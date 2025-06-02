#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use crate::runner::model::{ComparisonCondition, ComparisonOperator, Condition, ConditionGroup, ConditionOperator, PositionedValue, PropertyChainElement, PropertyPath, Rule, RuleReferenceCondition, RuleSet, RuleValue, SourcePosition};

    #[test]
    fn test_comparison_operator_display() {
        assert_eq!(ComparisonOperator::GreaterThanOrEqual.to_string(), "is greater than or equal to");

        assert_eq!(ComparisonOperator::LessThanOrEqual.to_string(), "is less than or equal to");

        assert_eq!(ComparisonOperator::EqualTo.to_string(), "is equal to");
        assert_eq!(ComparisonOperator::NotEqualTo.to_string(), "is not equal to");
        assert_eq!(ComparisonOperator::SameAs.to_string(), "is the same as");
        assert_eq!(ComparisonOperator::NotSameAs.to_string(), "is not the same as");

        assert_eq!(ComparisonOperator::LaterThan.to_string(), "is later than");
        assert_eq!(ComparisonOperator::EarlierThan.to_string(), "is earlier than");

        assert_eq!(ComparisonOperator::GreaterThan.to_string(), "is greater than");
        assert_eq!(ComparisonOperator::LessThan.to_string(), "is less than");

        assert_eq!(ComparisonOperator::In.to_string(), "is in");
        assert_eq!(ComparisonOperator::NotIn.to_string(), "is not in");
        assert_eq!(ComparisonOperator::Contains.to_string(), "contains");
    }

    #[test]
    fn test_rule_value_display() {
        // Test Number
        let num_value = RuleValue::Number(42.5);
        assert_eq!(num_value.to_string(), "42.5");

        // Test String
        let str_value = RuleValue::String("hello world".to_string());
        assert_eq!(str_value.to_string(), "\"hello world\"");

        // Test Date
        let date_value = RuleValue::Date(NaiveDate::from_ymd_opt(2023, 12, 25).unwrap());
        assert_eq!(date_value.to_string(), "date(2023-12-25)");

        // Test Boolean
        let bool_value_true = RuleValue::Boolean(true);
        let bool_value_false = RuleValue::Boolean(false);
        assert_eq!(bool_value_true.to_string(), "true");
        assert_eq!(bool_value_false.to_string(), "false");

        // Test List
        let list_value = RuleValue::List(vec![
            RuleValue::Number(1.0),
            RuleValue::String("test".to_string()),
            RuleValue::Boolean(true),
        ]);
        assert_eq!(list_value.to_string(), "[1, \"test\", true]");

        // Test empty list
        let empty_list = RuleValue::List(vec![]);
        assert_eq!(empty_list.to_string(), "[]");
    }

    #[test]
    fn test_positioned_value_creation() {
        // Test new
        let pos_val = PositionedValue::new("test".to_string());
        assert_eq!(pos_val.value, "test");
        assert!(pos_val.pos.is_none());

        // Test with_position
        let source_pos = SourcePosition {
            line: 1,
            start: 0,
            end: 4,
        };
        let pos_val_with_pos = PositionedValue::with_position("test".to_string(), Some(source_pos.clone()));
        assert_eq!(pos_val_with_pos.value, "test");
        assert!(pos_val_with_pos.pos.is_some());
        assert_eq!(pos_val_with_pos.pos.unwrap().line, 1);
    }

    #[test]
    fn test_positioned_value_from_implementations() {
        // Test From<String>
        let pos_val: PositionedValue<String> = "hello".to_string().into();
        assert_eq!(pos_val.value, "hello");
        assert!(pos_val.pos.is_none());

        // Test From<RuleValue>
        let rule_val = RuleValue::Number(42.0);
        let pos_val: PositionedValue<RuleValue> = rule_val.clone().into();
        match pos_val.value {
            RuleValue::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number variant"),
        }
        assert!(pos_val.pos.is_none());
    }

    #[test]
    fn test_rule_creation_and_modification() {
        let mut rule = Rule::new(
            Some("test_label".to_string()),
            "user".to_string(),
            "approved".to_string(),
        );

        assert_eq!(rule.label, Some("test_label".to_string()));
        assert_eq!(rule.selector, "user");
        assert_eq!(rule.outcome, "approved");
        assert!(rule.conditions.is_empty());
        assert!(rule.position.is_none());
        assert!(rule.selector_pos.is_none());

        // Test adding conditions
        let comparison_condition = ComparisonCondition {
            selector: PositionedValue::new("user".to_string()),
            property: PositionedValue::new("age".to_string()),
            operator: ComparisonOperator::GreaterThanOrEqual,
            value: PositionedValue::new(RuleValue::Number(18.0)),
            property_chain: None,
            left_property_path: None,
            right_property_path: None,
        };

        rule.add_condition(
            Condition::Comparison(comparison_condition),
            None, // First condition has no operator
        );

        assert_eq!(rule.conditions.len(), 1);
        assert!(rule.conditions[0].operator.is_none());

        // Add second condition with AND operator
        let second_condition = ComparisonCondition {
            selector: PositionedValue::new("user".to_string()),
            property: PositionedValue::new("status".to_string()),
            operator: ComparisonOperator::EqualTo,
            value: PositionedValue::new(RuleValue::String("active".to_string())),
            property_chain: None,
            left_property_path: None,
            right_property_path: None,
        };

        rule.add_condition(
            Condition::Comparison(second_condition),
            Some(ConditionOperator::And),
        );

        assert_eq!(rule.conditions.len(), 2);
        assert_eq!(rule.conditions[1].operator, Some(ConditionOperator::And));
    }

    #[test]
    fn test_rule_reference_condition() {
        let rule_ref = RuleReferenceCondition {
            selector: PositionedValue::new("user".to_string()),
            rule_name: PositionedValue::new("eligibility_check".to_string()),
        };

        assert_eq!(rule_ref.selector.value, "user");
        assert_eq!(rule_ref.rule_name.value, "eligibility_check");
    }

    #[test]
    fn test_property_path_and_chain() {
        let property_path = PropertyPath {
            properties: vec!["address".to_string(), "city".to_string()],
            selector: "user".to_string(),
        };

        assert_eq!(property_path.selector, "user");
        assert_eq!(property_path.properties.len(), 2);
        assert_eq!(property_path.properties[0], "address");
        assert_eq!(property_path.properties[1], "city");

        // Test property chain elements
        let chain_element_prop = PropertyChainElement::Property("name".to_string());
        let chain_element_sel = PropertyChainElement::Selector("user".to_string());

        match chain_element_prop {
            PropertyChainElement::Property(prop) => assert_eq!(prop, "name"),
            _ => panic!("Expected Property variant"),
        }

        match chain_element_sel {
            PropertyChainElement::Selector(sel) => assert_eq!(sel, "user"),
            _ => panic!("Expected Selector variant"),
        }
    }

    #[test]
    fn test_ruleset_operations() {
        let mut ruleset = RuleSet::new();
        assert!(ruleset.rules.is_empty());
        assert!(ruleset.rule_map.is_empty());
        assert!(ruleset.label_map.is_empty());

        // Add first rule with label
        let rule1 = Rule::new(
            Some("age_check".to_string()),
            "user".to_string(),
            "adult".to_string(),
        );
        ruleset.add_rule(rule1);

        // Add second rule without label
        let rule2 = Rule::new(None, "user".to_string(), "minor".to_string());
        ruleset.add_rule(rule2);

        assert_eq!(ruleset.rules.len(), 2);
        assert_eq!(ruleset.rule_map.len(), 2);
        assert_eq!(ruleset.label_map.len(), 1);

        // Test get_rule
        let retrieved_rule = ruleset.get_rule("adult");
        assert!(retrieved_rule.is_some());
        assert_eq!(retrieved_rule.unwrap().outcome, "adult");
        assert_eq!(retrieved_rule.unwrap().label, Some("age_check".to_string()));

        // Test get_rule_by_label
        let retrieved_by_label = ruleset.get_rule_by_label("age_check");
        assert!(retrieved_by_label.is_some());
        assert_eq!(retrieved_by_label.unwrap().outcome, "adult");

        // Test non-existent rule
        assert!(ruleset.get_rule("non_existent").is_none());
        assert!(ruleset.get_rule_by_label("non_existent").is_none());
    }

    #[test]
    fn test_condition_group() {
        let comparison_condition = ComparisonCondition {
            selector: PositionedValue::new("user".to_string()),
            property: PositionedValue::new("score".to_string()),
            operator: ComparisonOperator::GreaterThan,
            value: PositionedValue::new(RuleValue::Number(80.0)),
            property_chain: None,
            left_property_path: None,
            right_property_path: None,
        };

        let condition_group = ConditionGroup {
            condition: Condition::Comparison(comparison_condition),
            operator: Some(ConditionOperator::Or),
        };

        assert_eq!(condition_group.operator, Some(ConditionOperator::Or));
        match condition_group.condition {
            Condition::Comparison(_) => (), // Expected
            _ => panic!("Expected Comparison condition"),
        }
    }

    #[test]
    fn test_source_position() {
        let pos = SourcePosition {
            line: 5,
            start: 10,
            end: 20,
        };

        assert_eq!(pos.line, 5);
        assert_eq!(pos.start, 10);
        assert_eq!(pos.end, 20);
    }

    #[test]
    fn test_complex_comparison_condition() {
        let left_path = PropertyPath {
            properties: vec!["account".to_string(), "balance".to_string()],
            selector: "user".to_string(),
        };

        let right_path = PropertyPath {
            properties: vec!["limits".to_string(), "daily_max".to_string()],
            selector: "config".to_string(),
        };

        let property_chain = vec![
            PropertyChainElement::Property("profile".to_string()),
            PropertyChainElement::Selector("details".to_string()),
            PropertyChainElement::Property("verification_level".to_string()),
        ];

        let complex_condition = ComparisonCondition {
            selector: PositionedValue::new("transaction".to_string()),
            property: PositionedValue::new("amount".to_string()),
            operator: ComparisonOperator::LessThanOrEqual,
            value: PositionedValue::new(RuleValue::Number(1000.0)),
            property_chain: Some(property_chain),
            left_property_path: Some(left_path),
            right_property_path: Some(right_path),
        };

        assert_eq!(complex_condition.selector.value, "transaction");
        assert_eq!(complex_condition.property.value, "amount");
        assert_eq!(complex_condition.operator, ComparisonOperator::LessThanOrEqual);
        assert!(complex_condition.property_chain.is_some());
        assert!(complex_condition.left_property_path.is_some());
        assert!(complex_condition.right_property_path.is_some());

        let chain = complex_condition.property_chain.unwrap();
        assert_eq!(chain.len(), 3);
    }

    #[test]
    fn test_rule_value_equality() {
        let num1 = RuleValue::Number(42.0);
        let num2 = RuleValue::Number(42.0);
        let num3 = RuleValue::Number(43.0);

        assert_eq!(num1, num2);
        assert_ne!(num1, num3);

        let str1 = RuleValue::String("test".to_string());
        let str2 = RuleValue::String("test".to_string());
        let str3 = RuleValue::String("different".to_string());

        assert_eq!(str1, str2);
        assert_ne!(str1, str3);

        let bool1 = RuleValue::Boolean(true);
        let bool2 = RuleValue::Boolean(true);
        let bool3 = RuleValue::Boolean(false);

        assert_eq!(bool1, bool2);
        assert_ne!(bool1, bool3);

        let date1 = RuleValue::Date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
        let date2 = RuleValue::Date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
        let date3 = RuleValue::Date(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap());

        assert_eq!(date1, date2);
        assert_ne!(date1, date3);
    }

    #[test]
    fn test_nested_list_values() {
        let nested_list = RuleValue::List(vec![
            RuleValue::Number(1.0),
            RuleValue::List(vec![
                RuleValue::String("nested".to_string()),
                RuleValue::Boolean(true),
            ]),
            RuleValue::Number(2.0),
        ]);

        let expected = "[1, [\"nested\", true], 2]";
        assert_eq!(nested_list.to_string(), expected);
    }

    #[test]
    fn test_condition_operators() {
        assert_eq!(ConditionOperator::And, ConditionOperator::And);
        assert_eq!(ConditionOperator::Or, ConditionOperator::Or);
        assert_ne!(ConditionOperator::And, ConditionOperator::Or);
    }

    #[test]
    fn test_ruleset_default() {
        let ruleset = RuleSet::default();
        assert!(ruleset.rules.is_empty());
        assert!(ruleset.rule_map.is_empty());
        assert!(ruleset.label_map.is_empty());
    }

    #[test]
    fn test_rule_with_multiple_condition_types() {
        let mut rule = Rule::new(
            Some("mixed_conditions".to_string()),
            "application".to_string(),
            "processed".to_string(),
        );

        // Add comparison condition
        let comparison = ComparisonCondition {
            selector: PositionedValue::new("application".to_string()),
            property: PositionedValue::new("status".to_string()),
            operator: ComparisonOperator::EqualTo,
            value: PositionedValue::new(RuleValue::String("pending".to_string())),
            property_chain: None,
            left_property_path: None,
            right_property_path: None,
        };

        rule.add_condition(Condition::Comparison(comparison), None);

        // Add rule reference condition
        let rule_ref = RuleReferenceCondition {
            selector: PositionedValue::new("application".to_string()),
            rule_name: PositionedValue::new("eligibility_rules".to_string()),
        };

        rule.add_condition(
            Condition::RuleReference(rule_ref),
            Some(ConditionOperator::And),
        );

        assert_eq!(rule.conditions.len(), 2);

        // Check first condition
        match &rule.conditions[0].condition {
            Condition::Comparison(_) => (),
            _ => panic!("Expected Comparison condition"),
        }
        assert!(rule.conditions[0].operator.is_none());

        // Check second condition
        match &rule.conditions[1].condition {
            Condition::RuleReference(_) => (),
            _ => panic!("Expected RuleReference condition"),
        }
        assert_eq!(rule.conditions[1].operator, Some(ConditionOperator::And));
    }
}