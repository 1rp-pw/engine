#[cfg(test)]
mod tests {
    use crate::runner::model::{ComparisonOperator, RuleValue, SourcePosition};
    use crate::runner::trace::{
        ComparisonEvaluationTrace, ComparisonTrace, ConditionTrace, OutcomeTrace,
        PropertyCheckTrace, PropertyTrace, RuleReferenceTrace, RuleSetTrace, RuleTrace,
        SelectorTrace, TypedValue, ValueTrace,
    };
    use chrono::NaiveDate;
    use serde_json;

    #[test]
    fn test_source_position_serialization() {
        let pos = SourcePosition {
            line: 5,
            start: 10,
            end: 20,
        };

        let json = serde_json::to_value(&pos).unwrap();
        assert_eq!(json["line"], 5);
        assert_eq!(json["start"], 10);
        assert_eq!(json["end"], 20);
    }

    #[test]
    fn test_selector_trace_serialization() {
        let trace = SelectorTrace {
            value: "user".to_string(),
            pos: Some(SourcePosition {
                line: 1,
                start: 5,
                end: 9,
            }),
        };

        let json = serde_json::to_value(&trace).unwrap();
        assert_eq!(json["value"], "user");
        assert!(json["pos"].is_object());
    }

    #[test]
    fn test_selector_trace_no_position() {
        let trace = SelectorTrace {
            value: "user".to_string(),
            pos: None,
        };

        let json = serde_json::to_value(&trace).unwrap();
        assert_eq!(json["value"], "user");
        assert!(json.get("pos").is_none());
    }

    #[test]
    fn test_outcome_trace_serialization() {
        let trace = OutcomeTrace {
            value: "eligible".to_string(),
            pos: Some(SourcePosition {
                line: 2,
                start: 15,
                end: 23,
            }),
        };

        let json = serde_json::to_value(&trace).unwrap();
        assert_eq!(json["value"], "eligible");
        assert!(json["pos"].is_object());
    }

    #[test]
    fn test_property_trace_serialization() {
        let trace = PropertyTrace {
            value: serde_json::json!({"status": "active"}),
            path: "$.user.status".to_string(),
        };

        let json = serde_json::to_value(&trace).unwrap();
        assert_eq!(json["path"], "$.user.status");
        assert!(json["value"].is_object());
    }

    #[test]
    fn test_typed_value_from_rule_value_number() {
        let rule_value = RuleValue::Number(42.5);
        let typed_value = TypedValue::from(&rule_value);

        assert_eq!(typed_value.value_type, "number");
        assert_eq!(typed_value.value, serde_json::json!(42.5));
    }

    #[test]
    fn test_typed_value_from_rule_value_string() {
        let rule_value = RuleValue::String("hello".to_string());
        let typed_value = TypedValue::from(&rule_value);

        assert_eq!(typed_value.value_type, "string");
        assert_eq!(typed_value.value, serde_json::json!("hello"));
    }

    #[test]
    fn test_typed_value_from_rule_value_boolean() {
        let rule_value = RuleValue::Boolean(true);
        let typed_value = TypedValue::from(&rule_value);

        assert_eq!(typed_value.value_type, "boolean");
        assert_eq!(typed_value.value, serde_json::json!(true));
    }

    #[test]
    fn test_typed_value_from_rule_value_date() {
        let date = NaiveDate::from_ymd_opt(2023, 12, 25).unwrap();
        let rule_value = RuleValue::Date(date);
        let typed_value = TypedValue::from(&rule_value);

        assert_eq!(typed_value.value_type, "date");
        assert_eq!(typed_value.value, serde_json::json!("2023-12-25"));
    }

    #[test]
    fn test_typed_value_from_rule_value_list() {
        let rule_value = RuleValue::List(vec![
            RuleValue::String("item1".to_string()),
            RuleValue::Number(123.0),
            RuleValue::Boolean(false),
        ]);
        let typed_value = TypedValue::from(&rule_value);

        assert_eq!(typed_value.value_type, "list");
        let expected = serde_json::json!(["item1", 123.0, false]);
        assert_eq!(typed_value.value, expected);
    }

    #[test]
    fn test_typed_value_from_rule_value_nested_list() {
        let rule_value = RuleValue::List(vec![RuleValue::List(vec![RuleValue::String(
            "nested".to_string(),
        )])]);
        let typed_value = TypedValue::from(&rule_value);

        assert_eq!(typed_value.value_type, "list");
        // Nested lists should be null according to the comment
        assert_eq!(typed_value.value, serde_json::json!([null]));
    }

    #[test]
    fn test_rule_value_to_value_trace() {
        let pos = Some(SourcePosition {
            line: 1,
            start: 0,
            end: 5,
        });
        let rule_value = RuleValue::String("test".to_string());

        let value_trace = rule_value.to_value_trace(pos.clone());

        assert_eq!(value_trace.value_type, "string");
        assert_eq!(value_trace.value, serde_json::json!("test"));
        assert_eq!(value_trace.pos, pos);
    }

    #[test]
    fn test_rule_value_to_value_trace_no_position() {
        let rule_value = RuleValue::Number(42.0);
        let value_trace = rule_value.to_value_trace(None);

        assert_eq!(value_trace.value_type, "number");
        assert_eq!(value_trace.value, serde_json::json!(42.0));
        assert!(value_trace.pos.is_none());
    }

    #[test]
    fn test_comparison_evaluation_trace_serialization() {
        let trace = ComparisonEvaluationTrace {
            left_value: TypedValue {
                value: serde_json::json!(25),
                value_type: "number".to_string(),
            },
            right_value: TypedValue {
                value: serde_json::json!(18),
                value_type: "number".to_string(),
            },
            comparison_result: true,
        };

        let json = serde_json::to_value(&trace).unwrap();
        assert_eq!(json["left_value"]["value"], 25);
        assert_eq!(json["left_value"]["type"], "number");
        assert_eq!(json["right_value"]["value"], 18);
        assert_eq!(json["right_value"]["type"], "number");
        assert_eq!(json["comparison_result"], true);
    }

    #[test]
    fn test_property_check_trace_serialization() {
        let trace = PropertyCheckTrace {
            property_name: "ageEligible".to_string(),
            property_value: serde_json::json!({"Boolean": true}),
        };

        let json = serde_json::to_value(&trace).unwrap();
        assert_eq!(json["property_name"], "ageEligible");
        assert_eq!(json["property_value"]["Boolean"], true);
    }

    #[test]
    fn test_rule_reference_trace_serialization() {
        let trace = RuleReferenceTrace {
            selector: SelectorTrace {
                value: "account".to_string(),
                pos: None,
            },
            rule_name: "active".to_string(),
            referenced_rule_outcome: Some("account is active".to_string()),
            property_check: None,
            result: true,
        };

        let json = serde_json::to_value(&trace).unwrap();
        assert_eq!(json["selector"]["value"], "account");
        assert_eq!(json["rule_name"], "active");
        assert_eq!(json["referenced_rule_outcome"], "account is active");
        assert!(json.get("property_check").is_none());
        assert_eq!(json["result"], true);
    }

    #[test]
    fn test_rule_reference_trace_with_property_check() {
        let property_check = PropertyCheckTrace {
            property_name: "status".to_string(),
            property_value: serde_json::json!({"String": "pass"}),
        };

        let trace = RuleReferenceTrace {
            selector: SelectorTrace {
                value: "user".to_string(),
                pos: None,
            },
            rule_name: "background check".to_string(),
            referenced_rule_outcome: None,
            property_check: Some(property_check),
            result: false,
        };

        let json = serde_json::to_value(&trace).unwrap();
        assert_eq!(json["selector"]["value"], "user");
        assert_eq!(json["rule_name"], "background check");
        assert!(json.get("referenced_rule_outcome").is_none());
        assert!(json["property_check"].is_object());
        assert_eq!(json["property_check"]["property_name"], "status");
        assert_eq!(json["result"], false);
    }

    #[test]
    fn test_comparison_trace_serialization() {
        let trace = ComparisonTrace {
            selector: SelectorTrace {
                value: "user".to_string(),
                pos: Some(SourcePosition {
                    line: 1,
                    start: 0,
                    end: 4,
                }),
            },
            property: PropertyTrace {
                value: serde_json::json!(25),
                path: "$.user.age".to_string(),
            },
            operator: ComparisonOperator::GreaterThanOrEqual,
            value: ValueTrace {
                value: serde_json::json!(18),
                value_type: "number".to_string(),
                pos: None,
            },
            evaluation_details: Some(ComparisonEvaluationTrace {
                left_value: TypedValue {
                    value: serde_json::json!(25),
                    value_type: "number".to_string(),
                },
                right_value: TypedValue {
                    value: serde_json::json!(18),
                    value_type: "number".to_string(),
                },
                comparison_result: true,
            }),
            result: true,
        };

        let json = serde_json::to_value(&trace).unwrap();
        assert_eq!(json["selector"]["value"], "user");
        assert_eq!(json["property"]["path"], "$.user.age");
        assert_eq!(json["operator"], "GreaterThanOrEqual");
        assert_eq!(json["value"]["value"], 18);
        assert!(json["evaluation_details"].is_object());
        assert_eq!(json["result"], true);
    }

    #[test]
    fn test_condition_trace_comparison_variant() {
        let comparison_trace = ComparisonTrace {
            selector: SelectorTrace {
                value: "user".to_string(),
                pos: None,
            },
            property: PropertyTrace {
                value: serde_json::json!("active"),
                path: "$.user.status".to_string(),
            },
            operator: ComparisonOperator::EqualTo,
            value: ValueTrace {
                value: serde_json::json!("active"),
                value_type: "string".to_string(),
                pos: None,
            },
            evaluation_details: None,
            result: true,
        };

        let condition_trace = ConditionTrace::Comparison(comparison_trace);
        let json = serde_json::to_value(&condition_trace).unwrap();

        // Should serialize as the inner ComparisonTrace due to #[serde(untagged)]
        assert_eq!(json["selector"]["value"], "user");
        assert_eq!(json["property"]["path"], "$.user.status");
        assert_eq!(json["operator"], "EqualTo");
    }

    #[test]
    fn test_condition_trace_rule_reference_variant() {
        let rule_ref_trace = RuleReferenceTrace {
            selector: SelectorTrace {
                value: "account".to_string(),
                pos: None,
            },
            rule_name: "is active".to_string(),
            referenced_rule_outcome: Some("account active".to_string()),
            property_check: None,
            result: true,
        };

        let condition_trace = ConditionTrace::RuleReference(rule_ref_trace);
        let json = serde_json::to_value(&condition_trace).unwrap();

        // Should serialize as the inner RuleReferenceTrace due to #[serde(untagged)]
        assert_eq!(json["selector"]["value"], "account");
        assert_eq!(json["rule_name"], "is active");
        assert_eq!(json["referenced_rule_outcome"], "account active");
    }

    #[test]
    fn test_rule_trace_serialization() {
        let rule_trace = RuleTrace {
            label: Some("Age Check".to_string()),
            selector: SelectorTrace {
                value: "user".to_string(),
                pos: None,
            },
            outcome: OutcomeTrace {
                value: "age verified".to_string(),
                pos: None,
            },
            conditions: vec![],
            result: true,
        };

        let json = serde_json::to_value(&rule_trace).unwrap();
        assert_eq!(json["label"], "Age Check");
        assert_eq!(json["selector"]["value"], "user");
        assert_eq!(json["outcome"]["value"], "age verified");
        assert!(json["conditions"].is_array());
        assert_eq!(json["result"], true);
    }

    #[test]
    fn test_rule_trace_no_label() {
        let rule_trace = RuleTrace {
            label: None,
            selector: SelectorTrace {
                value: "user".to_string(),
                pos: None,
            },
            outcome: OutcomeTrace {
                value: "eligible".to_string(),
                pos: None,
            },
            conditions: vec![],
            result: false,
        };

        let json = serde_json::to_value(&rule_trace).unwrap();
        assert!(json.get("label").is_none());
        assert_eq!(json["selector"]["value"], "user");
        assert_eq!(json["outcome"]["value"], "eligible");
        assert_eq!(json["result"], false);
    }

    #[test]
    fn test_rule_set_trace_serialization() {
        let rule_set_trace = RuleSetTrace {
            execution: vec![RuleTrace {
                label: None,
                selector: SelectorTrace {
                    value: "user".to_string(),
                    pos: None,
                },
                outcome: OutcomeTrace {
                    value: "eligible".to_string(),
                    pos: None,
                },
                conditions: vec![],
                result: true,
            }],
        };

        let json = serde_json::to_value(&rule_set_trace).unwrap();
        assert!(json["execution"].is_array());
        assert_eq!(json["execution"].as_array().unwrap().len(), 1);
        assert_eq!(json["execution"][0]["outcome"]["value"], "eligible");
    }
}