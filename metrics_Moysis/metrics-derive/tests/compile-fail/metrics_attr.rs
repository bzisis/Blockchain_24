// Moysis Moysis Volos, Greece 29/06/2024.

extern crate reth_metrics_derive;
use reth_metrics_derive::Metrics;

fn main() {}

// Struct with no fields, derived Metrics trait.
#[derive(Metrics)]
struct CustomMetrics;

// Struct with duplicate #[metrics()] attribute.
#[derive(Metrics)]
#[metrics()]
#[metrics()]
struct CustomMetrics2;

// Struct with no fields, derived Metrics trait with duplicate #[metrics()] attribute.
#[derive(Metrics)]
#[metrics()]
struct CustomMetrics3;

// Struct with incorrect scope attribute value (not a string).
#[derive(Metrics)]
#[metrics(scope = value)]
struct CustomMetrics4;

// Struct with incorrect scope attribute value (not a string).
#[derive(Metrics)]
#[metrics(scope = 123)]
struct CustomMetrics5;

// Struct with valid scope attribute.
#[derive(Metrics)]
#[metrics(scope = "some-scope")]
struct CustomMetrics6;

// Struct with multiple scope attributes (only one is expected).
#[derive(Metrics)]
#[metrics(scope = "some_scope", scope = "another_scope")]
struct CustomMetrics7;

// Struct with incorrect separator attribute value (not a string).
#[derive(Metrics)]
#[metrics(separator = value)]
struct CustomMetrics8;

// Struct with incorrect separator attribute value (not a string).
#[derive(Metrics)]
#[metrics(separator = 123)]
struct CustomMetrics9;

// Struct with valid separator attribute.
#[derive(Metrics)]
#[metrics(separator = "x")]
struct CustomMetrics10;

// Struct with multiple separator attributes (only one is expected).
#[derive(Metrics)]
#[metrics(separator = "_", separator = ":")]
struct CustomMetrics11;

// Struct with unrecognized attribute (random).
#[derive(Metrics)]
#[metrics(random = "value")]
struct CustomMetrics12;

// Struct with unrecognized attribute (dynamic).
#[derive(Metrics)]
#[metrics(scope = "scope", dynamic = true)]
struct CustomMetrics13;