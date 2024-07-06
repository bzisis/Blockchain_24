
//!  The `tracing` module provides functionalities for setting up and configuring logging.
//!
//!  It includes structures and functions to create and manage various logging layers: stdout,
//!  file, or journald. The module's primary entry point is the `Tracer` struct, which can be
//!  configured to use different logging formats and destinations. If no layer is specified, it will
//!  default to stdout.
//!
//!  # Examples
//!
//!  Basic usage:
//!
//!  ```
//!  use reth_tracing::{
//!      LayerInfo, RethTracer, Tracer,
//!      tracing::level_filters::LevelFilter,
//!      LogFormat,
//!  };
//!
//!  fn main() -> eyre::Result<()> {
//!      let tracer = RethTracer::new().with_stdout(LayerInfo::new(
//!          LogFormat::Json,
//!          LevelFilter::INFO.to_string(),
//!          "debug".to_string(),
//!          None,
//!      ));
//!
//!      tracer.init()?;
//!
//!      // Your application logic here
//!
//!      Ok(())
//!  }
//!  ```
//!
//!  This example sets up a tracer with JSON format logging for journald and terminal-friendly
//! format  for file logging.


/// Configures the Rust documentation (rustdoc) generation for this crate.
///
/// Sets the URL for the logo that will appear in the generated documentation,
/// the URL for the favicon, and the base URL for the issue tracker.
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]

/// Conditionally enables warnings for unused crate dependencies, except in test code.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

/// Conditionally enables features for documenting configuration predicates on docs.rs.
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]


/// Re-export of the `tracing` crate.
pub use tracing;

/// Re-export of the `tracing_subscriber` crate.
pub use tracing_subscriber;


/// Re-export LogFormat
pub use formatter::LogFormat;

/// Re-export FileInfo and FileWorkerGuard from the layers module
pub use layers::{FileInfo, FileWorkerGuard};

/// Re-export TestTracer from the test_tracer module
pub use test_tracer::TestTracer;

/// Internal modules
mod formatter;
mod layers;
mod test_tracer;

/// External crates
use crate::layers::Layers;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


/// Tracer for application logging.
///
/// Manages the configuration and initialization of logging layers,
/// including standard output, optional journald, and optional file logging.
#[derive(Debug, Clone)]
pub struct RethTracer {
    /// Configuration for the stdout logging layer.
    stdout: LayerInfo,

    /// Optional filter for the journald logging layer.
    journald: Option<String>,

    /// Optional configuration for the file logging layer, including file information.
    file: Option<(LayerInfo, FileInfo)>,
}


impl RethTracer {
    /// Constructs a new `RethTracer` with default settings.
    ///
    /// Initializes with default stdout layer configuration.
    /// Journald and file layers are not set by default.
    pub fn new() -> Self {
        Self {
            stdout: LayerInfo::default(),
            journald: None,
            file: None,
        }
    }

    /// Sets a custom configuration for the stdout layer.
    ///
    /// # Arguments
    /// * `config` - The `LayerInfo` to use for the stdout layer.
    ///
    /// # Returns
    /// A new `RethTracer` instance with the updated stdout configuration.
    pub fn with_stdout(mut self, config: LayerInfo) -> Self {
        self.stdout = config;
        self
    }

    /// Sets the journald layer filter.
    ///
    /// # Arguments
    /// * `filter` - The filter to use for the journald layer.
    ///
    /// # Returns
    /// A new `RethTracer` instance with the updated journald filter.
    pub fn with_journald(mut self, filter: String) -> Self {
        self.journald = Some(filter);
        self
    }

    /// Sets the file layer configuration and associated file info.
    ///
    /// # Arguments
    /// * `config` - The `LayerInfo` to use for the file layer.
    /// * `file_info` - The `FileInfo` containing details about the log file.
    ///
    /// # Returns
    /// A new `RethTracer` instance with the updated file layer configuration.
    pub fn with_file(mut self, config: LayerInfo, file_info: FileInfo) -> Self {
        self.file = Some((config, file_info));
        self
    }
}


impl Default for RethTracer {
    /// Returns the default `RethTracer` instance.
    ///
    /// This function creates a new `RethTracer` with default settings,
    /// initializing with default stdout layer configuration.
    /// Journald and file layers are not set by default.
    fn default() -> Self {
        Self::new()
    }
}


/// Configuration for a logging layer.
///
/// This struct holds configuration parameters for a tracing layer, including
/// the format, filtering directives, optional coloring, and directive.
#[derive(Debug, Clone)]
pub struct LayerInfo {
    /// The format for log messages.
    pub format: LogFormat,

    /// Directive for filtering log messages.
    pub default_directive: String,

    /// Additional filtering parameters as a string.
    pub filters: String,

    /// Optional color configuration for the log messages.
    pub color: Option<String>,
}


impl LayerInfo {
    /// Constructs a new `LayerInfo` instance with the specified configuration.
    ///
    /// # Arguments
    /// * `format` - Specifies the format for log messages. Possible values are:
    ///   - `LogFormat::Json` for JSON formatting.
    ///   - `LogFormat::LogFmt` for logfmt (key=value) formatting.
    ///   - `LogFormat::Terminal` for human-readable, terminal-friendly formatting.
    /// * `default_directive` - Directive for filtering log messages.
    /// * `filters` - Additional filtering parameters as a string.
    /// * `color` - Optional color configuration for the log messages.
    pub const fn new(
        format: LogFormat,
        default_directive: String,
        filters: String,
        color: Option<String>,
    ) -> Self {
        Self { format, default_directive, filters, color }
    }
}


impl Default for LayerInfo {
    /// Provides default values for `LayerInfo`.
    ///
    /// By default, it uses the terminal log format, sets the default logging
    /// directive to INFO level, has no additional filters, and enables color
    /// configuration with ANSI colors enabled by default.
    fn default() -> Self {
        Self {
            format: LogFormat::Terminal,
            default_directive: LevelFilter::INFO.to_string(),
            filters: String::new(),
            color: Some("always".to_string()),
        }
    }
}


/// Trait defining a general interface for logging configuration.
///
/// The `Tracer` trait provides a standardized way to initialize logging configurations
/// in an application. Implementations of this trait can specify different logging setups,
/// such as standard output logging, file logging, journald logging, or custom logging
/// configurations tailored for specific environments (like testing).
pub trait Tracer {
    /// Initialize the logging configuration.
    ///
    /// # Returns
    /// An `eyre::Result` which is `Ok` with an optional `WorkerGuard` if a file layer is used,
    /// or an `Err` in case of an error during initialization.
    fn init(self) -> eyre::Result<Option<WorkerGuard>>;
}


impl Tracer for RethTracer {
    /// Initializes the logging system based on the configured layers.
    ///
    /// This method sets up the global tracing subscriber with the specified
    /// stdout, journald, and file layers.
    ///
    /// The default layer is stdout.
    ///
    /// # Returns
    /// An `eyre::Result` which is `Ok` with an optional `WorkerGuard` if a file layer is used,
    /// or an `Err` in case of an error during initialization.
    fn init(self) -> eyre::Result<Option<WorkerGuard>> {
        /// Create a new Layers instance to manage logging layers
        let mut layers = Layers::new();

        /// Add stdout layer with specified format, default directive, filters, and color
        layers.stdout(
            self.stdout.format,                     /// Log format (Json, LogFmt, Terminal)
            self.stdout.default_directive.parse()?, /// Default directive for filtering
            &self.stdout.filters,                   /// Additional filter directives
            self.stdout.color,                      /// Optional color configuration
        )?;

        /// Optionally add journald layer with the specified filter
        if let Some(config) = self.journald {
            layers.journald(&config)?;
        }

        /// Optionally add file logging layer with specified format, filters, and file info
        let file_guard = if let Some((config, file_info)) = self.file {
            Some(layers.file(config.format, &config.filters, file_info)?)
        } else {
            None
        };

        /// Attempt to initialize the tracing subscriber with the configured layers
        /// Ignore the error if the global default subscriber is already set
        let _ = tracing_subscriber::registry().with(layers.into_inner()).try_init();

        /// Return the file guard if file logging was enabled
        Ok(file_guard)
    }
}


/// Initializes a tracing subscriber for tests.
///
/// The filter is configurable via `RUST_LOG`.
///
/// # Note
///
/// The subscriber will silently fail if it could not be installed.
pub fn init_test_tracing() {
    /// Initialize a default TestTracer and attempt to initialize the tracing subscriber
    let _ = TestTracer::default().init();
}
