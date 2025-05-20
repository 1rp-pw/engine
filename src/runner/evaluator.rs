use crate::runner::error::RuleError;
use crate::runner::model::{Condition, ComparisonOperator, Rule, RuleSet, RuleValue};
use crate::runner::trace::{
    RuleSetTrace,
    RuleTrace,
    ConditionTrace,
    ComparisonTrace,
    ComparisonEvaluationTrace,
    RuleReferenceTrace,
    PropertyCheckTrace,
};

use serde_json::{json, Value};
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
        selector_pos: None,
        outcome: model_rule.outcome.clone(),
        outcome_pos: None,
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
        Condition::RuleReference { selector, rule_name } => {
            let selector_value = json.get(selector)
                .or_else(|| get_json_value_insensative(json, selector));
            let selector_exists = selector_value.is_some();
            let effective_selector = if selector_exists {
                selector.clone()
            } else {
                let transformed = transform_selector_name(selector);
                if json.get(&transformed).is_some() {
                    transformed
                } else {
                    return Ok((false, ConditionTrace::RuleReference(RuleReferenceTrace {
                        selector: selector.clone(),
                        rule_name: rule_name.clone(),
                        referenced_rule_outcome: None,
                        property_check: None,
                        result: false,
                    })));
                }
            };

            // Process just this specific rule reference
            let part = rule_name.trim();
            let mut overall_result = true;
            let mut referenced_outcome = None;
            let mut property_check = None;

            // Try to find a matching rule by exact outcome match first
            if let Some(referenced_rule) = rule_set.get_rule(part) {
                let (referenced_result, _) = evaluate_rule(referenced_rule, json, rule_set)?;
                if !referenced_result {
                    overall_result = false;
                }
                referenced_outcome = Some(referenced_rule.outcome.to_string());
            } else {
                // If no exact match, try to find a rule with similar description
                if let Some(referenced_rule) = rule_set.find_matching_rule(selector, part) {
                    let (referenced_result, _) = evaluate_rule(referenced_rule, json, rule_set)?;
                    if !referenced_result {
                        overall_result = false;
                    }
                    referenced_outcome = Some(referenced_rule.outcome.clone());
                } else if let Some(referenced_rule) = rule_set.get_rule_by_label(part) {
                    let (referenced_result, _) = evaluate_rule(referenced_rule, json, rule_set)?;
                    if !referenced_result {
                        overall_result = false;
                    }
                    referenced_outcome = Some(referenced_rule.outcome.clone());
                } else {
                    // If we still can't find a matching rule, try to infer a property
                    let possible_properties = crate::runner::utils::infer_possible_properties(part);
                    let mut property_found = false;

                    if let Some(obj) = json.get(&effective_selector) {
                        for property in &possible_properties {
                            if let Some(property_value) = obj.get(property) {
                                property_found = true;

                                if let Some(value_bool) = property_value.as_bool() {
                                    if !value_bool {
                                        overall_result = false;
                                    }
                                    property_check = Some(PropertyCheckTrace {
                                        property_name: property.clone(),
                                        property_value: json!({ "Boolean": value_bool }),
                                    });
                                } else if let Some(value_str) = property_value.as_str() {
                                    // For string values, consider "pass", "true", "yes", etc. as passing
                                    let lower_value = value_str.to_lowercase();
                                    let passes = lower_value == "pass" || lower_value == "true" ||
                                        lower_value == "yes" || lower_value == "passed" || lower_value == "valid";
                                    if !passes {
                                        overall_result = false;
                                    }
                                    property_check = Some(PropertyCheckTrace {
                                        property_name: property.clone(),
                                        property_value: json!({ "String": value_str }),
                                    });
                                } else if let Some(value_num) = property_value.as_f64() {
                                    // For numeric values, consider non-zero as passing
                                    if value_num == 0.0 {
                                        overall_result = false;
                                    }
                                    property_check = Some(PropertyCheckTrace {
                                        property_name: property.clone(),
                                        property_value: json!({ "Number": value_num }),
                                    });
                                } else {
                                    property_check = Some(PropertyCheckTrace {
                                        property_name: property.clone(),
                                        property_value: property_value.clone(),
                                    });
                                }

                                break;
                            }
                        }
                    }

                    if !property_found {
                        // If we can't find any corresponding property, assume true
                    }
                }
            }

            let rule_reference_trace = RuleReferenceTrace {
                selector: selector.clone(),
                rule_name: rule_name.clone(),
                referenced_rule_outcome: referenced_outcome,
                property_check: property_check,
                result: overall_result,
            };

            Ok((overall_result, ConditionTrace::RuleReference(rule_reference_trace)))
        },
        Condition::Comparison {
            selector,
            selector_pos,
            property,
            property_pos,
            operator,
            value ,
            value_pos,
        } => {
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
                    selector_pos: selector_pos.clone(),
                    property: property.clone(),
                    property_pos: property_pos.clone(),
                    operator: operator.clone(),
                    value: value.clone(),
                    value_pos: value_pos.clone(),
                    evaluation_details: None,
                    result: false,
                })))
            };

            // If the property doesn't exist, return false
            if json[effective_selector].get(property).is_none() {
                return Ok((false, ConditionTrace::Comparison(ComparisonTrace {
                    selector: selector.clone(),
                    selector_pos: selector_pos.clone(),
                    property: property.clone(),
                    property_pos: property_pos.clone(),
                    operator: operator.clone(),
                    value: value.clone(),
                    value_pos: value_pos.clone(),
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
                Err(_) => {
                    (false, None)
                }
            };

            let comparison_trace = ComparisonTrace {
                selector: selector.clone(),
                selector_pos: selector_pos.clone(),
                property: property.clone(),
                property_pos: property_pos.clone(),
                operator: operator.clone(),
                value: value.clone(),
                value_pos: value_pos.clone(),
                evaluation_details,
                result: comparison_result,
            };

            Ok((comparison_result, ConditionTrace::Comparison(comparison_trace)))
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
            if s.len() == 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
                match chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    Ok(date) => {
                        Ok(RuleValue::Date(date))
                    },
                    Err(_) => {
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
                    Ok(l > r)
                },
                _ => Err(RuleError::TypeError("LaterThan only works with dates".to_string())),
            }
        },
        ComparisonOperator::EarlierThan => {
            match (left, right) {
                (RuleValue::Date(l), RuleValue::Date(r)) => {
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

fn get_json_value_insensative<'a>(json: &'a serde_json::Value, key: &str) -> Option<&'a serde_json::Value> {
    if let Some(obj) = json.as_object() {
        let key_lower = key.to_lowercase();
        for (k, v) in obj {
            if k.to_lowercase() == key_lower {
                return Some(v);
            }
        }
    }
    None
}
