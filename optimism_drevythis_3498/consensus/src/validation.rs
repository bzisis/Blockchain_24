[package]
name = "reth-optimism-consensus"       # Specifies the name of the package as "reth-optimism-consensus".
version.workspace = true               # Inherits the version number from the workspace configuration.
edition.workspace = true               # Inherits the Rust edition (e.g., 2018, 2021) from the workspace configuration.
rust-version.workspace = true          # Inherits the Rust version from the workspace configuration.
license.workspace = true               # Inherits the license information from the workspace configuration.
homepage.workspace = true              # Inherits the homepage URL from the workspace configuration.
repository.workspace = true            # Inherits the repository URL from the workspace configuration.
exclude.workspace = true               # Inherits the exclude patterns (files or directories to be ignored) from the workspace configuration.

[lints]
workspace = true                       # Inherits linting rules and configurations from the workspace.

[dependencies]
# reth
reth-consensus-common.workspace = true # Uses the "reth-consensus-common" crate from the workspace.
reth-chainspec.workspace = true        # Uses the "reth-chainspec" crate from the workspace.
reth-primitives.workspace = true       # Uses the "reth-primitives" crate from the workspace.
reth-consensus.workspace = true        # Uses the "reth-consensus" crate from the workspace.

tracing.workspace = true               # Uses the "tracing" crate from the workspace.

[features]
optimism = ["reth-primitives/optimism"] # Defines a feature named "optimism" that includes the "reth-primitives/optimism" feature.
