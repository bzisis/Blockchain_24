// Moysis Moysis Volos, Greece 29/06/2024.

/// A test to check for compile-time errors.
///
/// This test uses the `trybuild` crate to verify that certain Rust source files
/// fail to compile as expected. It is useful for ensuring that incorrect code
/// produces the expected compiler errors.
#[test]
fn compile_test() {
    // Create a new instance of `TestCases` from the `trybuild` crate.
    let t = trybuild::TestCases::new();

    // Run compile-fail tests on all files in the "tests/compile-fail" directory.
    // These tests are expected to fail to compile.
    t.compile_fail("tests/compile-fail/*.rs");
}
