use serde_json::json;
mod runner;


#[cfg(test)]
mod tests {
    use super::*;
    use runner::parser::parse_rules;
    use runner::evaluator::evaluate_rule_set;
    use serde_json::json;

    #[test]
    fn test_greater_than_or_equal() {
        let rule_text = r#"
        A **Person** gets senior_discount
          if the __age__ of the **Person** is greater than or equal to 65.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case where condition is true
        let json_true = json!({
            "Person": {
                "age": 70
            }
        });
        let results_true = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["senior_discount"]);

        // Test case where condition is false
        let json_false = json!({
            "Person": {
                "age": 60
            }
        });
        let results_false = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["senior_discount"]);
    }

    #[test]
    fn test_less_than_or_equal() {
        let rule_text = r#"
        A **Person** gets child_discount
          if the __age__ of the **Person** is less than or equal to 12.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case where condition is true
        let json_true = json!({
            "Person": {
                "age": 10
            }
        });
        let results_true = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["child_discount"]);

        // Test case where condition is false
        let json_false = json!({
            "Person": {
                "age": 15
            }
        });
        let results_false = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["child_discount"]);
    }

    #[test]
    fn test_equal_to() {
        let rule_text = r#"
        A **Transaction** gets flagged
          if the __amount__ of the **Transaction** is equal to 1337.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case where condition is true
        let json_true = json!({
            "Transaction": {
                "amount": 1337
            }
        });
        let results_true = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["flagged"]);

        // Test case where condition is false
        let json_false = json!({
            "Transaction": {
                "amount": 1000
            }
        });
        let results_false = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["flagged"]);
    }

    #[test]
    fn test_not_equal_to() {
        let rule_text = r#"
        A **Transaction** gets normal
          if the __status__ of the **Transaction** is not equal to "flagged".
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case where condition is true
        let json_true = json!({
            "Transaction": {
                "status": "completed"
            }
        });
        let results_true = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["normal"]);

        // Test case where condition is false
        let json_false = json!({
            "Transaction": {
                "status": "flagged"
            }
        });
        let results_false = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["normal"]);
    }

    #[test]
    fn test_later_than() {
        let rule_text = r#"
        A **Subscription** is active
          if the __expiryDate__ of the **Subscription** is later than 2023-01-01.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case where condition is true
        let json_true = json!({
            "Subscription": {
                "expiryDate": "2023-12-31"
            }
        });
        let results_true = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["active"]);

        // Test case where condition is false
        let json_false = json!({
            "Subscription": {
                "expiryDate": "2022-12-31"
            }
        });
        let results_false = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["active"]);
    }

    #[test]
    fn test_earlier_than() {
        let rule_text = r#"
        A **Document** is archived
          if the __creationDate__ of the **Document** is earlier than 2020-01-01.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case where condition is true
        let json_true = json!({
            "document": {
                "creationDate": "2019-06-15"
            }
        });
        let results_true = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["archived"]);

        // Test case where condition is false
        let json_false = json!({
            "document": {
                "creationDate": "2022-03-10"
            }
        });
        let results_false = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["archived"]);
    }

    #[test]
    fn test_is_in() {
        let rule_text = r#"
        A **Product** gets on_sale
          if the __category__ of the **Product** is in ["electronics", "clothing", "books"].
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case where condition is true
        let json_true = json!({
            "Product": {
                "category": "electronics"
            }
        });
        let results_true = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["on_sale"]);

        // Test case where condition is false
        let json_false = json!({
            "Product": {
                "category": "furniture"
            }
        });
        let results_false = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["on_sale"]);
    }

    #[test]
    fn test_is_not_in() {
        let rule_text = r#"
        A **Product** gets full_price
          if the __category__ of the **Product** is not in ["electronics", "clothing", "books"].
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case where condition is true
        let json_true = json!({
            "Product": {
                "category": "furniture"
            }
        });
        let results_true = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["full_price"]);

        // Test case where condition is false
        let json_false = json!({
            "Product": {
                "category": "electronics"
            }
        });
        let results_false = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["full_price"]);
    }

    #[test]
    fn test_contains() {
        let rule_text = r#"
    A **Message** gets flagged
      if the __content__ of the **Message** contains "urgent".
    "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case where condition is true
        let json_true = json!({
        "Message": {
            "content": "This is an urgent message"
        }
    });
        let results_true = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["flagged"]);

        // Test case where condition is false
        let json_false = json!({
        "Message": {
            "content": "This is a normal message"
        }
    });
        let results_false = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["flagged"]);
    }

    #[test]
    fn test_rule_with_label() {
        let rule_text = r#"
        senior.discount. A **Person** gets senior_discount
          if the __age__ of the **Person** is greater than or equal to 65.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();
        assert_eq!(rule_set.rules.len(), 1);
        assert_eq!(rule_set.rules[0].label, Some("senior.discount".to_string()));

        // Test case where condition is true
        let json_true = json!({
            "Person": {
                "age": 70
            }
        });
        let results_true = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["senior_discount"]);
    }
}

#[test]
fn test_property_name_transformation() {
    let rule_text = r#"
    A **Person** gets discount
      if the __first name__ of the **Person** is equal to "John".
    "#;

    let rule_set = runner::parser::parse_rules(rule_text).unwrap();

    // Test case where condition is true
    let json_true = json!({
        "Person": {
            "firstName": "John"
        }
    });
    let results_true = runner::evaluator::evaluate_rule_set(&rule_set, &json_true).unwrap();
    assert!(results_true["discount"]);

    // Test case where condition is false
    let json_false = json!({
        "Person": {
            "firstName": "Jane"
        }
    });
    let results_false = runner::evaluator::evaluate_rule_set(&rule_set, &json_false).unwrap();
    assert!(!results_false["discount"]);
}

