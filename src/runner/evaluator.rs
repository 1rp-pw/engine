use crate::runner::error::RuleError;
use crate::runner::model::{Condition, ComparisonOperator, Rule, RuleSet, RuleValue, ConditionOperator};
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
use chrono::NaiveDate;

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
    // 1) evaluate each condition, collect its bool and its trace
    let mut results = Vec::new();
    let mut ops     = Vec::new();
    let mut condition_traces = Vec::new();

    for (i, cg) in model_rule.conditions.iter().enumerate() {
        let (res, trace) = evaluate_condition(&cg.condition, json, rule_set)?;
        results.push(res);
        condition_traces.push(trace);

        // record the operator that *follows* this condition (None for first)
        if let Some(op) = &cg.operator {
            ops.push(op.clone());
        } else if i != 0 {
            // fallback to AND if parser didn’t provide one
            ops.push(ConditionOperator::And);
        }
    }

    // 2) collapse all ANDs first
    let mut i = 0;
    while i < ops.len() {
        if ops[i] == ConditionOperator::And {
            // combine results[i] AND results[i+1] into results[i]
            results[i] = results[i] && results[i + 1];
            // drop results[i+1] and ops[i]
            results.remove(i + 1);
            ops.remove(i);
            // do not advance i, maybe there’s another AND here
        } else {
            i += 1;
        }
    }

    // 3) now fold OR across what remains
    let rule_result = results
        .into_iter()
        .fold(false, |acc, next| acc || next);

    // 4) build your trace object exactly as before
    let rule_trace = RuleTrace {
        label:    model_rule.label.clone(),
        selector: model_rule.selector.clone(),
        selector_pos: model_rule.selector_pos.clone(),
        outcome:  model_rule.outcome.clone(),
        outcome_pos: model_rule.position.clone(), // or wherever you keep it
        conditions: condition_traces,
        result:   rule_result,
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

            // Try exact outcome match first
            if let Some(referenced_rule) = rule_set.get_rule(part) {
                let (referenced_result, _) = evaluate_rule(referenced_rule, json, rule_set)?;
                if !referenced_result {
                    overall_result = false;
                }
                referenced_outcome = Some(referenced_rule.outcome.to_string());
            } else if let Some(referenced_rule) = rule_set.get_rule_by_label(part) {
                // Try exact label match
                let (referenced_result, _) = evaluate_rule(referenced_rule, json, rule_set)?;
                if !referenced_result {
                    overall_result = false;
                }
                referenced_outcome = Some(referenced_rule.outcome.clone());
            } else {
                // Try to find a rule with matching outcome (case insensitive, partial match)
                let mut found_rule = None;
                for rule in &rule_set.rules {
                    if rule.outcome.to_lowercase() == part.to_lowercase() ||
                        rule.outcome.to_lowercase().contains(&part.to_lowercase()) ||
                        part.to_lowercase().contains(&rule.outcome.to_lowercase()) {
                        found_rule = Some(rule);
                        break;
                    }
                }

                if let Some(referenced_rule) = found_rule {
                    let (referenced_result, _) = evaluate_rule(referenced_rule, json, rule_set)?;
                    if !referenced_result {
                        overall_result = false;
                    }
                    referenced_outcome = Some(referenced_rule.outcome.clone());
                } else {
                    // If we still can't find a matching rule, try to infer a property
                    let possible_properties = crate::runner::utils::infer_possible_properties(part);

                    if let Some(obj) = json.get(&effective_selector) {
                        for property in &possible_properties {
                            if let Some(property_value) = obj.get(property) {
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

// Main comparison dispatcher
fn evaluate_comparison(
    left: &RuleValue,
    operator: &ComparisonOperator,
    right: &RuleValue
) -> Result<bool, RuleError> {
    use ComparisonOperator::*;

    match operator {
        // Numeric comparisons
        GreaterThanOrEqual => compare_numbers_gte(left, right),
        LessThanOrEqual => compare_numbers_lte(left, right),
        GreaterThan => compare_numbers_gt(left, right),
        LessThan => compare_numbers_lt(left, right),

        // Equality comparisons
        EqualTo => compare_equal(left, right),
        NotEqualTo => compare_not_equal(left, right),
        SameAs => compare_equal(left, right), // Same logic as EqualTo
        NotSameAs => compare_not_equal(left, right), // Same logic as NotEqualTo

        // Date comparisons
        LaterThan => compare_dates_later(left, right),
        EarlierThan => compare_dates_earlier(left, right),

        // List operations
        In => compare_in_list(left, right),
        NotIn => compare_not_in_list(left, right),
        Contains => compare_contains(left, right),
    }
}

// Helper function to try to convert a string to a date
fn try_parse_date(value: &RuleValue) -> Option<NaiveDate> {
    if let RuleValue::String(s) = value {
        if s.len() == 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
            NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
        } else {
            None
        }
    } else {
        None
    }
}

// Coerce values to dates if possible
fn coerce_to_dates(left: &RuleValue, right: &RuleValue) -> Option<(NaiveDate, NaiveDate)> {
    let left_date = match left {
        RuleValue::Date(d) => Some(*d),
        _ => try_parse_date(left),
    };

    let right_date = match right {
        RuleValue::Date(d) => Some(*d),
        _ => try_parse_date(right),
    };

    match (left_date, right_date) {
        (Some(l), Some(r)) => Some((l, r)),
        _ => None,
    }
}

// Numeric comparison functions
fn compare_numbers_gte(left: &RuleValue, right: &RuleValue) -> Result<bool, RuleError> {
    match (left, right) {
        (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l >= r),
        _ => Err(RuleError::TypeError("GreaterThanOrEqual only works with numbers".to_string())),
    }
}

fn compare_numbers_lte(left: &RuleValue, right: &RuleValue) -> Result<bool, RuleError> {
    match (left, right) {
        (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l <= r),
        _ => Err(RuleError::TypeError("LessThanOrEqual only works with numbers".to_string())),
    }
}

fn compare_numbers_gt(left: &RuleValue, right: &RuleValue) -> Result<bool, RuleError> {
    match (left, right) {
        (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l > r),
        _ => Err(RuleError::TypeError("GreaterThan only works with numbers".to_string())),
    }
}

fn compare_numbers_lt(left: &RuleValue, right: &RuleValue) -> Result<bool, RuleError> {
    match (left, right) {
        (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l < r),
        _ => Err(RuleError::TypeError("LessThan only works with numbers".to_string())),
    }
}

// Equality comparison functions
fn compare_equal(left: &RuleValue, right: &RuleValue) -> Result<bool, RuleError> {
    match (left, right) {
        (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l == r),
        (RuleValue::String(l), RuleValue::String(r)) => Ok(l == r),
        (RuleValue::Date(l), RuleValue::Date(r)) => Ok(l == r),
        (RuleValue::Boolean(l), RuleValue::Boolean(r)) => Ok(l == r),
        _ => Err(RuleError::TypeError(format!("Cannot compare {:?} and {:?} for equality", left, right))),
    }
}

fn compare_not_equal(left: &RuleValue, right: &RuleValue) -> Result<bool, RuleError> {
    compare_equal(left, right).map(|result| !result)
}

// Date comparison functions
fn compare_dates_later(left: &RuleValue, right: &RuleValue) -> Result<bool, RuleError> {
    if let Some((l, r)) = coerce_to_dates(left, right) {
        Ok(l > r)
    } else {
        Err(RuleError::TypeError(format!("LaterThan requires date values, got {:?} and {:?}", left, right)))
    }
}

fn compare_dates_earlier(left: &RuleValue, right: &RuleValue) -> Result<bool, RuleError> {
    if let Some((l, r)) = coerce_to_dates(left, right) {
        Ok(l < r)
    } else {
        Err(RuleError::TypeError(format!("EarlierThan requires date values, got {:?} and {:?}", left, right)))
    }
}

// List operation functions
fn compare_in_list(left: &RuleValue, right: &RuleValue) -> Result<bool, RuleError> {
    match right {
        RuleValue::List(items) => {
            for item in items {
                if is_equal(left, item) {
                    return Ok(true);
                }
            }
            Ok(false)
        },
        _ => Err(RuleError::TypeError("Right operand of 'is in' must be a list".to_string())),
    }
}

fn compare_not_in_list(left: &RuleValue, right: &RuleValue) -> Result<bool, RuleError> {
    compare_in_list(left, right).map(|result| !result)
}

fn compare_contains(left: &RuleValue, right: &RuleValue) -> Result<bool, RuleError> {
    match left {
        RuleValue::String(l) => {
            match right {
                RuleValue::String(r) => Ok(l.contains(r)),
                _ => Err(RuleError::TypeError("String contains only works with string values".to_string())),
            }
        },
        RuleValue::List(items) => {
            for item in items {
                if is_equal(item, right) {
                    return Ok(true);
                }
            }
            Ok(false)
        },
        _ => Err(RuleError::TypeError("Contains only works with strings or lists".to_string())),
    }
}

// Helper function to check equality without returning Result
fn is_equal(left: &RuleValue, right: &RuleValue) -> bool {
    match (left, right) {
        (RuleValue::Number(l), RuleValue::Number(r)) => l == r,
        (RuleValue::String(l), RuleValue::String(r)) => l == r,
        (RuleValue::Date(l), RuleValue::Date(r)) => l == r,
        (RuleValue::Boolean(l), RuleValue::Boolean(r)) => l == r,
        _ => false,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_numbers() {
        let five = RuleValue::Number(5.0);
        let ten = RuleValue::Number(10.0);

        assert_eq!(compare_numbers_gt(&ten, &five).unwrap(), true);
        assert_eq!(compare_numbers_gt(&five, &ten).unwrap(), false);
        assert_eq!(compare_numbers_gte(&five, &five).unwrap(), true);
        assert_eq!(compare_numbers_lt(&five, &ten).unwrap(), true);
        assert_eq!(compare_numbers_lte(&ten, &ten).unwrap(), true);
    }

    #[test]
    fn test_compare_dates() {
        let date1 = RuleValue::Date(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        let date2 = RuleValue::Date(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
        let date_str = RuleValue::String("2020-06-15".to_string());

        assert_eq!(compare_dates_earlier(&date1, &date2).unwrap(), true);
        assert_eq!(compare_dates_later(&date2, &date1).unwrap(), true);
        assert_eq!(compare_dates_earlier(&date_str, &date2).unwrap(), true);
    }

    #[test]
    fn test_compare_equality() {
        let str1 = RuleValue::String("hello".to_string());
        let str2 = RuleValue::String("hello".to_string());
        let str3 = RuleValue::String("world".to_string());

        assert_eq!(compare_equal(&str1, &str2).unwrap(), true);
        assert_eq!(compare_equal(&str1, &str3).unwrap(), false);
        assert_eq!(compare_not_equal(&str1, &str3).unwrap(), true);
    }

    #[test]
    fn test_list_operations() {
        let value = RuleValue::String("apple".to_string());
        let list = RuleValue::List(vec![
            RuleValue::String("apple".to_string()),
            RuleValue::String("banana".to_string()),
        ]);

        assert_eq!(compare_in_list(&value, &list).unwrap(), true);
        assert_eq!(compare_contains(&list, &value).unwrap(), true);

        let missing = RuleValue::String("orange".to_string());
        assert_eq!(compare_in_list(&missing, &list).unwrap(), false);
        assert_eq!(compare_not_in_list(&missing, &list).unwrap(), true);
    }
}