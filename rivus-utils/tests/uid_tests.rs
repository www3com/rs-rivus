
#[cfg(test)]
mod uid_comprehensive_tests {
    use rivus_utils::uid::{int_to_str, str_to_int};

    #[test]
    fn test_str_to_int_basic() {
        // Test basic string to int conversion
        let result = str_to_int("ABC");
        assert!(result.is_ok());
        
        let value = result.unwrap();
        assert!(value > 0);
    }

    #[test]
    fn test_str_to_int_empty_string() {
        let result = str_to_int("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_str_to_int_single_char() {
        let test_cases = vec![
            ("A", 0),
            ("B", 1),
            ("Z", 25),
            ("a", 26),
            ("b", 27),
            ("z", 51),
            ("0", 52),
            ("1", 53),
            ("9", 61),
            ("+", 62),
            ("/", 63),
        ];

        for (input, expected) in test_cases {
            let result = str_to_int(input);
            assert!(result.is_ok(), "Failed for input: {}", input);
            assert_eq!(result.unwrap(), expected, "Wrong result for input: {}", input);
        }
    }

    #[test]
    fn test_str_to_int_invalid_chars() {
        let invalid_inputs = vec![
            "@", // Invalid character
            "#", // Invalid character
            "$", // Invalid character
            "%", // Invalid character
            " ", // Space
            "\t", // Tab
            "\n", // Newline
        ];

        for input in invalid_inputs {
            let result = str_to_int(input);
            assert!(result.is_err(), "Should fail for invalid input: {}", input);
        }
    }

    #[test]
    fn test_str_to_int_max_length() {
        // Test maximum length string (10 characters)
        let max_length_string = "ABCDEFGHIJ";
        let result = str_to_int(max_length_string);
        assert!(result.is_ok(), "Should succeed for 10-character string");
    }

    #[test]
    fn test_str_to_int_too_long() {
        // Test string that's too long (11 characters)
        let too_long_string = "ABCDEFGHIJK";
        let result = str_to_int(too_long_string);
        assert!(result.is_err(), "Should fail for string longer than 10 characters");
    }

    #[test]
    fn test_int_to_str_basic() {
        // Test basic int to string conversion
        let result = int_to_str(12345);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_int_to_str_zero() {
        let result = int_to_str(0);
        assert_eq!(result, "");
    }

    #[test]
    fn test_int_to_str_single_values() {
        let test_cases = vec![
            (0, ""),
            (1, "B"),
            (25, "Z"),
            (26, "a"),
            (51, "z"),
            (52, "0"),
            (61, "9"),
            (62, "+"),
            (63, "/"),
        ];

        for (input, expected) in test_cases {
            let result = int_to_str(input);
            assert_eq!(result, expected, "Wrong result for input: {}", input);
        }
    }

    #[test]
    fn test_roundtrip_conversion() {
        // Test that str_to_int and int_to_str are inverse operations
        let test_strings = vec![
            "A",
            "AB",
            "ABC",
            "Test123",
            "HelloWorld",
            "A1B2C3",
            "+/",
            "abc123",
        ];

        for original in test_strings {
            // Convert string to int
            let int_result = str_to_int(original);
            if let Ok(int_value) = int_result {
                // Convert int back to string
                let converted_back = int_to_str(int_value);
                
                // The conversion might not be exactly the same due to the nature of the encoding
                // but we can test that it's consistent
                let reconverted = str_to_int(&converted_back);
                assert!(reconverted.is_ok(), "Roundtrip failed for: {}", original);
                assert_eq!(reconverted.unwrap(), int_value, "Roundtrip value mismatch for: {}", original);
            }
        }
    }

    #[test]
    fn test_large_values() {
        // Test with larger integer values
        let large_values = vec![
            1000000,
            10000000,
            100000000,
            1000000000,
            10000000000,
            100000000000,
            1000000000000,
        ];

        for value in large_values {
            let str_result = int_to_str(value);
            assert!(!str_result.is_empty() || value == 0, "Empty result for value: {}", value);
            
            // Test roundtrip
            let back_to_int = str_to_int(&str_result);
            assert!(back_to_int.is_ok(), "Failed to convert back to int: {}", str_result);
            assert_eq!(back_to_int.unwrap(), value, "Roundtrip failed for value: {}", value);
        }
    }

    #[test]
    fn test_boundary_values() {
        // Test boundary values for the encoding
        let boundary_tests = vec![
            ("A", 0),
            ("Z", 25),
            ("a", 26),
            ("z", 51),
            ("0", 52),
            ("9", 61),
            ("+", 62),
            ("/", 63),
        ];

        for (char_str, expected_int) in boundary_tests {
            let int_result = str_to_int(char_str);
            assert!(int_result.is_ok(), "Failed to convert boundary char: {}", char_str);
            assert_eq!(int_result.unwrap(), expected_int, "Wrong int for boundary char: {}", char_str);
        }
    }

    #[test]
    fn test_mixed_case_strings() {
        // Test strings with mixed case and numbers (only supported characters)
        let mixed_strings = vec![
            "AbC123",
            "XyZ789",
            "ABC123",
            "A1B2C3",
            "Test123",
        ];

        for mixed_str in mixed_strings {
            let result = str_to_int(mixed_str);
            assert!(result.is_ok(), "Failed to convert mixed case string: {}", mixed_str);
            
            let int_value = result.unwrap();
            let back_to_str = int_to_str(int_value);
            
            // Verify roundtrip consistency
            let roundtrip = str_to_int(&back_to_str);
            assert!(roundtrip.is_ok(), "Roundtrip failed for mixed string: {}", mixed_str);
            assert_eq!(roundtrip.unwrap(), int_value, "Roundtrip value mismatch for: {}", mixed_str);
        }
    }

    #[test]
    fn test_special_characters() {
        // Test strings containing special characters + and /
        let special_strings = vec![
            "++",
            "//",
            "+/",
            "/+",
            "A+B",
            "C/D",
            "Test+123",
            "ABC/DEF",
        ];

        for special_str in special_strings {
            let result = str_to_int(special_str);
            assert!(result.is_ok(), "Failed to convert special char string: {}", special_str);
        }
    }
}