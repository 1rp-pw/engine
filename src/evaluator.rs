// src/evaluator.rs
use crate::error::RuleError;
use crate::model::{Condition, ComparisonOperator, Rule, RuleSet, RuleValue};
use serde_json::Value;
use std::collections::HashMap;

pub fn evaluate_rule_set(
    rule_set: &RuleSet,
    json: &Value
) -> Result<HashMap<String, bool>, RuleError> {
    let mut results = HashMap::new();

    for rule in &rule_set.rules {
        let result = evaluate_rule(rule, json, rule_set)?;
        results.insert(rule.outcome.clone(), result);
    }

    Ok(results)
}

pub fn evaluate_rule(
    rule: &Rule,
    json: &Value,
    rule_set: &RuleSet
) -> Result<bool, RuleError> {
    for condition in &rule.conditions {
        if !evaluate_condition(condition, json, rule_set)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn evaluate_condition(
    condition: &Condition,
    json: &Value,
    rule_set: &RuleSet
) -> Result<bool, RuleError> {
    match condition {
        Condition::Comparison { selector, property, operator, value } => {
            let json_value = extract_value_from_json(json, selector, property)?;
            evaluate_comparison(&json_value, operator, value)
        },
        Condition::RuleReference { selector, rule_name } => {
            // Check if the selector exists in the JSON
            if !json.get(selector).is_some() {
                return Ok(false);
            }

            // Find the referenced rule
            let referenced_rule = rule_set.get_rule(rule_name)
                .ok_or_else(|| RuleError::ReferenceError(format!("Referenced rule '{}' not found", rule_name)))?;

            evaluate_rule(referenced_rule, json, rule_set)
        },
    }
}

fn extract_value_from_json(
    json: &Value,
    selector: &str,
    property: &str
) -> Result<RuleValue, RuleError> {
    let obj = json.get(selector)
        .ok_or_else(|| RuleError::EvaluationError(format!("Selector '{}' not found in JSON", selector)))?;

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
                if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    return Ok(RuleValue::Date(date));
                }
            }
            Ok(RuleValue::String(s.clone()))
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
        ComparisonOperator::EqualTo => {
            match (left, right) {
                (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l == r),
                (RuleValue::String(l), RuleValue::String(r)) => Ok(l == r),
                (RuleValue::Date(l), RuleValue::Date(r)) => Ok(l == r),
                (RuleValue::Boolean(l), RuleValue::Boolean(r)) => Ok(l == r),
                _ => Err(RuleError::TypeError(format!("Cannot compare {:?} and {:?} with EqualTo", left, right))),
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
        ComparisonOperator::LaterThan => {
            match (left, right) {
                (RuleValue::Date(l), RuleValue::Date(r)) => Ok(l > r),
                _ => Err(RuleError::TypeError("LaterThan only works with dates".to_string())),
            }
        },
        ComparisonOperator::GreaterThan => {
            match (left, right) {
                (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l > r),
                _ => Err(RuleError::TypeError("GreaterThan only works with numbers".to_string())),
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
    }
}
