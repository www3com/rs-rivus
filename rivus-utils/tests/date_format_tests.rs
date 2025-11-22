use chrono::NaiveDateTime;
use serde::{Serialize, Serializer};

#[cfg(test)]
mod date_format_tests {
    use rivus_utils::date_format;
    use super::*;

    #[test]
    fn test_serialize_with_custom_format() {
        // Create a test datetime
        let dt_str = "2023-12-25 15:30:45";
        let dt = NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S").unwrap();
        
        // Test serialization with custom format
        let format = "%Y-%m-%d %H:%M:%S";
        let mut serializer = serde_json::Serializer::new(Vec::new());
        
        let result = date_format::serialize_with_custom_format(&Some(dt), format, &mut serializer);
        assert!(result.is_ok());
    }

    #[test]
    fn test_serialize_with_custom_format_none() {
        // Test serialization with None value
        let format = "%Y-%m-%d %H:%M:%S";
        let mut serializer = serde_json::Serializer::new(Vec::new());
        
        let result = date_format::serialize_with_custom_format(&None, format, &mut serializer);
        assert!(result.is_ok());
    }

    #[test]
    fn test_standard_format() {
        // Test the standard format module
        let dt_str = "2023-12-25 15:30:45";
        let dt = NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S").unwrap();
        
        let mut serializer = serde_json::Serializer::new(Vec::new());
        let result = date_format::standard::serialize(&Some(dt), &mut serializer);
        assert!(result.is_ok());
    }

    #[test]
    fn test_date_only_format() {
        // Test the date_only format module
        let dt_str = "2023-12-25 15:30:45";
        let dt = NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S").unwrap();
        
        let mut serializer = serde_json::Serializer::new(Vec::new());
        let result = date_format::date_only::serialize(&Some(dt), &mut serializer);
        assert!(result.is_ok());
    }

    #[test]
    fn test_standard_format_none() {
        // Test standard format with None
        let mut serializer = serde_json::Serializer::new(Vec::new());
        let result = date_format::standard::serialize(&None, &mut serializer);
        assert!(result.is_ok());
    }

    #[test]
    fn test_date_only_format_none() {
        // Test date_only format with None
        let mut serializer = serde_json::Serializer::new(Vec::new());
        let result = date_format::date_only::serialize(&None, &mut serializer);
        assert!(result.is_ok());
    }

    #[test]
    fn test_different_datetime_formats() {
        let test_cases = vec![
            ("2023-01-01 00:00:00", "%Y-%m-%d %H:%M:%S"),
            ("2023-12-31 23:59:59", "%Y-%m-%d %H:%M:%S"),
            ("2023-06-15 12:30:45", "%Y-%m-%d %H:%M:%S"),
        ];

        for (dt_str, format) in test_cases {
            let dt = NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S").unwrap();
            
            let mut serializer = serde_json::Serializer::new(Vec::new());
            let result = date_format::serialize_with_custom_format(&Some(dt), format, &mut serializer);
            assert!(result.is_ok(), "Failed to serialize datetime: {}", dt_str);
        }
    }

    #[test]
    fn test_edge_case_datetimes() {
        // Test edge cases like leap years, different months, etc.
        let edge_cases = vec![
            "2020-02-29 12:00:00", // Leap year
            "2023-02-28 23:59:59", // Non-leap year
            "2023-01-01 00:00:00", // Start of year
            "2023-12-31 23:59:59", // End of year
        ];

        for dt_str in edge_cases {
            let dt = NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S").unwrap();
            
            // Test standard format
            let mut serializer = serde_json::Serializer::new(Vec::new());
            let result = date_format::standard::serialize(&Some(dt), &mut serializer);
            assert!(result.is_ok(), "Failed to serialize edge case: {}", dt_str);
            
            // Test date_only format
            let mut serializer = serde_json::Serializer::new(Vec::new());
            let result = date_format::date_only::serialize(&Some(dt), &mut serializer);
            assert!(result.is_ok(), "Failed to serialize edge case with date_only: {}", dt_str);
        }
    }

    #[test]
    fn test_custom_format_variations() {
        let dt_str = "2023-12-25 15:30:45";
        let dt = NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S").unwrap();
        
        let formats = vec![
            "%Y-%m-%d %H:%M:%S",
            "%Y/%m/%d %H:%M:%S",
            "%d-%m-%Y %H:%M:%S",
            "%Y-%m-%d",
            "%H:%M:%S",
            "%Y",
            "%m",
            "%d",
        ];

        for format in formats {
            let mut serializer = serde_json::Serializer::new(Vec::new());
            let result = date_format::serialize_with_custom_format(&Some(dt), format, &mut serializer);
            assert!(result.is_ok(), "Failed to serialize with format: {}", format);
        }
    }
}