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
        let (results_false, _trace_false) = evaluate_rule_set(&rule_set, &json_false_short).unwrap();
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
        let (results_missing, _trace_missing) = evaluate_rule_set(&rule_set, &json_missing).unwrap();
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
            ("is greater than", 5, "longname", true),     // 8 > 5
            ("is greater than", 10, "longname", false),   // 8 > 10 = false
            ("is less than", 10, "short", true),          // 5 < 10
            ("is less than", 3, "short", false),          // 5 < 3 = false
            ("is greater than or equal to", 5, "hello", true),  // 5 >= 5
            ("is less than or equal to", 5, "hello", true),     // 5 <= 5
            ("is equal to", 4, "test", true),             // 4 == 4
            ("is not equal to", 5, "test", true),         // 4 != 5
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
                results["valid"], expected,
                "Failed for operator '{}' with threshold {} and name '{}' (length {})",
                operator, threshold, name, name.len()
            );
        }
    }

    #[test]
    fn test_existing_simple_rule_still_works() {
        let input = r#"A **user** passes the test if __age__ of **user** is greater than or equal to 18."#;

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
        let input = r#"A **user** is valid if the length of __name__ of **user** is greater than 5."#;

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
        assert!(results["premium"], "Should match exact property name 'is_premium'");
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
        assert!(results["premium"], "Should match transformed property name 'isPremium'");
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
        assert!(results["premium"], "Should match case-insensitive property name 'IS_PREMIUM'");
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
        assert!(results["premium"], "Should match case-insensitive transformed property name 'ISPREMIUM'");
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
        assert!(results["valid"], "Should transform 'first name' to 'firstName'");
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
        assert!(results["valid"], "Should prefer exact match 'is_premium' over 'isPremium'");
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
        assert!(results["valid"], "Should handle nested property transformations");
    }

    #[test]
    fn test_date_property_transformation() {
        let input = r#"A **user** is young if __birth_date__ of **user** is later than "2000-01-01"."#;
        let rule_set = parse_rules(input).unwrap();

        let json_data = json!({
            "user": {
                "birthDate": "2005-06-15"  // Transformed property name
            }
        });

        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(results["young"], "Should handle date comparison with transformed property");
    }

    #[test]
    fn test_length_with_property_transformation() {
        let input = r#"A **user** is valid if the length of __full_name__ of **user** is greater than 5."#;
        let rule_set = parse_rules(input).unwrap();

        let json_data = json!({
            "user": {
                "fullName": "John Doe Smith"  // Transformed property name
            }
        });

        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(results["valid"], "Should handle length calculation with transformed property");
    }

    #[test]
    fn test_property_not_found_error() {
        let input = r#"A **user** is valid if __nonexistent_property__ of **user** is equal to true."#;
        let rule_set = parse_rules(input).unwrap();

        let json_data = json!({
            "user": {
                "some_other_property": true
            }
        });

        let (results, _trace) = evaluate_rule_set(&rule_set, &json_data).unwrap();
        assert!(!results["valid"], "Should return false when property is not found");
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
}



