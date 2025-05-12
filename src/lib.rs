// src/lib.rs
pub mod error;
pub mod model;
pub mod parser;
pub mod evaluator;

#[cfg(test)]
mod tests {
    use super::*;
    use parser::parse_rules;
    use evaluator::evaluate_rule_set;
    use serde_json::json;

    #[test]
    fn test_greater_than_or_equal() {
        let rule_text = r#"
        A Person gets senior discount
          if the __age__ of the Person is greater than or equal to 65.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case where condition is true
        let json_true = json!({
            "Person": {
                "age": 70
            }
        });
        let results_true = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["senior discount"]);

        // Test case where condition is false
        let json_false = json!({
            "Person": {
                "age": 60
            }
        });
        let results_false = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["senior discount"]);
    }

    #[test]
    fn test_equal_to() {
        let rule_text = r#"
        A Transaction gets flagged
          if the __amount__ of the Transaction is equal to 1337.
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
    fn test_later_than() {
        let rule_text = r#"
        A Subscription gets active
          if the __expiryDate__ of the Subscription is later than date(2023-01-01).
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
    fn test_is_in() {
        let rule_text = r#"
        A Product gets on_sale
          if the __category__ of the Product is in ["electronics", "clothing", "books"].
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
    fn test_rule_reference() {
        let rule_text = r#"
        A Person gets driving_license
          if the __age__ of the Person is greater than or equal to 17
          and the Person passes driving_test.

        A Person passes driving_test
          if the __testScore__ of the Person is greater than 70.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case where all conditions are true
        let json_true = json!({
            "Person": {
                "age": 18,
                "testScore": 75
            }
        });
        let results_true = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["driving_license"]);

        // Test case where one condition is false
        let json_false = json!({
            "Person": {
                "age": 18,
                "testScore": 65
            }
        });
        let results_false = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["driving_license"]);
    }
}
