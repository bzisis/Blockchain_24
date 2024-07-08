// Conditional compilation: Includes the `p2p` module only when the `optimism` feature is enabled.
#[cfg(feature = "optimism")]
mod p2p;

// Conditional compilation: Includes the `utils` module only when the `optimism` feature is enabled.
#[cfg(feature = "optimism")]
mod utils;

// Defines the `main` function as a constant function (`const fn`), which is not executable.
// This effectively compiles to an empty main function, suitable for cases where the main logic
// is conditionally included based on features or configurations.
const fn main() {}
