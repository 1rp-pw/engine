#[cfg(test)]
mod tests {
    use crate::runner::model::{ComparisonOperator, RuleValue, Condition, ConditionOperator};
    use chrono::NaiveDate;
    use crate::runner::parser::parse_rules;

    #[test]
    fn test_parse_simple_rule() {
        let input = r#"A **user** passes the test if __age__ of **user** is greater than or equal to 18."#;

        let result = parse_rules(input);
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
        let input = r#"Age Check. A **person** is eligible if __age__ of **user** is greater than 21."#;

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
            Some(ConditionOperator::And) => {},
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
            Some(ConditionOperator::Or) => {},
            _ => panic!("Expected OR operator"),
        }
    }

    #[test]
    fn test_parse_string_comparison() {
        let input = r#"
A **user** is valid if __status__ of **user** is equal to "active".
        "#;

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
            Condition::Comparison(comp) => {
                match &comp.value.value {
                    RuleValue::Boolean(b) => assert!(*b),
                    _ => panic!("Expected boolean value"),
                }
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_list_operations() {
        let input = r#"A **user** is valid if __role__ of **user** is in ["admin", "moderator", "user"]."#;

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
                assert_eq!(path.properties, vec!["id", "group"]);
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_property_to_property_comparison() {
        let input = r#"
A **user** is valid if __age__ of **user** is greater than __min_age__ of **config**.
        "#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];

        match &rule.conditions[0].condition {
            Condition::Comparison(comp) => {
                assert!(comp.left_property_path.is_some());
                assert!(comp.right_property_path.is_some());

                let left_path = comp.left_property_path.as_ref().unwrap();
                assert_eq!(left_path.selector, "user");
                assert_eq!(left_path.properties, vec!["age"]);

                let right_path = comp.right_property_path.as_ref().unwrap();
                assert_eq!(right_path.selector, "config");
                assert_eq!(right_path.properties, vec!["minAge"]);
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_all_comparison_operators() {
        let operators = vec![
            ("is greater than or equal to", ComparisonOperator::GreaterThanOrEqual),
            ("is at least", ComparisonOperator::GreaterThanOrEqual),

            ("is less than or equal to", ComparisonOperator::LessThanOrEqual),
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
            let input = format!(r#"A **user** passes the test if __value__ of **user** {} 10."#, op_str);
            let result = parse_rules(&input);
            if let Err(ref e) = result {
                println!("Parse error for '{}': {:?}", op_str, e);
            }
            assert!(result.is_ok(), "Failed to parse operator: {}", op_str);

            let rule_set = result.unwrap();
            let rule = &rule_set.rules[0];

            match &rule.conditions[0].condition {
                Condition::Comparison(comp) => {
                    assert_eq!(comp.operator, expected_op, "Operator mismatch for: {}", op_str);
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
            let input = format!(r#"A **user** passes the test if __role__ of **user** {} ["admin", "user"]."#, op_str);
            let result = parse_rules(&input);
            assert!(result.is_ok(), "Failed to parse list operator: {}", op_str);

            let rule_set = result.unwrap();
            let rule = &rule_set.rules[0];

            match &rule.conditions[0].condition {
                Condition::Comparison(comp) => {
                    assert_eq!(comp.operator, expected_op, "Operator mismatch for: {}", op_str);
                }
                _ => panic!("Expected comparison condition for operator: {}", op_str),
            }
        }
    }

    #[test]
    fn test_parse_various_outcome_verbs() {
        let verbs = vec!["gets", "passes", "is", "has", "receives", "qualifies for", "meets", "satisfies"];

        for verb in verbs {
            let input = format!(r#"A **user** {} approved if __age__ of **user** is greater than 18."#, verb);
            let result = parse_rules(&input);
            assert!(result.is_ok(), "Failed to parse verb: {}", verb);

            let rule_set = result.unwrap();
            let rule = &rule_set.rules[0];
            assert_eq!(rule.outcome, "approved", "Outcome mismatch for verb: {}", verb);
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
                Condition::Comparison(comp) => {
                    match &comp.value.value {
                        RuleValue::Date(d) => {
                            let expected = NaiveDate::from_ymd_opt(2023, 12, 1).unwrap();
                            assert_eq!(*d, expected);
                        }
                        _ => panic!("Expected date value"),
                    }
                }
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
            Condition::Comparison(comp) => {
                match &comp.value.value {
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
                }
            }
            _ => panic!("Expected comparison condition"),
        }
    }

    #[test]
    fn test_parse_property_name_transformation() {
        let input = r#"
A **user** is valid if __first name__ of **user** is equal to "John".
        "#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let rule = &rule_set.rules[0];

        match &rule.conditions[0].condition {
            Condition::Comparison(comp) => {
                // Property should be transformed from "first name" to "firstName"
                if let Some(path) = &comp.left_property_path {
                    assert_eq!(path.properties, vec!["firstName"]);
                }
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
}