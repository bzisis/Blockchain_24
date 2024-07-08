use reth_chainspec::{ChainSpec, OptimismHardfork};
use reth_ethereum_forks::{EthereumHardfork, Head};

/// Returns the spec id at the given timestamp.
///
/// Note: This function is intended to be used only after the merge when hardforks are activated by
/// timestamp. It checks which Optimism hardfork is active at the given timestamp and returns the
/// corresponding `SpecId`.
pub fn revm_spec_by_timestamp_after_bedrock(
    chain_spec: &ChainSpec,
    timestamp: u64,
) -> revm_primitives::SpecId {
    // Check if the Fjord hardfork is active at the given timestamp.
    if chain_spec.fork(OptimismHardfork::Fjord).active_at_timestamp(timestamp) {
        revm_primitives::FJORD
    // Check if the Ecotone hardfork is active at the given timestamp.
    } else if chain_spec.fork(OptimismHardfork::Ecotone).active_at_timestamp(timestamp) {
        revm_primitives::ECOTONE
    // Check if the Canyon hardfork is active at the given timestamp.
    } else if chain_spec.fork(OptimismHardfork::Canyon).active_at_timestamp(timestamp) {
        revm_primitives::CANYON
    // Check if the Regolith hardfork is active at the given timestamp.
    } else if chain_spec.fork(OptimismHardfork::Regolith).active_at_timestamp(timestamp) {
        revm_primitives::REGOLITH
    // Default to the Bedrock hardfork if none of the above are active.
    } else {
        revm_primitives::BEDROCK
    }
}

/// Returns the `revm_spec` based on the spec configuration.
///
/// This function determines the active hardfork at the current block and returns the corresponding
/// `SpecId`. It includes both Optimism and Ethereum hardforks.
pub fn revm_spec(chain_spec: &ChainSpec, block: &Head) -> revm_primitives::SpecId {
    // Check if each Optimism hardfork is active at the current block.
    if chain_spec.fork(OptimismHardfork::Fjord).active_at_head(block) {
        revm_primitives::FJORD
    } else if chain_spec.fork(OptimismHardfork::Ecotone).active_at_head(block) {
        revm_primitives::ECOTONE
    } else if chain_spec.fork(OptimismHardfork::Canyon).active_at_head(block) {
        revm_primitives::CANYON
    } else if chain_spec.fork(OptimismHardfork::Regolith).active_at_head(block) {
        revm_primitives::REGOLITH
    } else if chain_spec.fork(OptimismHardfork::Bedrock).active_at_head(block) {
        revm_primitives::BEDROCK
    // Check if each Ethereum hardfork is active at the current block.
    } else if chain_spec.fork(EthereumHardfork::Prague).active_at_head(block) {
        revm_primitives::PRAGUE
    } else if chain_spec.fork(EthereumHardfork::Cancun).active_at_head(block) {
        revm_primitives::CANCUN
    } else if chain_spec.fork(EthereumHardfork::Shanghai).active_at_head(block) {
        revm_primitives::SHANGHAI
    } else if chain_spec.fork(EthereumHardfork::Paris).active_at_head(block) {
        revm_primitives::MERGE
    } else if chain_spec.fork(EthereumHardfork::London).active_at_head(block) {
        revm_primitives::LONDON
    } else if chain_spec.fork(EthereumHardfork::Berlin).active_at_head(block) {
        revm_primitives::BERLIN
    } else if chain_spec.fork(EthereumHardfork::Istanbul).active_at_head(block) {
        revm_primitives::ISTANBUL
    } else if chain_spec.fork(EthereumHardfork::Petersburg).active_at_head(block) {
        revm_primitives::PETERSBURG
    } else if chain_spec.fork(EthereumHardfork::Byzantium).active_at_head(block) {
        revm_primitives::BYZANTIUM
    } else if chain_spec.fork(EthereumHardfork::SpuriousDragon).active_at_head(block) {
        revm_primitives::SPURIOUS_DRAGON
    } else if chain_spec.fork(EthereumHardfork::Tangerine).active_at_head(block) {
        revm_primitives::TANGERINE
    } else if chain_spec.fork(EthereumHardfork::Homestead).active_at_head(block) {
        revm_primitives::HOMESTEAD
    } else if chain_spec.fork(EthereumHardfork::Frontier).active_at_head(block) {
        revm_primitives::FRONTIER
    } else {
        // Panic if no hardforks are active, which is considered invalid.
        panic!(
            "invalid hardfork chainspec: expected at least one hardfork, got {:?}",
            chain_spec.hardforks
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reth_chainspec::ChainSpecBuilder;

    #[test]
    fn test_revm_spec_by_timestamp_after_merge() {
        #[inline(always)]
        fn op_cs(f: impl FnOnce(ChainSpecBuilder) -> ChainSpecBuilder) -> ChainSpec {
            // Create a mainnet chain spec for the Optimism chain (chain ID 10).
            let cs = ChainSpecBuilder::mainnet().chain(reth_chainspec::Chain::from_id(10));
            f(cs).build()
        }

        // Test if the correct SpecId is returned based on the activation timestamp.
        assert_eq!(
            revm_spec_by_timestamp_after_bedrock(&op_cs(|cs| cs.fjord_activated()), 0),
            revm_primitives::FJORD
        );
        assert_eq!(
            revm_spec_by_timestamp_after_bedrock(&op_cs(|cs| cs.ecotone_activated()), 0),
            revm_primitives::ECOTONE
        );
        assert_eq!(
            revm_spec_by_timestamp_after_bedrock(&op_cs(|cs| cs.canyon_activated()), 0),
            revm_primitives::CANYON
        );
        assert_eq!(
            revm_spec_by_timestamp_after_bedrock(&op_cs(|cs| cs.bedrock_activated()), 0),
            revm_primitives::BEDROCK
        );
        assert_eq!(
            revm_spec_by_timestamp_after_bedrock(&op_cs(|cs| cs.regolith_activated()), 0),
            revm_primitives::REGOLITH
        );
    }

    #[test]
    fn test_to_revm_spec() {
        #[inline(always)]
        fn op_cs(f: impl FnOnce(ChainSpecBuilder) -> ChainSpecBuilder) -> ChainSpec {
            // Create a mainnet chain spec for the Optimism chain (chain ID 10).
            let cs = ChainSpecBuilder::mainnet().chain(reth_chainspec::Chain::from_id(10));
            f(cs).build()
        }

        // Test if the correct SpecId is returned based on the head state.
        assert_eq!(
            revm_spec(&op_cs(|cs| cs.fjord_activated()), &Head::default()),
            revm_primitives::FJORD
        );
        assert_eq!(
            revm_spec(&op_cs(|cs| cs.ecotone_activated()), &Head::default()),
            revm_primitives::ECOTONE
        );
        assert_eq!(
            revm_spec(&op_cs(|cs| cs.canyon_activated()), &Head::default()),
            revm_primitives::CANYON
        );
        assert_eq!(
            revm_spec(&op_cs(|cs| cs.bedrock_activated()), &Head::default()),
            revm_primitives::BEDROCK
        );
        assert_eq!(
            revm_spec(&op_cs(|cs| cs.regolith_activated()), &Head::default()),
            revm_primitives::REGOLITH
        );
    }
}
