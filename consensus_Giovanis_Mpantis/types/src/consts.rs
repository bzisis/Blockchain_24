pub mod altair {
    /// Index of the flag indicating a timely source in participation flags.
    pub const TIMELY_SOURCE_FLAG_INDEX: usize = 0;
    /// Index of the flag indicating a timely target in participation flags.
    pub const TIMELY_TARGET_FLAG_INDEX: usize = 1;
    /// Index of the flag indicating a timely head in participation flags.
    pub const TIMELY_HEAD_FLAG_INDEX: usize = 2;
    
    /// Weight assigned to a timely source in participation.
    pub const TIMELY_SOURCE_WEIGHT: u64 = 14;
    /// Weight assigned to a timely target in participation.
    pub const TIMELY_TARGET_WEIGHT: u64 = 26;
    /// Weight assigned to a timely head in participation.
    pub const TIMELY_HEAD_WEIGHT: u64 = 14;
    
    /// Weight assigned to a sync reward in participation.
    pub const SYNC_REWARD_WEIGHT: u64 = 2;
    /// Weight assigned to a proposer in participation.
    pub const PROPOSER_WEIGHT: u64 = 8;
    /// Total weight denominator used in participation calculations.
    pub const WEIGHT_DENOMINATOR: u64 = 64;
    
    /// Number of sync committee subnets.
    pub const SYNC_COMMITTEE_SUBNET_COUNT: u64 = 4;
    /// Target number of aggregators per sync subcommittee.
    pub const TARGET_AGGREGATORS_PER_SYNC_SUBCOMMITTEE: u64 = 16;

    /// Array of participation flag weights.
    pub const PARTICIPATION_FLAG_WEIGHTS: [u64; NUM_FLAG_INDICES] = [
        TIMELY_SOURCE_WEIGHT,
        TIMELY_TARGET_WEIGHT,
        TIMELY_HEAD_WEIGHT,
    ];

    /// Number of participation flag indices.
    pub const NUM_FLAG_INDICES: usize = 3;
}
pub mod bellatrix {
    /// Number of intervals per slot.
    pub const INTERVALS_PER_SLOT: u64 = 3;
}
pub mod deneb {
    /// Re-exports the versioned hash version for KZG.
    pub use crate::VERSIONED_HASH_VERSION_KZG;
}
