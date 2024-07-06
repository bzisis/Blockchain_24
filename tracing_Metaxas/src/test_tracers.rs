/// Import WorkerGuard from the tracing_appender::non_blocking module
use tracing_appender::non_blocking::WorkerGuard;

/// Import EnvFilter from the tracing_subscriber module
use tracing_subscriber::EnvFilter;

/// Import the Tracer trait from the current crate
use crate::Tracer;

/// Initializes a tracing subscriber specifically tailored for testing purposes.
///
/// The subscriber's filter level is configurable via the `RUST_LOG` environment variable.
///
/// # Note
///
/// If initialization fails, the subscriber will silently fail, resulting in no logging output.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct TestTracer;

impl Tracer for TestTracer {
    /// Initializes the tracing subscriber for testing.
    ///
    /// # Returns
    ///
    /// Returns an `Ok(None)` indicating successful initialization with no associated `WorkerGuard`.
    /// If initialization fails, an error is returned.
    fn init(self) -> eyre::Result<Option<WorkerGuard>> {
        // Attempt to initialize a basic tracing subscriber with formatted output to stderr.
        let _ = tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env()) // Configures filter level from RUST_LOG
            .with_writer(std::io::stderr) // Directs output to stderr
            .try_init(); // Tries to initialize the subscriber

        Ok(None) // Return Ok(None) as there's no associated WorkerGuard for this test setup
    }
}
