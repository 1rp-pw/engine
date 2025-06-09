#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use serde_json::{json};
    use std::collections::{HashMap, HashSet};
    use crate::runner::error::RuleError;
    use crate::runner::evaluator::{
        compare_contains, compare_dates_earlier, compare_dates_later, compare_equal,
        compare_in_list, compare_not_equal, compare_not_in_list, compare_numbers_gt,
        compare_numbers_gte, compare_numbers_lt, compare_numbers_lte, evaluate_rule_set,
        evaluate_rule, evaluate_comparison_condition, convert_json_to_rule_value,
        find_effective_selector, extract_value_from_json, compare_is_empty, compare_is_not_empty,
        evaluate_rule_set_with_trace, evaluate_rule_with_trace
    };
    use crate::runner::model::{
        RuleValue, Rule, RuleSet, Condition, ComparisonCondition, RuleReferenceCondition,
        ComparisonOperator, ConditionGroup, ConditionOperator, PropertyPath, PropertyChainElement,
        PositionedValue
    };

    // Basic comparison tests (existing)
    #[test]
    fn test_compare_numbers() {
        let five = RuleValue::Number(5.0);
        let ten = RuleValue::Number(10.0);

        assert_eq!(compare_numbers_gt(&ten, &five).unwrap(), true);
        assert_eq!(compare_numbers_gt(&five, &ten).unwrap(), false);
        assert_eq!(compare_numbers_gte(&five, &five).unwrap(), true);
        assert_eq!(compare_numbers_lt(&five, &ten).unwrap(), true);
        assert_eq!(compare_numbers_lte(&ten, &ten).unwrap(), true);
    }

    #[test]
    fn test_compare_dates() {
        let date1 = RuleValue::Date(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        let date2 = RuleValue::Date(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
        let date_str = RuleValue::String("2020-06-15".to_string());

        assert_eq!(compare_dates_earlier(&date1, &date2).unwrap(), true);
        assert_eq!(compare_dates_later(&date2, &date1).unwrap(), true);
        assert_eq!(compare_dates_earlier(&date_str, &date2).unwrap(), true);
    }

    #[test]
    fn test_compare_equality() {
        let str1 = RuleValue::String("hello".to_string());
        let str2 = RuleValue::String("hello".to_string());
        let str3 = RuleValue::String("world".to_string());

        assert_eq!(compare_equal(&str1, &str2).unwrap(), true);
        assert_eq!(compare_equal(&str1, &str3).unwrap(), false);
        assert_eq!(compare_not_equal(&str1, &str3).unwrap(), true);
    }

    #[test]
    fn test_list_operations() {
        let value = RuleValue::String("apple".to_string());
        let list = RuleValue::List(vec![
            RuleValue::String("apple".to_string()),
            RuleValue::String("banana".to_string()),
        ]);

        assert_eq!(compare_in_list(&value, &list).unwrap(), true);
        assert_eq!(compare_contains(&list, &value).unwrap(), true);

        let missing = RuleValue::String("orange".to_string());
        assert_eq!(compare_in_list(&missing, &list).unwrap(), false);
        assert_eq!(compare_not_in_list(&missing, &list).unwrap(), true);
    }

    // New comprehensive tests

    #[test]
    fn test_convert_json_to_rule_value() {
        // Test number conversion
        let json_num = json!(42.5);
        let rule_val = convert_json_to_rule_value(&json_num).unwrap();
        assert!(matches!(rule_val, RuleValue::Number(42.5)));

        // Test string conversion
        let json_str = json!("test string");
        let rule_val = convert_json_to_rule_value(&json_str).unwrap();
        assert!(matches!(rule_val, RuleValue::String(s) if s == "test string"));

        // Test boolean conversion
        let json_bool = json!(true);
        let rule_val = convert_json_to_rule_value(&json_bool).unwrap();
        assert!(matches!(rule_val, RuleValue::Boolean(true)));

        // Test date string conversion
        let json_date = json!("2023-12-25");
        let rule_val = convert_json_to_rule_value(&json_date).unwrap();
        assert!(matches!(rule_val, RuleValue::Date(_)));

        // Test array conversion
        let json_array = json!(["item1", "item2"]);
        let rule_val = convert_json_to_rule_value(&json_array).unwrap();
        if let RuleValue::List(items) = rule_val {
            assert_eq!(items.len(), 2);
            assert!(matches!(items[0], RuleValue::String(ref s) if s == "item1"));
            assert!(matches!(items[1], RuleValue::String(ref s) if s == "item2"));
        } else {
            panic!("Expected List variant");
        }
    }

    #[test]
    fn test_find_effective_selector() {
        let json = json!({
            "user": {"name": "John"},
            "User": {"name": "Jane"},
            "userProfile": {"age": 30}
        });

        // Test exact match
        let result = find_effective_selector("user", &json).unwrap();
        assert_eq!(result, Some("user".to_string()));

        // Test case-insensitive match - should return actual key from JSON
        let result = find_effective_selector("USER", &json).unwrap();
        assert_eq!(result, Some("User".to_string()));

        // Test camelCase transformation
        let result = find_effective_selector("user profile", &json).unwrap();
        assert_eq!(result, Some("userProfile".to_string()));

        // Test non-existent selector
        let result = find_effective_selector("nonexistent", &json).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_value_from_json() {
        let json = json!({
            "user": {
                "name": "John",
                "age": 25,
                "active": true,
                "joinDate": "2023-01-15"
            }
        });

        // Test string extraction
        let result = extract_value_from_json(&json, "user", "name").unwrap();
        assert!(matches!(result, RuleValue::String(s) if s == "John"));

        // Test number extraction
        let result = extract_value_from_json(&json, "user", "age").unwrap();
        assert!(matches!(result, RuleValue::Number(25.0)));

        // Test boolean extraction
        let result = extract_value_from_json(&json, "user", "active").unwrap();
        assert!(matches!(result, RuleValue::Boolean(true)));

        // Test date string extraction
        let result = extract_value_from_json(&json, "user", "joinDate").unwrap();
        assert!(matches!(result, RuleValue::Date(_)));
    }

    #[test]
    fn test_comparison_condition_evaluation() {
        let json = json!({
            "user": {
                "age": 25,
                "name": "John",
                "active": true
            }
        });

        // Test numeric comparison
        let condition = ComparisonCondition {
            selector: PositionedValue { value: "user".to_string(), pos: None },
            property: PositionedValue { value: "age".to_string(), pos: None },
            operator: ComparisonOperator::GreaterThan,
            value: PositionedValue { value: RuleValue::Number(20.0), pos: None },
            left_property_path: None,
            right_property_path: None,
            property_chain: None,
        };

        let (result, _trace) = evaluate_comparison_condition(&condition, &json).unwrap();
        assert_eq!(result, true);

        // Test string equality
        let condition = ComparisonCondition {
            selector: PositionedValue { value: "user".to_string(), pos: None },
            property: PositionedValue { value: "name".to_string(), pos: None },
            operator: ComparisonOperator::EqualTo,
            value: PositionedValue { value: RuleValue::String("John".to_string()), pos: None },
            left_property_path: None,
            right_property_path: None,
            property_chain: None,
        };

        let (result, _trace) = evaluate_comparison_condition(&condition, &json).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_rule_evaluation() {
        let json = json!({
            "user": {
                "age": 25,
                "name": "John"
            }
        });

        let rule = Rule {
            label: Some("test rule".to_string()),
            selector: "user".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "user".to_string(), pos: None },
                        property: PositionedValue { value: "age".to_string(), pos: None },
                        operator: ComparisonOperator::GreaterThan,
                        value: PositionedValue { value: RuleValue::Number(18.0), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: None,
                }
            ],
            outcome: "adult".to_string(),
            position: None,
        };

        let rule_map: HashMap<String, usize> = HashMap::new();
        let label_map: HashMap<String, usize> = HashMap::new();
        let mut evaluation_stack = HashSet::new();
        let mut call_path = Vec::new();

        let rule_set = RuleSet { rules: vec![], rule_map, label_map, cache: crate::runner::model::PerformanceCache::new() };
        let (result, _trace) = evaluate_rule(&rule, &json, &rule_set, &mut evaluation_stack, &mut call_path).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_multiple_conditions_with_and() {
        let json = json!({
        "user": {
            "age": 25,
            "name": "John",
            "active": true
        }
    });

        let rule = Rule {
            label: Some("adult active user".to_string()),
            selector: "user".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "user".to_string(), pos: None },
                        property: PositionedValue { value: "age".to_string(), pos: None },
                        operator: ComparisonOperator::GreaterThanOrEqual,
                        value: PositionedValue { value: RuleValue::Number(18.0), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: None, // Remove the operator from the first condition
                },
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "user".to_string(), pos: None },
                        property: PositionedValue { value: "active".to_string(), pos: None },
                        operator: ComparisonOperator::EqualTo,
                        value: PositionedValue { value: RuleValue::Boolean(true), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: Some(ConditionOperator::And), // Move the operator to the second condition
                }
            ],
            outcome: "valid_user".to_string(),
            position: None,
        };

        let rule_map: HashMap<String, usize> = HashMap::new();
        let label_map: HashMap<String, usize> = HashMap::new();

        let mut evaluation_stack = HashSet::new();
        let mut call_path = Vec::new();

        let rule_set = RuleSet { rules: vec![], rule_map, label_map, cache: crate::runner::model::PerformanceCache::new() };
        let (result, _trace) = evaluate_rule(&rule, &json, &rule_set, &mut evaluation_stack, &mut call_path).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_multiple_conditions_with_or() {
        let json = json!({
            "user": {
                "age": 16,
                "vip": true
            }
        });

        let rule = Rule {
            label: Some("eligible user".to_string()),
            selector: "user".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "user".to_string(), pos: None },
                        property: PositionedValue { value: "age".to_string(), pos: None },
                        operator: ComparisonOperator::GreaterThanOrEqual,
                        value: PositionedValue { value: RuleValue::Number(18.0), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: None,
                },
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "user".to_string(), pos: None },
                        property: PositionedValue { value: "vip".to_string(), pos: None },
                        operator: ComparisonOperator::EqualTo,
                        value: PositionedValue { value: RuleValue::Boolean(true), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: Some(ConditionOperator::Or),
                }
            ],
            outcome: "eligible".to_string(),
            position: None,
        };

        let rule_map: HashMap<String, usize> = HashMap::new();
        let label_map: HashMap<String, usize> = HashMap::new();

        let mut evaluation_stack = HashSet::new();
        let mut call_path = Vec::new();

        let rule_set = RuleSet { rules: vec![], rule_map, label_map, cache: crate::runner::model::PerformanceCache::new() };
        let (result, _trace) = evaluate_rule(&rule, &json, &rule_set, &mut evaluation_stack, &mut call_path).unwrap();
        assert_eq!(result, true); // Should be true because vip is true, even though age < 18
    }

    #[test]
    fn test_rule_reference_condition() {
        let json = json!({
            "user": { "age": 25 },
            "account": { "type": "premium" }
        });

        // Create referenced rule
        let age_rule = Rule {
            label: Some("age check".to_string()),
            selector: "user".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "user".to_string(), pos: None },
                        property: PositionedValue { value: "age".to_string(), pos: None },
                        operator: ComparisonOperator::GreaterThanOrEqual,
                        value: PositionedValue { value: RuleValue::Number(18.0), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: None,
                }
            ],
            outcome: "adult".to_string(),
            position: None,
        };

        // Create main rule that references the age rule
        let main_rule = Rule {
            label: Some("main rule".to_string()),
            selector: "global".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::RuleReference(RuleReferenceCondition {
                        selector: PositionedValue { value: "".to_string(), pos: None },
                        rule_name: PositionedValue { value: "adult".to_string(), pos: None },
                    }),
                    operator: None,
                }
            ],
            outcome: "global".to_string(),
            position: None,
        };

        let mut rule_map = HashMap::new();
        rule_map.insert("adult".to_string(), 0);    // age_rule is at index 0
        rule_map.insert("global".to_string(), 1);   // main_rule is at index 1

        let mut label_map = HashMap::new();
        label_map.insert("age check".to_string(), 0);
        label_map.insert("main rule".to_string(), 1);

        let mut evaluation_stack = HashSet::new();
        let mut call_path = Vec::new();

        let rule_set = RuleSet {
            rules: vec![age_rule, main_rule.clone()],
            rule_map,
            label_map,
            cache: crate::runner::model::PerformanceCache::new()
        };

        let (result, _trace) = evaluate_rule(&main_rule, &json, &rule_set, &mut evaluation_stack, &mut call_path).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_rule_set_evaluation() {
        let json = json!({
            "user": { "age": 25, "active": true }
        });

        let age_rule = Rule {
            label: Some("age check".to_string()),
            selector: "user".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "user".to_string(), pos: None },
                        property: PositionedValue { value: "age".to_string(), pos: None },
                        operator: ComparisonOperator::GreaterThanOrEqual,
                        value: PositionedValue { value: RuleValue::Number(18.0), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: None,
                }
            ],
            outcome: "adult".to_string(),
            position: None,
        };

        let global_rule = Rule {
            label: Some("global rule".to_string()),
            selector: "global".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::RuleReference(RuleReferenceCondition {
                        selector: PositionedValue { value: "".to_string(), pos: None },
                        rule_name: PositionedValue { value: "adult".to_string(), pos: None },
                    }),
                    operator: None,
                }
            ],
            outcome: "global".to_string(),
            position: None,
        };

        let mut rule_map = HashMap::new();
        rule_map.insert("adult".to_string(), 0);    // age_rule is at index 0
        rule_map.insert("global".to_string(), 1);   // global_rule is at index 1

        let mut label_map = HashMap::new();
        label_map.insert("age check".to_string(), 0);
        label_map.insert("global rule".to_string(), 1);

        let rule_set = RuleSet {
            rules: vec![age_rule, global_rule],
            rule_map,
            label_map,
            cache: crate::runner::model::PerformanceCache::new()
        };

        let (results, _trace) = evaluate_rule_set(&rule_set, &json).unwrap();

        assert_eq!(results.get("global"), Some(&true));
        assert_eq!(results.get("adult"), Some(&true));
    }

    #[test]
    fn test_cross_object_comparison() {
        let json = json!({
            "user": { "age": 25 },
            "requirement": { "minAge": 18 }
        });

        let condition = ComparisonCondition {
            selector: PositionedValue { value: "user".to_string(), pos: None },
            property: PositionedValue { value: "age".to_string(), pos: None },
            operator: ComparisonOperator::GreaterThanOrEqual,
            value: PositionedValue { value: RuleValue::Number(0.0), pos: None }, // Not used in cross-object
            left_property_path: Some(PropertyPath {
                selector: "user".to_string(),
                properties: vec!["age".to_string()],
            }),
            right_property_path: Some(PropertyPath {
                selector: "requirement".to_string(),
                properties: vec!["minAge".to_string()],
            }),
            property_chain: None,
        };

        let (result, _trace) = evaluate_comparison_condition(&condition, &json).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_property_chain_access() {
        let json = json!({
            "user": {
                "profile": {
                    "settings": {
                        "theme": "dark"
                    }
                }
            }
        });

        let condition = ComparisonCondition {
            selector: PositionedValue { value: "user".to_string(), pos: None },
            property: PositionedValue { value: "theme".to_string(), pos: None },
            operator: ComparisonOperator::EqualTo,
            value: PositionedValue { value: RuleValue::String("dark".to_string()), pos: None },
            left_property_path: None,
            right_property_path: None,
            property_chain: Some(vec![
                PropertyChainElement::Property("profile".to_string()),
                PropertyChainElement::Property("settings".to_string()),
            ]),
        };

        let (result, _trace) = evaluate_comparison_condition(&condition, &json).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_case_insensitive_json_access() {
        let json = json!({
            "User": {
                "Name": "John",
                "AGE": 25
            }
        });

        // Test case-insensitive selector access
        let result = find_effective_selector("user", &json).unwrap();
        assert_eq!(result, Some("User".to_string()));

        // Test case-insensitive property access
        let result = extract_value_from_json(&json, "User", "name").unwrap();
        assert!(matches!(result, RuleValue::String(s) if s == "John"));

        let result = extract_value_from_json(&json, "User", "age").unwrap();
        assert!(matches!(result, RuleValue::Number(25.0)));
    }

    #[test]
    fn test_date_string_coercion() {
        let date1 = RuleValue::String("2023-01-01".to_string());
        let date2 = RuleValue::Date(NaiveDate::from_ymd_opt(2023, 6, 15).unwrap());

        assert_eq!(compare_dates_earlier(&date1, &date2).unwrap(), true);
        assert_eq!(compare_dates_later(&date2, &date1).unwrap(), true);
    }

    #[test]
    fn test_string_contains_operation() {
        let haystack = RuleValue::String("Hello World".to_string());
        let needle = RuleValue::String("World".to_string());
        let missing = RuleValue::String("Missing".to_string());

        assert_eq!(compare_contains(&haystack, &needle).unwrap(), true);
        assert_eq!(compare_contains(&haystack, &missing).unwrap(), false);
    }

    #[test]
    fn test_complex_list_operations() {
        let mixed_list = RuleValue::List(vec![
            RuleValue::String("apple".to_string()),
            RuleValue::Number(42.0),
            RuleValue::Boolean(true),
        ]);

        let string_val = RuleValue::String("apple".to_string());
        let number_val = RuleValue::Number(42.0);
        let bool_val = RuleValue::Boolean(true);
        let missing_val = RuleValue::String("orange".to_string());

        assert_eq!(compare_contains(&mixed_list, &string_val).unwrap(), true);
        assert_eq!(compare_contains(&mixed_list, &number_val).unwrap(), true);
        assert_eq!(compare_contains(&mixed_list, &bool_val).unwrap(), true);
        assert_eq!(compare_contains(&mixed_list, &missing_val).unwrap(), false);

        assert_eq!(compare_in_list(&string_val, &mixed_list).unwrap(), true);
        assert_eq!(compare_not_in_list(&missing_val, &mixed_list).unwrap(), true);
    }

    #[test]
    fn test_failed_comparisons() {
        let json = json!({
            "user": {
                "name": "John"
            }
        });

        // Test with non-existent selector
        let condition = ComparisonCondition {
            selector: PositionedValue { value: "nonexistent".to_string(), pos: None },
            property: PositionedValue { value: "name".to_string(), pos: None },
            operator: ComparisonOperator::EqualTo,
            value: PositionedValue { value: RuleValue::String("John".to_string()), pos: None },
            left_property_path: None,
            right_property_path: None,
            property_chain: None,
        };

        let (result, _trace) = evaluate_comparison_condition(&condition, &json).unwrap();
        assert_eq!(result, false);

        // Test with non-existent property
        let condition = ComparisonCondition {
            selector: PositionedValue { value: "user".to_string(), pos: None },
            property: PositionedValue { value: "nonexistent".to_string(), pos: None },
            operator: ComparisonOperator::EqualTo,
            value: PositionedValue { value: RuleValue::String("John".to_string()), pos: None },
            left_property_path: None,
            right_property_path: None,
            property_chain: None,
        };

        let (result, _trace) = evaluate_comparison_condition(&condition, &json).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_error_cases() {
        // Test type mismatch in comparisons
        let string_val = RuleValue::String("hello".to_string());
        let number_val = RuleValue::Number(42.0);

        // Should error when trying to do numeric comparison with string
        let result = compare_numbers_gt(&string_val, &number_val);
        assert!(result.is_err());

        // Should error when trying to compare incompatible types for equality
        let result = compare_equal(&string_val, &number_val);
        assert!(result.is_err());
    }

    #[test]
    fn test_infinite_loop_detection() {
        let json = json!({
        "person": {
            "age": 25,
            "driving_test_score": 85
        }
    });

        // Create the cyclic rules from your example

        // Rule 1: A **person** follows rule 1 if age >= 18 and follows rule 2
        let rule1 = Rule {
            label: None,
            selector: "person".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "person".to_string(), pos: None },
                        property: PositionedValue { value: "age".to_string(), pos: None },
                        operator: ComparisonOperator::GreaterThanOrEqual,
                        value: PositionedValue { value: RuleValue::Number(18.0), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: None,
                },
                ConditionGroup {
                    condition: Condition::RuleReference(RuleReferenceCondition {
                        selector: PositionedValue { value: "".to_string(), pos: None },
                        rule_name: PositionedValue { value: "rule 2".to_string(), pos: None },
                    }),
                    operator: Some(ConditionOperator::And),
                }
            ],
            outcome: "rule 1".to_string(),
            position: None,
        };

        // Rule 2: A **person** follows rule 2 if driving_test_score >= 60 and follows rule 3
        let rule2 = Rule {
            label: None,
            selector: "person".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "person".to_string(), pos: None },
                        property: PositionedValue { value: "driving_test_score".to_string(), pos: None },
                        operator: ComparisonOperator::GreaterThanOrEqual,
                        value: PositionedValue { value: RuleValue::Number(60.0), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: None,
                },
                ConditionGroup {
                    condition: Condition::RuleReference(RuleReferenceCondition {
                        selector: PositionedValue { value: "".to_string(), pos: None },
                        rule_name: PositionedValue { value: "rule 3".to_string(), pos: None },
                    }),
                    operator: Some(ConditionOperator::And),
                }
            ],
            outcome: "rule 2".to_string(),
            position: None,
        };

        // Rule 3: A **person** follows rule 3 if passes eye test and follows rule 1 (CYCLE!)
        let rule3 = Rule {
            label: None,
            selector: "person".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::RuleReference(RuleReferenceCondition {
                        selector: PositionedValue { value: "person".to_string(), pos: None },
                        rule_name: PositionedValue { value: "passes an eye test".to_string(), pos: None },
                    }),
                    operator: None,
                },
                ConditionGroup {
                    condition: Condition::RuleReference(RuleReferenceCondition {
                        selector: PositionedValue { value: "".to_string(), pos: None },
                        rule_name: PositionedValue { value: "rule 1".to_string(), pos: None },
                    }),
                    operator: Some(ConditionOperator::And),
                }
            ],
            outcome: "rule 3".to_string(),
            position: None,
        };

        // Global rule that starts the evaluation
        let global_rule = Rule {
            label: None,
            selector: "person".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::RuleReference(RuleReferenceCondition {
                        selector: PositionedValue { value: "".to_string(), pos: None },
                        rule_name: PositionedValue { value: "rule 1".to_string(), pos: None },
                    }),
                    operator: None,
                }
            ],
            outcome: "full driving license".to_string(),
            position: None,
        };

        // Set up the rule set
        let mut rule_map = HashMap::new();
        rule_map.insert("rule 1".to_string(), 0);
        rule_map.insert("rule 2".to_string(), 1);
        rule_map.insert("rule 3".to_string(), 2);
        rule_map.insert("full driving license".to_string(), 3);

        let rule_set = RuleSet {
            rules: vec![rule1, rule2, rule3, global_rule],
            rule_map,
            label_map: HashMap::new(),
            cache: crate::runner::model::PerformanceCache::new(),
        };

        // Test that cycle detection catches the infinite loop
        let result = evaluate_rule_set(&rule_set, &json);

        match result {
            Err(RuleError::EvaluationError(msg)) => {
                // Check that the error message contains information about the cycle
                assert!(msg.contains("Infinite loop detected"));
                assert!(msg.contains("rule 1"));
                assert!(msg.contains("rule 2"));
                assert!(msg.contains("rule 3"));
            },
            Ok(_) => {
                panic!("Expected infinite loop error, but evaluation succeeded");
            },
            Err(other_error) => {
                panic!("Expected infinite loop error, but got: {:?}", other_error);
            }
        }
    }

    #[test]
    fn test_empty_string_operations() {
        let empty_string = RuleValue::String("".to_string());
        let non_empty_string = RuleValue::String("hello".to_string());

        // Test is empty
        assert_eq!(compare_is_empty(&empty_string).unwrap(), true);
        assert_eq!(compare_is_empty(&non_empty_string).unwrap(), false);

        // Test is not empty
        assert_eq!(compare_is_not_empty(&empty_string).unwrap(), false);
        assert_eq!(compare_is_not_empty(&non_empty_string).unwrap(), true);
    }

    #[test]
    fn test_empty_list_operations() {
        let empty_list = RuleValue::List(vec![]);
        let non_empty_list = RuleValue::List(vec![
            RuleValue::String("item1".to_string()),
            RuleValue::Number(42.0),
        ]);

        // Test is empty
        assert_eq!(compare_is_empty(&empty_list).unwrap(), true);
        assert_eq!(compare_is_empty(&non_empty_list).unwrap(), false);

        // Test is not empty
        assert_eq!(compare_is_not_empty(&empty_list).unwrap(), false);
        assert_eq!(compare_is_not_empty(&non_empty_list).unwrap(), true);
    }

    #[test]
    fn test_empty_operations_with_unsupported_types() {
        let number_value = RuleValue::Number(42.0);
        let boolean_value = RuleValue::Boolean(true);
        let date_value = RuleValue::Date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());

        // Test that unsupported types return errors
        assert!(compare_is_empty(&number_value).is_err());
        assert!(compare_is_empty(&boolean_value).is_err());
        assert!(compare_is_empty(&date_value).is_err());

        assert!(compare_is_not_empty(&number_value).is_err());
        assert!(compare_is_not_empty(&boolean_value).is_err());
        assert!(compare_is_not_empty(&date_value).is_err());
    }

    #[test]
    fn test_no_false_positive_cycle_detection() {
        let json = json!({
        "person": {
            "age": 25,
            "has_license": true
        }
    });

        // Create a rule that references another rule but doesn't create a cycle
        let age_check_rule = Rule {
            label: None,
            selector: "person".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "person".to_string(), pos: None },
                        property: PositionedValue { value: "age".to_string(), pos: None },
                        operator: ComparisonOperator::GreaterThanOrEqual,
                        value: PositionedValue { value: RuleValue::Number(18.0), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: None,
                }
            ],
            outcome: "is adult".to_string(),
            position: None,
        };

        let main_rule = Rule {
            label: None,
            selector: "person".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::RuleReference(RuleReferenceCondition {
                        selector: PositionedValue { value: "".to_string(), pos: None },
                        rule_name: PositionedValue { value: "is adult".to_string(), pos: None },
                    }),
                    operator: None,
                },
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "person".to_string(), pos: None },
                        property: PositionedValue { value: "has_license".to_string(), pos: None },
                        operator: ComparisonOperator::EqualTo,
                        value: PositionedValue { value: RuleValue::Boolean(true), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: Some(ConditionOperator::And),
                }
            ],
            outcome: "can drive".to_string(),
            position: None,
        };

        let mut rule_map = HashMap::new();
        rule_map.insert("is adult".to_string(), 0);
        rule_map.insert("can drive".to_string(), 1);

        let rule_set = RuleSet {
            rules: vec![age_check_rule, main_rule],
            rule_map,
            label_map: HashMap::new(),
            cache: crate::runner::model::PerformanceCache::new(),
        };

        // This should succeed without any cycle detection errors
        let result = evaluate_rule_set(&rule_set, &json);

        match result {
            Ok((results, _trace)) => {
                assert_eq!(results.get("can drive"), Some(&true));
                assert_eq!(results.get("is adult"), Some(&true));
            },
            Err(e) => {
                panic!("Valid rule evaluation should not fail: {:?}", e);
            }
        }
    }

    // ERROR TRACE TESTS - Testing the new error tracing functionality

    #[test]
    fn test_error_trace_with_missing_selector() {
        let json = json!({
            "user": {
                "age": 25,
                "name": "John"
            }
        });

        let rule = Rule {
            label: Some("test rule".to_string()),
            selector: "nonexistent_selector".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "nonexistent_selector".to_string(), pos: None },
                        property: PositionedValue { value: "age".to_string(), pos: None },
                        operator: ComparisonOperator::GreaterThan,
                        value: PositionedValue { value: RuleValue::Number(18.0), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: None,
                }
            ],
            outcome: "adult".to_string(),
            position: None,
        };

        let rule_set = RuleSet {
            rules: vec![rule.clone()],
            rule_map: {
                let mut map = HashMap::new();
                map.insert("adult".to_string(), 0);
                map
            },
            label_map: HashMap::new(),
            cache: crate::runner::model::PerformanceCache::new(),
        };

        // Test with trace-preserving evaluation
        let result = evaluate_rule_set_with_trace(&rule_set, &json);

        // Should succeed but with false result (missing selectors return false, not error)
        assert!(result.is_success());
        assert!(result.trace.is_some());

        let trace = result.unwrap_trace();
        assert_eq!(trace.execution.len(), 1);
        assert_eq!(trace.execution[0].result, false);
        assert_eq!(trace.execution[0].outcome.value, "adult");
        assert!(!trace.execution[0].conditions.is_empty());
    }

    #[test]
    fn test_error_trace_with_missing_property() {
        let json = json!({
            "user": {
                "name": "John"
                // Missing "age" property
            }
        });

        let rule = Rule {
            label: Some("age check".to_string()),
            selector: "user".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "user".to_string(), pos: None },
                        property: PositionedValue { value: "age".to_string(), pos: None },
                        operator: ComparisonOperator::GreaterThan,
                        value: PositionedValue { value: RuleValue::Number(18.0), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: None,
                }
            ],
            outcome: "adult".to_string(),
            position: None,
        };

        let rule_set = RuleSet {
            rules: vec![rule.clone()],
            rule_map: {
                let mut map = HashMap::new();
                map.insert("adult".to_string(), 0);
                map
            },
            label_map: HashMap::new(),
            cache: crate::runner::model::PerformanceCache::new(),
        };

        // Test with trace-preserving evaluation
        let result = evaluate_rule_set_with_trace(&rule_set, &json);

        // Should succeed but with false result (missing properties return false, not error)
        assert!(result.is_success());
        assert!(result.trace.is_some());

        let trace = result.unwrap_trace();
        assert_eq!(trace.execution.len(), 1);
        assert_eq!(trace.execution[0].result, false);
        assert_eq!(trace.execution[0].outcome.value, "adult");
        assert_eq!(trace.execution[0].conditions.len(), 1);
    }

    #[test]
    fn test_error_trace_with_infinite_loop() {
        let json = json!({
            "person": {
                "age": 25,
                "driving_test_score": 85
            }
        });

        // Create cyclic rules that will cause infinite loop
        let rule1 = Rule {
            label: None,
            selector: "person".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "person".to_string(), pos: None },
                        property: PositionedValue { value: "age".to_string(), pos: None },
                        operator: ComparisonOperator::GreaterThanOrEqual,
                        value: PositionedValue { value: RuleValue::Number(18.0), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: None,
                },
                ConditionGroup {
                    condition: Condition::RuleReference(RuleReferenceCondition {
                        selector: PositionedValue { value: "".to_string(), pos: None },
                        rule_name: PositionedValue { value: "rule 2".to_string(), pos: None },
                    }),
                    operator: Some(ConditionOperator::And),
                }
            ],
            outcome: "rule 1".to_string(),
            position: None,
        };

        let rule2 = Rule {
            label: None,
            selector: "person".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::RuleReference(RuleReferenceCondition {
                        selector: PositionedValue { value: "".to_string(), pos: None },
                        rule_name: PositionedValue { value: "rule 1".to_string(), pos: None },
                    }),
                    operator: None,
                }
            ],
            outcome: "rule 2".to_string(),
            position: None,
        };

        let global_rule = Rule {
            label: None,
            selector: "person".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::RuleReference(RuleReferenceCondition {
                        selector: PositionedValue { value: "".to_string(), pos: None },
                        rule_name: PositionedValue { value: "rule 1".to_string(), pos: None },
                    }),
                    operator: None,
                }
            ],
            outcome: "global".to_string(),
            position: None,
        };

        let mut rule_map = HashMap::new();
        rule_map.insert("rule 1".to_string(), 0);
        rule_map.insert("rule 2".to_string(), 1);
        rule_map.insert("global".to_string(), 2);

        let rule_set = RuleSet {
            rules: vec![rule1, rule2, global_rule],
            rule_map,
            label_map: HashMap::new(),
            cache: crate::runner::model::PerformanceCache::new(),
        };

        // Test with trace-preserving evaluation
        let result = evaluate_rule_set_with_trace(&rule_set, &json);

        // Should have error but also trace
        assert!(result.is_failure());
        assert!(result.trace.is_some());

        // Verify error is about infinite loop
        if let Err(RuleError::EvaluationError(msg)) = &result.result {
            assert!(msg.contains("Infinite loop detected"));
        } else {
            panic!("Expected infinite loop error");
        }

        // Verify trace contains partial execution information
        let trace = result.unwrap_trace();
        assert!(!trace.execution.is_empty());
        
        // The trace should contain the partial evaluation before the loop was detected
        let first_rule_trace = &trace.execution[0];
        assert_eq!(first_rule_trace.outcome.value, "global");
        assert_eq!(first_rule_trace.result, false);
    }

    #[test]
    fn test_error_trace_with_rule_reference_failure() {
        let json = json!({
            "user": {
                "age": 25
            }
        });

        // Create a rule that references a non-existent rule
        let main_rule = Rule {
            label: Some("main rule".to_string()),
            selector: "user".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::RuleReference(RuleReferenceCondition {
                        selector: PositionedValue { value: "".to_string(), pos: None },
                        rule_name: PositionedValue { value: "nonexistent_rule".to_string(), pos: None },
                    }),
                    operator: None,
                }
            ],
            outcome: "result".to_string(),
            position: None,
        };

        let global_rule = Rule {
            label: None,
            selector: "global".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::RuleReference(RuleReferenceCondition {
                        selector: PositionedValue { value: "".to_string(), pos: None },
                        rule_name: PositionedValue { value: "result".to_string(), pos: None },
                    }),
                    operator: None,
                }
            ],
            outcome: "global".to_string(),
            position: None,
        };

        let mut rule_map = HashMap::new();
        rule_map.insert("result".to_string(), 0);
        rule_map.insert("global".to_string(), 1);

        let rule_set = RuleSet {
            rules: vec![main_rule, global_rule],
            rule_map,
            label_map: HashMap::new(),
            cache: crate::runner::model::PerformanceCache::new(),
        };

        // Test with trace-preserving evaluation
        let result = evaluate_rule_set_with_trace(&rule_set, &json);

        // Should succeed (non-existent rules default to false, not error)
        assert!(result.is_success());
        assert!(result.trace.is_some());

        let trace = result.unwrap_trace();
        // Should have traces for both rules
        assert_eq!(trace.execution.len(), 2);
        
        // Global rule should be false because referenced rule is false
        let global_trace = &trace.execution[0];
        assert_eq!(global_trace.outcome.value, "global");
        assert_eq!(global_trace.result, false);

        // Main rule should be false because it references non-existent rule
        let main_trace = &trace.execution[1];
        assert_eq!(main_trace.outcome.value, "result");
        assert_eq!(main_trace.result, false);
    }

    #[test]
    fn test_error_trace_preserves_successful_conditions_before_failure() {
        let json = json!({
            "user": {
                "age": 25,
                "active": true
                // Missing "score" property
            }
        });

        let rule = Rule {
            label: Some("complex rule".to_string()),
            selector: "user".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "user".to_string(), pos: None },
                        property: PositionedValue { value: "age".to_string(), pos: None },
                        operator: ComparisonOperator::GreaterThan,
                        value: PositionedValue { value: RuleValue::Number(18.0), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: None,
                },
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "user".to_string(), pos: None },
                        property: PositionedValue { value: "active".to_string(), pos: None },
                        operator: ComparisonOperator::EqualTo,
                        value: PositionedValue { value: RuleValue::Boolean(true), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: Some(ConditionOperator::And),
                },
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "user".to_string(), pos: None },
                        property: PositionedValue { value: "score".to_string(), pos: None },
                        operator: ComparisonOperator::GreaterThan,
                        value: PositionedValue { value: RuleValue::Number(80.0), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: Some(ConditionOperator::And),
                }
            ],
            outcome: "qualified".to_string(),
            position: None,
        };

        let rule_set = RuleSet {
            rules: vec![rule.clone()],
            rule_map: {
                let mut map = HashMap::new();
                map.insert("qualified".to_string(), 0);
                map
            },
            label_map: HashMap::new(),
            cache: crate::runner::model::PerformanceCache::new(),
        };

        // Test with trace-preserving evaluation
        let result = evaluate_rule_set_with_trace(&rule_set, &json);

        // Should succeed but with false result (missing score property)
        assert!(result.is_success());
        assert!(result.trace.is_some());

        let trace = result.unwrap_trace();
        assert_eq!(trace.execution.len(), 1);
        
        let rule_trace = &trace.execution[0];
        assert_eq!(rule_trace.outcome.value, "qualified");
        assert_eq!(rule_trace.result, false); // AND operation fails due to missing score
        
        // Should have all three condition traces
        assert_eq!(rule_trace.conditions.len(), 3);
        
        // Verify the first two conditions were evaluated successfully
        // (even though final result is false due to missing property)
        use crate::runner::trace::ConditionTrace;
        
        if let ConditionTrace::Comparison(comp_trace) = &rule_trace.conditions[0] {
            assert_eq!(comp_trace.result, true); // age > 18
        } else {
            panic!("Expected comparison trace");
        }
        
        if let ConditionTrace::Comparison(comp_trace) = &rule_trace.conditions[1] {
            assert_eq!(comp_trace.result, true); // active == true
        } else {
            panic!("Expected comparison trace");
        }
        
        if let ConditionTrace::Comparison(comp_trace) = &rule_trace.conditions[2] {
            assert_eq!(comp_trace.result, false); // score is missing
        } else {
            panic!("Expected comparison trace");
        }
    }

    #[test]
    fn test_error_trace_individual_rule_evaluation() {
        let json = json!({
            "user": {
                "age": 25
            }
        });

        let rule = Rule {
            label: Some("age check".to_string()),
            selector: "user".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "user".to_string(), pos: None },
                        property: PositionedValue { value: "age".to_string(), pos: None },
                        operator: ComparisonOperator::GreaterThan,
                        value: PositionedValue { value: RuleValue::Number(18.0), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: None,
                }
            ],
            outcome: "adult".to_string(),
            position: None,
        };

        let rule_set = RuleSet {
            rules: vec![],
            rule_map: HashMap::new(),
            label_map: HashMap::new(),
            cache: crate::runner::model::PerformanceCache::new(),
        };

        let mut evaluation_stack = HashSet::new();
        let mut call_path = Vec::new();

        // Test individual rule evaluation with trace
        let result = evaluate_rule_with_trace(&rule, &json, &rule_set, &mut evaluation_stack, &mut call_path);

        // Should succeed
        assert!(result.is_ok());
        let (rule_result, rule_trace) = result.unwrap();
        assert_eq!(rule_result, true);
        assert_eq!(rule_trace.outcome.value, "adult");
        assert_eq!(rule_trace.result, true);
        assert_eq!(rule_trace.conditions.len(), 1);
    }

    #[test]
    fn test_case_insensitive_string_comparison() {
        let json = json!({
            "user": {
                "name": "John",
                "status": "ACTIVE"
            }
        });

        // Test case-insensitive equality (default behavior)
        let condition = ComparisonCondition {
            selector: PositionedValue { value: "user".to_string(), pos: None },
            property: PositionedValue { value: "name".to_string(), pos: None },
            operator: ComparisonOperator::EqualTo,
            value: PositionedValue { value: RuleValue::String("JOHN".to_string()), pos: None },
            left_property_path: None,
            right_property_path: None,
            property_chain: None,
        };

        let (result, _trace) = evaluate_comparison_condition(&condition, &json).unwrap();
        assert_eq!(result, true); // Should match case-insensitively

        // Test case-insensitive with different case
        let condition = ComparisonCondition {
            selector: PositionedValue { value: "user".to_string(), pos: None },
            property: PositionedValue { value: "status".to_string(), pos: None },
            operator: ComparisonOperator::EqualTo,
            value: PositionedValue { value: RuleValue::String("active".to_string()), pos: None },
            left_property_path: None,
            right_property_path: None,
            property_chain: None,
        };

        let (result, _trace) = evaluate_comparison_condition(&condition, &json).unwrap();
        assert_eq!(result, true); // Should match case-insensitively
    }

    #[test]
    fn test_case_sensitive_exactly_equal_comparison() {
        let json = json!({
            "user": {
                "name": "John",
                "status": "ACTIVE"
            }
        });

        // Test case-sensitive exact equality
        let condition = ComparisonCondition {
            selector: PositionedValue { value: "user".to_string(), pos: None },
            property: PositionedValue { value: "name".to_string(), pos: None },
            operator: ComparisonOperator::ExactlyEqualTo,
            value: PositionedValue { value: RuleValue::String("John".to_string()), pos: None },
            left_property_path: None,
            right_property_path: None,
            property_chain: None,
        };

        let (result, _trace) = evaluate_comparison_condition(&condition, &json).unwrap();
        assert_eq!(result, true); // Should match exactly

        // Test case-sensitive exact equality that should fail
        let condition = ComparisonCondition {
            selector: PositionedValue { value: "user".to_string(), pos: None },
            property: PositionedValue { value: "name".to_string(), pos: None },
            operator: ComparisonOperator::ExactlyEqualTo,
            value: PositionedValue { value: RuleValue::String("JOHN".to_string()), pos: None },
            left_property_path: None,
            right_property_path: None,
            property_chain: None,
        };

        let (result, _trace) = evaluate_comparison_condition(&condition, &json).unwrap();
        assert_eq!(result, false); // Should NOT match due to case difference

        // Test case-sensitive exact equality with status
        let condition = ComparisonCondition {
            selector: PositionedValue { value: "user".to_string(), pos: None },
            property: PositionedValue { value: "status".to_string(), pos: None },
            operator: ComparisonOperator::ExactlyEqualTo,
            value: PositionedValue { value: RuleValue::String("active".to_string()), pos: None },
            left_property_path: None,
            right_property_path: None,
            property_chain: None,
        };

        let (result, _trace) = evaluate_comparison_condition(&condition, &json).unwrap();
        assert_eq!(result, false); // Should NOT match due to case difference
    }

    #[test]
    fn test_case_insensitive_contains_operation() {
        let json = json!({
            "user": {
                "description": "This user is a PREMIUM member"
            }
        });

        // Test case-insensitive contains
        let condition = ComparisonCondition {
            selector: PositionedValue { value: "user".to_string(), pos: None },
            property: PositionedValue { value: "description".to_string(), pos: None },
            operator: ComparisonOperator::Contains,
            value: PositionedValue { value: RuleValue::String("premium".to_string()), pos: None },
            left_property_path: None,
            right_property_path: None,
            property_chain: None,
        };

        let (result, _trace) = evaluate_comparison_condition(&condition, &json).unwrap();
        assert_eq!(result, true); // Should match case-insensitively

        // Test case-insensitive contains with different case
        let condition = ComparisonCondition {
            selector: PositionedValue { value: "user".to_string(), pos: None },
            property: PositionedValue { value: "description".to_string(), pos: None },
            operator: ComparisonOperator::Contains,
            value: PositionedValue { value: RuleValue::String("USER".to_string()), pos: None },
            left_property_path: None,
            right_property_path: None,
            property_chain: None,
        };

        let (result, _trace) = evaluate_comparison_condition(&condition, &json).unwrap();
        assert_eq!(result, true); // Should match case-insensitively
    }

    #[test]
    fn test_case_insensitive_list_operations() {
        let json = json!({
            "user": {
                "role": "ADMIN"
            }
        });

        // Test case-insensitive list membership
        let condition = ComparisonCondition {
            selector: PositionedValue { value: "user".to_string(), pos: None },
            property: PositionedValue { value: "role".to_string(), pos: None },
            operator: ComparisonOperator::In,
            value: PositionedValue { 
                value: RuleValue::List(vec![
                    RuleValue::String("admin".to_string()),
                    RuleValue::String("user".to_string()),
                    RuleValue::String("guest".to_string())
                ]), 
                pos: None 
            },
            left_property_path: None,
            right_property_path: None,
            property_chain: None,
        };

        let (result, _trace) = evaluate_comparison_condition(&condition, &json).unwrap();
        assert_eq!(result, true); // Should match case-insensitively
    }

    #[test]
    fn test_comparison_of_trace_preserving_vs_regular_evaluation() {
        let json = json!({
            "user": {
                "age": 25,
                "name": "John"
            }
        });

        let rule = Rule {
            label: Some("user check".to_string()),
            selector: "user".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::Comparison(ComparisonCondition {
                        selector: PositionedValue { value: "user".to_string(), pos: None },
                        property: PositionedValue { value: "age".to_string(), pos: None },
                        operator: ComparisonOperator::GreaterThan,
                        value: PositionedValue { value: RuleValue::Number(18.0), pos: None },
                        left_property_path: None,
                        right_property_path: None,
                        property_chain: None,
                    }),
                    operator: None,
                }
            ],
            outcome: "adult".to_string(),
            position: None,
        };

        let global_rule = Rule {
            label: None,
            selector: "global".to_string(),
            selector_pos: None,
            conditions: vec![
                ConditionGroup {
                    condition: Condition::RuleReference(RuleReferenceCondition {
                        selector: PositionedValue { value: "".to_string(), pos: None },
                        rule_name: PositionedValue { value: "adult".to_string(), pos: None },
                    }),
                    operator: None,
                }
            ],
            outcome: "global".to_string(),
            position: None,
        };

        let mut rule_map = HashMap::new();
        rule_map.insert("adult".to_string(), 0);
        rule_map.insert("global".to_string(), 1);

        let rule_set = RuleSet {
            rules: vec![rule, global_rule],
            rule_map,
            label_map: HashMap::new(),
            cache: crate::runner::model::PerformanceCache::new(),
        };

        // Test both evaluation methods
        let regular_result = evaluate_rule_set(&rule_set, &json);
        let trace_result = evaluate_rule_set_with_trace(&rule_set, &json);

        // Both should succeed with the same results
        assert!(regular_result.is_ok());
        assert!(trace_result.is_success());

        let (regular_results, regular_trace) = regular_result.unwrap();
        
        // Both should have traces, and they should be similar
        assert!(trace_result.trace.is_some());
        assert!(trace_result.is_success());
        
        let trace_results = trace_result.result.as_ref().unwrap();
        let enhanced_trace = trace_result.trace.as_ref().unwrap();

        // Results should be identical
        assert_eq!(&regular_results, trace_results);
        
        assert_eq!(regular_trace.execution.len(), enhanced_trace.execution.len());
        assert_eq!(regular_trace.execution[0].result, enhanced_trace.execution[0].result);
        assert_eq!(regular_trace.execution[1].result, enhanced_trace.execution[1].result);
    }
}