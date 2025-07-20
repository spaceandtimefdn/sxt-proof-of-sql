#[cfg(feature = "std")]
use sysinfo::System;
use tracing::{trace, Level};

/// Logs the memory usage of the system at the TRACE level.
///
/// This function logs the available memory, used memory, and the percentage of memory used.
/// It only logs this information if the TRACE level is enabled in the tracing configuration.
///
/// # Arguments
///
/// * `name` - A string slice that holds the name to be included in the log message.
#[expect(clippy::cast_precision_loss)]
pub fn log_memory_usage(name: &str) {
    #[cfg(feature = "std")]
    if tracing::level_enabled!(Level::TRACE) {
        let mut system = System::new_all();
        system.refresh_memory();

        let available_memory = system.available_memory() as f64 / (1024.0 * 1024.0);
        let used_memory = system.used_memory() as f64 / (1024.0 * 1024.0);
        let percentage_memory_used = (used_memory / (used_memory + available_memory)) * 100.0;

        trace!(
            "{} Available memory: {:.2} MB, Used memory: {:.2} MB, Percentage memory used: {:.2}%",
            name,
            available_memory,
            used_memory,
            percentage_memory_used
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_memory_usage_does_not_panic() {
        // Test that the function doesn't panic when called
        log_memory_usage("Test");
        log_memory_usage("");
        log_memory_usage("Test with spaces and symbols !@#$%");
    }

    #[test]
    fn test_log_memory_usage_with_various_names() {
        // Test with different name formats
        let test_names = [
            "Start",
            "End", 
            "Middle", 
            "Process Step 1",
            "UTF-8 ñáme",
            "Name with numbers 123",
            "UPPERCASE",
            "lowercase",
            "MixedCase",
        ];

        for name in &test_names {
            log_memory_usage(name);
        }
    }

    #[test]
    fn test_log_memory_usage_empty_string() {
        // Test with empty string - should not panic
        log_memory_usage("");
    }

    #[test]
    fn test_log_memory_usage_long_string() {
        // Test with a very long string
        let long_name = "a".repeat(1000);
        log_memory_usage(&long_name);
    }

    #[test]
    fn test_log_memory_usage_special_characters() {
        // Test with special characters that might cause issues in logs
        log_memory_usage("Test\nwith\nnewlines");
        log_memory_usage("Test\twith\ttabs");
        log_memory_usage("Test with \"quotes\"");
        log_memory_usage("Test with 'single quotes'");
        log_memory_usage("Test with backslash \\");
    }

    
}
