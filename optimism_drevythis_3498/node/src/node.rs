//! Optimism Node types config.

use crate::{
    args::RollupArgs,
    txpool::{OpTransactionPool, OpTransactionValidator},
    OptimismEngineTypes,
};
use reth_basic_payload_builder::{BasicPayloadJobGenerator, BasicPayloadJobGeneratorConfig};
use reth_evm::ConfigureEvm;
use reth_evm_optimism::{OpExecutorProvider, OptimismEvmConfig};
use reth_network::{NetworkHandle, NetworkManager};
use reth_node_builder::{
    components::{
        ComponentsBuilder, ConsensusBuilder, ExecutorBuilder, NetworkBuilder,
        PayloadServiceBuilder, PoolBuilder,
    },
    node::{FullNodeTypes, NodeTypes},
    BuilderContext, Node, PayloadBuilderConfig,
};
use reth_optimism_consensus::OptimismBeaconConsensus;
use reth_payload_builder::{PayloadBuilderHandle, PayloadBuilderService};
use reth_provider::CanonStateSubscriptions;
use reth_tracing::tracing::{debug, info};
use reth_transaction_pool::{
    blobstore::DiskFileBlobStore, CoinbaseTipOrdering, TransactionPool,
    TransactionValidationTaskExecutor,
};
use std::sync::Arc;

/// Type configuration for a regular Optimism node.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct OptimismNode {
    /// Additional Optimism args
    pub args: RollupArgs,
}

impl OptimismNode {
    /// Creates a new instance of the Optimism node type.
    pub const fn new(args: RollupArgs) -> Self {
        Self { args }
    }

    /// Returns the components for the given [`RollupArgs`].
    pub fn components<Node>(
        args: RollupArgs,
    ) -> ComponentsBuilder<
        Node,
        OptimismPoolBuilder,
        OptimismPayloadBuilder,
        OptimismNetworkBuilder,
        OptimismExecutorBuilder,
        OptimismConsensusBuilder,
    >
    where
        Node: FullNodeTypes<Engine = OptimismEngineTypes>,
    {
        let RollupArgs { disable_txpool_gossip, compute_pending_block, .. } = args;
        
        // Initialize and configure various components of the node using Builders
        ComponentsBuilder::default()
            .node_types::<Node>()
            .pool(OptimismPoolBuilder::default())
            .payload(OptimismPayloadBuilder::new(
                compute_pending_block,
                OptimismEvmConfig::default(),
            ))
            .network(OptimismNetworkBuilder { disable_txpool_gossip })
            .executor(OptimismExecutorBuilder::default())
            .consensus(OptimismConsensusBuilder::default())
    }
}

impl<N> Node<N> for OptimismNode
where
    N: FullNodeTypes<Engine = OptimismEngineTypes>,
{
    type ComponentsBuilder = ComponentsBuilder<
        N,
        OptimismPoolBuilder,
        OptimismPayloadBuilder,
        OptimismNetworkBuilder,
        OptimismExecutorBuilder,
        OptimismConsensusBuilder,
    >;

    /// Retrieves the components builder for the Optimism node.
    fn components_builder(self) -> Self::ComponentsBuilder {
        let Self { args } = self;
        Self::components(args)
    }
}

impl NodeTypes for OptimismNode {
    type Primitives = ();
    type Engine = OptimismEngineTypes;
}

/// A regular optimism evm and executor builder.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct OptimismExecutorBuilder;

impl<Node> ExecutorBuilder<Node> for OptimismExecutorBuilder
where
    Node: FullNodeTypes,
{
    type EVM = OptimismEvmConfig;
    type Executor = OpExecutorProvider<Self::EVM>;

    async fn build_evm(
        self,
        ctx: &BuilderContext<Node>,
    ) -> eyre::Result<(Self::EVM, Self::Executor)> {
        let chain_spec = ctx.chain_spec();
        let evm_config = OptimismEvmConfig::default();
        
        // Create an executor provider using the chain spec and EVM configuration
        let executor = OpExecutorProvider::new(chain_spec, evm_config);

        Ok((evm_config, executor))
    }
}

/// A basic optimism transaction pool.
///
/// This contains various settings that can be configured and take precedence over the node's
/// config.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct OptimismPoolBuilder;

impl<Node> PoolBuilder<Node> for OptimismPoolBuilder
where
    Node: FullNodeTypes,
{
    type Pool = OpTransactionPool<Node::Provider, DiskFileBlobStore>;

    async fn build_pool(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::Pool> {
        let data_dir = ctx.config().datadir();
        let blob_store = DiskFileBlobStore::open(data_dir.blobstore(), Default::default())?;
        
        // Configure transaction validation tasks for the transaction pool
        let validator = TransactionValidationTaskExecutor::eth_builder(ctx.chain_spec())
            .with_head_timestamp(ctx.head().timestamp)
            .kzg_settings(ctx.kzg_settings()?)
            .with_additional_tasks(1)
            .build_with_tasks(
                ctx.provider().clone(),
                ctx.task_executor().clone(),
                blob_store.clone(),
            )
            .map(OpTransactionValidator::new);

        // Create the transaction pool with configured validator and settings
        let transaction_pool = reth_transaction_pool::Pool::new(
            validator,
            CoinbaseTipOrdering::default(),
            blob_store,
            ctx.pool_config(),
        );
        
        // Log initialization message
        info!(target: "reth::cli", "Transaction pool initialized");
        
        let transactions_path = data_dir.txpool_transactions();

        // spawn txpool maintenance task
        {
            let pool = transaction_pool.clone();
            let chain_events = ctx.provider().canonical_state_stream();
            let client = ctx.provider().clone();
            let transactions_backup_config =
                reth_transaction_pool::maintain::LocalTransactionBackupConfig::with_local_txs_backup(transactions_path);

            // Spawn tasks for maintaining transaction pool integrity
            ctx.task_executor().spawn_critical_with_graceful_shutdown_signal(
                "local transactions backup task",
                |shutdown| {
                    reth_transaction_pool::maintain::backup_local_transactions_task(
                        shutdown,
                        pool.clone(),
                        transactions_backup_config,
                    )
                },
            );

            // spawn the maintenance task
            ctx.task_executor().spawn_critical(
                "txpool maintenance task",
                reth_transaction_pool::maintain::maintain_transaction_pool_future(
                    client,
                    pool,
                    chain_events,
                    ctx.task_executor().clone(),
                    Default::default(),
                ),
            );
            
            debug!(target: "reth::cli", "Spawned txpool maintenance task");
        }

        Ok(transaction_pool)
    }
}

/// A basic optimism payload service builder
#[derive(Debug, Default, Clone)]
pub struct OptimismPayloadBuilder<EVM = OptimismEvmConfig> {
    /// By default the pending block equals the latest block
    /// to save resources and not leak txs from the tx-pool,
    /// this flag enables computing of the pending block
    /// from the tx-pool instead.
    ///
    /// If `compute_pending_block` is not enabled, the payload builder
    /// will use the payload attributes from the latest block. Note
    /// that this flag is not yet functional.
    pub compute_pending_block: bool,
    /// The EVM configuration to use for the payload builder.
    pub evm_config: EVM,
}

impl<EVM> OptimismPayloadBuilder<EVM> {
    /// Create a new instance with the given `compute_pending_block` flag and evm config.
    pub const fn new(compute_pending_block: bool, evm_config: EVM) -> Self {
        Self { compute_pending_block, evm_config }
    }
}

impl<Node, EVM, Pool> PayloadServiceBuilder<Node, Pool> for OptimismPayloadBuilder<EVM>
where
    Node: FullNodeTypes<Engine = OptimismEngineTypes>,
    Pool: TransactionPool + Unpin + 'static,
    EVM: ConfigureEvm,
{
    async fn spawn_payload_service(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> eyre::Result<PayloadBuilderHandle<Node::Engine>> {
        // Initialize the payload builder with chain spec and EVM configuration
        let payload_builder = reth_optimism_payload_builder::OptimismPayloadBuilder::new(
            ctx.chain_spec(),
            self.evm_config,
        )
        .set_compute_pending_block(self.compute_pending_block);
        
        let conf = ctx.payload_builder_config();

        // Configure payload job generator settings
        let payload_job_config = BasicPayloadJobGeneratorConfig::default()
            .interval(conf.interval())
            .deadline(conf.deadline())
            .max_payload_tasks(conf.max_payload_tasks())
            // no extradata for OP
            .extradata(Default::default());

        // Create a basic payload job generator with configured settings
        let payload_generator = BasicPayloadJobGenerator::with_builder(
            ctx.provider().clone(),
            pool,
            ctx.task_executor().clone(),
            payload_job_config,
            ctx.chain_spec(),
            payload_builder,
        );
        
        // Create payload builder service and spawn it as a task
        let (payload_service, payload_builder) =
            PayloadBuilderService::new(payload_generator, ctx.provider().canonical_state_stream());

        ctx.task_executor().spawn_critical("payload builder service", Box::pin(payload_service));

        Ok(payload_builder)
    }
}

/// A basic optimism network builder.
#[derive(Debug, Default, Clone)]
pub struct OptimismNetworkBuilder {
    /// Disable transaction pool gossip
    pub disable_txpool_gossip: bool,
}

impl<Node, Pool> NetworkBuilder<Node, Pool> for OptimismNetworkBuilder
where
    Node: FullNodeTypes,
    Pool: TransactionPool + Unpin + 'static,
{
    async fn build_network(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> eyre::Result<NetworkHandle> {
        let Self { disable_txpool_gossip } = self;

        let args = &ctx.config().network;

        // Build network configuration with custom settings
        let network_builder = ctx
            .network_config_builder()?
            // purposefully disable discv4
            .disable_discv4_discovery()
            // apply discovery settings
            .apply(|mut builder| {
                let rlpx_socket = (args.addr, args.port).into();

                if !args.discovery.disable_discovery {
                    builder = builder.discovery_v5(args.discovery.discovery_v5_builder(
                        rlpx_socket,
                        ctx.chain_spec().bootnodes().unwrap_or_default(),
                    ));
                }

                builder
            });

        // Create network configuration based on the builder
        let mut network_config = ctx.build_network_config(network_builder);

        // Configure network settings based on node parameters
        network_config.tx_gossip_disabled = disable_txpool_gossip;

        // Build network manager with the configured network config
        let network = NetworkManager::builder(network_config).await?;

        // Start network with initialized network manager and transaction pool
        let handle = ctx.start_network(network, pool);

        Ok(handle)
    }
}

/// A basic optimism consensus builder.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct OptimismConsensusBuilder;

impl<Node> ConsensusBuilder<Node> for OptimismConsensusBuilder
where
    Node: FullNodeTypes,
{
    type Consensus = Arc<dyn reth_consensus::Consensus>;

    async fn build_consensus(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::Consensus> {
        // Determine the type of consensus based on development environment
        if ctx.is_dev() {
            Ok(Arc::new(reth_auto_seal_consensus::AutoSealConsensus::new(ctx.chain_spec())))
        } else {
            Ok(Arc::new(OptimismBeaconConsensus::new(ctx.chain_spec())))
        }
    }
}
