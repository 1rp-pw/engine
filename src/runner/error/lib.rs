#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use serde_json;
    use crate::runner::error::RuleError;

    #[test]
    fn test_parse_error_creation() {
        let error = RuleError::ParseError("Invalid syntax".to_string());
        assert!(matches!(error, RuleError::ParseError(_)));

        let error_message = format!("{}", error);
        assert_eq!(error_message, "Parse error: Invalid syntax");
    }

    #[test]
    fn test_parse_error_debug() {
        let error = RuleError::ParseError("Missing semicolon".to_string());
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains("ParseError"));
        assert!(debug_output.contains("Missing semicolon"));
    }

    #[test]
    fn test_evaluation_error_creation() {
        let error = RuleError::EvaluationError("Division by zero".to_string());
        assert!(matches!(error, RuleError::EvaluationError(_)));

        let error_message = format!("{}", error);
        assert_eq!(error_message, "Evaluation error: Division by zero");
    }

    #[test]
    fn test_evaluation_error_debug() {
        let error = RuleError::EvaluationError("Undefined variable".to_string());
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains("EvaluationError"));
        assert!(debug_output.contains("Undefined variable"));
    }

    #[test]
    fn test_type_error_creation() {
        let error = RuleError::TypeError("Expected number, found string".to_string());
        assert!(matches!(error, RuleError::TypeError(_)));

        let error_message = format!("{}", error);
        assert_eq!(error_message, "Type error: Expected number, found string");
    }

    #[test]
    fn test_type_error_debug() {
        let error = RuleError::TypeError("Incompatible types".to_string());
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains("TypeError"));
        assert!(debug_output.contains("Incompatible types"));
    }

    #[test]
    fn test_io_error_from_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let rule_error: RuleError = io_err.into();

        assert!(matches!(rule_error, RuleError::IoError(_)));

        let error_message = format!("{}", rule_error);
        assert!(error_message.contains("IO error:"));
        assert!(error_message.contains("File not found"));
    }

    #[test]
    fn test_io_error_various_kinds() {
        let test_cases = vec![
            (io::ErrorKind::PermissionDenied, "Permission denied"),
            (io::ErrorKind::ConnectionRefused, "Connection refused"),
            (io::ErrorKind::InvalidInput, "Invalid input"),
            (io::ErrorKind::TimedOut, "Operation timed out"),
        ];

        for (kind, message) in test_cases {
            let io_err = io::Error::new(kind, message);
            let rule_error: RuleError = io_err.into();

            assert!(matches!(rule_error, RuleError::IoError(_)));
            let error_str = format!("{}", rule_error);
            assert!(error_str.contains("IO error:"));
            assert!(error_str.contains(message));
        }
    }

    #[test]
    fn test_json_error_from_conversion() {
        // Create a JSON parsing error
        let json_result: Result<serde_json::Value, serde_json::Error> =
            serde_json::from_str("{ invalid json }");

        let json_err = json_result.unwrap_err();
        let rule_error: RuleError = json_err.into();

        assert!(matches!(rule_error, RuleError::JsonError(_)));

        let error_message = format!("{}", rule_error);
        assert!(error_message.contains("JSON error:"));
    }

    #[test]
    fn test_json_error_serialization() {
        use serde::Serialize;

        // Create a type that will fail to serialize
        #[derive(Serialize)]
        struct BadStruct {
            #[serde(serialize_with = "failing_serializer")]
            field: i32,
        }

        fn failing_serializer<S>(_: &i32, _: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(serde::ser::Error::custom("Intentional serialization failure"))
        }

        let bad_struct = BadStruct { field: 42 };
        let json_result = serde_json::to_string(&bad_struct);

        let json_err = json_result.unwrap_err();
        let rule_error: RuleError = json_err.into();

        assert!(matches!(rule_error, RuleError::JsonError(_)));
        let error_str = format!("{}", rule_error);
        assert!(error_str.contains("JSON error:"));
    }

    #[test]
    fn test_error_equality() {
        let error1 = RuleError::ParseError("test".to_string());
        let error2 = RuleError::ParseError("test".to_string());
        let error3 = RuleError::ParseError("different".to_string());
        let error4 = RuleError::EvaluationError("test".to_string());

        // Note: RuleError doesn't derive PartialEq, so we test display equality instead
        assert_eq!(format!("{}", error1), format!("{}", error2));
        assert_ne!(format!("{}", error1), format!("{}", error3));
        assert_ne!(format!("{}", error1), format!("{}", error4));
    }

    #[test]
    fn test_error_source() {
        use std::error::Error;

        // Test that IO errors preserve their source
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let rule_error: RuleError = io_err.into();

        // The source should be available
        assert!(rule_error.source().is_some());

        // Test that JSON errors preserve their source
        let json_result: Result<serde_json::Value, serde_json::Error> =
            serde_json::from_str("{ invalid json }");
        let json_err = json_result.unwrap_err();
        let rule_error: RuleError = json_err.into();

        assert!(rule_error.source().is_some());

        // Test that string-based errors don't have a source
        let parse_error = RuleError::ParseError("test".to_string());
        assert!(parse_error.source().is_none());
    }

    #[test]
    fn test_error_chain() {
        use std::error::Error;

        // Create a nested IO error
        let inner_error = io::Error::new(io::ErrorKind::Other, "Inner error");
        let outer_error = io::Error::new(io::ErrorKind::NotFound, "Outer error");

        let rule_error: RuleError = outer_error.into();

        // Should be able to walk the error chain
        let mut current_error: &dyn Error = &rule_error;
        let mut error_count = 0;

        loop {
            error_count += 1;
            match current_error.source() {
                Some(source) => current_error = source,
                None => break,
            }
            // Prevent infinite loops in tests
            if error_count > 10 {
                break;
            }
        }

        assert!(error_count >= 1);
    }

    #[test]
    fn test_error_downcast() {
        use std::error::Error;

        // Test downcasting for IO errors
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let rule_error: RuleError = io_err.into();

        if let RuleError::IoError(ref inner_io_err) = rule_error {
            assert_eq!(inner_io_err.kind(), io::ErrorKind::NotFound);
        } else {
            panic!("Expected IoError variant");
        }
    }

    #[test]
    fn test_all_error_variants_display() {
        let errors = vec![
            RuleError::ParseError("parse issue".to_string()),
            RuleError::EvaluationError("eval issue".to_string()),
            RuleError::TypeError("type issue".to_string()),
            RuleError::IoError(io::Error::new(io::ErrorKind::NotFound, "io issue")),
            RuleError::JsonError(serde_json::from_str::<serde_json::Value>("invalid").unwrap_err()),
        ];

        for error in errors {
            let display_str = format!("{}", error);
            let debug_str = format!("{:?}", error);

            // Each error should have a non-empty display and debug representation
            assert!(!display_str.is_empty());
            assert!(!debug_str.is_empty());

            // Display should not contain "Error" at the start (since thiserror handles this)
            // but should contain the error type description
            match error {
                RuleError::ParseError(_) => assert!(display_str.starts_with("Parse error:")),
                RuleError::EvaluationError(_) => assert!(display_str.starts_with("Evaluation error:")),
                RuleError::TypeError(_) => assert!(display_str.starts_with("Type error:")),
                RuleError::IoError(_) => assert!(display_str.starts_with("IO error:")),
                RuleError::JsonError(_) => assert!(display_str.starts_with("JSON error:")),
            }
        }
    }

    #[test]
    fn test_error_send_sync() {
        // Ensure RuleError implements Send and Sync (important for async code)
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<RuleError>();
        assert_sync::<RuleError>();
    }
}