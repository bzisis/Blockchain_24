//! Pruning and full node arguments
//! 
//! The `PruningArgs` struct provides options for controlling pruning behavior and full node operation. It includes a flag (`full`) that,
//! when set, prioritizes running the node in full mode where only the most recent [`MINIMUM_PRUNING_DISTANCE`] block states are retained.
//! 
//! The `prune_config` method of `PruningArgs` computes and returns a `PruneConfig` based on the current settings. If `full` is false, no
//! pruning configuration is returned. Otherwise, it configures pruning with specific parameters such as block intervals and pruning modes
//! for various data segments like sender recovery, transaction lookup, receipts, account history, and storage history. It also includes
//! a receipts log filter based on the deposit contract configuration.
//! 
//! Tests are provided to validate the default behavior of the `PruningArgs`, ensuring that when parsed without additional arguments,
//! it correctly defaults to the expected configuration.

use clap::Args;
use reth_chainspec::ChainSpec;
use reth_config::config::PruneConfig;
use reth_prune_types::{PruneMode, PruneModes, ReceiptsLogPruneConfig, MINIMUM_PRUNING_DISTANCE};

/// Parameters for pruning and full node
#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
#[command(next_help_heading = "Pruning")]
pub struct PruningArgs {
    /// Run full node. Only the most recent [`MINIMUM_PRUNING_DISTANCE`] block states are stored.
    /// This flag takes priority over pruning configuration in reth.toml.
    #[arg(long, default_value_t = false)]
    pub full: bool,
}

impl PruningArgs {
    /// Returns pruning configuration.
    pub fn prune_config(&self, chain_spec: &ChainSpec) -> Option<PruneConfig> {
        if !self.full {
            return None
        }
        Some(PruneConfig {
            block_interval: 5,
            segments: PruneModes {
                sender_recovery: Some(PruneMode::Full),
                transaction_lookup: None,
                receipts: chain_spec
                    .deposit_contract
                    .as_ref()
                    .map(|contract| PruneMode::Before(contract.block)),
                account_history: Some(PruneMode::Distance(MINIMUM_PRUNING_DISTANCE)),
                storage_history: Some(PruneMode::Distance(MINIMUM_PRUNING_DISTANCE)),
                receipts_log_filter: ReceiptsLogPruneConfig(
                    chain_spec
                        .deposit_contract
                        .as_ref()
                        .map(|contract| (contract.address, PruneMode::Before(contract.block)))
                        .into_iter()
                        .collect(),
                ),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// A helper type to parse Args more easily
    #[derive(Parser)]
    struct CommandParser<T: Args> {
        #[command(flatten)]
        args: T,
    }

    #[test]
    fn pruning_args_sanity_check() {
        let default_args = PruningArgs::default();
        let args = CommandParser::<PruningArgs>::parse_from(["reth"]).args;
        assert_eq!(args, default_args);
    }
}
