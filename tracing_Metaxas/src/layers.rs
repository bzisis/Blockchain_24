/// Imports and Dependencies
/// This section includes further necessary imports and dependencies for logging, file handling,
/// and setting up the tracing subscriber.

/// Importing Path and PathBuf from the standard library for file system path manipulation.
use std::path::{Path, PathBuf};

/// Importing RollingConditionBasic and RollingFileAppender from the rolling_file crate for log file rotation.
use rolling_file::{RollingConditionBasic, RollingFileAppender};

/// Importing WorkerGuard from the tracing_appender crate to ensure logs are flushed on shutdown.
use tracing_appender::non_blocking::WorkerGuard;

/// Importing additional components from the tracing_subscriber crate for configuring the tracing subscriber.
use tracing_subscriber::{filter::Directive, EnvFilter, Layer, Registry};

/// Importing the LogFormat enum from the crate's formatter module.
use crate::formatter::LogFormat;



/// This section defines a type alias for a worker guard used in file logging.
/// A worker guard returned by the file layer.
///
/// When a guard is dropped, all events currently in-memory are flushed to the log file this guard
/// belongs to.
pub type FileWorkerGuard = tracing_appender::non_blocking::WorkerGuard;


/// This section defines a type alias for a boxed tracing layer, making it easier to handle layers in a dynamic context.
/// A boxed tracing [Layer].
///
/// This type alias represents a dynamically dispatched tracing layer
/// that is both `Send` and `Sync`, allowing it to be used in concurrent contexts.
/// The `S` generic parameter represents the subscriber type the layer is composed with.
pub(crate) type BoxedLayer<S> = Box<dyn Layer<S> + Send + Sync>;


/// Default log file name for the application.
///
/// This constant specifies the default file name used for logging application logs.
pub const RETH_LOG_FILE_NAME: &str = "reth.log";


/// Default EnvFilter Directives
/// This section defines a constant array of default directives for the EnvFilter, 
/// which disables high-frequency debug logs from certain libraries.

/// Default [directives](Directive) for [`EnvFilter`] which disables high-frequency debug logs from
/// `hyper`, `trust-dns`, `jsonrpsee-server`, and `discv5`.
///
/// This constant specifies an array of directives to configure the [`EnvFilter`].
/// It disables debug logs for high-frequency loggers to prevent log flooding.
const DEFAULT_ENV_FILTER_DIRECTIVES: [&str; 5] = [
    "hyper::proto::h1=off",       /// Disable debug logs from the HTTP/1 protocol implementation in hyper.
    "trust_dns_proto=off",        /// Disable debug logs from the trust-dns protocol implementation.
    "trust_dns_resolver=off",     /// Disable debug logs from the trust-dns resolver.
    "discv5=off",                 /// Disable debug logs from the discv5 protocol.
    "jsonrpsee-server=off",       // Disable debug logs from the jsonrpsee server.
];



/// Manages the collection of layers for a tracing subscriber.
///
/// `Layers` acts as a container for different logging layers such as stdout, file, or journald.
/// Each layer can be configured separately and then combined into a tracing subscriber.
pub(crate) struct Layers {
    inner: Vec<BoxedLayer<Registry>>,
}

impl Layers {
    /// Creates a new `Layers` instance.
    pub(crate) fn new() -> Self {
        Self { inner: vec![] }
    }

    /// Consumes the `Layers` instance, returning the inner vector of layers.
    pub(crate) fn into_inner(self) -> Vec<BoxedLayer<Registry>> {
        self.inner
    }

    /// Adds a journald layer to the layers collection.
    ///
    /// # Arguments
    /// * `filter` - A string containing additional filter directives for this layer.
    ///
    /// # Returns
    /// An `eyre::Result<()>` indicating the success or failure of the operation.
    pub(crate) fn journald(&mut self, filter: &str) -> eyre::Result<()> {
        let journald_filter = build_env_filter(None, filter)?;
        let layer = tracing_journald::layer()?.with_filter(journald_filter).boxed();
        self.inner.push(layer);
        Ok(())
    }

    /// Adds a stdout layer with specified formatting and filtering.
    ///
    /// # Type Parameters
    /// * `S` - The type of subscriber that will use these layers.
    ///
    /// # Arguments
    /// * `format` - The log message format.
    /// * `directive` - Directive for the default logging level.
    /// * `filter` - Additional filter directives as a string.
    /// * `color` - Optional color configuration for the log messages.
    ///
    /// # Returns
    /// An `eyre::Result<()>` indicating the success or failure of the operation.
    pub(crate) fn stdout(
        &mut self,
        format: LogFormat,
        default_directive: Directive,
        filters: &str,
        color: Option<String>,
    ) -> eyre::Result<()> {
        let filter = build_env_filter(Some(default_directive), filters)?;
        let layer = format.apply(filter, color, None);
        self.inner.push(layer.boxed());
        Ok(())
    }

    /// Adds a file logging layer to the layers collection.
    ///
    /// # Arguments
    /// * `format` - The format for log messages.
    /// * `filter` - Additional filter directives as a string.
    /// * `file_info` - Information about the log file including path and rotation strategy.
    ///
    /// # Returns
    /// An `eyre::Result<FileWorkerGuard>` representing the file logging worker.
    pub(crate) fn file(
        &mut self,
        format: LogFormat,
        filter: &str,
        file_info: FileInfo,
    ) -> eyre::Result<FileWorkerGuard> {
        let (writer, guard) = file_info.create_log_writer();
        let file_filter = build_env_filter(None, filter)?;
        let layer = format.apply(file_filter, None, Some(writer));
        self.inner.push(layer);
        Ok(guard)
    }
}

/// FileInfo Struct Definition
/// This section defines the `FileInfo` struct, which holds configuration information for file logging.

/// Holds configuration information for file logging.
///
/// Contains details about the log file's path, name, size, and rotation strategy.
#[derive(Debug, Clone)]
pub struct FileInfo {
    dir: PathBuf,           /// Directory where the log files will be stored.
    file_name: String,      /// Name of the log file.
    max_size_bytes: u64,    /// Maximum size of the log file in bytes before it gets rotated.
    max_files: usize,       /// Maximum number of rotated log files to keep.
}


/// FileInfo Implementation 
/// This section implements methods for the `FileInfo` struct, providing functionality for file logging configuration.

impl FileInfo {
    /// Creates a new `FileInfo` instance.
    ///
    /// # Arguments
    /// * `dir` - The directory where log files will be stored.
    /// * `max_size_bytes` - The maximum size of the log file in bytes before rotation.
    /// * `max_files` - The maximum number of rotated log files to keep.
    pub fn new(dir: PathBuf, max_size_bytes: u64, max_files: usize) -> Self {
        Self { 
            dir, 
            file_name: RETH_LOG_FILE_NAME.to_string(), // Default log file name
            max_size_bytes, 
            max_files 
        }
    }

    /// Creates the log directory if it doesn't exist.
    ///
    /// Ensures that the directory specified in `dir` exists, creating it if necessary.
    ///
    /// # Returns
    /// A reference to the path of the log directory.
    fn create_log_dir(&self) -> &Path {
        let log_dir: &Path = self.dir.as_ref();
        if !log_dir.exists() {
            std::fs::create_dir_all(log_dir).expect("Could not create log directory");
        }
        log_dir
    }

    /// Creates a non-blocking writer for the log file.
    ///
    /// Initializes a non-blocking log writer with file rotation based on size.
    ///
    /// # Returns
    /// A tuple containing the non-blocking writer and its associated worker guard.
    fn create_log_writer(&self) -> (tracing_appender::non_blocking::NonBlocking, WorkerGuard) {
        let log_dir = self.create_log_dir();
        let (writer, guard) = tracing_appender::non_blocking(
            RollingFileAppender::new(
                log_dir.join(&self.file_name),                  // Path to the log file
                RollingConditionBasic::new().max_size(self.max_size_bytes), // Rotation condition based on file size
                self.max_files,                                // Number of rotated files to keep
            ).expect("Could not initialize file logging"),
        );
        (writer, guard)
    }
}


/// Build Environment Filter
/// This function creates an `EnvFilter` for logging, combining default and additional directives.

/// Builds an environment filter for logging.
///
/// The events are filtered by `default_directive`, unless overridden by `RUST_LOG`.
///
/// # Arguments
/// * `default_directive` - An optional `Directive` that sets the default directive.
/// * `directives` - Additional directives as a comma-separated string.
///
/// # Returns
/// An `eyre::Result<EnvFilter>` that can be used to configure a tracing subscriber.
fn build_env_filter(
    default_directive: Option<Directive>,
    directives: &str,
) -> eyre::Result<EnvFilter> {
    // Initialize the EnvFilter builder with the optional default directive
    let env_filter = if let Some(default_directive) = default_directive {
        EnvFilter::builder().with_default_directive(default_directive).from_env_lossy()
    } else {
        EnvFilter::builder().from_env_lossy()
    };

    // Combine the default directives with additional directives
    DEFAULT_ENV_FILTER_DIRECTIVES
        .into_iter()  // Iterate over the default directives
        .chain(directives.split(',').filter(|d| !d.is_empty()))  // Add additional directives, filtering out empty strings
        .try_fold(env_filter, |env_filter, directive| {
            // Parse and add each directive to the EnvFilter
            Ok(env_filter.add_directive(directive.parse()?))
        })
}
