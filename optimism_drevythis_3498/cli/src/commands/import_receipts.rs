//! Command that imports OP mainnet receipts from Bedrock datadir, exported via
//! <https://github.com/testinprod-io/op-geth/pull/1>.

use clap::Parser; // Importing the Parser trait from clap for command line parsing.
use reth_cli_commands::common::{AccessRights, Environment, EnvironmentArgs}; // Importing common structures for CLI commands.
use reth_db::tables; // Importing database table definitions.
use reth_db_api::{database::Database, transaction::DbTx}; // Importing traits for database operations and transactions.
use reth_downloaders::{
    file_client::{ChunkedFileReader, DEFAULT_BYTE_LEN_CHUNK_CHAIN_FILE}, // Importing file client for chunked file reading.
    file_codec_ovm_receipt::HackReceiptFileCodec, // Importing codec for decoding OVM receipts.
    receipt_file_client::ReceiptFileClient, // Importing file client for receipt files.
};
use reth_execution_types::ExecutionOutcome; // Importing structure for execution outcomes.
use reth_node_core::version::SHORT_VERSION; // Importing the node version.
use reth_optimism_primitives::bedrock_import::is_dup_tx; // Importing function to check for duplicate transactions.
use reth_primitives::Receipts; // Importing receipts type.
use reth_provider::{
    OriginalValuesKnown, ProviderFactory, StageCheckpointReader, StateWriter,
    StaticFileProviderFactory, StaticFileWriter, StatsReader,
}; // Importing various traits for providers and writers.
use reth_stages::StageId; // Importing stage identifiers.
use reth_static_file_types::StaticFileSegment; // Importing types for static file segments.
use std::path::{Path, PathBuf}; // Importing path types from the standard library.
use tracing::{debug, error, info, trace}; // Importing macros for logging.

/// Initializes the database with the genesis block.
#[derive(Debug, Parser)]
pub struct ImportReceiptsOpCommand {
    #[command(flatten)]
    env: EnvironmentArgs, // Flattening the environment arguments.

    /// Chunk byte length to read from file.
    #[arg(long, value_name = "CHUNK_LEN", verbatim_doc_comment)]
    chunk_len: Option<u64>, // Optional argument for the chunk byte length.

    /// The path to a receipts file for import. File must use `HackReceiptFileCodec` (used for
    /// exporting OP chain segment below Bedrock block via testinprod/op-geth).
    ///
    /// <https://github.com/testinprod-io/op-geth/pull/1>
    #[arg(value_name = "IMPORT_PATH", verbatim_doc_comment)]
    path: PathBuf, // Path to the receipts file.
}

impl ImportReceiptsOpCommand {
    /// Execute `import` command
    pub async fn execute(self) -> eyre::Result<()> {
        // Log the start of the import process.
        info!(target: "reth::cli", "reth {} starting", SHORT_VERSION);

        // Log the chunk byte length being used.
        debug!(target: "reth::cli",
            chunk_byte_len=self.chunk_len.unwrap_or(DEFAULT_BYTE_LEN_CHUNK_CHAIN_FILE),
            "Chunking receipts import"
        );

        // Initialize the environment with read/write access rights.
        let Environment { provider_factory, .. } = self.env.init(AccessRights::RW)?;

        // Import receipts from the file.
        import_receipts_from_file(
            provider_factory,
            self.path,
            self.chunk_len,
            |first_block, receipts: &mut Receipts| {
                let mut total_filtered_out_dup_txns = 0;
                for (index, receipts_for_block) in receipts.iter_mut().enumerate() {
                    if is_dup_tx(first_block + index as u64) {
                        receipts_for_block.clear();
                        total_filtered_out_dup_txns += 1;
                    }
                }
                total_filtered_out_dup_txns
            },
        )
        .await
    }
}

/// Imports receipts to static files. Takes a filter callback as parameter, that returns the total
/// number of filtered out receipts.
///
/// Caution! Filter callback must replace completely filtered out receipts for a block, with empty
/// vectors, rather than `vec!(None)`. This is since the code for writing to static files, expects
/// indices in the [`Receipts`] list, to map to sequential block numbers.
pub async fn import_receipts_from_file<DB, P, F>(
    provider_factory: ProviderFactory<DB>, // The provider factory.
    path: P, // Path to the receipts file.
    chunk_len: Option<u64>, // Optional chunk length.
    mut filter: F, // Filter callback.
) -> eyre::Result<()>
where
    DB: Database, // Database trait bound.
    P: AsRef<Path>, // Path trait bound.
    F: FnMut(u64, &mut Receipts) -> usize, // Filter callback trait bound.
{
    let provider = provider_factory.provider_rw()?; // Get a read/write provider.
    let static_file_provider = provider_factory.static_file_provider(); // Get the static file provider.

    // Ensure transactions exist before importing receipts.
    let total_imported_txns = static_file_provider
        .count_entries::<tables::Transactions>()
        .expect("transaction static files must exist before importing receipts");
    let highest_block_transactions = static_file_provider
        .get_highest_static_file_block(StaticFileSegment::Transactions)
        .expect("transaction static files must exist before importing receipts");

    // Read stage checkpoints from the database.
    for stage in StageId::ALL {
        let checkpoint = provider.get_stage_checkpoint(stage)?;
        trace!(target: "reth::cli",
            ?stage,
            ?checkpoint,
            "Read stage checkpoints from db"
        );
    }

    // Prepare a transaction for writing to storage.
    let tx = provider.into_tx();
    let mut total_decoded_receipts = 0; // Initialize decoded receipts counter.
    let mut total_filtered_out_dup_txns = 0; // Initialize filtered out duplicate transactions counter.

    // Open the receipts file for reading in chunks.
    let mut reader = ChunkedFileReader::new(path, chunk_len).await?;

    // Process each chunk from the file.
    while let Some(file_client) =
        reader.next_chunk::<ReceiptFileClient<HackReceiptFileCodec>>().await?
    {
        // Create a new file client from the chunk.
        let ReceiptFileClient {
            mut receipts,
            first_block,
            total_receipts: total_receipts_chunk,
            ..
        } = file_client;

        // Mark these as decoded.
        total_decoded_receipts += total_receipts_chunk;

        // Apply the filter callback to the receipts.
        total_filtered_out_dup_txns += filter(first_block, &mut receipts);

        // Log the import of the receipt file chunk.
        info!(target: "reth::cli",
            first_receipts_block=?first_block,
            total_receipts_chunk,
            "Importing receipt file chunk"
        );

        // Create an execution outcome with the receipts.
        let execution_outcome =
            ExecutionOutcome::new(Default::default(), receipts, first_block, Default::default());

        // Get a static file producer for writing receipts.
        let static_file_producer =
            static_file_provider.get_writer(first_block, StaticFileSegment::Receipts)?;

        // Write the receipts to storage.
        execution_outcome.write_to_storage::<DB::TXMut>(
            &tx,
            Some(static_file_producer),
            OriginalValuesKnown::Yes,
        )?;
    }

    // Commit the transaction.
    tx.commit()?;
    static_file_provider.commit()?; // Commit the static file provider.

    // Check if any receipts were imported.
    if total_decoded_receipts == 0 {
        error!(target: "reth::cli", "No receipts were imported, ensure the receipt file is valid and not empty");
        return Ok(())
    }

    // Get the total number of imported receipts.
    let total_imported_receipts = static_file_provider
        .count_entries::<tables::Receipts>()
        .expect("static files must exist after ensuring we decoded more than zero");

    // Check if the total decoded receipts match the imported ones, considering filtered duplicates.
    if total_imported_receipts + total_filtered_out_dup_txns != total_decoded_receipts {
        error!(target: "reth::cli",
            total_decoded_receipts,
            total_imported_receipts,
            total_filtered_out_dup_txns,
            "Receipts were partially imported"
        );
    }

    // Check if the total imported receipts match the imported transactions.
    if total_imported_receipts != total_imported_txns {
        error!(target: "reth::cli",
            total_imported_receipts,
            total_imported_txns,
            "Receipts inconsistent with transactions"
        );
    }

    // Get the highest block number for receipts and transactions.
    let highest_block_receipts = static_file_provider
        .get_highest_static_file_block(StaticFileSegment::Receipts)
        .expect("static files must exist after ensuring we decoded more than zero");

    // Check if the highest block number for receipts matches the highest block number for transactions.
    if highest_block_receipts != highest_block_transactions {
        error!(target: "reth::cli",
            highest_block_receipts,
            highest_block_transactions,
            "Height of receipts inconsistent with transactions"
        );
    }

    // Log the final import statistics.
    info!(target: "reth::cli",
        total_imported_receipts,
        total_decoded_receipts,
        total_filtered_out_dup_txns,
        "Receipt file imported"
    );

    // Return success result.
    Ok(())
}
