// Import necessary modules and components
use node::NodeTestContext;
use reth::{
    args::{DiscoveryArgs, NetworkArgs, RpcServerArgs}, // Arguments for network, discovery, and RPC server
    builder::{NodeBuilder, NodeConfig, NodeHandle}, // Components to build a node
    tasks::TaskManager, // Task manager to handle async tasks
};
use reth_chainspec::ChainSpec; // Chain specification for the blockchain
use reth_db::{test_utils::TempDatabase, DatabaseEnv}; // Database components
use reth_node_builder::{
    components::NodeComponentsBuilder, FullNodeTypesAdapter, Node, NodeAdapter, RethFullAdapter,
}; // Node builder components
use reth_provider::providers::BlockchainProvider; // Blockchain provider
use std::sync::Arc; // Arc for thread-safe reference counting
use tracing::{span, Level}; // Tracing for logging
use wallet::Wallet; // Wallet module

// Define modules for organizing code
pub mod node;           // Module for test nodes
pub mod transaction;    // Module for transaction operations
pub mod wallet;         // Module for wallet operations
mod payload;            // Module for payload operations
mod network;            // Module for network operations
mod engine_api;         // Module for engine API operations
mod rpc;                // Module for RPC operations
mod traits;             // Module for helper traits

// Function to set up test nodes
pub async fn setup<N>(
    num_nodes: usize,                  // Number of nodes to create
    chain_spec: Arc<ChainSpec>,        // Chain specification
    is_dev: bool,                      // Development mode flag
) -> eyre::Result<(Vec<NodeHelperType<N>>, TaskManager, Wallet)> // Return result with nodes, task manager, and wallet
where
    N: Default + Node<TmpNodeAdapter<N>>, // Constraints for node type
{
    let tasks = TaskManager::current();   // Get current task manager
    let exec = tasks.executor();          // Get task executor

    // Network configuration with discovery disabled
    let network_config = NetworkArgs {
        discovery: DiscoveryArgs { disable_discovery: true, ..DiscoveryArgs::default() },
        ..NetworkArgs::default()
    };

    // Create a vector to hold the nodes
    let mut nodes: Vec<NodeTestContext<_>> = Vec::with_capacity(num_nodes);

    for idx in 0..num_nodes {
        // Configuration for each node
        let node_config = NodeConfig::test()
            .with_chain(chain_spec.clone()) // Use the provided chain spec
            .with_network(network_config.clone()) // Use the provided network config
            .with_unused_ports() // Use random unused ports
            .with_rpc(RpcServerArgs::default().with_unused_ports().with_http()) // Configure RPC server with unused ports and HTTP
            .set_dev(is_dev); // Set development mode if specified

        let span = span!(Level::INFO, "node", idx); // Logging span for each node
        let _enter = span.enter();                 // Enter the logging span
        let NodeHandle { node, node_exit_future: _ } = NodeBuilder::new(node_config.clone())
            .testing_node(exec.clone()) // Use the task executor
            .node(Default::default()) // Default node configuration
            .launch()
            .await?; // Launch the node

        let mut node = NodeTestContext::new(node).await?; // Initialize node context

        // Connect each node to the previous node
        if let Some(previous_node) = nodes.last_mut() {
            previous_node.connect(&mut node).await; // Connect to the previous node
        }

        // Connect the last node to the first node if there are more than two
        if idx + 1 == num_nodes && num_nodes > 2 {
            if let Some(first_node) = nodes.first_mut() {
                node.connect(first_node).await; // Connect to the first node
            }
        }

        nodes.push(node); // Add the node to the list
    }

    // Return the list of nodes, task manager, and a wallet with the chain ID
    Ok((nodes, tasks, Wallet::default().with_chain_id(chain_spec.chain().into())))
}

// Type aliases for convenience
type TmpDB = Arc<TempDatabase<DatabaseEnv>>; // Alias for a temporary database
type TmpNodeAdapter<N> = FullNodeTypesAdapter<N, TmpDB, BlockchainProvider<TmpDB>>; // Alias for a temporary node adapter

type Adapter<N> = NodeAdapter<
    RethFullAdapter<TmpDB, N>,
    <<N as Node<TmpNodeAdapter<N>>>::ComponentsBuilder as NodeComponentsBuilder<
        RethFullAdapter<TmpDB, N>,
    >>::Components,
>; // Alias for a node adapter

// Type alias for a node helper type
pub type NodeHelperType<N> = NodeTestContext<Adapter<N>>;