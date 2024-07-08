use alloy_primitives::B256;
use futures_util::{Stream, StreamExt};
use reth_config::Config;
use reth_consensus::Consensus;
use reth_db_api::database::Database;
use reth_downloaders::{
    bodies::bodies::BodiesDownloaderBuilder, file_client::FileClient,
    headers::reverse_headers::ReverseHeadersDownloaderBuilder,
};
use reth_errors::ProviderError;
use reth_evm_optimism::OpExecutorProvider;
use reth_network_p2p::{
    bodies::downloader::BodyDownloader,
    headers::downloader::{HeaderDownloader, SyncTarget},
};
use reth_node_events::node::NodeEvent;
use reth_provider::{BlockNumReader, ChainSpecProvider, HeaderProvider, ProviderFactory};
use reth_prune::PruneModes;
use reth_stages::{sets::DefaultStages, Pipeline, StageSet};
use reth_stages_types::StageId;
use reth_static_file::StaticFileProducer;
use std::sync::Arc;
use tokio::sync::watch;

/// Builds the import pipeline for the blockchain.
///
/// Depending on the configuration, the pipeline will either execute all stages or only those that
/// don't require the state.
///
/// # Parameters
/// - `config`: The configuration object which contains settings for the stages.
/// - `provider_factory`: Factory for providing data access to the blockchain.
/// - `consensus`: Arc-wrapped consensus object to manage consensus-related operations.
/// - `file_client`: Arc-wrapped file client for accessing blockchain files.
/// - `static_file_producer`: Producer for serving static files.
/// - `disable_exec`: A flag to disable execution stages if set to true.
///
/// # Returns
/// A result containing the built pipeline and a stream of node events.
pub async fn build_import_pipeline<DB, C>(
    config: &Config,
    provider_factory: ProviderFactory<DB>,
    consensus: &Arc<C>,
    file_client: Arc<FileClient>,
    static_file_producer: StaticFileProducer<DB>,
    disable_exec: bool,
) -> eyre::Result<(Pipeline<DB>, impl Stream<Item = NodeEvent>)>
where
    DB: Database + Clone + Unpin + 'static,
    C: Consensus + 'static,
{
    // Ensure the file client has canonical blocks; if not, exit early with an error.
    if !file_client.has_canonical_blocks() {
        eyre::bail!("unable to import non canonical blocks");
    }

    // Retrieve the latest block number stored in the database.
    let last_block_number = provider_factory.last_block_number()?;
    
    // Get the sealed header for the latest block. If not found, return an error.
    let local_head = provider_factory
        .sealed_header(last_block_number)?
        .ok_or(ProviderError::HeaderNotFound(last_block_number.into()))?;

    // Initialize the header downloader using the reverse headers downloader builder.
    let mut header_downloader = ReverseHeadersDownloaderBuilder::new(config.stages.headers)
        .build(file_client.clone(), consensus.clone())
        .into_task();
    
    // Update the local head and sync target for the header downloader.
    header_downloader.update_local_head(local_head);
    header_downloader.update_sync_target(SyncTarget::Tip(file_client.tip().unwrap()));

    // Initialize the body downloader using the bodies downloader builder.
    let mut body_downloader = BodiesDownloaderBuilder::new(config.stages.bodies)
        .build(file_client.clone(), consensus.clone(), provider_factory.clone())
        .into_task();
    
    // Set the download range for the body downloader based on the file client's min and max blocks.
    body_downloader
        .set_download_range(file_client.min_block().unwrap()..=file_client.max_block().unwrap())
        .expect("failed to set download range");

    // Create a watch channel for broadcasting the tip.
    let (tip_tx, tip_rx) = watch::channel(B256::ZERO);
    
    // Initialize the executor provider specific to Optimism.
    let executor = OpExecutorProvider::optimism(provider_factory.chain_spec());

    // Determine the maximum block to sync, defaulting to 0 if none found.
    let max_block = file_client.max_block().unwrap_or(0);

    // Build the pipeline with the configured stages and the tip sender.
    let pipeline = Pipeline::builder()
        .with_tip_sender(tip_tx)
        .with_max_block(max_block)
        .add_stages(
            DefaultStages::new(
                provider_factory.clone(),
                tip_rx,
                consensus.clone(),
                header_downloader,
                body_downloader,
                executor,
                config.stages.clone(),
                PruneModes::default(),
            )
            .builder()
            .disable_all_if(&StageId::STATE_REQUIRED, || disable_exec), // Disable stages requiring state if execution is disabled.
        )
        .build(provider_factory, static_file_producer);

    // Convert pipeline events into a stream of node events.
    let events = pipeline.events().map(Into::into);

    // Return the constructed pipeline and the events stream.
    Ok((pipeline, events))
}
