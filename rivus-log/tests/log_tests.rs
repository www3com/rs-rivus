use rivus_log::{LogOptions, LogFile, init};
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_options_creation() {
        let options = LogOptions {
            level: "info".to_string(),
            output: "console".to_string(),
            file: None,
        };

        assert_eq!(options.level, "info");
        assert_eq!(options.output, "console");
        assert!(options.file.is_none());
    }

    #[test]
    fn test_log_options_with_file() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("logs");
        fs::create_dir(&log_path).unwrap();

        let file_config = LogFile {
            path: log_path.to_string_lossy().to_string(),
            prefix: "test_app".to_string(),
            max_size: Some(1024 * 1024), // 1MB
            max_age: Some(7), // 7 days
        };

        let options = LogOptions {
            level: "debug".to_string(),
            output: "console,file".to_string(),
            file: Some(file_config.clone()),
        };

        assert_eq!(options.level, "debug");
        assert_eq!(options.output, "console,file");
        assert!(options.file.is_some());
        
        let file = options.file.unwrap();
        assert_eq!(file.path, file_config.path);
        assert_eq!(file.prefix, file_config.prefix);
        assert_eq!(file.max_size, file_config.max_size);
        assert_eq!(file.max_age, file_config.max_age);
    }

    #[test]
    fn test_log_file_clone() {
        let file_config = LogFile {
            path: "/var/log".to_string(),
            prefix: "myapp".to_string(),
            max_size: Some(10 * 1024 * 1024),
            max_age: Some(30),
        };

        let cloned = file_config.clone();
        assert_eq!(cloned.path, file_config.path);
        assert_eq!(cloned.prefix, file_config.prefix);
        assert_eq!(cloned.max_size, file_config.max_size);
        assert_eq!(cloned.max_age, file_config.max_age);
    }

    #[test]
    fn test_log_options_serialization() {
        let options = LogOptions {
            level: "warn".to_string(),
            output: "file".to_string(),
            file: Some(LogFile {
                path: "/tmp/logs".to_string(),
                prefix: "test".to_string(),
                max_size: None,
                max_age: None,
            }),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("\"level\":\"warn\""));
        assert!(json.contains("\"output\":\"file\""));
        assert!(json.contains("\"path\":\"/tmp/logs\""));
        assert!(json.contains("\"prefix\":\"test\""));

        // Test JSON deserialization
        let deserialized: LogOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.level, options.level);
        assert_eq!(deserialized.output, options.output);
        assert!(deserialized.file.is_some());
    }

    #[test]
    fn test_init_with_console_only() {
        let options = LogOptions {
            level: "error".to_string(),
            output: "console".to_string(),
            file: None,
        };

        // This should not panic
        init(options);
        
        // Test that logging works after initialization
        tracing::error!("Test error message");
        tracing::info!("This should not be logged due to error level");
    }

    #[test]
    fn test_init_with_invalid_output() {
        let options = LogOptions {
            level: "info".to_string(),
            output: "invalid_output".to_string(),
            file: None,
        };

        // Should fall back to console
        init(options);
        tracing::info!("Should work with fallback to console");
    }

    #[test]
    fn test_init_with_console_and_file() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("logs");
        fs::create_dir(&log_path).unwrap();

        let options = LogOptions {
            level: "debug".to_string(),
            output: "console,file".to_string(),
            file: Some(LogFile {
                path: log_path.to_string_lossy().to_string(),
                prefix: "test".to_string(),
                max_size: Some(1024),
                max_age: Some(1),
            }),
        };

        // Should not panic
        init(options);
        tracing::debug!("Test debug message");
        tracing::info!("Test info message");
    }

    #[test]
    fn test_init_with_file_only() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("logs");
        fs::create_dir(&log_path).unwrap();

        let options = LogOptions {
            level: "trace".to_string(),
            output: "file".to_string(),
            file: Some(LogFile {
                path: log_path.to_string_lossy().to_string(),
                prefix: "file_only_test".to_string(),
                max_size: None,
                max_age: None,
            }),
        };

        // Should not panic
        init(options);
        tracing::trace!("Test trace message");
        tracing::debug!("Test debug message");
    }

    #[test]
    fn test_init_with_file_output_but_no_file_config() {
        let options = LogOptions {
            level: "info".to_string(),
            output: "file".to_string(),
            file: None,
        };

        // Should print warning and fall back to console
        init(options);
        tracing::info!("Should work with fallback to console due to missing file config");
    }

    #[test]
    fn test_multiple_init_calls() {
        let options1 = LogOptions {
            level: "info".to_string(),
            output: "console".to_string(),
            file: None,
        };

        let options2 = LogOptions {
            level: "debug".to_string(),
            output: "console".to_string(),
            file: None,
        };

        // First init should work
        init(options1);
        tracing::info!("First initialization");

        // Second init should handle the error gracefully
        init(options2);
        tracing::debug!("Second initialization attempt");
    }

    #[test]
    fn test_log_level_variations() {
        let levels = vec!["trace", "debug", "info", "warn", "error"];
        
        for level in levels {
            let options = LogOptions {
                level: level.to_string(),
                output: "console".to_string(),
                file: None,
            };
            
            init(options);
            
            // Test logging at different levels
            tracing::trace!("Trace message");
            tracing::debug!("Debug message");
            tracing::info!("Info message");
            tracing::warn!("Warning message");
            tracing::error!("Error message");
        }
    }
}