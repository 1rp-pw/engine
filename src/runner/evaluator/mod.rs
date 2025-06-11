mod lib;

use crate::runner::error::{RuleError, EvaluationResult, PartialRuleTrace};
use crate::runner::model::{Condition, ComparisonOperator, Rule, RuleSet, RuleValue, ConditionOperator, ComparisonCondition, RuleReferenceCondition, PropertyChainElement};
use crate::runner::trace::{RuleSetTrace, RuleTrace, ConditionTrace, ComparisonTrace, ComparisonEvaluationTrace, RuleReferenceTrace, PropertyCheckTrace, PropertyTrace, TypedValue, SelectorTrace, OutcomeTrace};

use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use chrono::NaiveDate;
use crate::runner::utils::{transform_property_name, names_match};

impl RuleError {
    pub fn infinite_loop_error(cycle_path: Vec<String>) -> Self {
        RuleError::EvaluationError(format!(
            "Infinite loop detected in rule evaluation: {} -> {}",
            cycle_path.join(" -> "),
            cycle_path[0]
        ))
    }
}

#[allow(dead_code)]
pub fn evaluate_rule_set_with_trace(
    rule_set: &RuleSet,
    json: &Value
) -> EvaluationResult<HashMap<String, bool>> {
    let mut all_traces: Vec<RuleTrace> = Vec::new();
    let mut results = HashMap::new();
    let mut processed_rules = HashSet::new();
    
    // Find global rule and handle potential error
    let global_rule = match crate::runner::utils::find_global_rule(&rule_set.rules) {
        Ok(rule) => rule,
        Err(error) => {
            // Even if we can't find global rule, return what trace we can
            let trace = RuleSetTrace { execution: all_traces };
            return EvaluationResult::failure(error, Some(trace));
        }
    };
    
    let mut evaluation_stack = HashSet::new();
    let mut call_path = Vec::new();

    // Evaluate global rule with trace preservation
    match evaluate_rule_with_trace(global_rule, json, rule_set, &mut evaluation_stack, &mut call_path) {
        Ok((result, rule_trace)) => {
            results.insert(global_rule.outcome.clone(), result);
            all_traces.push(rule_trace);
            processed_rules.insert(global_rule.outcome.clone());
        }
        Err((error, partial_trace)) => {
            // Convert partial trace and return failure with trace
            if let Some(trace) = partial_trace {
                all_traces.push(trace.to_rule_trace());
            }
            let rule_set_trace = RuleSetTrace { execution: all_traces };
            return EvaluationResult::failure(error, Some(rule_set_trace));
        }
    }

    // Continue with referenced rules
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

        // Process collected rules
        for (outcome, rule) in rules_to_process {
            let mut sub_evaluation_stack = HashSet::new();
            let mut sub_call_path = Vec::new();

            match evaluate_rule_with_trace(rule, json, rule_set, &mut sub_evaluation_stack, &mut sub_call_path) {
                Ok((sub_result, sub_trace)) => {
                    results.insert(outcome, sub_result);
                    all_traces.push(sub_trace);
                }
                Err((error, partial_trace)) => {
                    // On error, include partial trace and return failure
                    if let Some(trace) = partial_trace {
                        all_traces.push(trace.to_rule_trace());
                    }
                    let rule_set_trace = RuleSetTrace { execution: all_traces };
                    return EvaluationResult::failure(error, Some(rule_set_trace));
                }
            }
        }

        i += 1;
    }

    let rule_set_trace = RuleSetTrace { execution: all_traces };
    EvaluationResult::success(results, rule_set_trace)
}

#[allow(dead_code)]
pub fn evaluate_rule_set(
    rule_set: &RuleSet,
    json: &Value
) -> Result<(HashMap<String, bool>, RuleSetTrace), RuleError> {
    let global_rule= crate::runner::utils::find_global_rule(&rule_set.rules)?;
    let mut evaluation_stack = HashSet::new();
    let mut call_path = Vec::new();

    let (result, rule_trace) = evaluate_rule(global_rule, json, rule_set, &mut evaluation_stack, &mut call_path)?;
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
            let mut sub_evaluation_stack = HashSet::new();
            let mut sub_call_path = Vec::new();

            let (sub_result, sub_trace) = evaluate_rule(rule, json, rule_set, &mut sub_evaluation_stack, &mut sub_call_path)?;
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

/// Enhanced rule evaluation that preserves traces even on errors
pub fn evaluate_rule_with_trace(
    model_rule: &Rule,
    json: &Value,
    rule_set: &RuleSet,
    evaluation_stack: &mut HashSet<String>,
    call_path: &mut Vec<String>,
) -> Result<(bool, RuleTrace), (RuleError, Option<PartialRuleTrace>)> {
    // Initialize partial trace to capture progress
    let mut partial_trace = PartialRuleTrace::new(
        model_rule.label.clone(),
        model_rule.selector.clone(),
        model_rule.selector_pos.clone(),
        model_rule.outcome.clone(),
        model_rule.position.clone(),
    );

    // cycle check
    let rule_identifier = model_rule.outcome.clone();
    if evaluation_stack.contains(&rule_identifier) {
        call_path.push(rule_identifier.clone());
        let error = RuleError::infinite_loop_error(call_path.clone());
        partial_trace.set_error(format!("Infinite loop detected: {}", error));
        return Err((error, Some(partial_trace)));
    }
    evaluation_stack.insert(rule_identifier.clone());
    call_path.push(rule_identifier.clone());

    // evaluate each condition, collect results and traces
    let mut results = Vec::new();
    let mut ops = Vec::new();
    let mut condition_traces = Vec::new();

    for (i, cg) in model_rule.conditions.iter().enumerate() {
        match evaluate_condition_with_trace(&cg.condition, json, rule_set, evaluation_stack, call_path) {
            Ok((res, trace)) => {
                results.push(res);
                partial_trace.add_condition(trace.clone());
                condition_traces.push(trace);
            }
            Err((error, condition_trace)) => {
                // Add any partial condition trace we have
                if let Some(trace) = condition_trace {
                    partial_trace.add_condition(trace);
                }
                partial_trace.set_error(format!("Condition evaluation failed: {}", error));
                evaluation_stack.remove(&rule_identifier);
                call_path.pop();
                return Err((error, Some(partial_trace)));
            }
        }

        // record the operator that follows this condition
        if let Some(op) = &cg.operator {
            ops.push(op.clone());
        } else if i != 0 {
            ops.push(ConditionOperator::And);
        }
    }

    evaluation_stack.remove(&rule_identifier);
    call_path.pop();

    // collapse all ANDs first
    let mut i = 0;
    while i < ops.len() {
        if ops[i] == ConditionOperator::And {
            results[i] = results[i] && results[i + 1];
            results.remove(i + 1);
            ops.remove(i);
        } else {
            i += 1;
        }
    }

    // fold OR across what remains
    let rule_result = results
        .into_iter()
        .fold(false, |acc, next| acc || next);

    // build complete trace object
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

pub fn evaluate_rule(
    model_rule: &Rule,
    json: &Value,
    rule_set: &RuleSet,
    evaluation_stack: &mut HashSet<String>,
    call_path: &mut Vec<String>,
) -> Result<(bool, RuleTrace), RuleError> {
    // cycle check
    let rule_identifier = model_rule.outcome.clone();
    if evaluation_stack.contains(&rule_identifier) {
        call_path.push(rule_identifier.clone());
        return Err(RuleError::infinite_loop_error(call_path.clone()));
    }
    evaluation_stack.insert(rule_identifier.clone());
    call_path.push(rule_identifier.clone());

    // 1) evaluate each condition, collect its bool and its trace
    let mut results = Vec::new();
    let mut ops     = Vec::new();
    let mut condition_traces = Vec::new();

    for (i, cg) in model_rule.conditions.iter().enumerate() {
        let (res, trace) = evaluate_condition(&cg.condition, json, rule_set, evaluation_stack, call_path)?;
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

    evaluation_stack.remove(&rule_identifier);
    call_path.pop();

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

#[allow(dead_code)]
fn evaluate_condition_with_trace(
    condition: &Condition,
    json: &Value,
    rule_set: &RuleSet,
    evaluation_stack: &mut HashSet<String>,
    call_path: &mut Vec<String>,
) -> Result<(bool, ConditionTrace), (RuleError, Option<ConditionTrace>)> {
    match condition {
        Condition::RuleReference(ref_condition) => {
            match evaluate_rule_reference_condition_with_trace(ref_condition, json, rule_set, evaluation_stack, call_path) {
                Ok(result) => Ok(result),
                Err((error, trace)) => Err((error, trace))
            }
        },
        Condition::Comparison(comp_condition) => {
            match evaluate_comparison_condition_with_trace(comp_condition, json) {
                Ok(result) => Ok(result),
                Err((error, trace)) => Err((error, trace))
            }
        },
    }
}

fn evaluate_condition(
    condition: &Condition,
    json: &Value,
    rule_set: &RuleSet,
    evaluation_stack: &mut HashSet<String>,
    call_path: &mut Vec<String>,
) -> Result<(bool, ConditionTrace), RuleError> {
    match condition {
        Condition::RuleReference(ref_condition) => {
            evaluate_rule_reference_condition(ref_condition, json, rule_set, evaluation_stack, call_path)
        },
        Condition::Comparison(comp_condition) => {
            evaluate_comparison_condition(comp_condition, json)
        },
    }
}

fn evaluate_rule_reference_condition(
    condition: &RuleReferenceCondition,
    json: &Value,
    rule_set: &RuleSet,
    evaluation_stack: &mut HashSet<String>,
    call_path: &mut Vec<String>,
) -> Result<(bool, ConditionTrace), RuleError> {
    // Handle empty selector case (for label references)
    if condition.selector.value.is_empty() {
        // This is a label reference without a selector
        let rule_name = condition.rule_name.value.trim();

        // Try to find and evaluate the referenced rule
        if let Some((result, outcome)) = try_evaluate_by_rule(rule_name, json, rule_set, evaluation_stack, call_path)? {
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

    let (result, referenced_outcome, property_check) = if effective_selector.is_some() {
        // Selector exists in JSON - use it directly
        let part = condition.rule_name.value.trim();
        evaluate_rule_or_property(part, &effective_selector.unwrap(), json, rule_set, evaluation_stack, call_path)?
    } else {
        // Conceptual selector - try to evaluate the rule without requiring the selector to exist
        let part = condition.rule_name.value.trim();
        
        // First, try to find the rule globally (without a specific selector)
        if let Some((rule_result, outcome)) = try_evaluate_by_rule(part, json, rule_set, evaluation_stack, call_path)? {
            (rule_result, Some(outcome), None)
        } else {
            // If no global rule found, try to evaluate against all available objects in the JSON
            let mut found_any_match = false;
            let mut last_outcome = None;
            let mut last_property_check = None;
            
            if let Some(obj) = json.as_object() {
                for (key, _) in obj {
                    if let Ok((rule_result, outcome, prop_check)) = evaluate_rule_or_property(part, key, json, rule_set, evaluation_stack, call_path) {
                        if rule_result {
                            found_any_match = true;
                            last_outcome = outcome;
                            last_property_check = prop_check;
                            break; // Found a match, we can stop
                        }
                    }
                }
            }
            
            (found_any_match, last_outcome, last_property_check)
        }
    };

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

#[allow(dead_code)]
fn evaluate_rule_reference_condition_with_trace(
    condition: &RuleReferenceCondition,
    json: &Value,
    rule_set: &RuleSet,
    evaluation_stack: &mut HashSet<String>,
    call_path: &mut Vec<String>,
) -> Result<(bool, ConditionTrace), (RuleError, Option<ConditionTrace>)> {
    // Handle empty selector case (for label references)
    if condition.selector.value.is_empty() {
        let rule_name = condition.rule_name.value.trim();

        // Try to find and evaluate the referenced rule
        match try_evaluate_by_rule_with_trace(rule_name, json, rule_set, evaluation_stack, call_path) {
            Ok(Some((result, outcome))) => {
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
            Ok(None) => {
                // Rule not found
                return Ok((false, create_failed_rule_reference_trace(condition)));
            }
            Err((error, _)) => {
                // Error during rule evaluation
                let failed_trace = create_failed_rule_reference_trace(condition);
                return Err((error, Some(failed_trace)));
            }
        }
    }

    // Normal case with selector
    let effective_selector = match find_effective_selector(&condition.selector.value, json) {
        Ok(sel) => sel,
        Err(error) => {
            let failed_trace = create_failed_rule_reference_trace(condition);
            return Err((error, Some(failed_trace)));
        }
    };

    let part = condition.rule_name.value.trim();
    let (result, referenced_outcome, property_check) = if effective_selector.is_some() {
        // Selector exists in JSON - use it directly
        match evaluate_rule_or_property_with_trace(part, &effective_selector.unwrap(), json, rule_set, evaluation_stack, call_path) {
            Ok(result) => result,
            Err((error, _)) => {
                let failed_trace = create_failed_rule_reference_trace(condition);
                return Err((error, Some(failed_trace)));
            }
        }
    } else {
        // Conceptual selector - try to evaluate the rule without requiring the selector to exist
        
        // First, try to find the rule globally (without a specific selector)
        match try_evaluate_by_rule_with_trace(part, json, rule_set, evaluation_stack, call_path) {
            Ok(Some((rule_result, outcome))) => {
                (rule_result, Some(outcome), None)
            }
            Ok(None) => {
                // If no global rule found, try to evaluate against all available objects in the JSON
                let mut found_any_match = false;
                let mut last_outcome = None;
                let mut last_property_check = None;
                
                if let Some(obj) = json.as_object() {
                    for (key, _) in obj {
                        match evaluate_rule_or_property_with_trace(part, key, json, rule_set, evaluation_stack, call_path) {
                            Ok((rule_result, outcome, prop_check)) => {
                                if rule_result {
                                    found_any_match = true;
                                    last_outcome = outcome;
                                    last_property_check = prop_check;
                                    break; // Found a match, we can stop
                                }
                            }
                            Err(_) => {
                                // Ignore errors and continue to next key
                                continue;
                            }
                        }
                    }
                }
                
                (found_any_match, last_outcome, last_property_check)
            }
            Err((error, _)) => {
                let failed_trace = create_failed_rule_reference_trace(condition);
                return Err((error, Some(failed_trace)));
            }
        }
    };

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

#[allow(dead_code)]
fn evaluate_comparison_condition_with_trace(
    condition: &ComparisonCondition,
    json: &Value
) -> Result<(bool, ConditionTrace), (RuleError, Option<ConditionTrace>)> {
    // Check if this is a cross-object comparison
    if let Some(left_path) = &condition.left_property_path {
        return match evaluate_cross_object_comparison(condition, left_path, json) {
            Ok(result) => Ok(result),
            Err(error) => {
                let failed_trace = create_failed_comparison_trace(condition, None);
                Err((error, Some(failed_trace)))
            }
        };
    }

    // Check if this is a chained property access
    if let Some(property_chain) = &condition.property_chain {
        return match evaluate_chained_comparison_condition(condition, property_chain, json) {
            Ok(result) => Ok(result),
            Err(error) => {
                let failed_trace = create_failed_comparison_trace(condition, None);
                Err((error, Some(failed_trace)))
            }
        };
    }

    // Original simple property condition logic
    let effective_selector = match find_effective_selector(&condition.selector.value, json) {
        Ok(Some(sel)) => sel,
        Ok(None) => {
            return Ok((false, create_failed_comparison_trace(condition, None)));
        }
        Err(error) => {
            let failed_trace = create_failed_comparison_trace(condition, None);
            return Err((error, Some(failed_trace)));
        }
    };

    // Check if property exists
    let property_value = json.get(&effective_selector)
        .and_then(|obj| obj.get(&condition.property.value));

    if property_value.is_none() {
        return Ok((false, create_failed_comparison_trace(condition, Some(&effective_selector))));
    }

    // Extract and evaluate the comparison
    let json_value = match extract_value_from_json(json, &effective_selector, &condition.property.value) {
        Ok(value) => value,
        Err(error) => {
            let failed_trace = create_failed_comparison_trace(condition, Some(&effective_selector));
            return Err((error, Some(failed_trace)));
        }
    };
    
    let (comparison_result, evaluation_details) = match perform_comparison(
        &json_value,
        &condition.operator,
        &condition.value.value
    ) {
        Ok(result) => result,
        Err(error) => {
            let failed_trace = create_failed_comparison_trace(condition, Some(&effective_selector));
            return Err((error, Some(failed_trace)));
        }
    };

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

fn evaluate_rule_or_property(
    rule_name: &str,
    effective_selector: &str,
    json: &Value,
    rule_set: &RuleSet,
    evaluation_stack: &mut HashSet<String>,
    call_path: &mut Vec<String>,
) -> Result<(bool, Option<String>, Option<PropertyCheckTrace>), RuleError> {
    // Try to find a matching rule first
    if let Some((result, outcome)) = try_evaluate_by_rule(rule_name, json, rule_set, evaluation_stack, call_path)? {
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
    rule_set: &RuleSet,
    evaluation_stack: &mut HashSet<String>,
    call_path: &mut Vec<String>,
) -> Result<Option<(bool, String)>, RuleError> {
    // Try exact outcome match
    if let Some(rule) = rule_set.get_rule(rule_name) {
        let (result, _) = evaluate_rule(rule, json, rule_set, evaluation_stack, call_path)?;
        return Ok(Some((result, rule.outcome.clone())));
    }

    // Try exact label match
    if let Some(rule) = rule_set.get_rule_by_label(rule_name) {
        let (result, _) = evaluate_rule(rule, json, rule_set, evaluation_stack, call_path)?;
        return Ok(Some((result, rule.outcome.clone())));
    }

    // Try fuzzy matching
    if let Some(rule) = find_rule_fuzzy_match(rule_name, rule_set) {
        let (result, _) = evaluate_rule(rule, json, rule_set, evaluation_stack, call_path)?;
        return Ok(Some((result, rule.outcome.clone())));
    }

    Ok(None)
}

fn find_rule_fuzzy_match<'a>(rule_name: &str, rule_set: &'a RuleSet) -> Option<&'a Rule> {
    // Check cache first
    if let Ok(cache) = rule_set.cache.rule_fuzzy_matches.read() {
        if let Some(cached_outcome) = cache.get(rule_name) {
            return cached_outcome.as_ref().and_then(|outcome| rule_set.get_rule(outcome));
        }
    }

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
    let mut found_outcome: Option<String> = None;
    
    // Pre-allocate lowercase strings to avoid repeated allocations in loop
    let rule_outcomes_lower: Vec<_> = rule_set.rules
        .iter()
        .map(|rule| rule.outcome.to_lowercase())
        .collect();
    
    for (rule, outcome_lower) in rule_set.rules.iter().zip(rule_outcomes_lower.iter()) {
        // Check if the rule outcome matches the cleaned rule name
        if outcome_lower == &cleaned_rule_name {
            found_outcome = Some(rule.outcome.clone());
            break;
        }

        // Check if either contains the other
        if outcome_lower.contains(&cleaned_rule_name) || cleaned_rule_name.contains(outcome_lower) {
            found_outcome = Some(rule.outcome.clone());
            break;
        }

        // Original checks with full rule_name
        if outcome_lower == &rule_name_lower ||
            outcome_lower.contains(&rule_name_lower) ||
            rule_name_lower.contains(outcome_lower) {
            found_outcome = Some(rule.outcome.clone());
            break;
        }
    }

    // Cache the result
    if let Ok(mut cache) = rule_set.cache.rule_fuzzy_matches.write() {
        cache.insert(rule_name.to_string(), found_outcome.clone());
    }

    found_outcome.and_then(|outcome| rule_set.get_rule(&outcome))
}

#[allow(dead_code)]
fn try_evaluate_by_rule_with_trace(
    rule_name: &str,
    json: &Value,
    rule_set: &RuleSet,
    evaluation_stack: &mut HashSet<String>,
    call_path: &mut Vec<String>,
) -> Result<Option<(bool, String)>, (RuleError, Option<PartialRuleTrace>)> {
    // Try exact outcome match
    if let Some(rule) = rule_set.get_rule(rule_name) {
        match evaluate_rule_with_trace(rule, json, rule_set, evaluation_stack, call_path) {
            Ok((result, _)) => return Ok(Some((result, rule.outcome.clone()))),
            Err((error, partial_trace)) => return Err((error, partial_trace))
        }
    }
    
    // Try exact label match
    if let Some(rule) = rule_set.get_rule_by_label(rule_name) {
        match evaluate_rule_with_trace(rule, json, rule_set, evaluation_stack, call_path) {
            Ok((result, _)) => return Ok(Some((result, rule.outcome.clone()))),
            Err((error, partial_trace)) => return Err((error, partial_trace))
        }
    }
    
    // Try fuzzy matching
    if let Some(rule) = find_rule_fuzzy_match(rule_name, rule_set) {
        match evaluate_rule_with_trace(rule, json, rule_set, evaluation_stack, call_path) {
            Ok((result, _)) => return Ok(Some((result, rule.outcome.clone()))),
            Err((error, partial_trace)) => return Err((error, partial_trace))
        }
    }
    
    Ok(None)
}

#[allow(dead_code)]
fn evaluate_rule_or_property_with_trace(
    rule_name: &str,
    effective_selector: &str,
    json: &Value,
    rule_set: &RuleSet,
    evaluation_stack: &mut HashSet<String>,
    call_path: &mut Vec<String>,
) -> Result<(bool, Option<String>, Option<PropertyCheckTrace>), (RuleError, Option<PartialRuleTrace>)> {
    // Try to find a matching rule first
    match try_evaluate_by_rule_with_trace(rule_name, json, rule_set, evaluation_stack, call_path) {
        Ok(Some((result, outcome))) => {
            return Ok((result, Some(outcome), None));
        }
        Ok(None) => {
            // Continue to property evaluation
        }
        Err((error, partial_trace)) => {
            return Err((error, partial_trace));
        }
    }

    // If no rule found, try to evaluate as a property
    match try_evaluate_as_property(rule_name, effective_selector, json) {
        Ok(Some(property_check)) => {
            let result = evaluate_property_result(&property_check);
            Ok((result, None, Some(property_check)))
        }
        Ok(None) => {
            // If neither rule nor property found, assume true (free text condition)
            Ok((true, None, None))
        }
        Err(error) => {
            Err((error, None))
        }
    }
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

fn calculate_length_of(value: &Value) -> Result<f64, RuleError> {
    match value {
        Value::String(s) => Ok(s.len() as f64),
        Value::Array(arr) => Ok(arr.len() as f64),
        Value::Object(obj) => Ok(obj.len() as f64), // Object property count
        Value::Null => Ok(0.0),
        _ => Err(RuleError::EvaluationError(
            format!("Cannot calculate length of {:?}", value)
        ))
    }
}

fn calculate_number_of(value: &Value) -> Result<f64, RuleError> {
    match value {
        Value::Array(arr) => Ok(arr.len() as f64),
        Value::Null => Ok(0.0),
        _ => Err(RuleError::EvaluationError(
            format!("Cannot calculate number of {:?} for non array", value)
        ))
    }
}

// ===== Comparison Evaluation =====
#[allow(dead_code)]
fn evaluate_comparison_condition(
    condition: &ComparisonCondition,
    json: &Value
) -> Result<(bool, ConditionTrace), RuleError> {
    // Check if this is a cross-object comparison
    if let Some(left_path) = &condition.left_property_path {
        return evaluate_cross_object_comparison(condition, left_path, json);
    }

    // Check if this is a chained property access
    if let Some(property_chain) = &condition.property_chain {
        return evaluate_chained_comparison_condition(condition, property_chain, json);
    }

    // Original simple property condition logic
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

fn evaluate_cross_object_comparison(
    condition: &ComparisonCondition,
    left_path: &crate::runner::model::PropertyPath,
    json: &Value
) -> Result<(bool, ConditionTrace), RuleError> {
    if is_length_of_operation(left_path) {
        return evaluate_length_of_comparison(condition, left_path, json);
    }
    if is_number_of_operation(left_path) {
        return evaluate_number_of_comparison(condition, left_path, json);
    }

    // Resolve left property path
    let (left_value, left_path_str) = resolve_property_path(left_path, json)?;
    if left_value.is_none() {
        return Ok((false, create_failed_comparison_trace_with_path(condition, &left_path_str)));
    }

    let left_rule_value = convert_json_to_rule_value(left_value.unwrap())?;

    let (comparison_result, evaluation_details) = if let Some(right_path) = &condition.right_property_path {
        // Property-to-property comparison
        let (right_value, _right_path_str) = resolve_property_path(right_path, json)?;

        if right_value.is_none() {
            return Ok((false, create_failed_comparison_trace_with_path(condition, &left_path_str)));
        }

        let right_rule_value = convert_json_to_rule_value(right_value.unwrap())?;
        perform_comparison(&left_rule_value, &condition.operator, &right_rule_value)?
    } else {
        // Property-to-value comparison
        perform_comparison(&left_rule_value, &condition.operator, &condition.value.value)?
    };

    // Build the trace
    let comparison_trace = ComparisonTrace {
        selector: SelectorTrace {
            value: left_path.selector.clone(),
            pos: None,
        },
        property: PropertyTrace {
            value: left_value.unwrap().clone(),
            path: left_path_str.clone(),
        },
        operator: condition.operator.clone(),
        value: condition.value.value.to_value_trace(condition.value.pos.clone()),
        evaluation_details,
        result: comparison_result,
    };

    Ok((comparison_result, ConditionTrace::Comparison(comparison_trace)))
}

fn evaluate_length_of_comparison(
    condition: &ComparisonCondition,
    left_path: &crate::runner::model::PropertyPath,
    json: &Value
) -> Result<(bool, ConditionTrace), RuleError> {
    let mut actual_path = left_path.clone();
    actual_path.properties.pop();

    let (target_value, mut path_str) = resolve_property_path(&actual_path, json)?;
    if target_value.is_none() {
        path_str = format!("{}.length", path_str);
        return Ok((false, create_failed_comparison_trace_with_path(condition, &path_str)));
    }

    // Calculate length
    let length = calculate_length_of(target_value.unwrap())?;
    let length_rule_value = RuleValue::Number(length);

    // Perform comparison
    let (comparison_result, evaluation_details) = perform_comparison(
        &length_rule_value,
        &condition.operator,
        &condition.value.value
    )?;

    // Build the trace with length information
    let length_path = format!("{}.length", path_str);
    let comparison_trace = ComparisonTrace {
        selector: SelectorTrace {
            value: left_path.selector.clone(),
            pos: None,
        },
        property: PropertyTrace {
            value: serde_json::json!(length), // Show the calculated length
            path: length_path,
        },
        operator: condition.operator.clone(),
        value: condition.value.value.to_value_trace(condition.value.pos.clone()),
        evaluation_details,
        result: comparison_result,
    };

    Ok((comparison_result, ConditionTrace::Comparison(comparison_trace)))
}

fn evaluate_number_of_comparison(
    condition: &ComparisonCondition,
    left_path: &crate::runner::model::PropertyPath,
    json: &Value
) -> Result<(bool, ConditionTrace), RuleError> {
    let mut actual_path = left_path.clone();
    actual_path.properties.pop();

    let (target_value, mut path_str) = resolve_property_path(&actual_path, json)?;
    if target_value.is_none() {
        path_str = format!("{}.length", path_str);
        return Ok((false, create_failed_comparison_trace_with_path(condition, &path_str)));
    }

    // Calculate length
    let number = calculate_number_of(target_value.unwrap())?;
    let number_rule_value = RuleValue::Number(number);

    // Perform comparison
    let (comparison_result, evaluation_details) = perform_comparison(
        &number_rule_value,
        &condition.operator,
        &condition.value.value
    )?;

    // Build the trace with length information
    let number_path = format!("{}.number", path_str);
    let comparison_trace = ComparisonTrace {
        selector: SelectorTrace {
            value: left_path.selector.clone(),
            pos: None,
        },
        property: PropertyTrace {
            value: serde_json::json!(number), // Show the calculated length
            path: number_path,
        },
        operator: condition.operator.clone(),
        value: condition.value.value.to_value_trace(condition.value.pos.clone()),
        evaluation_details,
        result: comparison_result,
    };

    Ok((comparison_result, ConditionTrace::Comparison(comparison_trace)))
}

fn is_length_of_operation(path: &crate::runner::model::PropertyPath) -> bool {
    path.properties.last() == Some(&"__length_of__".to_string())
}
fn is_number_of_operation(path: &crate::runner::model::PropertyPath) -> bool {
    path.properties.last() == Some(&"__number_of__".to_string())
}


fn resolve_property_path<'a>(
    path: &crate::runner::model::PropertyPath,
    json: &'a Value
) -> Result<(Option<&'a Value>, String), RuleError> {
    let mut path_parts = vec![path.selector.clone()];
    let mut current_value = json;

    // Check if the selector contains dots (nested path)
    if path.selector.contains('.') {
        // Handle nested selector path
        let selector_parts: Vec<&str> = path.selector.split('.').collect();
        for part in &selector_parts {
            let effective_part = find_effective_selector(part, current_value)?;
            if effective_part.is_none() {
                return Ok((None, format!("$.{}", path.selector)));
            }
            let final_part = effective_part.unwrap();
            current_value = current_value.get(&final_part)
                .ok_or_else(|| RuleError::EvaluationError(format!("Selector part '{}' not found", final_part)))?;
            path_parts.push(final_part);
        }
    } else {
        // Handle regular single selector
        let effective_selector = find_effective_selector(&path.selector, json)?;
        if effective_selector.is_none() {
            return Ok((None, format!("$.{}", path.selector)));
        }

        let final_selector = effective_selector.unwrap();
        current_value = json.get(&final_selector)
            .ok_or_else(|| RuleError::EvaluationError(format!("Selector '{}' not found", final_selector)))?;
        path_parts[0] = final_selector.clone(); // Use the actual key from JSON
    }

    let is_length_of_operator = is_length_of_operation(path);
    let is_number_of_operator = is_number_of_operation(path);
    
    let properties_to_process = if is_length_of_operator {
        &path.properties[..path.properties.len() - 1]
    } else if is_number_of_operator {
        &path.properties[..path.properties.len() - 1]
    } else {
        &path.properties[..]
    };

    // Follow the property chain - properties are already in correct traversal order
    // For "__date of birth__ of **person** of **driving test**", we get properties: ["person", "date of birth"]
    // And we traverse: driving test -> person -> date of birth
    for property in properties_to_process.iter() {
        let mut found_property = None;
        let mut actual_property_name = property.clone();

        if let Some(prop_value) = current_value.get(property) {
            found_property = Some(prop_value);
        } else if let Some(prop_value) = get_json_value_insensitive(current_value, property) {
            found_property = Some(prop_value);
            // Find the actual property name in the JSON for path tracking
            if let Some(obj) = current_value.as_object() {
                for (key, _) in obj {
                    if names_match(property, key) {
                        actual_property_name = key.clone();
                        break;
                    }
                }
            }
        } else {
            let transformed_property = transform_property_name(property);
            if let Some(prop_value) = current_value.get(&transformed_property) {
                found_property = Some(prop_value);
                actual_property_name = transformed_property.clone();
            } else if let Some(prop_value) = get_json_value_insensitive(current_value, &transformed_property) {
                found_property = Some(prop_value);
                // Find the actual property name in the JSON for path tracking
                if let Some(obj) = current_value.as_object() {
                    for (key, _) in obj {
                        if names_match(&transformed_property, key) {
                            actual_property_name = key.clone();
                            break;
                        }
                    }
                }
            }
        }

        if let Some(prop_value) = found_property {
            current_value = prop_value;
            path_parts.push(actual_property_name);
        } else {
            return Ok((None, format!("$.{}", path_parts.join("."))));
        }
    }

    if is_length_of_operator {
        calculate_length_of(current_value)?;
        path_parts.push("length".to_string());
        let path_str = format!("$.{}", path_parts.join("."));
        return Ok((Some(current_value), path_str));
    }

    let path_str = format!("$.{}", path_parts.join("."));
    Ok((Some(current_value), path_str))
}

#[allow(dead_code)]
fn evaluate_chained_comparison_condition(
    condition: &ComparisonCondition,
    property_chain: &[PropertyChainElement],
    json: &Value
) -> Result<(bool, ConditionTrace), RuleError> {
    // Resolve the chained property access
    let (final_value, path) = resolve_chained_property_access(
        &condition.property.value,
        &condition.selector.value,
        property_chain,
        json
    )?;

    if final_value.is_none() {
        return Ok((false, create_failed_comparison_trace_with_path(condition, &path)));
    }

    // Extract and evaluate the comparison
    let json_value = convert_json_to_rule_value(final_value.unwrap())?;
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
            value: final_value.unwrap().clone(),
            path: path.clone(),
        },
        operator: condition.operator.clone(),
        value: condition.value.value.to_value_trace(condition.value.pos.clone()),
        evaluation_details,
        result: comparison_result,
    };

    Ok((comparison_result, ConditionTrace::Comparison(comparison_trace)))
}

fn resolve_chained_property_access<'a>(
    first_property: &str,
    final_selector: &str,
    chain: &[PropertyChainElement],
    json: &'a Value
) -> Result<(Option<&'a Value>, String), RuleError> {
    let mut path_parts = Vec::new();

    // Start with the final selector
    let effective_selector = find_effective_selector(final_selector, json)?;

    if effective_selector.is_none() {
        return Ok((None, format!("$.{}", final_selector)));
    }

    let final_sel = effective_selector.unwrap();
    let mut current_value = json.get(&final_sel)
        .ok_or_else(|| RuleError::EvaluationError(format!("Selector '{}' not found", final_sel)))?;

    path_parts.push(final_sel);

    // Follow the chain
    for element in chain {
        match element {
            PropertyChainElement::Property(property) => {
                if let Some(prop_value) = current_value.get(property) {
                    current_value = prop_value;
                    path_parts.push(property.clone());
                } else if let Some(prop_value) = get_json_value_insensitive(current_value, property) {
                    current_value = prop_value;
                    path_parts.push(property.clone());
                } else {
                    return Ok((None, format!("$.{}", path_parts.join("."))));
                }
            }
            PropertyChainElement::Selector(selector) => {
                if let Some(sel_value) = current_value.get(selector) {
                    current_value = sel_value;
                    path_parts.push(selector.clone());
                } else if let Some(sel_value) = get_json_value_insensitive(current_value, selector) {
                    current_value = sel_value;
                    path_parts.push(selector.clone());
                } else {
                    return Ok((None, format!("$.{}", path_parts.join("."))));
                }
            }
        }
    }

    // Finally, get the first property
    if let Some(final_prop_value) = current_value.get(first_property) {
        current_value = final_prop_value;
        path_parts.push(first_property.to_string());
    } else if let Some(final_prop_value) = get_json_value_insensitive(current_value, first_property) {
        current_value = final_prop_value;
        path_parts.push(first_property.to_string());
    } else {
        return Ok((None, format!("$.{}", path_parts.join("."))));
    }

    let path = format!("$.{}", path_parts.join("."));
    Ok((Some(current_value), path))
}

fn convert_json_to_rule_value(value: &Value) -> Result<RuleValue, RuleError> {
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

    // Try fuzzy matching with all JSON keys
    if let Some(obj) = json.as_object() {
        for (key, _) in obj {
            if names_match(selector, key) {
                return Ok(Some(key.clone())); // Return the actual key from JSON
            }
        }
    }

    Ok(None)
}

#[allow(dead_code)]
fn find_effective_selector_with_mapping(selector: &str, json: &Value, rule_set: &RuleSet) -> Result<Option<String>, RuleError> {
    // First resolve the selector through mappings
    let actual_selector = rule_set.resolve_selector(selector);
    
    // Then find the effective selector in JSON
    find_effective_selector(&actual_selector, json)
}

#[allow(dead_code)]
fn resolve_property_path_with_mapping<'a>(
    path: &crate::runner::model::PropertyPath,
    json: &'a Value,
    rule_set: &RuleSet
) -> Result<(Option<&'a Value>, String), RuleError> {
    let mut path_parts = vec![path.selector.clone()];

    // Find effective selector with mapping support
    let effective_selector = find_effective_selector_with_mapping(&path.selector, json, rule_set)?;
    if effective_selector.is_none() {
        return Ok((None, format!("$.{}", path.selector)));
    }

    let final_selector = effective_selector.unwrap();
    let mut current_value = json.get(&final_selector)
        .ok_or_else(|| RuleError::EvaluationError(format!("Selector '{}' not found", final_selector)))?;

    path_parts[0] = final_selector.clone(); // Use the actual key from JSON

    let is_length_of_operator = is_length_of_operation(path);
    let is_number_of_operator = is_number_of_operation(path);
    
    let properties_to_process = if is_length_of_operator {
        &path.properties[..path.properties.len() - 1]
    } else if is_number_of_operator {
        &path.properties[..path.properties.len() - 1]
    } else {
        &path.properties[..]
    };

    // Follow the property chain - properties are already in correct traversal order
    // For "__date of birth__ of **person** of **driving test**", we get properties: ["person", "date of birth"]
    // And we traverse: driving test -> person -> date of birth
    for property in properties_to_process.iter() {
        let mut found_property = None;
        let mut actual_property_name = property.clone();

        if let Some(prop_value) = current_value.get(property) {
            found_property = Some(prop_value);
        } else if let Some(prop_value) = get_json_value_insensitive(current_value, property) {
            found_property = Some(prop_value);
            // Find the actual property name in the JSON for path tracking
            if let Some(obj) = current_value.as_object() {
                for (key, _) in obj {
                    if names_match(property, key) {
                        actual_property_name = key.clone();
                        break;
                    }
                }
            }
        } else {
            let transformed_property = transform_property_name(property);
            if let Some(prop_value) = current_value.get(&transformed_property) {
                found_property = Some(prop_value);
                actual_property_name = transformed_property.clone();
            } else if let Some(prop_value) = get_json_value_insensitive(current_value, &transformed_property) {
                found_property = Some(prop_value);
                // Find the actual property name in the JSON for path tracking
                if let Some(obj) = current_value.as_object() {
                    for (key, _) in obj {
                        if names_match(&transformed_property, key) {
                            actual_property_name = key.clone();
                            break;
                        }
                    }
                }
            }
        }

        if let Some(value) = found_property {
            current_value = value;
            path_parts.push(actual_property_name);
        } else {
            let path_so_far = format!("$.{}", path_parts.join("."));
            return Ok((None, format!("{}.{}", path_so_far, property)));
        }
    }

    let final_path = format!("$.{}", path_parts.join("."));
    Ok((Some(current_value), final_path))
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

fn create_failed_comparison_trace_with_path(
    condition: &ComparisonCondition,
    path: &str
) -> ConditionTrace {
    ConditionTrace::Comparison(ComparisonTrace {
        selector: SelectorTrace {
            value: condition.selector.value.clone(),
            pos: condition.selector.pos.clone(),
        },
        property: PropertyTrace {
            value: Value::Null,
            path: path.to_string(),
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
    // Check if selector contains dots (nested path)
    if selector.contains('.') {
        return extract_value_from_nested_selector(json, selector, property);
    }

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
        let transformed_property = transform_selector_name(property);
        if let Some(val) = obj.get(&transformed_property) {
            val
        } else {
            return Err(RuleError::EvaluationError(
                format!("Property '{}' not found in selector '{}'", property, selector)
            ));
        }
    };

    convert_json_to_rule_value(value)
}

fn extract_value_from_nested_selector(
    json: &Value,
    nested_selector: &str,
    property: &str
) -> Result<RuleValue, RuleError> {
    // Split the nested selector by dots (e.g., "top.second" -> ["top", "second"])
    let path_parts: Vec<&str> = nested_selector.split('.').collect();
    
    // Navigate through the JSON using the path
    let mut current_value = json;
    
    for part in &path_parts {
        // Try to navigate to the next level
        if let Some(next_val) = current_value.get(part) {
            current_value = next_val;
        } else if let Some(next_val) = get_json_value_insensitive(current_value, part) {
            current_value = next_val;
        } else {
            let transformed_part = transform_selector_name(part);
            if let Some(next_val) = current_value.get(&transformed_part) {
                current_value = next_val;
            } else if let Some(next_val) = get_json_value_insensitive(current_value, &transformed_part) {
                current_value = next_val;
            } else {
                return Err(RuleError::EvaluationError(
                    format!("Path segment '{}' not found in nested selector '{}'", part, nested_selector)
                ));
            }
        }
    }

    // Now extract the property from the final object
    let value = if let Some(val) = current_value.get(property) {
        val
    } else if let Some(val) = get_json_value_insensitive(current_value, property) {
        val
    } else {
        let transformed_property = transform_selector_name(property);
        if let Some(val) = current_value.get(&transformed_property) {
            val
        } else if let Some(val) = get_json_value_insensitive(current_value, &transformed_property) {
            val
        } else {
            return Err(RuleError::EvaluationError(
                format!("Property '{}' not found in nested selector '{}'", property, nested_selector)
            ));
        }
    };

    convert_json_to_rule_value(value)
}

// Rest of the comparison functions remain exactly the same as before...
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
        ExactlyEqualTo => compare_exactly_equal(left, right),
        NotEqualTo => compare_not_equal(left, right),

        // Date comparisons
        LaterThan => compare_dates_later(left, right),
        EarlierThan => compare_dates_earlier(left, right),

        // List operations
        In => compare_in_list(left, right),
        NotIn => compare_not_in_list(left, right),
        Contains => compare_contains(left, right),
        
        // Empty checks (only use left operand, ignore right)
        IsEmpty => compare_is_empty(left),
        IsNotEmpty => compare_is_not_empty(left),
        
        // Duration comparison
        Within => compare_within(left, right),
    }
}

// Helper function to try to convert a string to a date
fn try_parse_date(value: &RuleValue) -> Option<NaiveDate> {
    if let RuleValue::String(s) = value {
        match s.len() {
            10 => {
                if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    Some(date)
                } else {
                    None
                }
            },
            18 => {
                if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
                    Some(date)
                } else {
                    None
                }
            }
            24 => {
                if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%dT%H:%M:%S.%fZ") {
                    Some(date)
                } else {
                    None
                }
            }
            _ => None
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

fn coerce_to_date(value: &RuleValue) -> Option<NaiveDate> {
    let date = match value {
        RuleValue::Date(d) => Some(*d),
        _ => try_parse_date(value),
    };
    
    match date {
        Some(d) => Some(d),
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

// Equality comparison functions (case-insensitive by default)
fn compare_equal(left: &RuleValue, right: &RuleValue) -> Result<bool, RuleError> {
    match (left, right) {
        (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l == r),
        (RuleValue::String(l), RuleValue::String(r)) => Ok(l.to_lowercase() == r.to_lowercase()),
        (RuleValue::Date(l), RuleValue::Date(r)) => Ok(l == r),
        (RuleValue::Boolean(l), RuleValue::Boolean(r)) => Ok(l == r),
        _ => Err(RuleError::TypeError(format!("Cannot compare {:?} and {:?} for equality", left, right))),
    }
}

// Case-sensitive equality comparison
fn compare_exactly_equal(left: &RuleValue, right: &RuleValue) -> Result<bool, RuleError> {
    match (left, right) {
        (RuleValue::Number(l), RuleValue::Number(r)) => Ok(l == r),
        (RuleValue::String(l), RuleValue::String(r)) => Ok(l == r),
        (RuleValue::Date(l), RuleValue::Date(r)) => Ok(l == r),
        (RuleValue::Boolean(l), RuleValue::Boolean(r)) => Ok(l == r),
        _ => Err(RuleError::TypeError(format!("Cannot compare {:?} and {:?} for exact equality", left, right))),
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
                RuleValue::String(r) => Ok(l.to_lowercase().contains(&r.to_lowercase())),
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

// Helper function to check equality without returning Result (case-insensitive for strings)
fn is_equal(left: &RuleValue, right: &RuleValue) -> bool {
    match (left, right) {
        (RuleValue::Number(l), RuleValue::Number(r)) => l == r,
        (RuleValue::String(l), RuleValue::String(r)) => l.to_lowercase() == r.to_lowercase(),
        (RuleValue::Date(l), RuleValue::Date(r)) => l == r,
        (RuleValue::Boolean(l), RuleValue::Boolean(r)) => l == r,
        _ => false,
    }
}

// Empty check functions
pub fn compare_is_empty(value: &RuleValue) -> Result<bool, RuleError> {
    match value {
        RuleValue::String(s) => Ok(s.is_empty()),
        RuleValue::List(items) => Ok(items.is_empty()),
        _ => Err(RuleError::TypeError("IsEmpty only works with strings or lists".to_string())),
    }
}

pub fn compare_is_not_empty(value: &RuleValue) -> Result<bool, RuleError> {
    compare_is_empty(value).map(|result| !result)
}

fn compare_within(left: &RuleValue, right: &RuleValue) -> Result<bool, RuleError> {
    match right {
        RuleValue::Duration(duration) => {
            let date_value = coerce_to_date(left)
                .ok_or_else(|| RuleError::TypeError(
                    format!("Within operator requires a date or convertible value, got {:?}", left)
                ))?;

            let now = chrono::Utc::now().naive_utc().date();
            let diff_days = (date_value - now).num_days().abs() as f64;
            let duration_days = duration.to_seconds() / 86400.0;
            Ok(diff_days <= duration_days)
        },
        _ => Err(RuleError::TypeError(
            "Within operator requires a duration as the right operand".to_string()
        )),
    }
}

fn transform_selector_name(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }
    
    if words.len() == 1 {
        return words[0].to_lowercase();
    }

    // Pre-allocate with reasonable capacity to avoid reallocations
    let estimated_size = name.len(); // Conservative estimate
    let mut result = String::with_capacity(estimated_size);
    result.push_str(&words[0].to_lowercase());
    
    for word in &words[1..] {
        if !word.is_empty() {
            // Capitalize first letter
            if let Some(first_char) = word.chars().next() {
                result.extend(first_char.to_uppercase());
                if word.len() > 1 {
                    result.push_str(&word[1..].to_lowercase());
                }
            }
        }
    }

    result
}

fn get_json_value_insensitive<'a>(json: &'a serde_json::Value, key: &str) -> Option<&'a serde_json::Value> {
    if let Some(obj) = json.as_object() {
        // First try exact match (most common case)
        if let Some(value) = obj.get(key) {
            return Some(value);
        }
        
        // Then try fuzzy matching (camelCase, snake_case, spaces)
        for (k, v) in obj {
            if names_match(key, k) {
                return Some(v);
            }
        }
    }
    None
}