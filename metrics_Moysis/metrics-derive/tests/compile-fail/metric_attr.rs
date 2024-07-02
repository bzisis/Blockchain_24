// Moysis Moysis Volos, Greece 29/06/2024.

// Import the `metrics` crate, which provides functionality for metrics management.
extern crate metrics;

// Import the `reth_metrics_derive` crate, which enables deriving metrics-related traits and functionality.
extern crate reth_metrics_derive;

// Import specific types or traits from the `metrics` crate.
use metrics::Gauge;

// Import the `Metrics` trait from the `reth_metrics_derive` crate.
use reth_metrics_derive::Metrics;

fn main() {}

// Struct with a single Gauge metric.
#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics {
    gauge: Gauge,
}

// Struct with a Gauge metric annotated with `metric` attribute.
#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics2 {
    #[metric()]
    gauge: Gauge,
}

// Struct with a Gauge metric and incorrect attribute usage (random attribute).
#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics3 {
    #[metric(random = "value")]
    gauge: Gauge,
    // Note: 'random' is not a recognized attribute for metrics.
}

// Struct with a Gauge metric and incorrect attribute usage (describe attribute).
#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics4 {
    #[metric(describe = 123)]
    gauge: Gauge,
    // Note: 'describe' expects a string, not an integer.
}

// Struct with a Gauge metric and incorrect attribute usage (rename attribute).
#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics5 {
    #[metric(rename = 123)]
    gauge: Gauge,
    // Note: 'rename' expects a string, not an integer.
}

// Struct with a Gauge metric and redundant describe attribute.
#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics6 {
    #[metric(describe = "", describe = "")]
    gauge: Gauge,
    // Note: 'describe' should be used only once with a single string value.
}

// Struct with a Gauge metric and redundant rename attribute.
#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics7 {
    #[metric(rename = "_gauge", rename = "_gauge")]
    gauge: Gauge,
    // Note: 'rename' should be used only once with a single string value.
}

// Struct with a non-metric type annotated as a metric.
#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics8 {
    #[metric(describe = "")]
    gauge: String,
    // Note: Gauge type is expected for metrics, but `String` is provided here.
}