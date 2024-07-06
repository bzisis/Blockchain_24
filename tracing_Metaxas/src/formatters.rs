/// Imports and Dependencies
/// This section includes necessary imports and dependencies for the logging and tracing functionalities.

/// Importing the BoxedLayer type from the crate::layers module.
use crate::layers::BoxedLayer;

/// Importing ValueEnum from the clap crate, which is used for command-line argument parsing.
use clap::ValueEnum;

/// Importing fmt and Display from the standard library for formatting.
use std::{fmt, fmt::Display};

/// Importing NonBlocking from the tracing_appender crate, which provides non-blocking logging capabilities.
use tracing_appender::non_blocking::NonBlocking;

/// Importing various components from the tracing_subscriber crate for setting up logging and tracing.
use tracing_subscriber::{EnvFilter, Layer, Registry};



/// Log Format Enumeration
/// This section defines an enumeration that specifies the supported logging formats.

// Represents the logging format.
#[derive(Debug, Copy, Clone, ValueEnum, Eq, PartialEq)]
pub enum LogFormat {
    /// JSON format for logs.
    /// This format outputs log records as JSON objects, 
    /// which is useful for structured logging.
    Json,

    /// logfmt (key=value) format for logs.
    /// This concise, human-readable format is often used in command-line applications.
    LogFmt,

    /// Terminal-friendly format for logs.
    /// This format is designed to be easily read in terminal interfaces.
    Terminal,
}


/// LogFormat Implementation
/// This section implements the LogFormat enum, providing a method to apply the selected logging format
/// to create a new tracing layer with additional configurations for filtering and output.

impl LogFormat {
    /// Applies the specified logging format to create a new layer.
    ///
    /// This method constructs a tracing layer with the selected format,
    /// along with additional configurations for filtering and output.
    ///
    /// # Arguments
    /// * `filter` - An `EnvFilter` used to determine which log records to output.
    /// * `color` - An optional string that enables or disables ANSI color codes in the logs.
    /// * `file_writer` - An optional `NonBlocking` writer for directing logs to a file.
    ///
    /// # Returns
    /// A `BoxedLayer<Registry>` that can be added to a tracing subscriber.
    pub fn apply(
        &self,
        filter: EnvFilter,
        color: Option<String>,
        file_writer: Option<NonBlocking>,
    ) -> BoxedLayer<Registry> {
        /// Determine if ANSI colors should be used in the logs
        let ansi = if let Some(color) = color {
            std::env::var("RUST_LOG_STYLE").map(|val| val != "never").unwrap_or(color != "never")
        } else {
            false
        };

        /// Determine if log targets should be shown in the logs
        let target = std::env::var("RUST_LOG_TARGET")
            /// `RUST_LOG_TARGET` always overrides default behavior
            .map(|val| val != "0")
            .unwrap_or_else(|_|
                /// If `RUST_LOG_TARGET` is not set, show target in logs only if the max enabled
                /// level is higher than INFO (DEBUG, TRACE)
                filter.max_level_hint().map_or(true, |max_level| max_level > tracing::Level::INFO));

        /// Match the logging format and create the appropriate layer
        match self {
            Self::Json => {
                /// Create a JSON formatted layer
                let layer =
                    tracing_subscriber::fmt::layer().json().with_ansi(ansi).with_target(target);

                /// Use the provided file_writer if available
                if let Some(writer) = file_writer {
                    layer.with_writer(writer).with_filter(filter).boxed()
                } else {
                    layer.with_filter(filter).boxed()
                }
            }
            Self::LogFmt => {
                /// Create a logfmt (key=value) formatted layer
                tracing_logfmt::layer().with_filter(filter).boxed()
            }
            Self::Terminal => {
                /// Create a terminal-friendly formatted layer
                let layer = tracing_subscriber::fmt::layer().with_ansi(ansi).with_target(target);

                /// Use the provided file_writer if available
                if let Some(writer) = file_writer {
                    layer.with_writer(writer).with_filter(filter).boxed()
                } else {
                    layer.with_filter(filter).boxed()
                }
            }
        }
    }
}


/// LogFormat Display Implementation 
/// This section implements the Display trait for the LogFormat enum,
/// allowing instances of LogFormat to be formatted as strings.

impl Display for LogFormat {
    /// Formats the LogFormat as a string.
    ///
    /// This method matches the LogFormat variant and writes a corresponding
    /// string representation to the provided formatter.
    ///
    /// # Arguments
    /// * `f` - A mutable reference to a fmt::Formatter, used for writing the formatted string.
    ///
    /// # Returns
    /// A fmt::Result indicating success or failure.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Match the LogFormat variant and write the corresponding string representation.
        match self {
            Self::Json => write!(f, "json"),
            Self::LogFmt => write!(f, "logfmt"),
            Self::Terminal => write!(f, "terminal"),
        }
    }
}
