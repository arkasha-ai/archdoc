//! Test utilities for golden tests

use std::fs;
use std::path::Path;

/// Read a file and return its contents
pub fn read_test_file(path: &str) -> String {
    fs::read_to_string(path).expect(&format!("Failed to read test file: {}", path))
}

/// Write content to a file for testing
pub fn write_test_file(path: &str, content: &str) {
    fs::write(path, content).expect(&format!("Failed to write test file: {}", path))
}

/// Compare two strings and panic if they don't match
pub fn assert_strings_equal(actual: &str, expected: &str, message: &str) {
    if actual != expected {
        panic!("{}: Strings do not match\nActual:\n{}\nExpected:\n{}", message, actual, expected);
    }
}