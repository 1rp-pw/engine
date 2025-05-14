use crate::runner::error::RuleError;
use crate::runner::model::{Condition, ComparisonOperator, Rule, RuleSet, RuleValue};
use crate::runner::trace::{RuleSetTrace, RuleTrace, ConditionTrace, ComparisonTrace, ComparisonEvaluationTrace, RuleReferenceTrace};

use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub fn evaluate_rule_set(
    rule_set: &RuleSet,
    json: &Value
) -> Result<(HashMap<String, bool>, RuleSetTrace), RuleError> {
    let global_rule= crate::runner::utils::find_global_rule(&rule_set.rules)?;
    let (result, rule_trace) = evaluate_rule(global_rule, json, rule_set)?;

    let mut results = HashMap::new();
    results.insert(global_rule.outcome.clone(), result);

    let mut all_traces = vec![rule_trace];

    let mut processed_rules = HashSet::new();
    processed_rules.insert(global_rule.outcome.clone());

    let mut i = 0;
    while i < all_traces.len() {
        let mut rules_to_process = Vec::new();
        {
            let trace = &all_traces[i];
            for condition in &trace.conditions {
                if let ConditionTrace::RuleReference(ref_trace) = condition {
                    if let Some(outcome) = &ref_trace.referenced_rule_outcome {
                        if !processed_rules.contains(outcome) {
                            if let Some(rule) = rule_set.get_rule(outcome) {
                                rules_to_process.push((outcome.clone(), rule));
                                processed_rules.insert(outcome.clone());
                            }
                        }
                    }
                }
            }
        }

        // Then process the collected rules and modify all_traces
        for (outcome, rule) in rules_to_process {
            let (sub_result, sub_trace) = evaluate_rule(rule, json, rule_set)?;
            results.insert(outcome, sub_result);
            all_traces.push(sub_trace);
        }

        i += 1;
    }
    
    let rule_set_trace = RuleSetTrace {
        execution: all_traces,
    };

    Ok((results, rule_set_trace))
}

pub fn evaluate_rule(
    model_rule: &Rule,
    json: &Value,
    rule_set: &RuleSet
) -> Result<(bool, RuleTrace), RuleError> {
    let mut condition_traces = Vec::new();
    let mut rule_result = true;
    
    for condition in &model_rule.conditions {
        let (condition_result, condition_trace) = evaluate_condition(condition, json, rule_set)?;
        condition_traces.push(condition_trace);
        if !condition_result {
            rule_result = false;
        }
    }
    
    let rule_trace = RuleTrace {
        label: model_rule.label.clone(),
        selector: model_rule.selector.clone(),
        outcome: model_rule.outcome.clone(),
        conditions: condition_traces,
        result: rule_result,
    };
    
    Ok((rule_result, rule_trace))
}

fn evaluate_condition(
    condition: &Condition,
    json: &Value,
    rule_set: &RuleSet
) -> Result<(bool, ConditionTrace), RuleError> {
    match condition {
        Condition::Comparison { selector, property, operator, value } => {
            // Try exact match first
            let selector_exists = json.get(selector).is_some();

            // If exact match fails, try transformed selector
            let transformed_selector = if !selector_exists {
                transform_selector_name(selector)
            } else {
                selector.clone()
            };

            // If neither selector exists in the JSON, return false
            let effective_selector = if selector_exists {
                selector
            } else if json.get(&transformed_selector).is_some() {
                &transformed_selector
            } else {
                return Ok((false, ConditionTrace::Comparison(ComparisonTrace {
                    selector: selector.clone(),
                    property: property.clone(),
                    operator: operator.clone(),
                    value: value.clone(),
                    evaluation_details: None,
                    result: false,
                })))
            };

            // If the property doesn't exist, return false
            if json[effective_selector].get(property).is_none() {
                return Ok((false, ConditionTrace::Comparison(ComparisonTrace {
                    selector: selector.clone(),
                    property: property.clone(),
                    operator: operator.clone(),
                    value: value.clone(),
                    evaluation_details: None,
                    result: false,
                })))
            }

            let json_value = extract_value_from_json(json, effective_selector, property)?;
            let (comparison_result, evaluation_details) = match evaluate_comparison(&json_value, operator, value) {
                Ok(res) => {
                    let details = ComparisonEvaluationTrace {
                        left_value: json_value.clone(),
                        right_value: value.clone(),
                        comparison_result: res,
                    };
                    (res, Some(details))
                },
                Err(e) => {
                    println!("Comparison Error {}", e);
                    (false, None)
                }
            };
            
            let comparison_trace = ComparisonTrace {
                selector: selector.clone(),
                property: property.clone(),
                operator: operator.clone(),
                value: value.clone(),
                evaluation_details,
                result: comparison_result,
            };
            
            Ok((comparison_result, ConditionTrace::Comparison(comparison_trace)))
        },
        Condition::RuleReference { selector, rule_name } => {
            let selector_exists = json.get(selector).is_some();
            if !selector_exists && json.get(&transform_selector_name(selector)).is_none() {
                return Ok((false, ConditionTrace::RuleReference(RuleReferenceTrace {
                    selector: selector.clone(),
                    rule_name: rule_name.clone(),
                    referenced_rule_outcome: None,
                    result: false,
                })));
            }

            let rule_parts: Vec<&str> = rule_name.split(" and ").collect();
            let mut overall_result = true;
            let mut referenced_outcome = None;

            for part in &rule_parts {
                let part = part.trim();

                // Try to find a matching rule
                match rule_set.find_matching_rule(selector, part) {
                    Some(referenced_rule) => {
                        println!("Found matching rule: {} gets {}", referenced_rule.selector, referenced_rule.outcome);
                        let (referenced_result, _) = evaluate_rule(referenced_rule, json, rule_set)?;

                        if !referenced_result {
                            overall_result = false;
                        }

                        // Store the outcome for tracing
                        if referenced_outcome.is_none() {
                            referenced_outcome = Some(referenced_rule.outcome.clone());
                        }
                    },
                    None => {
                        // If we can't find a matching rule, assume true for now
                        println!("Warning: No matching rule found for '{}' with description '{}', assuming true", selector, part);
                    }
                }
            }

            // // Try to find a matching rule
            // let (rule_reference_result, referenced_rule_outcome) = match rule_set.find_matching_rule(selector, rule_name) {
            //     Some(referenced_rule) => {
            //         println!("Found matching rule: {} gets {}", referenced_rule.selector, referenced_rule.outcome);
            //         let (referenced_result, _) = evaluate_rule(referenced_rule, json, rule_set)?;
            //         (referenced_result, Some(referenced_rule.outcome.clone()))
            //     },
            //     None => {
            //         // If we can't find a matching rule, assume true
            //         println!("Warning: No matching rule found for '{}' with description '{}', assuming true", selector, rule_name);
            //         (true, None)
            //     }
            // };
            
            let rule_reference_trace = RuleReferenceTrace {
                selector: selector.clone(),
                rule_name: rule_name.clone(),
                referenced_rule_outcome: referenced_outcome,
                result: overall_result,
            };
            
            Ok((overall_result, ConditionTrace::RuleReference(rule_reference_trace)))
        },
    }
}

fn extract_value_from_json(
    json: &Value,
    selector: &str,
    property: &str
) -> Result<RuleValue, RuleError> {
    let obj = if let Some(obj) = json.get(selector) {
        obj
    } else {
        // Try transformed selector (camelCase)
        let transformed_selector = transform_selector_name(selector);
        json.get(&transformed_selector)
            .ok_or_else(|| RuleError::EvaluationError(format!("Selector '{}' (or '{}') not found in JSON", selector, transformed_selector)))?
    };

    let value = obj.get(property)
        .ok_or_else(|| RuleError::EvaluationError(format!("Property '{}' not found in selector '{}'", property, selector)))?;

    match value {
        Value::Number(n) => {
            if let Some(num) = n.as_f64() {
                Ok(RuleValue::Number(num))
            } else {
                Err(RuleError::TypeError(format!("Could not convert number to f64: {:?}", n)))
            }
        },
        Value::String(s) => {
            // Try to parse as date if it looks like a date (YYYY-MM-DD format)
            if s.len() == 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
                println!("Attempting to parse date: {}", s);
                match chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    Ok(date) => {
                        println!("Successfully parsed date: {}", date);
                        Ok(RuleValue::Date(date))
                    },
                    Err(e) => {
                        println!("Failed to parse date '{}': {}", s, e);
                        Ok(RuleValue::String(s.clone()))
                    }
                }
            } else {
                Ok(RuleValue::String(s.clone()))
            }
        },
        Value::Bool(b) => Ok(RuleValue::Boolean(*b)),
        Value::Array(arr) => {
            let mut values = Vec::new();
            for item in arr {
                // This is a simplified conversion - you might want to handle nested arrays differently
                if let Some(s) = item.as_str() {
                    values.push(RuleValue::String(s.to_string()));
                } else if let Some(n) = item.as_f64() {
                    values.push(RuleValue::Number(n));
                } else if let Some(b) = item.as_bool() {
                    values.push(RuleValue::Boolean(b));
                } else {
                    return Err(RuleError::TypeError(format!("Unsupported array item type: {:?}", item)));
                }
            }
            Ok(RuleValue::List(values))
        },
        _ => Err(RuleError::TypeError(format!("Unsupported JSON value type: {:?}", value))),
    }
}

fn evaluate_comparison(
    left: &RuleValue,
    operator: &ComparisonOperator,
    right: &RuleValue
) -> Result<bool, RuleError> {
    match operator {
        ComparisonOperator::GreaterThanOrEqual => {
            match (left, right) {
                (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l >= r),
                _ => Err(RuleError::TypeError("GreaterThanOrEqual only works with numbers".to_string())),
            }
        },
        ComparisonOperator::LessThanOrEqual => {
            match (left, right) {
                (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l <= r),
                _ => Err(RuleError::TypeError("LessThanOrEqual only works with numbers".to_string())),
            }
        },
        ComparisonOperator::EqualTo => {
            match (left, right) {
                (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l == r),
                (RuleValue::String(l), RuleValue::String(r)) => Ok(l == r),
                (RuleValue::Date(l), RuleValue::Date(r)) => Ok(l == r),
                (RuleValue::Boolean(l), RuleValue::Boolean(r)) => Ok(l == r),
                _ => Err(RuleError::TypeError(format!("Cannot compare {:?} and {:?} with EqualTo", left, right))),
            }
        },
        ComparisonOperator::NotEqualTo => {
            match (left, right) {
                (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l != r),
                (RuleValue::String(l), RuleValue::String(r)) => Ok(l != r),
                (RuleValue::Date(l), RuleValue::Date(r)) => Ok(l != r),
                (RuleValue::Boolean(l), RuleValue::Boolean(r)) => Ok(l != r),
                _ => Err(RuleError::TypeError(format!("Cannot compare {:?} and {:?} with NotEqualTo", left, right))),
            }
        },
        ComparisonOperator::SameAs => {
            match (left, right) {
                (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l == r),
                (RuleValue::String(l), RuleValue::String(r)) => Ok(l == r),
                (RuleValue::Date(l), RuleValue::Date(r)) => Ok(l == r),
                (RuleValue::Boolean(l), RuleValue::Boolean(r)) => Ok(l == r),
                _ => Err(RuleError::TypeError(format!("Cannot compare {:?} and {:?} with SameAs", left, right))),
            }
        },
        ComparisonOperator::NotSameAs => {
            match (left, right) {
                (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l != r),
                (RuleValue::String(l), RuleValue::String(r)) => Ok(l != r),
                (RuleValue::Date(l), RuleValue::Date(r)) => Ok(l != r),
                (RuleValue::Boolean(l), RuleValue::Boolean(r)) => Ok(l != r),
                _ => Err(RuleError::TypeError(format!("Cannot compare {:?} and {:?} with NotSameAs", left, right))),
            }
        },
        ComparisonOperator::LaterThan => {
            match (left, right) {
                (RuleValue::Date(l), RuleValue::Date(r)) => {
                    println!("Comparing dates: {} > {}", l, r);
                    Ok(l > r)
                },
                _ => Err(RuleError::TypeError("LaterThan only works with dates".to_string())),
            }
        },
        ComparisonOperator::EarlierThan => {
            match (left, right) {
                (RuleValue::Date(l), RuleValue::Date(r)) => {
                    println!("Comparing dates: {} < {}", l, r);
                    Ok(l < r)
                },
                _ => Err(RuleError::TypeError(format!("EarlierThan only works with dates {} {}", left, right))),
            }
        },
        ComparisonOperator::GreaterThan => {
            match (left, right) {
                (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l > r),
                _ => Err(RuleError::TypeError("GreaterThan only works with numbers".to_string())),
            }
        },
        ComparisonOperator::LessThan => {
            match (left, right) {
                (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l < r),
                _ => Err(RuleError::TypeError("LessThan only works with numbers".to_string())),
            }
        },
        ComparisonOperator::In => {
            match right {
                RuleValue::List(items) => {
                    for item in items {
                        match (left, item) {
                            (RuleValue::Number(l), RuleValue::Number(r)) if l == r => return Ok(true),
                            (RuleValue::String(l), RuleValue::String(r)) if l == r => return Ok(true),
                            (RuleValue::Date(l), RuleValue::Date(r)) if l == r => return Ok(true),
                            (RuleValue::Boolean(l), RuleValue::Boolean(r)) if l == r => return Ok(true),
                            _ => continue,
                        }
                    }
                    Ok(false)
                },
                _ => Err(RuleError::TypeError("Right operand of 'is in' must be a list".to_string())),
            }
        },
        ComparisonOperator::NotIn => {
            match right {
                RuleValue::List(items) => {
                    for item in items {
                        match (left, item) {
                            (RuleValue::Number(l), RuleValue::Number(r)) if l == r => return Ok(false),
                            (RuleValue::String(l), RuleValue::String(r)) if l == r => return Ok(false),
                            (RuleValue::Date(l), RuleValue::Date(r)) if l == r => return Ok(false),
                            (RuleValue::Boolean(l), RuleValue::Boolean(r)) if l == r => return Ok(false),
                            _ => continue,
                        }
                    }
                    Ok(true)
                },
                _ => Err(RuleError::TypeError("Right operand of 'is not in' must be a list".to_string())),
            }
        },
        ComparisonOperator::Contains => {
            match (left, right) {
                (RuleValue::String(l), RuleValue::String(r)) => Ok(l.contains(r)),
                (RuleValue::List(items), _) => {
                    for item in items {
                        match (item, right) {
                            (RuleValue::Number(l), RuleValue::Number(r)) if l == r => return Ok(true),
                            (RuleValue::String(l), RuleValue::String(r)) if l == r => return Ok(true),
                            (RuleValue::Date(l), RuleValue::Date(r)) if l == r => return Ok(true),
                            (RuleValue::Boolean(l), RuleValue::Boolean(r)) if l == r => return Ok(true),
                            _ => continue,
                        }
                    }
                    Ok(false)
                },
                _ => Err(RuleError::TypeError("Contains only works with strings or lists".to_string())),
            }
        },
    }
}

fn transform_selector_name(name: &str) -> String {
    // Similar to transform_property_name but keeps first letter capitalized
    let words: Vec<&str> = name.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }

    let mut result = words[0].to_lowercase();
    for word in &words[1..] {
        if !word.is_empty() {
            result.push_str(&word[0..1].to_uppercase());
            result.push_str(&word[1..].to_lowercase());
        }
    }

    result
}


