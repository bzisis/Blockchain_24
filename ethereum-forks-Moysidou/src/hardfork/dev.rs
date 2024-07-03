use crate::{ChainHardforks, EthereumHardfork, ForkCondition};
use alloy_primitives::U256;
use once_cell::sync::Lazy;

/// Dev hardforks
pub static DEV_HARDFORKS: Lazy<ChainHardforks> = Lazy::new(|| {
    /// Initialize ChainHardforks with a vector of tuples containing hardforks and their conditions
    ChainHardforks::new(vec![
        /// Ethereum hardforks activated at block 0
        (EthereumHardfork::Frontier.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Homestead.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Dao.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Tangerine.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::SpuriousDragon.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Byzantium.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Constantinople.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Petersburg.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Istanbul.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Berlin.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::London.boxed(), ForkCondition::Block(0)),
        (
            EthereumHardfork::Paris.boxed(),
            ForkCondition::TTD { fork_block: None, total_difficulty: U256::ZERO },
        ),
        /// Shanghai and Cancun hardforks activated at timestamp 0
        (EthereumHardfork::Shanghai.boxed(), ForkCondition::Timestamp(0)),
        (EthereumHardfork::Cancun.boxed(), ForkCondition::Timestamp(0)),

        /// Optimism hardforks activated conditionally based on feature flag "optimism"
        #[cfg(feature = "optimism")]
        (crate::OptimismHardfork::Regolith.boxed(), ForkCondition::Timestamp(0)),
        #[cfg(feature = "optimism")]
        (crate::OptimismHardfork::Bedrock.boxed(), ForkCondition::Block(0)),
        #[cfg(feature = "optimism")]
        (crate::OptimismHardfork::Ecotone.boxed(), ForkCondition::Timestamp(0)),
    ])
});
