use reth_primitives::BlockNumber;

/// Events emitted by an `ExEx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// ExExEvent enum: Describes the purpose of the enum and provides details on its variant.
pub enum ExExEvent {
    /// Highest block processed by the `ExEx`.
    ///
    /// The `ExEx` must guarantee that it will not require all earlier blocks in the future,
    /// meaning that Reth is allowed to prune them.
    ///
    /// On reorgs, it's possible for the height to go down.
    FinishedHeight(BlockNumber),
}   // FinishedHeight variant: Explains what this variant represents, including the implications for block pruning and reorganization.
