// Moysis Moysis Volos, Greece 29/06/2024.

// Importing necessary modules and types for metric recording and testing
// `metrics`: Provides types and functions for metric recording and description
// `once_cell::sync::Lazy`: For lazily initializing static variables
// `reth_metrics_derive::Metrics`: Derive macro for generating metric-related code
// `serial_test::serial`: Ensures tests are run serially to prevent interference
// `std::collections::HashMap`: For storing metrics in a hash map
// `std::sync::Mutex`: For thread-safe mutable access to metrics
use metrics::{
    Counter, Gauge, Histogram, Key, KeyName, Label, Metadata, Recorder, SharedString, Unit,
};
use once_cell::sync::Lazy;
use reth_metrics_derive::Metrics;
use serial_test::serial;
use std::{collections::HashMap, sync::Mutex};

#[allow(dead_code)]
#[derive(Metrics)]
#[metrics(scope = "metrics_custom")]
/// A struct representing custom metrics with various field types.
/// Some fields are skipped, and some have custom descriptions or renames.
struct CustomMetrics {
    #[metric(skip)]
    skipped_field_a: u8, // This field is skipped and not included in the metrics.

    /// A gauge with doc comment description.
    gauge: Gauge, // This field is a Gauge metric with a description from the doc comment.

    #[metric(rename = "second_gauge", describe = "A gauge with metric attribute description.")]
    gauge2: Gauge, // This field is a Gauge metric with a custom name and description.

    #[metric(skip)]
    skipped_field_b: u16, // This field is skipped and not included in the metrics.

    /// Some doc comment
    #[metric(describe = "Metric attribute description will be preferred over doc comment.")]
    counter: Counter, // This field is a Counter metric with a description from the attribute, which takes precedence over the doc comment.

    #[metric(skip)]
    skipped_field_c: u32, // This field is skipped and not included in the metrics.

    #[metric(skip)]
    skipped_field_d: u64, // This field is skipped and not included in the metrics.

    /// A renamed histogram.
    #[metric(rename = "histogram")]
    histo: Histogram, // This field is a Histogram metric with a custom name.

    #[metric(skip)]
    skipped_field_e: u128, // This field is skipped and not included in the metrics.
}

#[allow(dead_code)]
#[derive(Metrics)]
#[metrics(dynamic = true)]
/// A struct representing metrics with a dynamic scope.
/// Some fields are skipped, and some have custom descriptions or renames.
struct DynamicScopeMetrics {
    #[metric(skip)]
    skipped_field_a: u8, // This field is skipped and not included in the metrics.

    /// A gauge with doc comment description.
    gauge: Gauge, // This field is a Gauge metric with a description from the doc comment.

    #[metric(rename = "second_gauge", describe = "A gauge with metric attribute description.")]
    gauge2: Gauge, // This field is a Gauge metric with a custom name and description.

    #[metric(skip)]
    skipped_field_b: u16, // This field is skipped and not included in the metrics.

    /// Some doc comment
    #[metric(describe = "Metric attribute description will be preferred over doc comment.")]
    counter: Counter, // This field is a Counter metric with a description from the attribute, which takes precedence over the doc comment.

    #[metric(skip)]
    skipped_field_c: u32, // This field is skipped and not included in the metrics.

    #[metric(skip)]
    skipped_field_d: u64, // This field is skipped and not included in the metrics.

    /// A renamed histogram.
    #[metric(rename = "histogram")]
    histo: Histogram, // This field is a Histogram metric with a custom name.

    #[metric(skip)]
    skipped_field_e: u128, // This field is skipped and not included in the metrics.
}

// A lazily initialized global test recorder.
static RECORDER: Lazy<TestRecorder> = Lazy::new(TestRecorder::new);

/// Tests the describe functionality of the metrics.
///
/// This function checks that the expected metrics are correctly described with the
/// appropriate types and descriptions.
///
/// # Arguments
///
/// * `scope` - A string slice representing the scope of the metrics to test.
fn test_describe(scope: &str) {
    // Ensure the number of metrics recorded matches the expected count.
    assert_eq!(RECORDER.metrics_len(), 4);

    // Check the description and type of the "gauge" metric.
    let gauge = RECORDER.get_metric(&format!("{scope}.gauge"));
    assert!(gauge.is_some());
    assert_eq!(
        gauge.unwrap(),
        TestMetric {
            ty: TestMetricTy::Gauge,
            description: Some("A gauge with doc comment description.".to_owned()),
            labels: None,
        }
    );

    // Check the description and type of the "second_gauge" metric.
    let second_gauge = RECORDER.get_metric(&format!("{scope}.second_gauge"));
    assert!(second_gauge.is_some());
    assert_eq!(
        second_gauge.unwrap(),
        TestMetric {
            ty: TestMetricTy::Gauge,
            description: Some("A gauge with metric attribute description.".to_owned()),
            labels: None,
        }
    );

    // Check the description and type of the "counter" metric.
    let counter = RECORDER.get_metric(&format!("{scope}.counter"));
    assert!(counter.is_some());
    assert_eq!(
        counter.unwrap(),
        TestMetric {
            ty: TestMetricTy::Counter,
            description: Some(
                "Metric attribute description will be preferred over doc comment.".to_owned()
            ),
            labels: None,
        }
    );

    // Check the description and type of the "histogram" metric.
    let histogram = RECORDER.get_metric(&format!("{scope}.histogram"));
    assert!(histogram.is_some());
    assert_eq!(
        histogram.unwrap(),
        TestMetric {
            ty: TestMetricTy::Histogram,
            description: Some("A renamed histogram.".to_owned()),
            labels: None,
        }
    );
}

/// Test to describe custom metrics.
///
/// This test sets up the global recorder, describes the `CustomMetrics` metrics, and
/// then verifies that the metrics are described correctly using the `test_describe` function.
#[test]
#[serial] // Ensures that this test runs serially to prevent interference with other tests.
fn describe_metrics() {
    // Enter the global recorder and set it for the duration of the test.
    let _guard = RECORDER.enter();

    // Describe the metrics for the `CustomMetrics` struct.
    CustomMetrics::describe();

    // Verify that the metrics are described correctly with the expected scope.
    test_describe("metrics_custom");
}

/// Test to describe dynamic scope metrics.
///
/// This test sets up the global recorder, describes the `DynamicScopeMetrics` metrics with
/// a dynamic scope, and then verifies that the metrics are described correctly using the `test_describe` function.
#[test]
#[serial] // Ensures that this test runs serially to prevent interference with other tests.
fn describe_dynamic_metrics() {
    // Enter the global recorder and set it for the duration of the test.
    let _guard = RECORDER.enter();

    // Define the dynamic scope for the metrics.
    let scope = "local_scope";

    // Describe the metrics for the `DynamicScopeMetrics` struct with the dynamic scope.
    DynamicScopeMetrics::describe(scope);

    // Verify that the metrics are described correctly with the specified dynamic scope.
    test_describe(scope);
}

/// Tests the registration of metrics.
///
/// This function checks that the expected metrics are correctly registered with the
/// appropriate types. It verifies that the metrics exist and their types match the expected values.
///
/// # Arguments
///
/// * `scope` - A string slice representing the scope of the metrics to test.
fn test_register(scope: &str) {
    // Ensure the number of metrics recorded matches the expected count.
    assert_eq!(RECORDER.metrics_len(), 4);

    // Check the registration and type of the "gauge" metric.
    let gauge = RECORDER.get_metric(&format!("{scope}.gauge"));
    assert!(gauge.is_some());
    assert_eq!(
        gauge.unwrap(),
        TestMetric { ty: TestMetricTy::Gauge, description: None, labels: None }
    );

    // Check the registration and type of the "second_gauge" metric.
    let second_gauge = RECORDER.get_metric(&format!("{scope}.second_gauge"));
    assert!(second_gauge.is_some());
    assert_eq!(
        second_gauge.unwrap(),
        TestMetric { ty: TestMetricTy::Gauge, description: None, labels: None }
    );

    // Check the registration and type of the "counter" metric.
    let counter = RECORDER.get_metric(&format!("{scope}.counter"));
    assert!(counter.is_some());
    assert_eq!(
        counter.unwrap(),
        TestMetric { ty: TestMetricTy::Counter, description: None, labels: None }
    );

    // Check the registration and type of the "histogram" metric.
    let histogram = RECORDER.get_metric(&format!("{scope}.histogram"));
    assert!(histogram.is_some());
    assert_eq!(
        histogram.unwrap(),
        TestMetric { ty: TestMetricTy::Histogram, description: None, labels: None }
    );
}

/// Test to register custom metrics.
///
/// This test sets up the global recorder, registers the `CustomMetrics` metrics,
/// and then verifies that the metrics are registered correctly using the `test_register` function.
#[test]
#[serial] // Ensures that this test runs serially to prevent interference with other tests.
fn register_metrics() {
    // Enter the global recorder and set it for the duration of the test.
    let _guard = RECORDER.enter();

    // Register the metrics for the `CustomMetrics` struct by creating a default instance.
    let _metrics = CustomMetrics::default();

    // Verify that the metrics are registered correctly with the expected scope.
    test_register("metrics_custom");
}

/// Test to register dynamic scope metrics.
///
/// This test sets up the global recorder, registers the `DynamicScopeMetrics` metrics
/// with a dynamic scope, and then verifies that the metrics are registered correctly using the `test_register` function.
#[test]
#[serial] // Ensures that this test runs serially to prevent interference with other tests.
fn register_dynamic_metrics() {
    // Enter the global recorder and set it for the duration of the test.
    let _guard = RECORDER.enter();

    // Define the dynamic scope for the metrics.
    let scope = "local_scope";

    // Register the metrics for the `DynamicScopeMetrics` struct by creating an instance with the dynamic scope.
    let _metrics = DynamicScopeMetrics::new(scope);

    // Verify that the metrics are registered correctly with the specified dynamic scope.
    test_register(scope);
}

/// Tests the functionality of metrics with labels.
///
/// This function checks that the expected metrics are correctly registered with the
/// appropriate labels. It verifies that the metrics exist and their labels match the expected values.
///
/// # Arguments
///
/// * `scope` - A string slice representing the scope of the metrics to test.
fn test_labels(scope: &str) {
    // Define the expected labels for the metrics.
    let test_labels = vec![Label::new("key", "value")];

    // Check the labels of the "gauge" metric.
    let gauge = RECORDER.get_metric(&format!("{scope}.gauge"));
    assert!(gauge.is_some());
    let labels = gauge.unwrap().labels;
    assert!(labels.is_some());
    assert_eq!(labels.unwrap(), test_labels);

    // Check the labels of the "second_gauge" metric.
    let second_gauge = RECORDER.get_metric(&format!("{scope}.second_gauge"));
    assert!(second_gauge.is_some());
    let labels = second_gauge.unwrap().labels;
    assert!(labels.is_some());
    assert_eq!(labels.unwrap(), test_labels);

    // Check the labels of the "counter" metric.
    let counter = RECORDER.get_metric(&format!("{scope}.counter"));
    assert!(counter.is_some());
    let labels = counter.unwrap().labels;
    assert!(labels.is_some());
    assert_eq!(labels.unwrap(), test_labels);

    // Check the labels of the "histogram" metric.
    let histogram = RECORDER.get_metric(&format!("{scope}.histogram"));
    assert!(histogram.is_some());
    let labels = histogram.unwrap().labels;
    assert!(labels.is_some());
    assert_eq!(labels.unwrap(), test_labels);
}

/// Test to validate custom metrics with labels.
///
/// This test sets up the global recorder, registers the `CustomMetrics` metrics with labels,
/// and then verifies that the metrics are registered correctly with the expected labels using the `test_labels` function.
#[test]
#[serial] // Ensures that this test runs serially to prevent interference with other tests.
fn label_metrics() {
    // Enter the global recorder and set it for the duration of the test.
    let _guard = RECORDER.enter();

    // Register the metrics for the `CustomMetrics` struct with labels.
    let _metrics = CustomMetrics::new_with_labels(&[("key", "value")]);

    // Verify that the metrics are registered correctly with the expected labels.
    test_labels("metrics_custom");
}

/// Test to validate dynamic scope metrics with labels.
///
/// This test sets up the global recorder, registers the `DynamicScopeMetrics` metrics with a dynamic scope and labels,
/// and then verifies that the metrics are registered correctly with the expected labels using the `test_labels` function.
#[test]
#[serial] // Ensures that this test runs serially to prevent interference with other tests.
fn dynamic_label_metrics() {
    // Enter the global recorder and set it for the duration of the test.
    let _guard = RECORDER.enter();

    // Define the dynamic scope for the metrics.
    let scope = "local_scope";

    // Register the metrics for the `DynamicScopeMetrics` struct with the dynamic scope and labels.
    let _metrics = DynamicScopeMetrics::new_with_labels(scope, &[("key", "value")]);

    // Verify that the metrics are registered correctly with the expected labels.
    test_labels(scope);
}

/// A test recorder for verifying metrics.
///
/// This struct holds a collection of recorded metrics and provides methods for managing
/// and querying these metrics during tests.
struct TestRecorder {
    /// A map of metric keys to their associated `TestMetric` details.
    ///
    /// The key is a string representing the metric name, and the value is a `TestMetric` struct
    /// containing the type, description, and labels of the metric.
    metrics: Mutex<HashMap<String, TestMetric>>,
}

/// An enumeration representing the type of a test metric.
///
/// This enum is used to differentiate between different types of metrics, such as counters, gauges, and histograms.
#[derive(PartialEq, Clone, Debug)]
enum TestMetricTy {
    /// A counter metric, which typically counts occurrences of an event.
    Counter,
    /// A gauge metric, which represents a value that can go up and down.
    Gauge,
    /// A histogram metric, which samples observations and counts them in configurable buckets.
    Histogram,
}

/// A struct representing a test metric with its type, description, and labels.
///
/// This struct is used to store detailed information about a metric, including its type,
/// an optional description, and optional labels.
#[derive(PartialEq, Clone, Debug)]
struct TestMetric {
    /// The type of the metric (e.g., counter, gauge, histogram).
    ty: TestMetricTy,
    /// An optional description of the metric.
    description: Option<String>,
    /// An optional vector of labels associated with the metric.
    labels: Option<Vec<Label>>,
}

impl TestRecorder {
    /// Creates a new `TestRecorder` instance.
    ///
    /// This initializes the `metrics` field with an empty `HashMap` wrapped in a `Mutex`.
    fn new() -> Self {
        Self { metrics: Mutex::new(HashMap::default()) }
    }

    /// Sets this recorder as the global recorder for the duration of the returned guard.
    ///
    /// This method installs the recorder as the global metrics recorder and returns a guard
    /// that clears the metrics when dropped.
    #[must_use]
    fn enter(&'static self) -> impl Drop {
        // Define a struct for resetting the recorder.
        struct Reset {
            recorder: &'static TestRecorder,
        }

        // Implement the `Drop` trait to clear the metrics when the guard is dropped.
        impl Drop for Reset {
            fn drop(&mut self) {
                self.recorder.clear();
            }
        }

        // Set this recorder as the global recorder.
        let _ = metrics::set_global_recorder(self);

        // Return the reset guard.
        Reset { recorder: self }
    }

    /// Returns the number of metrics recorded.
    ///
    /// This method locks the `metrics` mutex and returns the length of the `HashMap`.
    fn metrics_len(&self) -> usize {
        self.metrics.lock().expect("failed to lock metrics").len()
    }

    /// Retrieves a metric by its key.
    ///
    /// This method locks the `metrics` mutex and returns a cloned `TestMetric` if found.
    ///
    /// # Arguments
    /// * `key` - A string slice representing the key of the metric to retrieve.
    ///
    /// # Returns
    /// An `Option` containing the `TestMetric` if found, or `None` if not found.
    fn get_metric(&self, key: &str) -> Option<TestMetric> {
        self.metrics.lock().expect("failed to lock metrics").get(key).cloned()
    }

    /// Records a metric with its type, description, and labels.
    ///
    /// This method locks the `metrics` mutex and inserts a new `TestMetric` into the `HashMap`.
    ///
    /// # Arguments
    /// * `key` - A string slice representing the key of the metric.
    /// * `ty` - The type of the metric.
    /// * `description` - An optional description of the metric.
    /// * `labels` - An optional vector of labels associated with the metric.
    fn record_metric(
        &self,
        key: &str,
        ty: TestMetricTy,
        description: Option<String>,
        labels: Option<Vec<Label>>,
    ) {
        self.metrics
            .lock()
            .expect("failed to lock metrics")
            .insert(key.to_owned(), TestMetric { ty, description, labels });
    }

    /// Clears all recorded metrics.
    ///
    /// This method locks the `metrics` mutex and clears the `HashMap`.
    fn clear(&self) {
        self.metrics.lock().expect("failed to lock metrics").clear();
    }
}


impl Recorder for &'static TestRecorder {
    /// Describes a counter metric.
    ///
    /// This method records the description of a counter metric.
    ///
    /// # Arguments
    /// * `key` - The name of the metric.
    /// * `_unit` - The unit of the metric (unused).
    /// * `description` - The description of the metric.
    fn describe_counter(&self, key: KeyName, _unit: Option<Unit>, description: SharedString) {
        self.record_metric(
            key.as_str(),
            TestMetricTy::Counter,
            Some(description.into_owned()),
            None,
        )
    }

    /// Describes a gauge metric.
    ///
    /// This method records the description of a gauge metric.
    ///
    /// # Arguments
    /// * `key` - The name of the metric.
    /// * `_unit` - The unit of the metric (unused).
    /// * `description` - The description of the metric.
    fn describe_gauge(&self, key: KeyName, _unit: Option<Unit>, description: SharedString) {
        self.record_metric(key.as_str(), TestMetricTy::Gauge, Some(description.into_owned()), None)
    }

    /// Describes a histogram metric.
    ///
    /// This method records the description of a histogram metric.
    ///
    /// # Arguments
    /// * `key` - The name of the metric.
    /// * `_unit` - The unit of the metric (unused).
    /// * `description` - The description of the metric.
    fn describe_histogram(&self, key: KeyName, _unit: Option<Unit>, description: SharedString) {
        self.record_metric(
            key.as_str(),
            TestMetricTy::Histogram,
            Some(description.into_owned()),
            None,
        )
    }

    /// Registers a counter metric.
    ///
    /// This method records a counter metric with optional labels.
    ///
    /// # Arguments
    /// * `key` - The key of the metric.
    /// * `_metadata` - The metadata of the metric (unused).
    ///
    /// # Returns
    /// A `Counter` instance that does nothing.
    fn register_counter(&self, key: &Key, _metadata: &Metadata<'_>) -> Counter {
        let labels_vec: Vec<Label> = key.labels().cloned().collect();
        let labels = (!labels_vec.is_empty()).then_some(labels_vec);
        self.record_metric(key.name(), TestMetricTy::Counter, None, labels);
        Counter::noop()
    }

    /// Registers a gauge metric.
    ///
    /// This method records a gauge metric with optional labels.
    ///
    /// # Arguments
    /// * `key` - The key of the metric.
    /// * `_metadata` - The metadata of the metric (unused).
    ///
    /// # Returns
    /// A `Gauge` instance that does nothing.
    fn register_gauge(&self, key: &Key, _metadata: &Metadata<'_>) -> Gauge {
        let labels_vec: Vec<Label> = key.labels().cloned().collect();
        let labels = (!labels_vec.is_empty()).then_some(labels_vec);
        self.record_metric(key.name(), TestMetricTy::Gauge, None, labels);
        Gauge::noop()
    }

    /// Registers a histogram metric.
    ///
    /// This method records a histogram metric with optional labels.
    ///
    /// # Arguments
    /// * `key` - The key of the metric.
    /// * `_metadata` - The metadata of the metric (unused).
    ///
    /// # Returns
    /// A `Histogram` instance that does nothing.
    fn register_histogram(&self, key: &Key, _metadata: &Metadata<'_>) -> Histogram {
        let labels_vec: Vec<Label> = key.labels().cloned().collect();
        let labels = (!labels_vec.is_empty()).then_some(labels_vec);
        self.record_metric(key.name(), TestMetricTy::Histogram, None, labels);
        Histogram::noop()
    }
}
