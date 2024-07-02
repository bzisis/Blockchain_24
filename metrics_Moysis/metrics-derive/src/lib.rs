// Moysis Moysis Volos, Greece 29/06/2024.

//! This crate provides the [Metrics] derive macro.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png", // URL for the documentation logo
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256", // URL for the favicon
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/" // Base URL for the issue tracker
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))] // Warn about unused crate dependencies when not testing
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))] // Enable additional documentation features when using docs.rs

// Import necessary crates and modules for procedural macro functionality
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

// Module declarations for internal functionality
mod expand; // Module for expanding the derive macro
mod metric; // Module for handling metric-related logic
mod with_attrs; // Module for working with attributes

/// A procedural macro to derive the `Metrics` trait for a struct.
///
/// This macro generates code to create and manage metrics for the fields of a struct,
/// based on the provided `metrics` and `metric` attributes. It uses the `expand::derive`
/// function to perform the main logic of the macro expansion.
///
/// # Arguments
/// * `input` - The input token stream representing the struct for which the `Metrics` trait
///   should be derived.
///
/// # Returns
/// A `TokenStream` containing the generated code for the `Metrics` implementation.
#[proc_macro_derive(Metrics, attributes(metrics, metric))]
pub fn derive_metrics(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a `DeriveInput` syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    
    // Expand the macro using the `expand::derive` function, converting errors to compile errors.
    expand::derive(&input).unwrap_or_else(|err| err.to_compile_error()).into()
}