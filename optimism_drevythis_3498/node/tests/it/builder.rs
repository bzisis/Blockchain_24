//! Node builder setup tests.

// Import necessary crates and utilities
use reth_db::test_utils::create_test_rw_db;
use reth_node_api::FullNodeComponents;
use reth_node_builder::{NodeBuilder, NodeConfig};
use reth_node_optimism::node::OptimismNode;

// Unit test to validate basic setup of node builder
#[test]
fn test_basic_setup() {
    // Parse CLI -> config
    let config = NodeConfig::test(); // Obtain a test configuration for the node
    let db = create_test_rw_db(); // Create a test read-write database

    // Initialize a new NodeBuilder
    let _builder = NodeBuilder::new(config)
        .with_database(db) // Set up the node with the created database
        .with_types::<OptimismNode>() // Specify types associated with OptimismNode
        .with_components(OptimismNode::components(Default::default())) // Configure components for OptimismNode
        .on_component_initialized(move |ctx| {
            let _provider = ctx.provider(); // Retrieve provider from context when component is initialized
            Ok(()) // Return Ok(()) to indicate initialization success
        })
        .on_node_started(|_full_node| Ok(())) // Execute closure when node starts
        .on_rpc_started(|_ctx, handles| {
            let _client = handles.rpc.http_client(); // Access HTTP client from RPC handles
            Ok(()) // Return Ok(()) to indicate RPC start success
        })
        .extend_rpc_modules(|ctx| {
            let _ = ctx.config(); // Access configuration from context
            let _ = ctx.node().provider(); // Access provider from node in context

            Ok(()) // Return Ok(()) to indicate successful extension of RPC modules
        })
        .check_launch(); // Check if node can be launched without errors
}
