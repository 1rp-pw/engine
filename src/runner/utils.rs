use crate::runner::error::RuleError;
use crate::runner::model::{Condition, Rule};

fn find_referenced_outcomes(rules: &[Rule]) -> std::collections::HashSet<String> {
    let mut referenced = std::collections::HashSet::new();
    for rule in rules {
        for cond in &rule.conditions {
            if let Condition::RuleReference { selector: _, rule_name } = cond {
                for other_rule in rules {
                    if other_rule.outcome.contains(rule_name) || rule_name.contains(&other_rule.outcome) {
                        referenced.insert(other_rule.outcome.clone());
                    }
                }
            }
        }
    }
    referenced
}

pub fn find_global_rule<'a>(rules: &'a [Rule]) -> Result<&'a Rule, RuleError> {
    if rules.len() == 1 {
        return Ok(&rules[0])
    }

    let referenced = find_referenced_outcomes(rules);
    let globals: Vec<&Rule> = rules
        .iter()
        .filter(|r| !referenced.contains(&r.outcome))
        .collect();

    match globals.len() {
        1 => Ok(globals[0]),
        0 => Err(RuleError::ParseError("No global rule found".to_string())),
        _ => Err(RuleError::ParseError("Multiple global rules found".to_string())),
    }
}

pub fn transform_property_name(name: &str) -> String {
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

pub fn infer_possible_properties(rule_name: &str) -> Vec<String> {
    let mut properties = Vec::new();

    // Remove common qualification phrases
    let qualification_phrases = [
        "passes the", "passes", "qualifies for the", "qualifies for", "qualifies",
        "meets the", "meets", "satisfies the", "satisfies", "is eligible for the",
        "is eligible for", "is eligible", "has passed the", "has passed",
        "has qualified for the", "has qualified for", "has qualified",
        "is approved for the", "is approved for", "is approved"
    ];

    let mut cleaned = rule_name.to_string();
    for phrase in &qualification_phrases {
        cleaned = cleaned.replace(phrase, "");
    }
    cleaned = cleaned.trim().to_string();

    // Generate camelCase version
    let base_property = transform_property_name(&cleaned);

    // Add various suffixes that might indicate status
    properties.push(base_property.clone());
    properties.push(format!("{}Passed", base_property));
    properties.push(format!("{}Qualified", base_property));
    properties.push(format!("{}Eligible", base_property));
    properties.push(format!("{}Approved", base_property));
    properties.push(format!("{}Status", base_property));

    // Also try with just the last word if it's a compound phrase
    if let Some(last_word) = cleaned.split_whitespace().last() {
        let last_word_property = transform_property_name(last_word);
        properties.push(last_word_property.clone());
        properties.push(format!("{}Passed", last_word_property));
        properties.push(format!("{}Qualified", last_word_property));
        properties.push(format!("{}Eligible", last_word_property));
        properties.push(format!("{}Approved", last_word_property));
        properties.push(format!("{}Status", last_word_property));
    }

    properties
}