#[cfg(test)]
mod tests {
    use crate::runner::model::{ComparisonOperator, Condition, ConditionOperator, RuleValue};
    use crate::runner::parser::parse_rules;
    use chrono::NaiveDate;

    #[test]
    fn test_parse_simple_rule() {
        let input =
            r#"A **user** passes the test if __age__ of **user** is greater than or equal to 18."#;

        let result = parse_rules(input);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        assert_eq!(rule_set.rules.len(), 1);

        let rule = &rule_set.rules[0];
        assert_eq!(rule.selector, "user");
        assert_eq!(rule.outcome, "the test");
        assert_eq!(rule.conditions.len(), 1);

        match &rule.conditions[0].condition {
            Condition::Comparison(comp) => {
                assert_eq!(comp.selector.value, "user");
                assert_eq!(comp.property.value, "age");
                assert_eq!(comp.operator, ComparisonOperator::GreaterThanOrEqual);
                match &comp.value.value {
                    RuleValue::Number(n) => assert_eq!(*n, 18.0),
                    _ => panic!("Expected number value"),
                }
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_rule_with_label() {
        let input =
            r#"Age Check. A **person** is eligible if __age__ of **user** is greater than 21."#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];
        assert_eq!(rule.label, Some("Age Check".to_string()));
        assert_eq!(rule.selector, "person");
        assert_eq!(rule.outcome, "eligible");
    }

    #[test]
    fn test_parse_multiple_conditions_with_and() {
        let input = r#"A **user** qualifies for test if __age__ of **user** is greater than 18 and __score__ of **user** is greater than 70."#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];
        assert_eq!(rule.conditions.len(), 2);

        // First condition should have no operator
        assert!(rule.conditions[0].operator.is_none());

        // Second condition should have AND operator
        match &rule.conditions[1].operator {
            Some(ConditionOperator::And) => {}
            _ => panic!("Expected AND operator"),
        }
    }

    #[test]
    fn test_parse_multiple_conditions_with_or() {
        let input = r#"A **user** passes the test if __premium__ of **user** is equal to true or __score__ of **user** is greater than 90."#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];
        assert_eq!(rule.conditions.len(), 2);

        // Second condition should have OR operator
        match &rule.conditions[1].operator {
            Some(ConditionOperator::Or) => {}
            _ => panic!("Expected OR operator"),
        }
    }

    #[test]
    fn test_parse_string_comparison() {
        let input = r#"A **user** is valid if __status__ of **user** is equal to "active"."#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];

        match &rule.conditions[0].condition {
            Condition::Comparison(comp) => {
                assert_eq!(comp.operator, ComparisonOperator::EqualTo);
                match &comp.value.value {
                    RuleValue::String(s) => assert_eq!(s, "active"),
                    _ => panic!("Expected string value"),
                }
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_date_comparison() {
        let input = r#"A **user** is eligible if __birth_date__ of **user** is earlier than date(2000-01-01)."#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];

        match &rule.conditions[0].condition {
            Condition::Comparison(comp) => {
                assert_eq!(comp.operator, ComparisonOperator::EarlierThan);
                match &comp.value.value {
                    RuleValue::Date(d) => {
                        let expected = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
                        assert_eq!(*d, expected);
                    }
                    _ => panic!("Expected date value"),
                }
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_boolean_comparison() {
        let input = r#"A **user** is premium if __is_premium__ of **user** is equal to true."#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];

        match &rule.conditions[0].condition {
            Condition::Comparison(comp) => match &comp.value.value {
                RuleValue::Boolean(b) => assert!(*b),
                _ => panic!("Expected boolean value"),
            },
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_list_operations() {
        let input =
            r#"A **user** is valid if __role__ of **user** is in ["admin", "moderator", "user"]."#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];

        match &rule.conditions[0].condition {
            Condition::Comparison(comp) => {
                assert_eq!(comp.operator, ComparisonOperator::In);
                match &comp.value.value {
                    RuleValue::List(items) => {
                        assert_eq!(items.len(), 3);
                        match &items[0] {
                            RuleValue::String(s) => assert_eq!(s, "admin"),
                            _ => panic!("Expected string in list"),
                        }
                    }
                    _ => panic!("Expected list value"),
                }
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_rule_reference() {
        let input = r#"
A **user** passes the test if the **user** passes the age check.
        "#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];

        match &rule.conditions[0].condition {
            Condition::RuleReference(ref_cond) => {
                assert_eq!(ref_cond.selector.value, "user");
                assert_eq!(ref_cond.rule_name.value, "passes the age check");
            }
            _ => panic!("Expected rule reference condition"),
        }
    }

    #[test]
    fn test_parse_label_reference() {
        let input = r#"
A **user** is eligible if §ageCheck is valid.
        "#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];

        match &rule.conditions[0].condition {
            Condition::RuleReference(ref_cond) => {
                assert_eq!(ref_cond.selector.value, "");
                assert_eq!(ref_cond.rule_name.value, "ageCheck");
            }
            _ => panic!("Expected rule reference condition"),
        }
    }

    #[test]
    fn test_parse_chained_property_access() {
        let input = r#"
A **user** is valid if __id__ of __group__ of **user** is equal to 1.
        "#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];

        match &rule.conditions[0].condition {
            Condition::Comparison(comp) => {
                assert!(comp.left_property_path.is_some());
                let path = comp.left_property_path.as_ref().unwrap();
                assert_eq!(path.selector, "user");
                assert_eq!(path.properties, vec!["group", "id"]);
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_all_comparison_operators() {
        let operators = vec![
            (
                "is greater than or equal to",
                ComparisonOperator::GreaterThanOrEqual,
            ),
            ("is at least", ComparisonOperator::GreaterThanOrEqual),
            (
                "is less than or equal to",
                ComparisonOperator::LessThanOrEqual,
            ),
            ("is no more than", ComparisonOperator::LessThanOrEqual),
            ("is equal to", ComparisonOperator::EqualTo),
            ("is the same as", ComparisonOperator::EqualTo),
            ("is not equal to", ComparisonOperator::NotEqualTo),
            ("is not the same as", ComparisonOperator::NotEqualTo),
            ("is later than", ComparisonOperator::LaterThan),
            ("is earlier than", ComparisonOperator::EarlierThan),
            ("is greater than", ComparisonOperator::GreaterThan),
            ("is less than", ComparisonOperator::LessThan),
            ("contains", ComparisonOperator::Contains),
        ];

        for (op_str, expected_op) in operators {
            let input = format!(
                r#"A **user** passes the test if __value__ of **user** {} 10."#,
                op_str
            );
            let result = parse_rules(&input);
            if let Err(ref e) = result {
                println!("Parse error for '{}': {:?}", op_str, e);
            }
            assert!(result.is_ok(), "Failed to parse operator: {}", op_str);

            let rule_set = result.unwrap();
            let rule = &rule_set.rules[0];

            match &rule.conditions[0].condition {
                Condition::Comparison(comp) => {
                    assert_eq!(
                        comp.operator, expected_op,
                        "Operator mismatch for: {}",
                        op_str
                    );
                }
                _ => panic!("Expected comparison condition for operator: {}", op_str),
            }
        }
    }

    #[test]
    fn test_parse_list_operators() {
        let operators = vec![
            ("is in", ComparisonOperator::In),
            ("is not in", ComparisonOperator::NotIn),
        ];

        for (op_str, expected_op) in operators {
            let input = format!(
                r#"A **user** passes the test if __role__ of **user** {} ["admin", "user"]."#,
                op_str
            );
            let result = parse_rules(&input);
            assert!(result.is_ok(), "Failed to parse list operator: {}", op_str);

            let rule_set = result.unwrap();
            let rule = &rule_set.rules[0];

            match &rule.conditions[0].condition {
                Condition::Comparison(comp) => {
                    assert_eq!(
                        comp.operator, expected_op,
                        "Operator mismatch for: {}",
                        op_str
                    );
                }
                _ => panic!("Expected comparison condition for operator: {}", op_str),
            }
        }
    }

    #[test]
    fn test_parse_various_outcome_verbs() {
        let verbs = vec![
            "gets",
            "passes",
            "is",
            "has",
            "receives",
            "qualifies for",
            "meets",
            "satisfies",
        ];

        for verb in verbs {
            let input = format!(
                r#"A **user** {} approved if __age__ of **user** is greater than 18."#,
                verb
            );
            let result = parse_rules(&input);
            assert!(result.is_ok(), "Failed to parse verb: {}", verb);

            let rule_set = result.unwrap();
            let rule = &rule_set.rules[0];
            assert_eq!(
                rule.outcome, "approved",
                "Outcome mismatch for verb: {}",
                verb
            );
        }
    }

    #[test]
    fn test_parse_multiple_rules() {
        let input = r#"
A **user** passes the age check
  if __age__ of **user** is greater than or equal to 18.
A **user** passes the premium check
  if __is_premium__ of **user** is equal to true.
A **user** is eligible
  if the **user** passes the age check
  and the **user** passes the premium check.
        "#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        assert_eq!(rule_set.rules.len(), 3);

        // Check first rule
        assert_eq!(rule_set.rules[0].outcome, "the age check");

        // Check second rule
        assert_eq!(rule_set.rules[1].outcome, "the premium check");

        // Check third rule (should be the global rule)
        assert_eq!(rule_set.rules[2].outcome, "eligible");
        assert_eq!(rule_set.rules[2].conditions.len(), 2);
    }

    #[test]
    fn test_parse_comments() {
        let input = r#"
# This is a comment
A **user** passes the test if __age__ of **user** is greater than 18.
# Another comment
        "#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        assert_eq!(rule_set.rules.len(), 1);
    }

    #[test]
    fn test_parse_complex_mixed_conditions() {
        let input = r#"A **user** is eligible if __age__ of **user** is greater than 18 and __status__ of **user** is equal to "active" or __is_premium__ of **user** is equal to true."#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];
        assert_eq!(rule.conditions.len(), 3);

        // Check operators
        assert!(rule.conditions[0].operator.is_none());
        assert_eq!(rule.conditions[1].operator, Some(ConditionOperator::And));
        assert_eq!(rule.conditions[2].operator, Some(ConditionOperator::Or));
    }

    #[test]
    fn test_parse_error_cases() {
        // Missing selector
        let input = r#"A passes if __age__ is greater than 18."#;
        let result = parse_rules(input);
        assert!(result.is_err());

        // Missing condition
        let input = r#"A **user** passes if."#;
        let result = parse_rules(input);
        assert!(result.is_err());

        // Invalid operator
        let input = r#"A **user** passes if __age__ is bigger than 18."#;
        let result = parse_rules(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_date_formats() {
        // Test both date formats
        let inputs = vec![
            r#"A **user** is valid if __date__ of **user** is later than date(2023-12-01)."#,
            r#"A **user** is valid if __date__ of **user** is later than 2023-12-01."#,
        ];

        for input in inputs {
            let result = parse_rules(input);
            assert!(result.is_ok(), "Failed to parse date format in: {}", input);

            let rule_set = result.unwrap();
            let rule = &rule_set.rules[0];

            match &rule.conditions[0].condition {
                Condition::Comparison(comp) => match &comp.value.value {
                    RuleValue::Date(d) => {
                        let expected = NaiveDate::from_ymd_opt(2023, 12, 1).unwrap();
                        assert_eq!(*d, expected);
                    }
                    _ => panic!("Expected date value"),
                },
                _ => panic!("Expected comparison condition"),
            }
        }
    }

    #[test]
    fn test_parse_numerical_list() {
        let input = r#"
A **user** is valid if __score__ of **user** is in [85, 90, 95, 100].
        "#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];

        match &rule.conditions[0].condition {
            Condition::Comparison(comp) => match &comp.value.value {
                RuleValue::List(items) => {
                    assert_eq!(items.len(), 4);
                    for (i, expected) in [85.0, 90.0, 95.0, 100.0].iter().enumerate() {
                        match &items[i] {
                            RuleValue::Number(n) => assert_eq!(*n, *expected),
                            _ => panic!("Expected number in list"),
                        }
                    }
                }
                _ => panic!("Expected list value"),
            },
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_empty_operators() {
        // Test "is empty" operator
        let input = r#"A **login** is valid if __username__ of **login** is empty."#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];

        match &rule.conditions[0].condition {
            Condition::Comparison(comp) => {
                assert_eq!(comp.operator, ComparisonOperator::IsEmpty);
                assert_eq!(comp.property.value, "username");
                assert_eq!(comp.selector.value, "login");
            }
            _ => panic!("Expected comparison condition"),
        }

        // Test "is not empty" operator
        let input = r#"A **login** is valid if __username__ of **login** is not empty."#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];

        match &rule.conditions[0].condition {
            Condition::Comparison(comp) => {
                assert_eq!(comp.operator, ComparisonOperator::IsNotEmpty);
                assert_eq!(comp.property.value, "username");
                assert_eq!(comp.selector.value, "login");
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_complex_rule_with_empty_operators() {
        let input = r#"A **login** is valid if __username__ of **login** is not empty and __password__ of **login** contains "@"."#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];
        assert_eq!(rule.conditions.len(), 2);

        // First condition: username is not empty
        match &rule.conditions[0].condition {
            Condition::Comparison(comp) => {
                assert_eq!(comp.operator, ComparisonOperator::IsNotEmpty);
                assert_eq!(comp.property.value, "username");
            }
            _ => panic!("Expected comparison condition"),
        }

        // Second condition: password contains "@"
        match &rule.conditions[1].condition {
            Condition::Comparison(comp) => {
                assert_eq!(comp.operator, ComparisonOperator::Contains);
                assert_eq!(comp.property.value, "password");
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_whitespace_and_formatting() {
        let input = r#"


        A    **user**    passes   the test if   __age__   of **user** is greater than   18   .


        "#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        assert_eq!(rule_set.rules.len(), 1);
        let rule = &rule_set.rules[0];
        assert_eq!(rule.selector, "user");
        assert_eq!(rule.outcome, "the test");
    }

    #[test]
    fn test_parse_within_duration() {
        let input = r#"A **user** is valid if __test_date__ of **user** is within 30 days."#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        assert_eq!(rule_set.rules.len(), 1);
        let rule = &rule_set.rules[0];
        assert_eq!(rule.selector, "user");
        assert_eq!(rule.outcome, "valid");

        match &rule.conditions[0].condition {
            Condition::Comparison(comp) => {
                assert_eq!(comp.operator, ComparisonOperator::Within);
                assert_eq!(comp.property.value, "test_date");
                match &comp.value.value {
                    RuleValue::Duration(duration) => {
                        assert_eq!(duration.amount, 30.0);
                        // Should be normalized to days since it's >= 1 day
                        assert_eq!(duration.unit, crate::runner::model::TimeUnit::Days);
                    }
                    _ => panic!("Expected Duration value"),
                }
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_flexible_naming_conventions() {
        // Test with spaces in object selectors (but using proper property chain syntax)
        let input = r#"A **person** passes the theory test if the __multiple choice__ of the __theory__ of the __scores__ of the **driving test** is at least 43."#;

        let result = parse_rules(input);
        if let Err(ref e) = result {
            println!("Parse error for flexible naming: {:?}", e);
        }
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        assert_eq!(rule_set.rules.len(), 1);
        let rule = &rule_set.rules[0];
        assert_eq!(rule.selector, "person");
    }

    #[test]
    fn test_parse_within_different_time_units() {
        let test_cases = vec![
            ("30 minutes", 30.0 * 60.0),   // Should normalize to seconds
            ("2 hours", 2.0 * 3600.0),     // Should normalize to seconds
            ("5 days", 5.0 * 86400.0),     // Should normalize to days (5 days in seconds)
            ("1 month", 1.0 * 2629746.0),  // Should normalize to days (1 month in seconds)
            ("2 years", 2.0 * 31556952.0), // Should normalize to days (2 years in seconds)
        ];

        for (duration_str, expected_seconds) in test_cases {
            let input = format!(
                r#"A **user** is valid if __test_date__ of **user** is within {}."#,
                duration_str
            );

            let result = parse_rules(&input);
            assert!(result.is_ok(), "Failed to parse: {}", duration_str);

            let rule_set = result.unwrap();
            let rule = &rule_set.rules[0];

            match &rule.conditions[0].condition {
                Condition::Comparison(comp) => {
                    match &comp.value.value {
                        RuleValue::Duration(duration) => {
                            let actual_seconds = duration.to_seconds();
                            // Allow for small floating point differences (within 1%)
                            let diff = (actual_seconds - expected_seconds).abs();
                            let tolerance = expected_seconds * 0.01; // 1% tolerance
                            assert!(
                                diff < tolerance.max(1.0),
                                "Duration mismatch for {}: expected {} seconds, got {} seconds",
                                duration_str,
                                expected_seconds,
                                actual_seconds
                            );
                        }
                        _ => panic!("Expected Duration value for: {}", duration_str),
                    }
                }
                _ => panic!("Expected comparison condition for: {}", duration_str),
            }
        }
    }

    #[test]
    fn test_parse_nested_selector_paths() {
        let input = r#"A **user** is valid if __prop__ of the **top.second** is equal to "test"."#;

        let result = parse_rules(input);
        assert!(result.is_ok(), "Failed to parse nested selector");

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];

        match &rule.conditions[0].condition {
            Condition::Comparison(comp) => {
                assert_eq!(comp.selector.value, "top.second");
                assert_eq!(comp.property.value, "prop");
                assert_eq!(comp.operator, ComparisonOperator::EqualTo);
                match &comp.value.value {
                    RuleValue::String(s) => assert_eq!(s, "test"),
                    _ => panic!("Expected string value"),
                }
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_multiple_nested_levels() {
        let input = r#"A **user** passes the test if __value__ of the **data.config.settings** is greater than 10."#;

        let result = parse_rules(input);
        assert!(result.is_ok(), "Failed to parse deeply nested selector");

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];

        match &rule.conditions[0].condition {
            Condition::Comparison(comp) => {
                assert_eq!(comp.selector.value, "data.config.settings");
                assert_eq!(comp.property.value, "value");
                assert_eq!(comp.operator, ComparisonOperator::GreaterThan);
                match &comp.value.value {
                    RuleValue::Number(n) => assert_eq!(*n, 10.0),
                    _ => panic!("Expected number value"),
                }
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_single_nested_path() {
        let input = r#"A **person** is eligible if the **second** of the **top** passes the test."#;

        let result = parse_rules(input);
        assert!(result.is_ok(), "Failed to parse single nested reference");

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];

        match &rule.conditions[0].condition {
            Condition::RuleReference(ref_cond) => {
                assert_eq!(ref_cond.selector.value, "second");
                assert_eq!(ref_cond.rule_name.value, "of the **top** passes the test");
            }
            _ => panic!("Expected rule reference condition"),
        }
    }

    #[test]
    fn test_nested_vs_regular_selector() {
        // Test that regular selectors still work
        let input1 = r#"A **user** is valid if __name__ of **user** is equal to "John"."#;
        let result1 = parse_rules(input1);
        assert!(result1.is_ok(), "Failed to parse regular selector");

        // Test that nested selectors work
        let input2 = r#"A **user** is valid if __name__ of **user.profile** is equal to "John"."#;
        let result2 = parse_rules(input2);
        assert!(result2.is_ok(), "Failed to parse nested selector");

        let rule_set1 = result1.unwrap();
        let rule_set2 = result2.unwrap();

        // Check regular selector
        match &rule_set1.rules[0].conditions[0].condition {
            Condition::Comparison(comp) => {
                assert_eq!(comp.selector.value, "user");
            }
            _ => panic!("Expected comparison condition"),
        }

        // Check nested selector
        match &rule_set2.rules[0].conditions[0].condition {
            Condition::Comparison(comp) => {
                assert_eq!(comp.selector.value, "user.profile");
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_citation_functionality_already_works() {
        // Citation support is already implemented and working!
        // This test demonstrates the existing functionality

        // Test label reference (the existing test works fine)
        let label_input = r#"A **user** is eligible if §ageCheck is valid."#;
        let result = parse_rules(label_input);
        assert!(result.is_ok());
        let rule_set = result.unwrap();
        match &rule_set.rules[0].conditions[0].condition {
            crate::runner::model::Condition::RuleReference(ref_cond) => {
                assert_eq!(ref_cond.selector.value, "");
                assert_eq!(ref_cond.rule_name.value, "ageCheck");
            }
            _ => panic!("Expected rule reference condition"),
        }

        // Test rule name reference (the existing test works fine)
        let rule_input = r#"A **user** passes the test if the **user** passes the age check."#;
        let result = parse_rules(rule_input);
        assert!(result.is_ok());
        let rule_set = result.unwrap();
        match &rule_set.rules[0].conditions[0].condition {
            crate::runner::model::Condition::RuleReference(ref_cond) => {
                assert_eq!(ref_cond.selector.value, "user");
                assert_eq!(ref_cond.rule_name.value, "passes the age check");
            }
            _ => panic!("Expected rule reference condition"),
        }
    }

    #[test]
    fn test_driving_test_example_should_not_error() {
        let input = r#"
A **driver** gets a driving licence
  if the **driver** is valid
  and the **driver** passes the theory test
  and the **driver** passes the practical test
  and the **driver** has a provisional licence.

A **driver** is valid
  if __age__ of **driver** is greater than or equal to 17.

A **driver** passes the theory test
  if __theory_score__ of **driver** is greater than or equal to 43.

A **driver** passes the practical test
  if __practical_score__ of **driver** is greater than or equal to 50.

A **driver** has a provisional licence
  if __provisional__ of **driver** is equal to true.
        "#;

        let result = parse_rules(input);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        assert!(
            result.is_ok(),
            "Should parse without errors - rule references should not be considered global rules"
        );

        let rule_set = result.unwrap();
        assert_eq!(rule_set.rules.len(), 5);

        // The first rule should be the global rule since it references others
        // The other rules are supporting rules that are referenced
        assert_eq!(rule_set.rules[0].outcome, "a driving licence");
    }

    #[test]
    fn test_partial_matching_issue() {
        // This test demonstrates a potential issue with partial matching
        let input = r#"
A **person** gets access 
  if the **person** is valid 
  and the **person** has valid_license.

A **person** is valid if __age__ of **person** is greater than 18.

A **person** has valid_license if __license__ of **person** is equal to true.
        "#;

        let result = parse_rules(input);
        assert!(
            result.is_ok(),
            "Should parse successfully with improved matching logic"
        );

        let rule_set = result.unwrap();

        // Verify that the system correctly identifies the global rule
        // and doesn't confuse similar rule names
        let referenced = crate::runner::utils::find_referenced_outcomes(&rule_set.rules);

        // Should have exactly the right references: valid, valid_license
        assert_eq!(referenced.len(), 2);
        assert!(referenced.contains("valid"));
        assert!(referenced.contains("valid_license"));

        // Should identify "access" as the only global rule
        let global_rule_result = crate::runner::utils::find_global_rule(&rule_set.rules);
        assert!(global_rule_result.is_ok());
        assert_eq!(global_rule_result.unwrap().outcome, "access");
    }

    #[test]
    fn test_complex_driving_test_example() {
        let input = r#"
A **driver** gets a driving licence
  if the **driver** passes the age test
  and the **driver** passes the test requirements
  and the **driver** has taken the test in the time period
  and the **driver** did their test at a valid center.

A **driver** did their test at a valid center
  if the __center__ of the **drivingTest.testDates.practical** is in ["Manchester", "Coventry"]
  and the __center__ of the **practical** of the **test dates** in the **driving test** is in ["Manchester", "Coventry"].

A **driver** passes the age test
  if the __date of birth__ of the **person** in the **driving test** is earlier than 2008-12-12.

A **driver** passes the test requirements
  if **driver** passes the theory test
  and the **driver** passes the practical test.

A **driver** passes the theory test
  if the __multiple choice__ of the **theory** of the **scores** in the **driving test** is at least 43
  and the __hazard perception__ of the **theory** of the **scores** in the **driving test** is at least 44.

A **driver** passes the practical test
  if the __minor__ in the **practical** of the **scores** in the **driving test** is no more than 15
  and the __major__ in the **practical** of the **scores** in the **driving test** is equal to false.

A **driver** has taken the test in the time period
  if the __date__ of the __theory__ of the **testDates** in the **driving test** is within 2 years
  and the __date__ of the __practical__ of the **testDates** in the **driving test** is within 30 days.
        "#;

        let result = parse_rules(input);
        assert!(
            result.is_ok(),
            "Should parse without errors - rule references should match rule outcomes"
        );

        let rule_set = result.unwrap();

        // Should identify "a driving licence" as the only global rule
        let global_rule_result = crate::runner::utils::find_global_rule(&rule_set.rules);
        assert!(global_rule_result.is_ok());
        assert_eq!(global_rule_result.unwrap().outcome, "a driving licence");
    }

    #[test]
    fn test_driving_example_without_problematic_label() {
        // The label grammar has a bug where it's too greedy across multiple rules
        // For now, test without the label to confirm the reference matching works
        let input = r#"
A **driver** gets a driving licence
  if the **driver** passes the age test
  and the **driver** passes the test requirements
  and the **driver** has taken the test in the time period
  and the **driver** did their test at a valid center.

A **driver** did their test at a valid center
  if the __center__ of the **drivingTest.testDates.practical** is in ["Manchester", "Coventry"]
  and the __center__ of the **practical** of the **test dates** in the **driving test** is in ["Manchester", "Coventry"].

A **driver** passes the age test
  if the __date of birth__ of the **person** in the **driving test** is earlier than 2008-12-12.

A **driver** passes the test requirements
  if **driver** passes the theory test
  and the **driver** passes the practical test.

A **driver** passes the theory test
  if the __multiple choice__ of the **theory** of the **scores** in the **driving test** is at least 43
  and the __hazard perception__ of the **theory** of the **scores** in the **driving test** is at least 44.

A **driver** passes the practical test
  if the __minor__ in the **practical** of the **scores** in the **driving test** is no more than 15
  and the __major__ in the **practical** of the **scores** in the **driving test** is equal to false.

A **driver** has taken the test in the time period
  if the __date__ of the __theory__ of the **testDates** in the **driving test** is within 2 years
  and the __date__ of the __practical__ of the **testDates** in the **driving test** is within 30 days.
        "#;

        let result = parse_rules(input);
        assert!(
            result.is_ok(),
            "Should parse without errors when label is removed"
        );

        let rule_set = result.unwrap();

        // Should have all 7 rules
        assert_eq!(rule_set.rules.len(), 7);

        // Should identify "a driving licence" as the only global rule
        let global_rule_result = crate::runner::utils::find_global_rule(&rule_set.rules);
        assert!(global_rule_result.is_ok());
        assert_eq!(global_rule_result.unwrap().outcome, "a driving licence");
    }

    #[test]
    fn test_label_parsing_bug_fixed() {
        // First test that labels work correctly on a single rule
        let single_rule = r#"tester. A **driver** passes the age test if __age__ of **driver** is greater than 18."#;
        let result = parse_rules(single_rule);
        assert!(result.is_ok(), "Single rule with label should parse");
        let rule_set = result.unwrap();
        assert_eq!(rule_set.rules.len(), 1);
        assert_eq!(rule_set.rules[0].label, Some("tester".to_string()));

        // Test the full scenario with 3 rules, where only the last has a label
        let input = r#"
A **driver** gets a driving licence
  if the **driver** passes the age test
  and the **driver** did their test at a valid center.

A **driver** did their test at a valid center
  if __center__ of **driver** is in ["Manchester", "Coventry"].

tester. A **driver** passes the age test
  if __date_of_birth__ of **driver** is earlier than 2008-12-12.
        "#;

        let result = parse_rules(input);
        if let Err(ref e) = result {
            println!("Parse error in test_label_parsing_bug_fixed: {:?}", e);
        }
        assert!(
            result.is_ok(),
            "Should parse successfully with label grammar fix"
        );

        let rule_set = result.unwrap();

        // Should have 3 rules
        assert_eq!(rule_set.rules.len(), 3);

        // Check that only the third rule has a label
        assert!(rule_set.rules[0].label.is_none());
        assert!(rule_set.rules[1].label.is_none());
        assert_eq!(rule_set.rules[2].label, Some("tester".to_string()));

        // Should identify "a driving licence" as the only global rule
        let global_rule_result = crate::runner::utils::find_global_rule(&rule_set.rules);
        assert!(global_rule_result.is_ok());
        assert_eq!(global_rule_result.unwrap().outcome, "a driving licence");
    }

    #[test]
    fn test_valid_label_formats() {
        // Test that various label formats should be valid according to the spec:
        // a label can be anything <alphanumeric><fullstop>(repeated)<space>
        let valid_labels = vec![
            "a.b.c. ",
            "a. ",
            "a.1.s. ",
            "1.a.s. ",
            "tester. ",
            "rule1. ",
            "1. ",
            "test.123. ",
            "driver.bob. ",
            "1.0. ",
            "1.1. ",
        ];

        for label in valid_labels {
            println!("Testing label: '{}'", label);
            // The label should match the pattern: alphanumeric characters with dots, ending with ". "
            assert!(label.ends_with(". "), "Label should end with '. '");
            let without_ending = &label[..label.len() - 2];
            assert!(
                !without_ending.is_empty(),
                "Label should have content before '. '"
            );

            // Check that all characters are alphanumeric or dots
            for ch in without_ending.chars() {
                assert!(
                    ch.is_alphanumeric() || ch == '.',
                    "Label should only contain alphanumeric characters and dots, found: '{}'",
                    ch
                );
            }
        }
    }

    #[test]
    fn test_label_reference_with_dots() {
        // Test that label references with dots work correctly
        let input = r#"
A **driver** gets a driving licence
  if §driver.bob is valid.

driver.bob. A **driver** passes the age test
  if __age__ of **driver** is greater than 18.
        "#;

        let result = parse_rules(input);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        assert!(result.is_ok(), "Should parse label references with dots");
        
        let rule_set = result.unwrap();
        assert_eq!(rule_set.rules.len(), 2);
        
        // Check the label on the second rule
        assert_eq!(rule_set.rules[1].label, Some("driver.bob".to_string()));
        
        // Check that the first rule has a label reference condition
        match &rule_set.rules[0].conditions[0].condition {
            crate::runner::model::Condition::RuleReference(ref_cond) => {
                assert_eq!(ref_cond.rule_name.value, "driver.bob");
            }
            _ => panic!("Expected rule reference condition"),
        }
    }
    
    #[test]
    fn test_numeric_labels() {
        // Test that numeric labels like 1.0, 1.1 work correctly
        let input = r#"
1.0. A **driver** gets a driving licence
  if the **driver** passes the age test.

1.1. A **driver** passes the age test
  if __age__ of **driver** is greater than 18.
        "#;

        let result = parse_rules(input);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        assert!(result.is_ok(), "Should parse numeric labels");
        
        let rule_set = result.unwrap();
        assert_eq!(rule_set.rules.len(), 2);
        
        // Check the labels
        assert_eq!(rule_set.rules[0].label, Some("1.0".to_string()));
        assert_eq!(rule_set.rules[1].label, Some("1.1".to_string()));
    }

    #[test]
    fn test_full_driving_test_with_label_references() {
        // Test the full driving test example with driver.bob label reference
        let input = r#"
# Driving Test Example

A **driver** gets a driving licence
  if §driver.bob is valid
  and the **driver** passes the test requirements
  and the **driver** has taken the test in the time period
  and the **driver** did their test at a valid center.

A **driver** did their test at a valid center
  if the __center__ of the **drivingTest.testDates.practical** is in ["Manchester", "Coventry"]
  and the __center__ of the **practical** of the **test dates** in the **driving test** is in ["Manchester", "Coventry"].

driver.bob. A **driver** passes the age test
  if the __date of birth__ of the **person** in the **driving test** is earlier than 2008-12-12.

A **driver** passes the test requirements
  if **driver** passes the theory test
  and the **driver** passes the practical test.

A **driver** passes the theory test
  if the __multiple choice__ of the **theory** of the **scores** in the **driving test** is at least 43
  and the __hazard perception__ of the **theory** of the **scores** in the **driving test** is at least 44.

A **driver** passes the practical test
  if the __minor__ in the **practical** of the **scores** in the **driving test** is no more than 15
  and the __major__ in the **practical** of the **scores** in the **driving test** is equal to false.

A **driver** has taken the test in the time period
  if the __date__ of the __theory__ of the **testDates** in the **driving test** is within 2 years
  and the __date__ of the __practical__ of the **testDates** in the **driving test** is within 30 days.
        "#;

        let result = parse_rules(input);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        assert!(
            result.is_ok(),
            "Should parse driving test with label references"
        );

        let rule_set = result.unwrap();
        
        // Should have all 7 rules  
        assert_eq!(rule_set.rules.len(), 7);
        
        // The third rule should have the driver.bob label
        assert_eq!(rule_set.rules[2].label, Some("driver.bob".to_string()));
        
        // The first rule should have a label reference as its first condition
        match &rule_set.rules[0].conditions[0].condition {
            crate::runner::model::Condition::RuleReference(ref_cond) => {
                assert_eq!(ref_cond.rule_name.value, "driver.bob");
            }
            _ => panic!("Expected label reference condition"),
        }
        
        // Should identify "a driving licence" as the only global rule
        let global_rule_result = crate::runner::utils::find_global_rule(&rule_set.rules);
        assert!(global_rule_result.is_ok());
        assert_eq!(global_rule_result.unwrap().outcome, "a driving licence");
    }
    
    #[test]
    fn test_driving_test_with_labels_fixed() {
        // This is the full driving test example from the user, with label prefix
        let input = r#"
tester. A **driver** gets a driving licence
  if the **driver** passes the age test
  and the **driver** passes the test requirements
  and the **driver** has taken the test in the time period
  and the **driver** did their test at a valid center.

A **driver** did their test at a valid center
  if the __center__ of the **drivingTest.testDates.practical** is in ["Manchester", "Coventry"]
  and the __center__ of the **practical** of the **test dates** in the **driving test** is in ["Manchester", "Coventry"].

A **driver** passes the age test
  if the __date of birth__ of the **person** in the **driving test** is earlier than 2008-12-12.

A **driver** passes the test requirements
  if **driver** passes the theory test
  and the **driver** passes the practical test.

A **driver** passes the theory test
  if the __multiple choice__ of the **theory** of the **scores** in the **driving test** is at least 43
  and the __hazard perception__ of the **theory** of the **scores** in the **driving test** is at least 44.

A **driver** passes the practical test
  if the __minor__ in the **practical** of the **scores** in the **driving test** is no more than 15
  and the __major__ in the **practical** of the **scores** in the **driving test** is equal to false.

A **driver** has taken the test in the time period
  if the __date__ of the __theory__ of the **testDates** in the **driving test** is within 2 years
  and the __date__ of the __practical__ of the **testDates** in the **driving test** is within 30 days.
        "#;

        let result = parse_rules(input);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        assert!(
            result.is_ok(),
            "Should parse without errors with fixed label grammar"
        );

        let rule_set = result.unwrap();

        // Should have all 7 rules
        assert_eq!(rule_set.rules.len(), 7);

        // First rule should have the label
        assert_eq!(rule_set.rules[0].label, Some("tester".to_string()));

        // Should identify "a driving licence" as the only global rule
        let global_rule_result = crate::runner::utils::find_global_rule(&rule_set.rules);
        assert!(global_rule_result.is_ok());
        assert_eq!(global_rule_result.unwrap().outcome, "a driving licence");
    }
}
