#[cfg(not(feature = "std"))]
use alloc::{
    collections::BTreeMap, /// Import BTreeMap from alloc crate for collections
    format,                /// Import format from alloc crate for string formatting
    string::{String, ToString}, /// Import String and ToString trait from alloc crate for string operations
    vec::Vec,              /// Import Vec from alloc crate for dynamic arrays
};

use crate::{hardforks::Hardforks, ForkCondition}; /// Import Hardforks trait and ForkCondition enum from local crate

/// A container to pretty-print a hardfork.
///
/// The fork is formatted depending on its fork condition:
///
/// - Block and timestamp based forks are formatted in the same manner (`{name} <({eip})>
///   @{condition}`)
/// - TTD based forks are formatted separately as `{name} <({eip})> @{ttd} (network is <not> known
///   to be merged)`
///
/// An optional EIP can be attached to the fork to display as well. This should generally be in the
/// form of just `EIP-x`, e.g. `EIP-1559`.
#[derive(Debug)]
struct DisplayFork {
    /// The name of the hardfork (e.g. Frontier)
    name: String,
    /// The fork condition
    activated_at: ForkCondition,
    /// An optional EIP (e.g. `EIP-1559`).
    eip: Option<String>,
}

impl core::fmt::Display for DisplayFork {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let name_with_eip = if let Some(eip) = &self.eip {
            format!("{} ({})", self.name, eip) /// Format name with EIP if present
        } else {
            self.name.clone() /// Otherwise, clone the name
        };

        match self.activated_at {
            ForkCondition::Block(at) | ForkCondition::Timestamp(at) => {
                write!(f, "{name_with_eip:32} @{at}")?; /// Format block or timestamp based fork
            }
            ForkCondition::TTD { fork_block, total_difficulty } => {
                write!(
                    f,
                    "{:32} @{} ({})",
                    name_with_eip,
                    total_difficulty,
                    if fork_block.is_some() {
                        "network is known to be merged"
                    } else {
                        "network is not known to be merged"
                    }
                )?;
            }
            ForkCondition::Never => unreachable!(),
        }

        Ok(())
    }
}

// Todo: This will result in dep cycle so currently commented out
// # Examples
//
// ```
// # use reth_chainspec::MAINNET;
// println!("{}", MAINNET.display_hardforks());
// ```
//
/// A container for pretty-printing a list of hardforks.
///
/// An example of the output:
///
/// ```text
/// Pre-merge hard forks (block based):
// - Frontier                         @0
// - Homestead                        @1150000
// - Dao                              @1920000
// - Tangerine                        @2463000
// - SpuriousDragon                   @2675000
// - Byzantium                        @4370000
// - Constantinople                   @7280000
// - Petersburg                       @7280000
// - Istanbul                         @9069000
// - MuirGlacier                      @9200000
// - Berlin                           @12244000
// - London                           @12965000
// - ArrowGlacier                     @13773000
// - GrayGlacier                      @15050000
// Merge hard forks:
// - Paris                            @58750000000000000000000 (network is known to be merged)
// Post-merge hard forks (timestamp based):
// - Shanghai                         @1681338455
/// ```
#[derive(Debug)]
pub struct DisplayHardforks {
    /// A list of pre-merge (block based) hardforks
    pre_merge: Vec<DisplayFork>,
    /// A list of merge (TTD based) hardforks
    with_merge: Vec<DisplayFork>,
    /// A list of post-merge (timestamp based) hardforks
    post_merge: Vec<DisplayFork>,
}

impl core::fmt::Display for DisplayHardforks {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fn format(
            header: &str,
            forks: &[DisplayFork],
            next_is_empty: bool,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            writeln!(f, "{header}:")?; /// Write header to output
            let mut iter = forks.iter().peekable(); /// Create iterator over forks with peeking capability
            while let Some(fork) = iter.next() {
                write!(f, "- {fork}")?; /// Write each fork with bullet point
                if !next_is_empty || iter.peek().is_some() {
                    writeln!(f)?; /// Write newline if not last item or next section is not empty
                }
            }
            Ok(())
        }

        /// Format pre-merge hardforks
        format(
            "Pre-merge hard forks (block based)",
            &self.pre_merge,
            self.with_merge.is_empty(),
            f,
        )?;

        /// Format merge hard forks section if not empty
        if !self.with_merge.is_empty() {
            format("Merge hard forks", &self.with_merge, self.post_merge.is_empty(), f)?;
        }

        /// Format post-merge hard forks section if not empty
        if !self.post_merge.is_empty() {
            format("Post-merge hard forks (timestamp based)", &self.post_merge, true, f)?;
        }

        Ok(())
    }
}

impl DisplayHardforks {
    /// Creates a new [`DisplayHardforks`] from an iterator of hardforks.
    pub fn new<H: Hardforks>(hardforks: &H, known_paris_block: Option<u64>) -> Self {
        let mut pre_merge = Vec::new(); /// Initialize vector for pre-merge hardforks
        let mut with_merge = Vec::new(); /// Initialize vector for merge hardforks
        let mut post_merge = Vec::new(); ////// Initialize vector for post-merge hardforks

        for (fork, condition) in hardforks.forks_iter() {
            let mut display_fork =
                DisplayFork { name: fork.name().to_string(), activated_at: condition, eip: None }; /// Create DisplayFork instance

            match condition {
                ForkCondition::Block(_) => {
                    pre_merge.push(display_fork); /// Push block-based fork to pre-merge vector
                }
                ForkCondition::TTD { total_difficulty, .. } => {
                    display_fork.activated_at =
                        ForkCondition::TTD { fork_block: known_paris_block, total_difficulty };
                    with_merge.push(display_fork); /// Push TTD-based fork to merge vector
                }
                ForkCondition::Timestamp(_) => {
                    post_merge.push(display_fork); /// Push timestamp-based fork to post-merge vector
                }
                ForkCondition::Never => continue, /// Skip Never variant
            }
        }

        Self { pre_merge, with_merge, post_merge } /// Return initialized DisplayHardForks instance
    }
}
