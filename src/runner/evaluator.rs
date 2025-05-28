use crate::runner::error::RuleError;
use crate::runner::model::{Condition, ComparisonOperator, Rule, RuleSet, RuleValue, ConditionOperator, SourcePosition, ComparisonCondition, RuleReferenceCondition};
use crate::runner::trace::{RuleSetTrace, RuleTrace, ConditionTrace, ComparisonTrace, ComparisonEvaluationTrace, RuleReferenceTrace, PropertyCheckTrace, PropertyTrace, TypedValue, SelectorTrace, OutcomeTrace};

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
            // fallback to AND if parser didn't provide one
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
            // do not advance i, maybe there's another AND here
        } else {
            i += 1;
        }
    }

    // 3) now fold OR across what remains
    let rule_result = results
        .into_iter()
        .fold(false, |acc, next| acc || next);

    // 4) build your trace object with the new structure
    let rule_trace = RuleTrace {
        label: model_rule.label.clone(),
        selector: SelectorTrace {
            value: model_rule.selector.clone(),
            pos: model_rule.selector_pos.clone(),
        },
        outcome: OutcomeTrace {
            value: model_rule.outcome.clone(),
            pos: model_rule.position.clone(),
        },
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
        Condition::RuleReference(ref_condition) => {
            evaluate_rule_reference_condition(ref_condition, json, rule_set)
        },
        Condition::Comparison(comp_condition) => {
            evaluate_comparison_condition(comp_condition, json)
        },
    }
}

fn evaluate_rule_reference_condition(
    condition: &RuleReferenceCondition,
    json: &Value,
    rule_set: &RuleSet
) -> Result<(bool, ConditionTrace), RuleError> {
    // Handle empty selector case (for label references)
    if condition.selector.value.is_empty() {
        // This is a label reference without a selector
        let rule_name = condition.rule_name.value.trim();

        // Try to find and evaluate the referenced rule
        if let Some((result, outcome)) = try_evaluate_by_rule(rule_name, json, rule_set)? {
            let rule_reference_trace = RuleReferenceTrace {
                selector: SelectorTrace {
                    value: String::new(),
                    pos: None,
                },
                rule_name: condition.rule_name.value.clone(),
                referenced_rule_outcome: Some(outcome),
                property_check: None,
                result,
            };
            return Ok((result, ConditionTrace::RuleReference(rule_reference_trace)));
        }

        // If no rule found, return false
        return Ok((false, create_failed_rule_reference_trace(condition)));
    }

    // Normal case with selector
    let effective_selector = find_effective_selector(&condition.selector.value, json)?;

    if effective_selector.is_none() {
        return Ok((false, create_failed_rule_reference_trace(condition)));
    }

    let part = condition.rule_name.value.trim();
    let (result, referenced_outcome, property_check) =
        evaluate_rule_or_property(part, &effective_selector.unwrap(), json, rule_set)?;

    let rule_reference_trace = RuleReferenceTrace {
        selector: SelectorTrace {
            value: condition.selector.value.clone(),
            pos: condition.selector.pos.clone(),
        },
        rule_name: condition.rule_name.value.clone(),
        referenced_rule_outcome: referenced_outcome,
        property_check,
        result,
    };

    Ok((result, ConditionTrace::RuleReference(rule_reference_trace)))
}

fn evaluate_rule_or_property(
    rule_name: &str,
    effective_selector: &str,
    json: &Value,
    rule_set: &RuleSet
) -> Result<(bool, Option<String>, Option<PropertyCheckTrace>), RuleError> {
    // Try to find a matching rule first
    if let Some((result, outcome)) = try_evaluate_by_rule(rule_name, json, rule_set)? {
        return Ok((result, Some(outcome), None));
    }

    // If no rule found, try to evaluate as a property
    if let Some(property_check) = try_evaluate_as_property(rule_name, effective_selector, json)? {
        let result = evaluate_property_result(&property_check);
        return Ok((result, None, Some(property_check)));
    }

    // If neither rule nor property found, assume true (free text condition)
    // This handles cases like "eye test" where no rule or property exists
    //eprintln!("Info: No rule or property found for '{}' - assuming true", rule_name);
    Ok((true, None, None))
}

fn try_evaluate_by_rule(
    rule_name: &str,
    json: &Value,
    rule_set: &RuleSet
) -> Result<Option<(bool, String)>, RuleError> {
    // Try exact outcome match
    if let Some(rule) = rule_set.get_rule(rule_name) {
        let (result, _) = evaluate_rule(rule, json, rule_set)?;
        return Ok(Some((result, rule.outcome.clone())));
    }

    // Try exact label match
    if let Some(rule) = rule_set.get_rule_by_label(rule_name) {
        let (result, _) = evaluate_rule(rule, json, rule_set)?;
        return Ok(Some((result, rule.outcome.clone())));
    }

    // Try fuzzy matching
    if let Some(rule) = find_rule_fuzzy_match(rule_name, rule_set) {
        let (result, _) = evaluate_rule(rule, json, rule_set)?;
        return Ok(Some((result, rule.outcome.clone())));
    }

    Ok(None)
}

fn find_rule_fuzzy_match<'a>(rule_name: &str, rule_set: &'a RuleSet) -> Option<&'a Rule> {
    let rule_name_lower = rule_name.to_lowercase();

    // First, try to match rules where the outcome contains key parts of the rule_name
    // For example: "passes the practical driving test" should match "the practical driving test"

    // Remove common prefixes that might be in the rule_name but not the outcome
    let prefixes_to_remove = ["passes the", "passes", "has", "has the", "is", "gets", "gets the"];
    let mut cleaned_rule_name = rule_name_lower.clone();

    for prefix in &prefixes_to_remove {
        if cleaned_rule_name.starts_with(prefix) {
            cleaned_rule_name = cleaned_rule_name[prefix.len()..].trim().to_string();
            break;
        }
    }

    // Now try to find a matching rule
    for rule in &rule_set.rules {
        let outcome_lower = rule.outcome.to_lowercase();

        // Check if the rule outcome matches the cleaned rule name
        if outcome_lower == cleaned_rule_name {
            return Some(rule);
        }

        // Check if either contains the other
        if outcome_lower.contains(&cleaned_rule_name) || cleaned_rule_name.contains(&outcome_lower) {
            return Some(rule);
        }

        // Original checks with full rule_name
        if outcome_lower == rule_name_lower ||
            outcome_lower.contains(&rule_name_lower) ||
            rule_name_lower.contains(&outcome_lower) {
            return Some(rule);
        }
    }

    None
}

fn try_evaluate_as_property(
    rule_name: &str,
    effective_selector: &str,
    json: &Value
) -> Result<Option<PropertyCheckTrace>, RuleError> {
    let possible_properties = crate::runner::utils::infer_possible_properties(rule_name);

    if let Some(obj) = json.get(effective_selector) {
        for property in &possible_properties {
            if let Some(property_value) = obj.get(property) {
                return Ok(Some(PropertyCheckTrace {
                    property_name: property.clone(),
                    property_value: convert_property_value(property_value),
                }));
            }
            // Also try case-insensitive match
            if let Some(property_value) = get_json_value_insensitive(obj, property) {
                return Ok(Some(PropertyCheckTrace {
                    property_name: property.clone(),
                    property_value: convert_property_value(property_value),
                }));
            }
        }
    } else if let Some(obj) = get_json_value_insensitive(json, effective_selector) {
        for property in &possible_properties {
            if let Some(property_value) = obj.get(property) {
                return Ok(Some(PropertyCheckTrace {
                    property_name: property.clone(),
                    property_value: convert_property_value(property_value),
                }));
            }
            // Also try case-insensitive match
            if let Some(property_value) = get_json_value_insensitive(obj, property) {
                return Ok(Some(PropertyCheckTrace {
                    property_name: property.clone(),
                    property_value: convert_property_value(property_value),
                }));
            }
        }
    }

    Ok(None)
}

fn convert_property_value(value: &Value) -> Value {
    match value {
        Value::Bool(b) => json!({ "Boolean": b }),
        Value::String(s) => json!({ "String": s }),
        Value::Number(n) => {
            if let Some(num) = n.as_f64() {
                json!({ "Number": num })
            } else {
                value.clone()
            }
        },
        _ => value.clone(),
    }
}

fn evaluate_property_result(property_check: &PropertyCheckTrace) -> bool {
    match &property_check.property_value {
        Value::Object(map) => {
            if let Some(Value::Bool(b)) = map.get("Boolean") {
                *b
            } else if let Some(Value::String(s)) = map.get("String") {
                let lower = s.to_lowercase();
                matches!(lower.as_str(), "pass" | "true" | "yes" | "passed" | "valid")
            } else if let Some(Value::Number(n)) = map.get("Number") {
                n.as_f64().map_or(false, |v| v != 0.0)
            } else {
                false
            }
        },
        _ => false,
    }
}

// ===== Comparison Evaluation =====

fn evaluate_comparison_condition(
    condition: &ComparisonCondition,
    json: &Value
) -> Result<(bool, ConditionTrace), RuleError> {
    // Find the effective selector
    let effective_selector = match find_effective_selector(&condition.selector.value, json)? {
        Some(sel) => sel,
        None => {
            return Ok((false, create_failed_comparison_trace(condition, None)));
        }
    };

    // Check if property exists
    let property_value = json.get(&effective_selector)
        .and_then(|obj| obj.get(&condition.property.value));

    if property_value.is_none() {
        return Ok((false, create_failed_comparison_trace(condition, Some(&effective_selector))));
    }

    // Extract and evaluate the comparison
    let json_value = extract_value_from_json(json, &effective_selector, &condition.property.value)?;
    let (comparison_result, evaluation_details) = perform_comparison(
        &json_value,
        &condition.operator,
        &condition.value.value
    )?;

    // Build the trace
    let comparison_trace = ComparisonTrace {
        selector: SelectorTrace {
            value: condition.selector.value.clone(),
            pos: condition.selector.pos.clone(),
        },
        property: PropertyTrace {
            value: property_value.unwrap().clone(),
            path: format!("$.{}.{}", effective_selector, condition.property.value),
        },
        operator: condition.operator.clone(),
        value: condition.value.value.to_value_trace(condition.value.pos.clone()),
        evaluation_details,
        result: comparison_result,
    };

    Ok((comparison_result, ConditionTrace::Comparison(comparison_trace)))
}

fn perform_comparison(
    json_value: &RuleValue,
    operator: &ComparisonOperator,
    value: &RuleValue
) -> Result<(bool, Option<ComparisonEvaluationTrace>), RuleError> {
    match evaluate_comparison(json_value, operator, value) {
        Ok(result) => {
            let details = ComparisonEvaluationTrace {
                left_value: TypedValue::from(json_value),
                right_value: TypedValue::from(value),
                comparison_result: result,
            };
            Ok((result, Some(details)))
        },
        Err(_) => Ok((false, None))
    }
}

// ===== Helper Functions =====

fn find_effective_selector(selector: &str, json: &Value) -> Result<Option<String>, RuleError> {
    // Try exact match first
    if json.get(selector).is_some() {
        return Ok(Some(selector.to_string()));
    }

    // Try case-insensitive match and return the actual key from the JSON
    if let Some(obj) = json.as_object() {
        let selector_lower = selector.to_lowercase();
        for (key, _) in obj {
            if key.to_lowercase() == selector_lower {
                return Ok(Some(key.clone())); // Return the actual key from JSON
            }
        }
    }

    // Try transformed selector (camelCase)
    let transformed = transform_selector_name(selector);
    if json.get(&transformed).is_some() {
        return Ok(Some(transformed));
    }

    // Try case-insensitive match on transformed selector
    if let Some(obj) = json.as_object() {
        let transformed_lower = transformed.to_lowercase();
        for (key, _) in obj {
            if key.to_lowercase() == transformed_lower {
                return Ok(Some(key.clone())); // Return the actual key from JSON
            }
        }
    }

    Ok(None)
}

fn create_failed_rule_reference_trace(condition: &RuleReferenceCondition) -> ConditionTrace {
    ConditionTrace::RuleReference(RuleReferenceTrace {
        selector: SelectorTrace {
            value: condition.selector.value.clone(),
            pos: condition.selector.pos.clone(),
        },
        rule_name: condition.rule_name.value.clone(),
        referenced_rule_outcome: None,
        property_check: None,
        result: false,
    })
}

fn create_failed_comparison_trace(
    condition: &ComparisonCondition,
    effective_selector: Option<&str>
) -> ConditionTrace {
    let path = if let Some(sel) = effective_selector {
        format!("$.{}.{}", sel, condition.property.value)
    } else {
        format!("$.{}.{}", condition.selector.value, condition.property.value)
    };

    ConditionTrace::Comparison(ComparisonTrace {
        selector: SelectorTrace {
            value: condition.selector.value.clone(),
            pos: condition.selector.pos.clone(),
        },
        property: PropertyTrace {
            value: Value::Null,
            path,
        },
        operator: condition.operator.clone(),
        value: condition.value.value.to_value_trace(condition.value.pos.clone()),
        evaluation_details: None,
        result: false,
    })
}

fn extract_value_from_json(
    json: &Value,
    selector: &str,
    property: &str
) -> Result<RuleValue, RuleError> {
    // First try to get the object using the selector
    let obj = if let Some(obj) = json.get(selector) {
        obj
    } else if let Some(obj) = get_json_value_insensitive(json, selector) {
        obj
    } else {
        let transformed_selector = transform_selector_name(selector);
        if let Some(obj) = json.get(&transformed_selector) {
            obj
        } else if let Some(obj) = get_json_value_insensitive(json, &transformed_selector) {
            obj
        } else {
            return Err(RuleError::EvaluationError(
                format!("Selector '{}' not found in JSON", selector)
            ));
        }
    };

    // Then try to get the property from the object (also case-insensitive)
    let value = if let Some(val) = obj.get(property) {
        val
    } else if let Some(val) = get_json_value_insensitive(obj, property) {
        val
    } else {
        return Err(RuleError::EvaluationError(
            format!("Property '{}' not found in selector '{}'", property, selector)
        ));
    };

    // Rest of the function remains the same...
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

fn get_json_value_insensitive<'a>(json: &'a serde_json::Value, key: &str) -> Option<&'a serde_json::Value> {
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