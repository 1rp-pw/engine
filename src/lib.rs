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

        let json_true = json!({
            "Person": {
                "age": 70
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["senior_discount"]);


        let json_false = json!({
            "Person": {
                "age": 60
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["senior_discount"]);
    }

    #[test]
    fn test_less_than_or_equal() {
        let rule_text = r#"
        A **Person** gets child_discount
          if the __age__ of the **Person** is less than or equal to 12.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        let json_true = json!({
            "Person": {
                "age": 10
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["child_discount"]);

        let json_false = json!({
            "Person": {
                "age": 15
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["child_discount"]);
    }

    #[test]
    fn test_equal_to() {
        let rule_text = r#"
        A **Transaction** gets flagged
          if the __amount__ of the **Transaction** is equal to 1337.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        let json_true = json!({
            "Transaction": {
                "amount": 1337
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["flagged"]);

        let json_false = json!({
            "Transaction": {
                "amount": 1000
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["flagged"]);
    }

    #[test]
    fn test_not_equal_to() {
        let rule_text = r#"
        A **Transaction** gets normal
          if the __status__ of the **Transaction** is not equal to "flagged".
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        let json_true = json!({
            "Transaction": {
                "status": "completed"
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["normal"]);

        let json_false = json!({
            "Transaction": {
                "status": "flagged"
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["normal"]);
    }

    #[test]
    fn test_later_than() {
        let rule_text = r#"
        A **Subscription** is active
          if the __expiry date__ of the **Subscription** is later than "2023-01-01".
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case where condition is true
        let json_true = json!({
            "Subscription": {
                "expiryDate": "2023-12-31"
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["active"]);

        // Test case where condition is false
        let json_false = json!({
            "Subscription": {
                "expiryDate": "2022-12-31"
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["active"]);
    }

    #[test]
    fn test_earlier_than() {
        let rule_text = r#"
A **Document** is archived
  if the __creation date__ of the **Document** is earlier than "2020-01-01".
"#;

        let rule_set = parse_rules(rule_text).expect("Parse error: failed to parse rules");

        // Test case where condition is true  
        let json_true = json!({
            "Document": {
                "creationDate": "2019-06-15"
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();

        //println!("Trace: {:#?}", trace_true);

        assert!(results_true["archived"]);
    }

    #[test]
    fn test_is_in() {
        let rule_text = r#"
        A **Product** gets on_sale
          if the __category__ of the **Product** is in ["electronics", "clothing", "books"].
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        let json_true = json!({
            "Product": {
                "category": "electronics"
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["on_sale"]);

        let json_false = json!({
            "Product": {
                "category": "furniture"
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["on_sale"]);
    }

    #[test]
    fn test_is_not_in() {
        let rule_text = r#"
        A **Product** gets full_price
          if the __category__ of the **Product** is not in ["electronics", "clothing", "books"].
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        let json_true = json!({
            "Product": {
                "category": "furniture"
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["full_price"]);

        let json_false = json!({
            "Product": {
                "category": "electronics"
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["full_price"]);
    }

    #[test]
    fn test_contains() {
        let rule_text = r#"
            A **Message** gets flagged
              if the __content__ of the **Message** contains "urgent".
            "#;

        let rule_set = parse_rules(rule_text).unwrap();

        let json_true = json!({
            "Message": {
                "content": "This is an urgent message"
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["flagged"]);

        let json_false = json!({
            "Message": {
                "content": "This is a normal message"
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
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

        let json_true = json!({
            "Person": {
                "age": 70
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["senior_discount"]);

        let json_false = json!({
            "Person": {
                "age": 60
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["senior_discount"]);
    }

    #[test]
    fn test_property_name_transformation() {
        let rule_text = r#"
    A **Person** gets discount
      if the __first name__ of the **Person** is equal to "John".
    "#;

        let rule_set = runner::parser::parse_rules(rule_text).unwrap();

        let json_true = serde_json::json!({
            "Person": {
                "firstName": "John"
            }
        });
        let (results_true, _trace_true) = runner::evaluator::evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["discount"]);

        let json_false = serde_json::json!({
            "Person": {
                "firstName": "Jane"
            }
        });
        let (results_false, _trace_false) = runner::evaluator::evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["discount"]);
    }

    #[test]
    fn test_super_simple() {
        let rule_text = r#"
        # Driving Test Rules
        A **Person** gets a full driving license
          if the __age__ of the **Person** is greater than or equal to 17
          and the __driving test score__ of the **Person** is greater than or equal to 60.
    "#;
        let rule_set = runner::parser::parse_rules(rule_text).unwrap();
        let json_true = serde_json::json!({
        "Person": {
            "age": 18,
            "drivingTestScore": 60
        }
    });
        let (result_true, _trace_true) = runner::evaluator::evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(result_true["a full driving license"]);

        let json_false = serde_json::json!({
            "Person": {
                "age": 18,
                "drivingTestScore": 59
            }
        });
        let (result_false, _trace_false) = runner::evaluator::evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!result_false["a full driving license"]);
    }

    #[test]
    fn test_reference() {
        let rule_text = r#"
        # Driving Test Rules
        A **Person** gets a full driving license
          if the __age__ of the **Person** is greater than or equal to 17
          and the **Person** passes the practical driving test
          and the **Person** passes the eye test.

        A **Person** passes the practical driving test
          if the __driving test score__ of the **Person** is greater than or equal to 60.
    "#;
        let rule_set = runner::parser::parse_rules(rule_text).unwrap();
        let json_true = serde_json::json!({
            "Person": {
                "age": 18,
                "drivingTestScore": 60
            }
        });
        let (result_true, _trace_true) = runner::evaluator::evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(result_true["a full driving license"]);

        let json_false = serde_json::json!({
            "Person": {
                "age": 18,
                "drivingTestScore": 59
            }
        });
        let (result_false, _trace_false) = runner::evaluator::evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!result_false["a full driving license"]);
    }

    #[test]
    fn test_or() {
        let rule_text = r#"
        # Driving Test Rules
        A **Person** gets a full driving license
          if the __age__ of the **Person** is greater than or equal to 17
          or the **Person** passes the practical driving test
          and the **Person** passes the eye test.

        A **Person** passes the practical driving test
          if the __driving test score__ of the **Person** is greater than or equal to 60.
    "#;
        let rule_set = runner::parser::parse_rules(rule_text).unwrap();
        let json_true = serde_json::json!({
            "Person": {
                "age": 18,
                "drivingTestScore": 60
            }
        });
        let (result_true, _trace_true) = runner::evaluator::evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(result_true["a full driving license"]);

        let json_true_or = serde_json::json!({
            "Person": {
                "age": 18,
                "drivingTestScore": 50
            }
        });
        let (result_true, _trace_true) = runner::evaluator::evaluate_rule_set(&rule_set, &json_true_or).unwrap();
        assert!(result_true["a full driving license"]);

        let json_false = serde_json::json!({
            "Person": {
                "age": 16,
                "drivingTestScore": 59
            }
        });
        let (result_false, _trace_false) = runner::evaluator::evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!result_false["a full driving license"]);
    }

    #[test]
    fn test_random_ref() {
        let rule_text = r#"
        # Driving Test Rules
        A **Person** gets a full driving license
          if the __age__ of the **Person** is greater than or equal to 17
          and the **Person** bob.

        A **Person** bob
          if the __driving test score__ of the **Person** is greater than or equal to 60.
    "#;
        let rule_set = runner::parser::parse_rules(rule_text).unwrap();
        let json_true = serde_json::json!({
            "Person": {
                "age": 18,
                "drivingTestScore": 60
            }
        });
        let (result_true, _trace_true) = runner::evaluator::evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(result_true["a full driving license"]);

        let json_false = serde_json::json!({
            "Person": {
                "age": 18,
                "drivingTestScore": 59
            }
        });
        let (result_false, _trace_false) = runner::evaluator::evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!result_false["a full driving license"]);
    }

    #[test]
    fn object_against_object() {
        let rule_text = r#"
        # Driving Test Rules
        A **Person** gets a full driving license
          if the __age__ of the **Person** is greater than or equal to 17
          and the **Person** passes the practical driving test
          and the **Person** passes the eye test.

        A **Person** passes the practical driving test
          if the __driving test__ of the **scores** is greater than or equal to 60.
    "#;
        let rule_set = runner::parser::parse_rules(rule_text).unwrap();
        let json_true = serde_json::json!({
            "Person": {
                "age": 18
            },
            "scores": {
                "drivingTest": 61,
                "theory": 5
            }
        });
        let (result_true, _trace_true) = runner::evaluator::evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(result_true["a full driving license"]);

        let json_false = serde_json::json!({
            "Person": {
                "age": 18
            },
            "scores": {
                "drivingTest": 59,
                "theory": 5
            }
        });
        let (result_false, _trace_false) = runner::evaluator::evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!result_false["a full driving license"]);
    }

    // #[test]
    // fn test_wierd_ref() {
    //     let rule_text = r#"
    //     An **employee** is Zoom Setup Aligned
    //       if **employee** is covered by at least one rule.
    //
    //     An **employee** is covered by at least one rule
    //       if **employee** satisfies rule 1 - No Zoom Profile
    //       or **employee** satisfies next-Criteria 2.
    //
    //     An  **employee** satisfies rule 1 - No Zoom Profile
    //       if __zoom setup__ of the **employee** is equal to "No Zoom Account".
    //
    //     1.Banker.Model. An **employee** satisfies next-Criteria 2
    //       if **employee** satisfies rule 2 - Banker Model
    //       or **employee** satisfies next-Criteria 3.
    //
    //     An **employee** satisfies rule 2 - Banker Model
    //       if __banker model list__ of the **employee** is equal to "Yes".
    //
    //     An **employee** satisfies next-Criteria 3
    //       if **employee** satisfies rule 3 - Recorded Zoom
    //       or **employee** satisfies next-Criteria 4.
    //
    //     An **employee** satisfies rule 3 - Recorded Zoom
    //       if __zoom setup__ of the **employee** is equal to "Recorded Zoom"
    //       and __zoom profile__ of the **employee** is equal to "Recorded Zoom".
    //
    //     An **employee** satisfies next-Criteria 4
    //       if **employee** satisfies rule 4 - Standard Zoom
    //       or **employee** satisfies next-Criteria 5.
    //
    //     An **employee** satisfies rule 4 - Standard Zoom
    //       if __zoom setup__ of the **employee** is equal to "Standard Zoom"
    //       and __zoom profile__ of the **employee** is equal to "Standard Zoom".
    //
    //     An **employee** satisfies next-Criteria 5
    //       if **employee** satisfies rule 5 - Disclaimer Zoom.
    //
    //     An **employee** satisfies rule 5 - Disclaimer Zoom
    //       if __zoom setup__ of the **employee** is equal to "Disclaimer Zoom"
    //       and __zoom profile__ of the **employee** is equal to "Disclaimer Zoom".
    //     "#;
    //     let rule_set = runner::parser::parse_rules(rule_text).unwrap();
    //     let json_true = serde_json::json!({
    //       "employee": {
    //         "soeId": "JM78873",
    //         "name": "Joey",
    //         "ZoomProfile": "Recorded Zoom",
    //         "BankerModelList": "No",
    //         "ZoomSetup": "No Zoom Account",
    //       }
    //     });
    //     let (result_true, _trace_true) = runner::evaluator::evaluate_rule_set(&rule_set, &json_true).unwrap();
    //     assert!(result_true["Zoom Setup Aligned"]);
    //
    //     let json_false = serde_json::json!({
    //       "employee": {
    //         "soeId": "JM78873",
    //         "name": "Joey",
    //         "ZoomProfile": "Recorded Zoom",
    //         "BankerModelList": "No",
    //         "ZoomSetup": "Beep",
    //       }
    //     });
    //     let (result_false, _trace_false) = runner::evaluator::evaluate_rule_set(&rule_set, &json_false).unwrap();
    //     assert!(!result_false["Zoom Setup Aligned"]);
    // }

    #[test]
    fn test_chained_property_access_success() {
        let rule_text = r#"
          A **user** can access a system
            if **user** is part of allowed group.

          A **user** is part of allowed group
            if __id__ of __group__ of **user** is in __allowed groups__ of __permissions__ of **service**.
        "#;
        let rule_set = parse_rules(rule_text).expect("Failed to parse rules");

        let data_true = json!({
            "user": {
                "group": {
                    "id": 1
                }
            },
            "service": {
                "permissions": {
                    "allowedGroups": [9, 8, 3, 1]
                }
            }
        });

        // Parse the rules
        let (result_true, _trace_true) = runner::evaluator::evaluate_rule_set(&rule_set, &data_true).unwrap();
        assert!(result_true["can access a system"]);

        let data_false = json!({
            "user": {
                "group": {
                    "id": 5
                }
            },
            "service": {
                "permissions": {
                    "allowedGroups": [9, 8, 3, 1]
                }
            }
        });

        // Evaluate the rules
        let (result_false, _trace_false) = runner::evaluator::evaluate_rule_set(&rule_set, &data_false).unwrap();
        assert!(!result_false["can access a system"]);
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that old simple property conditions still work
        let rule_text = r#"
          A **user** passes the test
            if __score__ of **user** is greater than 80.
        "#;

        let data = json!({
            "user": {
                "score": 85
            }
        });

        // Parse the rules
        let rule_set = parse_rules(rule_text).expect("Failed to parse rules");

        // Evaluate the rules
        let (results, _trace) = evaluate_rule_set(&rule_set, &data)
            .expect("Failed to evaluate rules");

        // Check that the user passes the test
        assert!(results.get("the test").unwrap_or(&false), "User should pass the test");
    }
}



