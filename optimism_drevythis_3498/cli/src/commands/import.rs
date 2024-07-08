//! Command that initializes the node by importing OP Mainnet chain segment below Bedrock, from a
//! file.
use clap::Parser; // Import the Parser trait from the clap crate for command line argument parsing.
use reth_cli_commands::common::{AccessRights, Environment, EnvironmentArgs}; // Import common structs for CLI commands.
use reth_consensus::noop::NoopConsensus; // Import a no-operation consensus mechanism.
use reth_db::tables; // Import database table definitions.
use reth_db_api::transaction::DbTx; // Import database transaction traits.
use reth_downloaders::file_client::{
    ChunkedFileReader, FileClient, DEFAULT_BYTE_LEN_CHUNK_CHAIN_FILE,
}; // Import file client and chunked file reader for handling blockchain files.
use reth_node_core::version::SHORT_VERSION; // Import the short version of the node.
use reth_optimism_primitives::bedrock_import::is_dup_tx; // Import function to check for duplicate transactions.
use reth_provider::StageCheckpointReader; // Import trait for reading stage checkpoints.
use reth_prune::PruneModes; // Import pruning modes for cleaning up the database.
use reth_stages::StageId; // Import stage identifiers.
use reth_static_file::StaticFileProducer; // Import static file producer for serving static files.
use std::{path::PathBuf, sync::Arc}; // Import standard library modules for file paths and reference counting.
use tracing::{debug, error, info}; // Import logging macros.

use crate::commands::build_pipeline::build_import_pipeline; // Import function to build the import pipeline.

/// Syncs RLP encoded blocks from a file.
#[derive(Debug, Parser)]
pub struct ImportOpCommand {
    #[command(flatten)]
    env: EnvironmentArgs, // Flattened command line arguments for the environment.

    /// Chunk byte length to read from file.
    #[arg(long, value_name = "CHUNK_LEN", verbatim_doc_comment)]
    chunk_len: Option<u64>, // Optional argument for the byte length of file chunks.

    /// The path to a block file for import.
    ///
    /// The online stages (headers and bodies) are replaced by a file import, after which the
    /// remaining stages are executed.
    #[arg(value_name = "IMPORT_PATH", verbatim_doc_comment)]
    path: PathBuf, // Path to the file containing blocks to import.
}

impl ImportOpCommand {
    /// Execute the `import` command.
    pub async fn execute(self) -> eyre::Result<()> {
        // Log the start of the import process with the node's version.
        info!(target: "reth::cli", "reth {} starting", SHORT_VERSION);

        // Log that stages requiring state changes are disabled.
        info!(target: "reth::cli",
            "Disabled stages requiring state, since cannot execute OVM state changes"
        );

        // Log the chunk byte length being used for file import.
        debug!(target: "reth::cli",
            chunk_byte_len=self.chunk_len.unwrap_or(DEFAULT_BYTE_LEN_CHUNK_CHAIN_FILE),
            "Chunking chain import"
        );

        // Initialize the environment with read/write access rights.
        let Environment { provider_factory, config, .. } = self.env.init(AccessRights::RW)?;

        // Use a no-operation consensus mechanism because we expect the inputs to be valid.
        let consensus = Arc::new(NoopConsensus::default());

        // Open the file for reading in chunks.
        let mut reader = ChunkedFileReader::new(&self.path, self.chunk_len).await?;

        // Initialize counters for decoded blocks and transactions.
        let mut total_decoded_blocks = 0;
        let mut total_decoded_txns = 0;
        let mut total_filtered_out_dup_txns = 0;

        // Process each chunk from the file.
        while let Some(mut file_client) = reader.next_chunk::<FileClient>().await? {
            // Log the start of importing a chain file chunk.
            info!(target: "reth::cli",
                "Importing chain file chunk"
            );

            // Get the tip of the chain from the file client.
            let tip = file_client.tip().ok_or(eyre::eyre!("file client has no tip"))?;
            // Log that the chain file chunk was read.
            info!(target: "reth::cli", "Chain file chunk read");

            // Update the counters with the number of blocks and transactions in the chunk.
            total_decoded_blocks += file_client.headers_len();
            total_decoded_txns += file_client.total_transactions();

            // Filter out duplicate transactions from the chunk.
            for (block_number, body) in file_client.bodies_iter_mut() {
                body.transactions.retain(|_| {
                    if is_dup_tx(block_number) {
                        total_filtered_out_dup_txns += 1;
                        return false
                    }
                    true
                })
            }

            // Build the import pipeline with the current configuration and file client.
            let (mut pipeline, events) = build_import_pipeline(
                &config,
                provider_factory.clone(),
                &consensus,
                Arc::new(file_client),
                StaticFileProducer::new(provider_factory.clone(), PruneModes::default()),
                true,
            )
            .await?;

            // Override the tip in the pipeline with the file client's tip.
            pipeline.set_tip(tip);
            // Log that the tip was manually set.
            debug!(target: "reth::cli", ?tip, "Tip manually set");

            // Get a provider for accessing blockchain data.
            let provider = provider_factory.provider()?;

            // Retrieve the latest block number from the stage checkpoint.
            let latest_block_number =
                provider.get_stage_checkpoint(StageId::Finish)?.map(|ch| ch.block_number);
            
            // Spawn a task to handle node events.
            tokio::spawn(reth_node_events::node::handle_events(
                None,
                latest_block_number,
                events,
                provider_factory.db_ref().clone(),
            ));

            // Run the pipeline to process the imported blocks.
            info!(target: "reth::cli", "Starting sync pipeline");
            tokio::select! {
                res = pipeline.run() => res?, // Run the pipeline and await its completion.
                _ = tokio::signal::ctrl_c() => {}, // Handle CTRL+C signal to gracefully stop.
            }
        }

        // After processing all chunks, get the provider to access final imported data.
        let provider = provider_factory.provider()?;

        // Get the total number of imported blocks and transactions from the database.
        let total_imported_blocks = provider.tx_ref().entries::<tables::HeaderNumbers>()?;
        let total_imported_txns = provider.tx_ref().entries::<tables::TransactionHashNumbers>()?;

        // Check if the total decoded blocks and transactions match the imported ones, considering filtered duplicates.
        if total_decoded_blocks != total_imported_blocks ||
            total_decoded_txns != total_imported_txns + total_filtered_out_dup_txns
        {
            // Log an error if the chain was only partially imported.
            error!(target: "reth::cli",
                total_decoded_blocks,
                total_imported_blocks,
                total_decoded_txns,
                total_filtered_out_dup_txns,
                total_imported_txns,
                "Chain was partially imported"
            );
        }

        // Log the final import statistics.
        info!(target: "reth::cli",
            total_imported_blocks,
            total_imported_txns,
            total_decoded_blocks,
            total_decoded_txns,
            total_filtered_out_dup_txns,
            "Chain file imported"
        );

        // Return success result.
        Ok(())
    }
}
