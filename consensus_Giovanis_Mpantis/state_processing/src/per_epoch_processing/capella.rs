/// This module handles the processing of historical summaries updates.
mod historical_summaries_update;

/// Re-export the `process_historical_summaries_update` function from the 
/// `historical_summaries_update` module, making it available at the crate level.
pub use historical_summaries_update::process_historical_summaries_update;
