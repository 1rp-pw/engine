mod runner;

#[cfg(test)]
mod tests {
    use super::*;
    use runner::evaluator::evaluate_rule_set;
    use runner::parser::parse_rules;
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
        let (results_true, _trace_true) =
            runner::evaluator::evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["discount"]);

        let json_false = serde_json::json!({
            "Person": {
                "firstName": "Jane"
            }
        });
        let (results_false, _trace_false) =
            runner::evaluator::evaluate_rule_set(&rule_set, &json_false).unwrap();
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
        let (result_true, _trace_true) =
            runner::evaluator::evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(result_true["a full driving license"]);

        let json_false = serde_json::json!({
            "Person": {
                "age": 18,
                "drivingTestScore": 59
            }
        });
        let (result_false, _trace_false) =
            runner::evaluator::evaluate_rule_set(&rule_set, &json_false).unwrap();
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
        let (result_true, _trace_true) =
            runner::evaluator::evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(result_true["a full driving license"]);

        let json_false = serde_json::json!({
            "Person": {
                "age": 18,
                "drivingTestScore": 59
            }
        });
        let (result_false, _trace_false) =
            runner::evaluator::evaluate_rule_set(&rule_set, &json_false).unwrap();
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
        let (result_true, _trace_true) =
            runner::evaluator::evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(result_true["a full driving license"]);

        let json_true_or = serde_json::json!({
            "Person": {
                "age": 18,
                "drivingTestScore": 50
            }
        });
        let (result_true, _trace_true) =
            runner::evaluator::evaluate_rule_set(&rule_set, &json_true_or).unwrap();
        assert!(result_true["a full driving license"]);

        let json_false = serde_json::json!({
            "Person": {
                "age": 16,
                "drivingTestScore": 59
            }
        });
        let (result_false, _trace_false) =
            runner::evaluator::evaluate_rule_set(&rule_set, &json_false).unwrap();
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
        let (result_true, _trace_true) =
            runner::evaluator::evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(result_true["a full driving license"]);

        let json_false = serde_json::json!({
            "Person": {
                "age": 18,
                "drivingTestScore": 59
            }
        });
        let (result_false, _trace_false) =
            runner::evaluator::evaluate_rule_set(&rule_set, &json_false).unwrap();
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
        let (result_true, _trace_true) =
            runner::evaluator::evaluate_rule_set(&rule_set, &json_true).unwrap();
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
        let (result_false, _trace_false) =
            runner::evaluator::evaluate_rule_set(&rule_set, &json_false).unwrap();
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
    //         "ZoomSetup": "Recorded Zoom",
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
        let (result_true, _trace_true) =
            runner::evaluator::evaluate_rule_set(&rule_set, &data_true).unwrap();
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
        let (result_false, _trace_false) =
            runner::evaluator::evaluate_rule_set(&rule_set, &data_false).unwrap();
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
        let (results, _trace) =
            evaluate_rule_set(&rule_set, &data).expect("Failed to evaluate rules");

        // Check that the user passes the test
        assert!(
            results.get("the test").unwrap_or(&false),
            "User should pass the test"
        );
    }

    #[test]
    fn test_string_length_greater_than() {
        let rule_text = r#"
        A **user** is valid if the length of __name__ of **user** is greater than 5.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case where name length is greater than 5
        let json_true = json!({
            "user": {
                "name": "bobby socks"  // 11 characters
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["valid"]);

        // Test case where name length is not greater than 5
        let json_false = json!({
            "user": {
                "name": "bob"  // 3 characters
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["valid"]);
    }

    #[test]
    fn test_array_length_greater_than() {
        let rule_text = r#"
        A **user** is valid if the length of __items__ of **user** is greater than 3.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case where array length is greater than 3
        let json_true = json!({
            "user": {
                "items": [1, 2, 3, 4, 5]  // 5 items
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["valid"]);

        // Test case where array length is not greater than 3
        let json_false = json!({
            "user": {
                "items": [1, 2]  // 2 items
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["valid"]);
    }

    #[test]
    fn test_array_number_greater_than() {
        let rule_text = r#"
        A **user** is valid if the number of __items__ of **user** is greater than 3.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case where array length is greater than 3
        let json_true = json!({
            "user": {
                "items": [1, 2, 3, 4, 5]  // 5 items
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["valid"]);

        // Test case where array length is not greater than 3
        let json_false = json!({
            "user": {
                "items": [1, 2]  // 2 items
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["valid"]);

        // Test case where "number of" is used on a string - should return an error
        let json_error = json!({
            "user": {
                "items": "tester"  // string, not an array
            }
        });
        let result = evaluate_rule_set(&rule_set, &json_error);
        assert!(result.is_err());
        // Optionally check that the error message indicates the invalid use of "number of" on a string
        if let Err(error) = result {
            let error_msg = error.to_string();
            assert!(
                error_msg.contains("number of")
                    || error_msg.contains("array")
                    || error_msg.contains("string")
            );
        }
    }

    #[test]
    fn test_string_length_equal_to() {
        let rule_text = r#"
        A **user** passes validation if the length of __password__ of **user** is equal to 8.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case where password length equals 8
        let json_true = json!({
            "user": {
                "password": "secure12"  // 8 characters
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["validation"]);

        // Test case where password length doesn't equal 8
        let json_false = json!({
            "user": {
                "password": "short"  // 5 characters
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["validation"]);
    }

    #[test]
    fn test_length_with_nested_properties() {
        let rule_text = r#"
        A **user** is valid if the length of __street__ of __address__ of **user** is greater than 10.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case with nested property having sufficient length
        let json_true = json!({
            "user": {
                "address": {
                    "street": "123 Very Long Street Name"  // 25 characters
                }
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["valid"]);

        // Test case with nested property having insufficient length
        let json_false = json!({
            "user": {
                "address": {
                    "street": "Short St"  // 8 characters
                }
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["valid"]);
    }

    #[test]
    fn test_empty_array_length() {
        let rule_text = r#"
        A **user** has items if the length of __items__ of **user** is greater than 0.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case with empty array
        let json_false = json!({
            "user": {
                "items": []  // 0 items
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["items"]);

        // Test case with non-empty array
        let json_true = json!({
            "user": {
                "items": ["one"]  // 1 item
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["items"]);
    }

    #[test]
    fn test_length_with_multiple_operators() {
        let rule_text = r#"
        A **user** is valid if the length of __name__ of **user** is greater than or equal to 3
          and the length of __name__ of **user** is less than or equal to 20.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case within valid range
        let json_true = json!({
            "user": {
                "name": "ValidName"  // 9 characters, between 3 and 20
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["valid"]);

        // Test case too short
        let json_false_short = json!({
            "user": {
                "name": "AB"  // 2 characters, less than 3
            }
        });
        let (results_false, _trace_false) =
            evaluate_rule_set(&rule_set, &json_false_short).unwrap();
        assert!(!results_false["valid"]);

        // Test case too long
        let json_false_long = json!({
            "user": {
                "name": "ThisNameIsWayTooLongToBeValid"  // 30 characters, more than 20
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false_long).unwrap();
        assert!(!results_false["valid"]);
    }

    #[test]
    fn test_length_of_object_properties() {
        let rule_text = r#"
        A **user** has sufficient data if the length of __profile__ of **user** is greater than 2.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case with object having more than 2 properties
        let json_true = json!({
            "user": {
                "profile": {
                    "name": "John",
                    "age": 30,
                    "email": "john@example.com"  // 3 properties
                }
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["sufficient data"]);

        // Test case with object having 2 or fewer properties
        let json_false = json!({
            "user": {
                "profile": {
                    "name": "John"  // 1 property
                }
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["sufficient data"]);
    }

    #[test]
    fn test_length_error_cases() {
        let rule_text = r#"
        A **user** is valid if the length of __value__ of **user** is greater than 5.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test case with null value
        let json_null = json!({
            "user": {
                "value": null
            }
        });
        let (results_null, _trace_null) = evaluate_rule_set(&rule_set, &json_null).unwrap();
        assert!(!results_null["valid"]); // Should fail gracefully, length of null is 0

        // Test case with missing property
        let json_missing = json!({
            "user": {
                "other_field": "value"
            }
        });
        let (results_missing, _trace_missing) =
            evaluate_rule_set(&rule_set, &json_missing).unwrap();
        assert!(!results_missing["valid"]); // Should fail when property doesn't exist
    }

    #[test]
    fn test_length_with_alternative_syntax() {
        // Test without "the" article
        let rule_text = r#"
        A **user** is valid if length of __name__ of **user** is greater than 3.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        let json_true = json!({
            "user": {
                "name": "ValidName"  // 9 characters
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["valid"]);
    }

    #[test]
    fn test_length_comparison_with_different_operators() {
        let operators_and_expected = vec![
            ("is greater than", 5, "longname", true),          // 8 > 5
            ("is greater than", 10, "longname", false),        // 8 > 10 = false
            ("is less than", 10, "short", true),               // 5 < 10
            ("is less than", 3, "short", false),               // 5 < 3 = false
            ("is greater than or equal to", 5, "hello", true), // 5 >= 5
            ("is less than or equal to", 5, "hello", true),    // 5 <= 5
            ("is equal to", 4, "test", true),                  // 4 == 4
            ("is not equal to", 5, "test", true),              // 4 != 5
        ];

        for (operator, threshold, name, expected) in operators_and_expected {
            let rule_text = format!(
                r#"A **user** is valid if the length of __name__ of **user** {} {}."#,
                operator, threshold
            );

            let rule_set = parse_rules(&rule_text).unwrap();

            let json_data = json!({
                "user": {
                    "name": name
                }
            });

            let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
            assert_eq!(
                results["valid"],
                expected,
                "Failed for operator '{}' with threshold {} and name '{}' (length {})",
                operator,
                threshold,
                name,
                name.len()
            );
        }
    }

    #[test]
    fn test_existing_simple_rule_still_works() {
        let input =
            r#"A **user** passes the test if __age__ of **user** is greater than or equal to 18."#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        assert_eq!(rule_set.rules.len(), 1);

        let rule = &rule_set.rules[0];
        assert_eq!(rule.selector, "user");
        assert_eq!(rule.outcome, "the test");
        assert_eq!(rule.conditions.len(), 1);
    }

    #[test]
    fn test_existing_string_comparison_still_works() {
        let input = r#"
A **user** is valid if __status__ of **user** is equal to "active".
        "#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let json_data = json!({
            "user": {
                "status": "active"
            }
        });

        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(results["valid"]);
    }

    #[test]
    fn test_existing_boolean_comparison_still_works() {
        let input = r#"A **user** is premium if __is_premium__ of **user** is equal to true."#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let json_data = json!({
            "user": {
                "is_premium": true
            }
        });

        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(results["premium"]);
    }

    #[test]
    fn test_existing_date_comparison_still_works() {
        let input = r#"A **user** is eligible if __birth_date__ of **user** is earlier than date(2000-01-01)."#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();
        let json_data = json!({
            "user": {
                "birth_date": "1995-06-15"
            }
        });

        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(results["eligible"]);
    }

    #[test]
    fn test_new_length_functionality_works() {
        let input =
            r#"A **user** is valid if the length of __name__ of **user** is greater than 5."#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();

        // Test with long name
        let json_true = json!({
            "user": {
                "name": "bobby socks"  // 11 characters
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["valid"]);

        // Test with short name
        let json_false = json!({
            "user": {
                "name": "bob"  // 3 characters
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false).unwrap();
        assert!(!results_false["valid"]);
    }

    #[test]
    fn test_both_old_and_new_in_same_rule() {
        let input = r#"
A **user** is valid if __age__ of **user** is greater than 18
  and the length of __name__ of **user** is greater than 3.
        "#;

        let result = parse_rules(input);
        assert!(result.is_ok());

        let rule_set = result.unwrap();

        // Both conditions true
        let json_true = json!({
            "user": {
                "age": 25,
                "name": "John Doe"
            }
        });
        let (results_true, _trace_true) = evaluate_rule_set(&rule_set, &json_true).unwrap();
        assert!(results_true["valid"]);

        // Age condition false
        let json_false_age = json!({
            "user": {
                "age": 16,
                "name": "John Doe"
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false_age).unwrap();
        assert!(!results_false["valid"]);

        // Name length condition false
        let json_false_name = json!({
            "user": {
                "age": 25,
                "name": "Jo"
            }
        });
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false_name).unwrap();
        assert!(!results_false["valid"]);
    }

    #[test]
    fn test_exact_property_name_match() {
        let input = r#"A **user** is premium if __is_premium__ of **user** is equal to true."#;
        let rule_set = parse_rules(input).unwrap();

        // JSON with exact property name match
        let json_data = json!({
            "user": {
                "is_premium": true
            }
        });

        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(
            results["premium"],
            "Should match exact property name 'is_premium'"
        );
    }

    #[test]
    fn test_transformed_property_name_match() {
        let input = r#"A **user** is premium if __is_premium__ of **user** is equal to true."#;
        let rule_set = parse_rules(input).unwrap();

        // JSON with camelCase property name
        let json_data = json!({
            "user": {
                "isPremium": true
            }
        });

        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(
            results["premium"],
            "Should match transformed property name 'isPremium'"
        );
    }

    #[test]
    fn test_case_insensitive_property_match() {
        let input = r#"A **user** is premium if __is_premium__ of **user** is equal to true."#;
        let rule_set = parse_rules(input).unwrap();

        // JSON with different case
        let json_data = json!({
            "user": {
                "IS_PREMIUM": true
            }
        });

        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(
            results["premium"],
            "Should match case-insensitive property name 'IS_PREMIUM'"
        );
    }

    #[test]
    fn test_case_insensitive_transformed_property_match() {
        let input = r#"A **user** is premium if __is_premium__ of **user** is equal to true."#;
        let rule_set = parse_rules(input).unwrap();

        // JSON with different case on transformed property
        let json_data = json!({
            "user": {
                "ISPREMIUM": true
            }
        });

        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(
            results["premium"],
            "Should match case-insensitive transformed property name 'ISPREMIUM'"
        );
    }

    #[test]
    fn test_multiple_word_property_transformation() {
        let input = r#"A **user** is valid if __first_name__ of **user** is equal to "John"."#;
        let rule_set = parse_rules(input).unwrap();

        // Test with exact match
        let json_exact = json!({
            "user": {
                "first_name": "John"
            }
        });
        let (results, _trace) = evaluate_rule_set(&rule_set, &json_exact).unwrap();
        assert!(results["valid"], "Should match exact 'first_name'");

        // Test with camelCase
        let json_camel = json!({
            "user": {
                "firstName": "John"
            }
        });
        let (results, _trace) = evaluate_rule_set(&rule_set, &json_camel).unwrap();
        assert!(results["valid"], "Should match camelCase 'firstName'");
    }

    #[test]
    fn test_space_separated_property_transformation() {
        let input = r#"A **user** is valid if __first name__ of **user** is equal to "John"."#;
        let rule_set = parse_rules(input).unwrap();

        // Test with camelCase transformation
        let json_camel = json!({
            "user": {
                "firstName": "John"
            }
        });
        let (results, _trace) = evaluate_rule_set(&rule_set, &json_camel).unwrap();
        assert!(
            results["valid"],
            "Should transform 'first name' to 'firstName'"
        );
    }

    #[test]
    fn test_property_precedence() {
        // If both exact and transformed properties exist, exact should take precedence
        let input = r#"A **user** is valid if __is_premium__ of **user** is equal to true."#;
        let rule_set = parse_rules(input).unwrap();

        let json_data = json!({
            "user": {
                "is_premium": true,    // Exact match
                "isPremium": false     // Transformed match
            }
        });

        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(
            results["valid"],
            "Should prefer exact match 'is_premium' over 'isPremium'"
        );
    }

    #[test]
    fn test_nested_property_transformation() {
        let input = r#"A **user** is valid if __street_name__ of __home_address__ of **user** is equal to "Main St"."#;
        let rule_set = parse_rules(input).unwrap();

        // Test with mixed property name styles
        let json_data = json!({
            "user": {
                "homeAddress": {  // Transformed selector property
                    "streetName": "Main St"  // Transformed nested property
                }
            }
        });

        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(
            results["valid"],
            "Should handle nested property transformations"
        );
    }

    #[test]
    fn test_date_property_transformation() {
        let input =
            r#"A **user** is young if __birth_date__ of **user** is later than "2000-01-01"."#;
        let rule_set = parse_rules(input).unwrap();

        let json_data = json!({
            "user": {
                "birthDate": "2005-06-15"  // Transformed property name
            }
        });

        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(
            results["young"],
            "Should handle date comparison with transformed property"
        );
    }

    #[test]
    fn test_length_with_property_transformation() {
        let input =
            r#"A **user** is valid if the length of __full_name__ of **user** is greater than 5."#;
        let rule_set = parse_rules(input).unwrap();

        let json_data = json!({
            "user": {
                "fullName": "John Doe Smith"  // Transformed property name
            }
        });

        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(
            results["valid"],
            "Should handle length calculation with transformed property"
        );
    }

    #[test]
    fn test_property_not_found_error() {
        let input =
            r#"A **user** is valid if __nonexistent_property__ of **user** is equal to true."#;
        let rule_set = parse_rules(input).unwrap();

        let json_data = json!({
            "user": {
                "some_other_property": true
            }
        });

        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(
            !results["valid"],
            "Should return false when property is not found"
        );
    }

    #[test]
    fn test_property_against_property() {
        let input = r#"A **user** is valid if __age__ of **user** is greater than __min_age__ of **config**."#;
        let rule_set = parse_rules(input).unwrap();

        let json_data = json!({
            "user": {
                "age": 25
            },
            "config": {
                "min_age": 18
            }
        });
        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(results["valid"]);
    }

    #[test]
    fn test_outcome_verbs_multi_word() {
        // Test that "is valid" works (single outcome_verb "is" + outcome "valid")
        let rule1 = r#"A **student** is valid
            if the __criminal_background_clear__ of the **background.criminal** in the **student** is equal to true."#;
        
        let result1 = parse_rules(rule1);
        assert!(result1.is_ok(), "Rule with 'is valid' should parse successfully");
        
        let rule_set1 = result1.unwrap();
        assert_eq!(rule_set1.rules[0].outcome, "valid");

        // Test that "has passed background verification" now works correctly
        let rule2 = r#"A **student** has passed background verification
            if the __criminal_background_clear__ of the **background.criminal** in the **student** is equal to true."#;
        
        let result2 = parse_rules(rule2);
        assert!(result2.is_ok(), "Rule with 'has passed background verification' should parse successfully");
        
        let rule_set2 = result2.unwrap();
        assert_eq!(rule_set2.rules[0].outcome, "passed background verification");
    }

    #[test]
    fn test_flexible_outcome_names() {
        // Test various outcome formats without hardcoded verbs
        let test_cases = vec![
            // Without any recognized verb - should take entire text as outcome
            ("A **user** can_access_admin_panel if __role__ of **user** is equal to \"admin\".", "can_access_admin_panel"),
            ("A **user** should-be-promoted if __performance__ of **user** is greater than 90.", "should-be-promoted"),
            ("A **user** will receive bonus payment if __sales__ of **user** is greater than 100000.", "will receive bonus payment"),
            
            // With recognized verbs
            ("A **user** gets premium_access if __tier__ of **user** is equal to \"gold\".", "premium_access"),
            ("A **user** is authorized_for_deletion if __admin__ of **user** is equal to true.", "authorized_for_deletion"),
            
            // Edge cases
            ("A **user** passed_all_checks if __verified__ of **user** is equal to true.", "passed_all_checks"),
            
            // Test the problematic case - should work since "has" is a recognized verb
            ("A **student** has background_verification_passed if __age__ of **student** is greater than 18.", "background_verification_passed"),
        ];

        for (rule_text, expected_outcome) in test_cases {
            let result = parse_rules(rule_text);
            assert!(result.is_ok(), "Failed to parse rule: {}\nError: {:?}", rule_text, result);
            
            let rule_set = result.unwrap();
            assert_eq!(rule_set.rules[0].outcome, expected_outcome, 
                "Incorrect outcome for rule: {}", rule_text);
        }
    }
    
    #[test]
    fn test_outcome_with_if_keyword() {
        // Test that the issue is with words containing "if"
        let problematic_cases = vec![
            ("A **student** has passed verification if __age__ of **student** is greater than 18.", "passed verification"),
            ("A **student** has qualified_verification if __age__ of **student** is greater than 18.", "qualified_verification"),
            ("A **student** has passed background verification if __age__ of **student** is greater than 18.", "passed background verification"),
        ];
        
        for (rule_text, _expected_outcome) in problematic_cases {
            let result = parse_rules(rule_text);
            // These should fail because "verification" contains "if"
            assert!(result.is_err(), "Expected parse error for rule with 'if' in outcome: {}", rule_text);
        }
        
        // Test workarounds
        let working_cases = vec![
            ("A **student** has passed_background_checks if __age__ of **student** is greater than 18.", "passed_background_checks"),
            ("A **student** passes background verification if __age__ of **student** is greater than 18.", "background verification"),
            ("A **student** completes verification if __age__ of **student** is greater than 18.", "verification"),
        ];
        
        for (rule_text, expected_outcome) in working_cases {
            let result = parse_rules(rule_text);
            assert!(result.is_ok(), "Failed to parse rule: {}\nError: {:?}", rule_text, result);
            
            let rule_set = result.unwrap();
            assert_eq!(rule_set.rules[0].outcome, expected_outcome, 
                "Incorrect outcome for rule: {}", rule_text);
        }
    }

    #[test]
    fn test_username_and_password_validation() {
        let input = r#"A **login** is valid
  if **login** passes the username tests
  and **login** passes the password tests.

A **login** passes the username tests
  if length of __username__ in **login** is at least 3.

A **login** passes the password tests
  if length of __password__ in **login** is at least 5."#;

        let rule_set = parse_rules(input).unwrap();

        let json_good = json!({
            "login": {
                "username": "john",
                "password": "<PASSWORD>"
            }
        });
        let (results_good, _trace_good) = evaluate_rule_set(&rule_set, &json_good).unwrap();
        assert!(results_good["valid"]);

        let json_bad_username = json!({
            "login": {
                "username": "jo",
                "password": "<PASSWORD>"
            }
        });
        let (results_bad_username, _trace_bad_username) =
            evaluate_rule_set(&rule_set, &json_bad_username).unwrap();
        assert!(!results_bad_username["valid"]);
    }

    #[test]
    fn test_user_defined_conceptual_object() {
        let input = r#"A **bob** is valid if **bob** passes the test.

A **bob** passes the test if __name__ of **user** is equal to "bob"."#;

        let rule_set = parse_rules(input).unwrap();

        let json_data = json!({
            "user": {
                "name": "bob"
            }
        });

        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(
            results["valid"],
            "Bob should be valid when user name is bob"
        );

        let json_data_false = json!({
            "user": {
                "name": "alice"
            }
        });

        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_data_false).unwrap();
        assert!(
            !results_false["valid"],
            "Bob should not be valid when user name is alice"
        );
    }

    #[test]
    fn full_driving_test() {
        let input = r#"A **driving test** gets a driving licence
  if the **driving test** passes the age test
  and the **driving test** passes the test requirements
  and the **driving test** has taken the test in the time period.

A **driving test** passes the age test
  if the __date of birth__ of the **person** of the **driving test** is earlier than 2008-12-12.

A **driving test** passes the test requirements
  if **driving test** passes the theory test
  and the **driving test** passes the practical test.

A **driving test** passes the theory test
  if the __multiple choice__ of the **theory** of the **scores** of the **driving test** is at least 43
  and the __hazard perception__ of the **theory** of the **scores** of the **driving test** is at least 44.

A **driving test** passes the practical test
  if the __minor__ of the __practical__ of the __scores__ of the **driving test** is no more than 15
  and the __major__ of the __practical__ of the __scores__ of the **driving test** is equal to false.

A **driving test** has taken the test in the time period
  if the __theory__ of the **testDates** of the **driving test** is within 2 years
  and the __practical__ of the **testDates** of the **driving test** is within 30 days."#;

        let rule_set = parse_rules(input).unwrap();

        let json_good = json!({
          "drivingTest": {
            "person": {
              "dateOfBirth": "1990-01-01",
              "name": "Bob"
            },
            "scores": {
              "practical": {
                "major": false,
                "minor": 13
              },
              "theory": {
                "hazardPerception": 75,
                "multipleChoice": 45
              }
            },
            "testDates": {
              "practical": "2025-06-01",
              "theory": "2024-12-12"
            }
          }
        });
        let (results_good, _trace_good) = evaluate_rule_set(&rule_set, &json_good).unwrap();
        assert!(results_good["a driving licence"]);

        let json_bad = json!({
          "drivingTest": {
            "person": {
              "dateOfBirth": "1990-01-01",
              "name": "Bob"
            },
            "scores": {
              "practical": {
                "major": true,
                "minor": 13
              },
              "theory": {
                "hazardPerception": 75,
                "multipleChoice": 45
              }
            },
            "testDates": {
              "practical": "2025-06-01",
              "theory": "2024-12-12"
            }
          }
        });
        let (results_bad, _trace_bad) = evaluate_rule_set(&rule_set, &json_bad).unwrap();
        assert!(!results_bad["a driving licence"]);
    }

    #[test]
    fn test_labels_in_evaluation_result() {
        use runner::evaluator::evaluate_rule_set_with_trace;

        let rules = r#"
A **driver** gets a driving licence
  if §driver.test is valid.

driver.test. A **driver** passes the age test
  if __age__ of **driver** is greater than or equal to 18.
        "#;

        let data = json!({
            "driver": {
                "age": 25
            }
        });

        let result = parse_rules(rules).unwrap();
        let eval_result = evaluate_rule_set_with_trace(&result, &data);

        assert!(eval_result.result.is_ok());
        assert!(eval_result.trace.is_some());

        // Check that the trace contains labeled rules
        let trace = eval_result.trace.unwrap();
        let mut found_label = false;
        for rule_trace in &trace.execution {
            if let Some(label) = &rule_trace.label {
                if label == "driver.test" {
                    found_label = true;
                    assert!(rule_trace.result, "Labeled rule should have passed");
                }
            }
        }
        assert!(
            found_label,
            "Should have found the driver.test label in trace"
        );
    }

    #[test]
    fn test_dollar_label_reference_evaluation() {
        use runner::evaluator::evaluate_rule_set_with_trace;

        let rules = r#"
A **user** gets access
  if $admin is valid
  or $manager succeeds.

admin. A **user** is admin
  if __role__ of **user** is equal to "admin".

manager. A **user** is manager
  if __role__ of **user** is equal to "manager".
        "#;

        // Test with admin role
        let admin_data = json!({
            "user": {
                "role": "admin"
            }
        });

        let result = parse_rules(rules).unwrap();
        let eval_result = evaluate_rule_set_with_trace(&result, &admin_data);

        assert!(eval_result.result.is_ok());
        let results = eval_result.result.unwrap();
        assert!(results["access"], "Admin should get access");

        // Test with manager role
        let manager_data = json!({
            "user": {
                "role": "manager"
            }
        });

        let eval_result2 = evaluate_rule_set_with_trace(&result, &manager_data);
        assert!(eval_result2.result.is_ok());
        let results2 = eval_result2.result.unwrap();
        assert!(results2["access"], "Manager should get access");

        // Test with neither role
        let user_data = json!({
            "user": {
                "role": "user"
            }
        });

        let eval_result3 = evaluate_rule_set_with_trace(&result, &user_data);
        assert!(eval_result3.result.is_ok());
        let results3 = eval_result3.result.unwrap();
        assert!(!results3["access"], "Regular user should not get access");
    }

    #[test]
    fn full_driving_test_custom_object() {
        let input = r#"A **driver** gets a driving licence
  if the **driver** passes the age test
  and the **driver** passes the test requirements
  and the **driver** has taken the test in the time period.

A **driver** passes the age test
  if the __date of birth__ of the **person** of the **driving test** is earlier than 2008-12-12.

A **driver** passes the test requirements
  if **driving test** passes the theory test
  and the **driving test** passes the practical test.

A **driver** passes the theory test
  if the __multiple choice__ of the __theory__ of the __scores__ of the **driving test** is at least 43
  and the __hazard perception__ of the __theory__ of the __scores__ of the **driving test** is at least 44.

A **driver** passes the practical test
  if the __minor__ of the __practical__ of the __scores__ in the **driving test** is no more than 15
  and the __major__ of the **practical** of the **scores** in the **driving test** is equal to false.

A **driver** has taken the test in the time period
  if the __theory__ of the __test dates__ in the **driving test** is within 2 years
  and the __practical__ of the __test dates__ in the **driving test** is within 30 days."#;

        let rule_set = parse_rules(input).unwrap();

        let json_good = json!({
          "drivingTest": {
            "person": {
              "dateOfBirth": "1990-01-01",
              "name": "Bob"
            },
            "scores": {
              "practical": {
                "major": false,
                "minor": 13
              },
              "theory": {
                "hazardPerception": 75,
                "multipleChoice": 45
              }
            },
            "testDates": {
              "practical": "2025-06-01",
              "theory": "2024-12-12"
            }
          }
        });
        let (results_good, _trace_good) = evaluate_rule_set(&rule_set, &json_good).unwrap();
        assert!(results_good["a driving licence"]);

        let json_bad = json!({
          "drivingTest": {
            "person": {
              "dateOfBirth": "1990-01-01",
              "name": "Bob"
            },
            "scores": {
              "practical": {
                "major": true,
                "minor": 13
              },
              "theory": {
                "hazardPerception": 75,
                "multipleChoice": 45
              }
            },
            "testDates": {
              "practical": "2025-06-01",
              "theory": "2024-12-12"
            }
          }
        });
        let (results_bad, _trace_bad) = evaluate_rule_set(&rule_set, &json_bad).unwrap();
        assert!(!results_bad["a driving licence"]);
    }

    #[test]
    fn test_nested_selector_evaluation() {
        let rule_text = r#"
        A **user** is valid if __prop__ of **top.second** is equal to "test".
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test successful case
        let json_success = json!({
            "top": {
                "second": {
                    "prop": "test"
                }
            }
        });
        let (results, _trace) = evaluate_rule_set(&rule_set, &json_success).unwrap();
        assert!(results["valid"]);

        // Test failure case
        let json_failure = json!({
            "top": {
                "second": {
                    "prop": "wrong"
                }
            }
        });
        let (results, _trace) = evaluate_rule_set(&rule_set, &json_failure).unwrap();
        assert!(!results["valid"]);
    }

    #[test]
    fn test_deep_nested_selector_evaluation() {
        let rule_text = r#"
        A **user** passes the test if __value__ of **data.config.settings** is greater than 10.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test successful case
        let json_success = json!({
            "data": {
                "config": {
                    "settings": {
                        "value": 15
                    }
                }
            }
        });
        let (results, _trace) = evaluate_rule_set(&rule_set, &json_success).unwrap();
        assert!(results["the test"]);

        // Test failure case
        let json_failure = json!({
            "data": {
                "config": {
                    "settings": {
                        "value": 5
                    }
                }
            }
        });
        let (results, _trace) = evaluate_rule_set(&rule_set, &json_failure).unwrap();
        assert!(!results["the test"]);
    }

    #[test]
    fn test_nested_selector_case_insensitive() {
        let rule_text = r#"
        A **user** is valid if __myProp__ of **Top.Second** is equal to "test".
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test with different case combinations
        let json_test = json!({
            "top": {
                "second": {
                    "my_prop": "test"
                }
            }
        });
        let (results, _trace) = evaluate_rule_set(&rule_set, &json_test).unwrap();
        assert!(results["valid"]);
    }

    #[test]
    fn test_nested_selector_with_backward_compatibility() {
        // Test that regular selectors still work
        let rule_text1 = r#"
        A **user** is valid if __name__ of **user** is equal to "John".
        "#;

        let rule_set1 = parse_rules(rule_text1).unwrap();
        let json1 = json!({
            "user": {
                "name": "John"
            }
        });
        let (results1, _trace1) = evaluate_rule_set(&rule_set1, &json1).unwrap();
        assert!(results1["valid"]);

        // Test nested selectors
        let rule_text2 = r#"
        A **user** is valid if __name__ of **user.profile** is equal to "John".
        "#;

        let rule_set2 = parse_rules(rule_text2).unwrap();
        let json2 = json!({
            "user": {
                "profile": {
                    "name": "John"
                }
            }
        });
        let (results2, _trace2) = evaluate_rule_set(&rule_set2, &json2).unwrap();
        assert!(results2["valid"]);
    }

    #[test]
    fn test_nested_selector_error_handling() {
        let rule_text = r#"
        A **user** is valid if __prop__ of **nonexistent.path** is equal to "test".
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test with missing path
        let json_missing = json!({
            "user": {
                "name": "John"
            }
        });

        let result = evaluate_rule_set(&rule_set, &json_missing);
        match result {
            Ok((results, _)) => {
                // The evaluation succeeded but the result should be false
                // since the path doesn't exist
                assert!(
                    !results["valid"],
                    "Should return false when nested path doesn't exist"
                );
            }
            Err(_) => {
                // Also acceptable - immediate error is fine
            }
        }
    }

    #[test]
    fn test_nested_selector_path_format() {
        let rule_text = r#"
        A **user** is valid if __center__ of **drivingTest.testDates.practical** is equal to "Manchester".
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        let json_data = json!({
            "drivingTest": {
                "testDates": {
                    "practical": {
                        "center": "Manchester"
                    }
                }
            }
        });

        let eval_result =
            crate::runner::evaluator::evaluate_rule_set_with_trace(&rule_set, &json_data);
        assert!(eval_result.result.is_ok());

        let results = eval_result.result.unwrap();
        assert!(results["valid"]);

        let trace = eval_result.trace.unwrap();
        let rule_trace = &trace.execution[0];
        let condition_trace = &rule_trace.conditions[0];

        // Check that the path is correctly formatted as $.drivingTest.testDates.practical.center
        // and NOT $.drivingTest.testDates.practical.drivingTest.testDates.practical.center
        if let runner::trace::ConditionTrace::Comparison(comp_trace) = condition_trace {
            assert_eq!(
                comp_trace.property.path,
                "$.drivingTest.testDates.practical.center"
            );
        } else {
            panic!("Expected comparison trace");
        }
    }

    #[test]
    fn test_age_comparison_operators() {
        // Test older than operator
        let rule_text = r#"
        A **driver** is allowed to drive
          if the __date_of_birth__ of the **driver** is older than 18 years.
        "#;

        let rule_set = parse_rules(rule_text).unwrap();

        // Test driver who is 24 years old (allowed to drive)
        let json_driver = json!({
            "driver": {
                "date_of_birth": "2000-01-01"
            }
        });
        let (results_driver, _trace) = evaluate_rule_set(&rule_set, &json_driver).unwrap();
        assert!(results_driver["allowed to drive"]);

        // Test young driver who is 16 years old (not allowed to drive)
        let json_young_driver = json!({
            "driver": {
                "date_of_birth": "2008-01-01"
            }
        });
        let (results_young, _trace) = evaluate_rule_set(&rule_set, &json_young_driver).unwrap();
        assert!(!results_young["allowed to drive"]);

        // Test younger than operator
        let rule_text_child = r#"
        A **child** gets child discount
          if the __date_of_birth__ of the **child** is younger than 12 years.
        "#;

        let rule_set_child = parse_rules(rule_text_child).unwrap();

        // Test child who is 4 years old (gets child discount)
        let json_child = json!({
            "child": {
                "date_of_birth": "2020-06-15"
            }
        });
        let (results_child, _trace) = evaluate_rule_set(&rule_set_child, &json_child).unwrap();
        assert!(results_child["child discount"]);

        // Test teenager who is 14 years old (not a child for discount)
        let json_teen = json!({
            "child": {
                "date_of_birth": "2010-01-01"
            }
        });
        let (results_teen, _trace) = evaluate_rule_set(&rule_set_child, &json_teen).unwrap();
        assert!(!results_teen["child discount"]);

        // Test combined age rules with single golden rule
        let rule_text_combined = r#"
        A **person** gets age_appropriate_benefit
          if the **person** is a senior citizen
          or the **person** is a young child.

        A **person** is a senior citizen
          if the __date_of_birth__ of the **person** is older than 65 years.

        A **person** is a young child
          if the __date_of_birth__ of the **person** is younger than 5 years.
        "#;

        let rule_set_combined = parse_rules(rule_text_combined).unwrap();

        // Test senior who is 74 years old (gets benefit)
        let json_senior = json!({
            "person": {
                "date_of_birth": "1950-03-20"
            }
        });
        let (results_senior, _trace) = evaluate_rule_set(&rule_set_combined, &json_senior).unwrap();
        assert!(results_senior["age_appropriate_benefit"]);

        // Test toddler who is 3 years old (gets benefit)
        let json_toddler = json!({
            "person": {
                "date_of_birth": "2021-06-15"
            }
        });
        let (results_toddler, _trace) =
            evaluate_rule_set(&rule_set_combined, &json_toddler).unwrap();
        assert!(results_toddler["age_appropriate_benefit"]);

        // Test middle-aged person who is 40 years old (no benefit)
        let json_middle = json!({
            "person": {
                "date_of_birth": "1984-03-20"
            }
        });
        let (results_middle, _trace) = evaluate_rule_set(&rule_set_combined, &json_middle).unwrap();
        assert!(!results_middle["age_appropriate_benefit"]);
    }

    #[test]
    fn test_long_university_admission_policy() {
        // This test reproduces the issue with line 123 parsing error
        let rules = r#"# University Admission Policy - Comprehensive Example
# This policy demonstrates all features of the policy language system
# Covers student admission to various programs with complex requirements

# Golden Rule - Entry point for policy evaluation
A **student** gets university admission
  if the **student** meets basic eligibility requirements
  and §academic.standards is valid
  and the **student** qualifies for their chosen program
  and $financial.verification is satisfied
  and the **student** has completed application requirements
  and the **student** has passed background verification.

# Basic eligibility with multiple comparison types
A **student** meets basic eligibility requirements
  if the __age__ of the **applicant** in the **student** is at least 16
  and the __age__ of the **applicant** in the **student** is no more than 65
  and the __citizenship status__ of the **applicant** in the **student** contains "eligible"
  and the __application date__ of the **submission** in the **student** is later than date(2025-01-01)
  and the __application date__ of the **submission** in the **student** is within 90 days
  and the __country of origin__ of the **applicant** in the **student** is not in ["restricted_country_1", "restricted_country_2"].

# Labeled rule for academic standards
academic.standards. A **student** meets academic standards
  if the **student** has sufficient academic background
  and the __cumulative gpa__ of the **transcripts.undergraduate** in the **student** is greater than 2.5
  and the __graduation date__ of the **previous education** in the **student** is earlier than 2030-12-31
  and the __english proficiency verified__ of the **language tests** in the **student** is equal to true.

# Academic background verification with nested objects
A **student** has sufficient academic background
  if the __completion status__ of the **transcripts.undergraduate.degree** is the same as "completed"
  and the __credit hours__ of the **transcripts.undergraduate** in the **student** is at least 120
  and the __math requirement met__ of the **prerequisites** in the **student** is not the same as false
  and the __science requirement met__ of the **prerequisites** in the **student** is equal to true.

# Program-specific qualification rules
A **student** qualifies for their chosen program
  if the **student** meets undergraduate program requirements
  or the **student** meets graduate program requirements
  or the **student** meets doctoral program requirements.

A **student** meets undergraduate program requirements
  if the __program type__ of the **application.program** is in ["bachelor_arts", "bachelor_science", "bachelor_engineering"]
  and the __sat score__ of the **standardized tests** in the **student** is at least 1200
  and the __high school gpa__ of the **transcripts.secondary** in the **student** is greater than 3.0
  and §extracurricular.activity succeeds.

A **student** meets graduate program requirements
  if the __program type__ of the **application.program** is in ["master_arts", "master_science", "master_business"]
  and the __gre score__ of the **standardized tests** in the **student** is at least 310
  and the __undergraduate gpa__ of the **transcripts.undergraduate** in the **student** is greater than 3.25
  and the **student** has relevant work experience
  and $research.experience is approved.

A **student** meets doctoral program requirements
  if the __program type__ of the **application.program** is in ["phd", "doctorate"]
  and the __gre score__ of the **standardized tests** in the **student** is at least 320
  and the __masters gpa__ of the **transcripts.graduate** in the **student** is at least 3.5
  and the **student** has published research
  and §advisor.approval is certified.

# Work experience verification
A **student** has relevant work experience
  if the __years of experience__ of the **employment.history** in the **student** is at least 2
  and the __field relevance__ of the **employment.current** in the **student** contains "related"
  and the __employment verification__ of the **employment.references** in the **student** is equal to true.

# Research experience verification
research.experience. A **student** has research experience
  if the __research projects count__ of the **academic.research** in the **student** is greater than 1
  and the __publication count__ of the **academic.publications** in the **student** is at least 1
  and the __conference presentations__ of the **academic.conferences** in the **student** is no more than 10.

# Extracurricular activities
extracurricular.activity. A **student** has extracurricular involvement
  if the __volunteer hours__ of the **community.service** in the **student** is at least 50
  or the __leadership roles__ of the **organizations** in the **student** is greater than 0
  or the __sports participation__ of the **athletics** in the **student** is equal to true.

# Published research verification
A **student** has published research
  if the __peer reviewed papers__ of the **academic.publications__ in the **student** is at least 3
  and the __citation count__ of the **academic.impact** in the **student** is greater than 10
  and the __first author papers__ of the **academic.publications** in the **student** is at least 1.

# Advisor approval for doctoral programs
advisor.approval. A **student** has advisor approval
  if the __faculty sponsor confirmed__ of the **advisor.agreement** in the **student** is not the same as false
  and the __research alignment__ of the **advisor.fit__ in the **student** contains "excellent"
  and the __funding availability__ of the **advisor.resources** in the **student** is equal to true.

# Financial verification
financial.verification. A **student** meets financial requirements
  if the **student** has proof of financial support
  and the __tuition payment method__ of the **finances.payment** in the **student** is in ["scholarship", "loan", "self_pay", "employer_sponsored"]
  and the __financial aid processed__ of the **finances.aid** in the **student** is the same as true.

A **student** has proof of financial support
  if the __bank balance__ of the **finances.accounts** in the **student** is at least 50000
  or the __scholarship amount__ of the **finances.awards** in the **student** is greater than 25000
  or the __loan approval__ of the **finances.loans** in the **student** is equal to true.

# Application requirements completion
A **student** has completed application requirements
  if the **student** has submitted all documents
  and the **student** has paid application fees
  and the __interview completed__ of the **admission.process** in the **student** is not the same as false
  and the __recommendation letters count__ of the **references** in the **student** is at least 3.

A **student** has submitted all documents
  if the __transcripts submitted__ of the **documents.academic** in the **student** is equal to true
  and the __personal statement submitted__ of the **documents.essays** in the **student** is equal to true
  and the __test scores submitted__ of the **documents.testing** in the **student** is equal to true
  and the __application form completed__ of the **documents.forms** in the **student** is the same as true.

A **student** has paid application fees
  if the __application fee paid__ of the **finances.fees** in the **student** is equal to true
  and the __payment date__ of the **finances.payment** in the **student** is within 30 days
  and the __payment amount__ of the **finances.payment** in the **student** is at least 75.

# Background verification
A **student** has passed background verification
  if the __criminal background clear__ of the **background.criminal** in the **student** is equal to true
  and the __academic integrity violations__ of the **background.academic** in the **student** is equal to false
  and the __visa status valid__ of the **background.immigration** in the **student** is not the same as false
  and the __health clearance__ of the **background.medical** in the **student** contains "approved"
  and the __emergency contact provided__ of the **background.contacts** in the **student** is the same as true."#;

        // This should parse successfully
        let result = parse_rules(rules);
        assert!(result.is_ok(), "Failed to parse university admission rules: {:?}", result);

        // Test with sample data
        let json_data = json!({
            "student": {
                "applicant": {
                    "age": 20,
                    "citizenship_status": "eligible",
                    "country_of_origin": "USA"
                },
                "submission": {
                    "application_date": "2025-06-01"
                },
                "transcripts": {
                    "undergraduate": {
                        "cumulative_gpa": 3.5,
                        "degree": {
                            "completion_status": "completed"
                        },
                        "credit_hours": 120
                    }
                },
                "previous_education": {
                    "graduation_date": "2024-05-15"
                },
                "language_tests": {
                    "english_proficiency_verified": true
                },
                "prerequisites": {
                    "math_requirement_met": true,
                    "science_requirement_met": true
                },
                "application": {
                    "program": {
                        "program_type": "master_science"
                    }
                },
                "standardized_tests": {
                    "gre_score": 320
                },
                "employment": {
                    "history": {
                        "years_of_experience": 3
                    },
                    "current": {
                        "field_relevance": "directly_related"
                    },
                    "references": {
                        "employment_verification": true
                    }
                },
                "academic": {
                    "research": {
                        "research_projects_count": 2
                    },
                    "publications": {
                        "publication_count": 2,
                        "peer_reviewed_papers": 4,
                        "first_author_papers": 2
                    },
                    "conferences": {
                        "conference_presentations": 3
                    },
                    "impact": {
                        "citation_count": 15
                    }
                },
                "community": {
                    "service": {
                        "volunteer_hours": 100
                    }
                },
                "finances": {
                    "accounts": {
                        "bank_balance": 60000
                    },
                    "payment": {
                        "tuition_payment_method": "self_pay",
                        "payment_date": "2025-06-01",
                        "payment_amount": 100
                    },
                    "fees": {
                        "application_fee_paid": true
                    },
                    "aid": {
                        "financial_aid_processed": true
                    }
                },
                "admission": {
                    "process": {
                        "interview_completed": true
                    }
                },
                "references": {
                    "recommendation_letters_count": 3
                },
                "documents": {
                    "academic": {
                        "transcripts_submitted": true
                    },
                    "essays": {
                        "personal_statement_submitted": true
                    },
                    "testing": {
                        "test_scores_submitted": true
                    },
                    "forms": {
                        "application_form_completed": true
                    }
                },
                "background": {
                    "criminal": {
                        "criminal_background_clear": true
                    },
                    "academic": {
                        "academic_integrity_violations": false
                    },
                    "immigration": {
                        "visa_status_valid": true
                    },
                    "medical": {
                        "health_clearance": "approved"
                    },
                    "contacts": {
                        "emergency_contact_provided": true
                    }
                }
            }
        });

        let rule_set = result.unwrap();
        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(results["university admission"], "Student should get university admission");
    }

    #[test]
    fn test_simple_peer_reviewed_papers_rule() {
        // Simplified test to isolate the parsing issue
        let rules = r#"
A **student** has published research
  if the __peer reviewed papers__ of the **academic.publications** in the **student** is at least 3."#;

        let result = parse_rules(rules);
        assert!(result.is_ok(), "Failed to parse peer reviewed papers rule: {:?}", result);
    }

    #[test]
    fn test_property_with_spaces_and_dots() {
        // Test various property names with spaces and nested paths
        let test_cases = vec![
            // Simple property with spaces
            r#"A **student** is valid if the __peer reviewed papers__ of the **student** is at least 3."#,
            // Nested selector with property with spaces
            r#"A **student** is valid if the __peer reviewed papers__ of the **academic.publications** is at least 3."#,
            // Full structure as in the failing test
            r#"A **student** is valid if the __peer reviewed papers__ of the **academic.publications** in the **student** is at least 3."#,
        ];

        for (i, rule) in test_cases.iter().enumerate() {
            let result = parse_rules(rule);
            assert!(result.is_ok(), "Test case {} failed to parse: {:?}\nRule: {}", i, result, rule);
        }
    }

    #[test]
    fn test_multiple_rules_with_peer_reviewed() {
        // Test with multiple rules to see if context affects parsing
        let rules = r#"
# Golden rule
A **student** gets admission
  if the **student** has extracurricular involvement
  and the **student** has published research
  and the **student** has advisor approval.

# Extracurricular activities
extracurricular.activity. A **student** has extracurricular involvement
  if the __volunteer hours__ of the **community.service** in the **student** is at least 50
  or the __leadership roles__ of the **organizations** in the **student** is greater than 0
  or the __sports participation__ of the **athletics** in the **student** is equal to true.

# Published research verification
A **student** has published research
  if the __peer reviewed papers__ of the **academic.publications** in the **student** is at least 3
  and the __citation count__ of the **academic.impact** in the **student** is greater than 10
  and the __first author papers__ of the **academic.publications** in the **student** is at least 1.

# Advisor approval for doctoral programs
advisor.approval. A **student** has advisor approval
  if the __faculty sponsor confirmed__ of the **advisor.agreement** in the **student** is not the same as false
  and the __research alignment__ of the **advisor.fit** in the **student** contains "excellent"
  and the __funding availability__ of the **advisor.resources** in the **student** is equal to true."#;

        let result = parse_rules(rules);
        assert!(result.is_ok(), "Failed to parse rules with peer reviewed papers: {:?}", result);
    }

    #[test]
    fn test_of_in_combination() {
        // Test the specific pattern that seems to be failing
        let test_cases = vec![
            // Using "of" twice
            r#"A **student** is valid if the __prop__ of the **sub** of the **student** is at least 3."#,
            // Using "in" twice  
            r#"A **student** is valid if the __prop__ in the **sub** in the **student** is at least 3."#,
            // Using "of" then "in" - this is what's failing
            r#"A **student** is valid if the __prop__ of the **sub** in the **student** is at least 3."#,
            // The actual failing case
            r#"A **student** is valid if the __peer reviewed papers__ of the **academic.publications** in the **student** is at least 3."#,
        ];

        for (i, rule) in test_cases.iter().enumerate() {
            println!("Testing case {}: {}", i, rule);
            let result = parse_rules(rule);
            if result.is_err() {
                println!("Error in case {}: {:?}", i, result);
            }
            assert!(result.is_ok(), "Test case {} failed to parse: {:?}\nRule: {}", i, result, rule);
        }
    }
}
