use crate::runner::error::RuleError;
use crate::runner::model::{Condition, Rule};

#[allow(dead_code)]
pub fn find_referenced_outcomes(rules: &[Rule]) -> std::collections::HashSet<String> {
    let mut referenced = std::collections::HashSet::new();

    for rule in rules {
        for condition_group in &rule.conditions {
            match &condition_group.condition {
                Condition::RuleReference(ref_condition) => {
                    let rule_name = &ref_condition.rule_name.value;

                    // Find all rules that this reference might match
                    for other_rule in rules {
                        // Check if this rule matches by label
                        let label_match = other_rule
                            .label
                            .as_ref()
                            .map_or(false, |label| label == rule_name);

                        // Check if this rule matches by exact outcome
                        let outcome_match = other_rule.outcome == *rule_name;

                        // Check if this rule matches by partial outcome (case insensitive)
                        // Improve matching logic to be more precise
                        let rule_name_lower = rule_name.to_lowercase();
                        let outcome_lower = other_rule.outcome.to_lowercase();

                        // More conservative partial matching:
                        // Focus on significant words and avoid common stop words
                        let partial_match = if rule_name_lower.len() >= 3
                            && outcome_lower.len() >= 3
                        {
                            // Common stop words that shouldn't be used for matching
                            let stop_words: std::collections::HashSet<&str> = [
                                "the",
                                "a",
                                "an",
                                "is",
                                "are",
                                "was",
                                "were",
                                "has",
                                "have",
                                "had",
                                "gets",
                                "passes",
                                "of",
                                "meets",
                                "qualifies",
                                "for",
                                "satisfies",
                                "achieves",
                                "completes",
                                "fulfills",
                                "obtains",
                                "receives",
                            ]
                            .iter()
                            .cloned()
                            .collect();

                            let reference_words: std::collections::HashSet<&str> = rule_name_lower
                                .split_whitespace()
                                .filter(|word| !stop_words.contains(word) && word.len() > 2)
                                .collect();
                            let outcome_words: std::collections::HashSet<&str> = outcome_lower
                                .split_whitespace()
                                .filter(|word| !stop_words.contains(word) && word.len() > 2)
                                .collect();

                            // Need at least one significant word to match
                            if reference_words.is_empty() || outcome_words.is_empty() {
                                false
                            } else {
                                let matching_words =
                                    reference_words.intersection(&outcome_words).count();
                                let reference_word_count = reference_words.len();

                                // For single significant word, require exact match
                                if reference_word_count == 1 {
                                    matching_words == 1
                                } else {
                                    // For multi-word, require ALL significant words to match for precise matching
                                    // This prevents "theory test" from matching "practical test"
                                    matching_words == reference_word_count
                                }
                            }
                        } else {
                            // For very short strings, require exact match
                            false
                        };

                        if label_match || outcome_match || partial_match {
                            referenced.insert(other_rule.outcome.clone());
                        }
                    }
                }
                Condition::Comparison(_) => {
                    // Comparison conditions don't reference other rules
                }
            }
        }
    }

    referenced
}

#[allow(dead_code)]
pub fn find_global_rule(rules: &[Rule]) -> Result<&Rule, RuleError> {
    if rules.len() == 1 {
        return Ok(&rules[0]);
    }

    let referenced = find_referenced_outcomes(rules);
    let globals: Vec<&Rule> = rules
        .iter()
        .filter(|r| !referenced.contains(&r.outcome))
        .collect();

    match globals.len() {
        1 => Ok(globals[0]),
        0 => Err(RuleError::ParseError("No global rule found".to_string())),
        _ => {
            let rule_list: Vec<String> = globals
                .iter()
                .map(|r| {
                    if let Some(label) = &r.label {
                        format!("'{}' ({})", label, r.outcome)
                    } else {
                        format!("'{}'", r.outcome)
                    }
                })
                .collect();
            Err(RuleError::ParseError(format!(
                "Multiple global rules found: {}. There should be only one golden rule that is not referenced by other rules.",
                rule_list.join(", ")
            )))
        }
    }
}

pub fn transform_property_name(name: &str) -> String {
    let words: Vec<&str> = name
        .split(&[' ', '_'][..])
        .filter(|s| !s.is_empty())
        .collect();
    if words.is_empty() {
        return String::new();
    }

    if words.len() == 1 {
        return words[0].to_string();
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
        "passes the",
        "passes",
        "qualifies for the",
        "qualifies for",
        "qualifies",
        "meets the",
        "meets",
        "satisfies the",
        "satisfies",
        "is eligible for the",
        "is eligible for",
        "is eligible",
        "has passed the",
        "has passed",
        "has qualified for the",
        "has qualified for",
        "has qualified",
        "is approved for the",
        "is approved for",
        "is approved",
    ];

    let mut cleaned = rule_name.to_string();
    for phrase in &qualification_phrases {
        cleaned = cleaned.replace(phrase, "");
    }
    cleaned = cleaned.trim().to_string();

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

#[allow(dead_code)]
pub fn transform_selector_name(name: &str) -> String {
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

/// Normalize a name by converting it to multiple possible formats
pub fn normalize_name(name: &str) -> Vec<String> {
    let mut variants = Vec::new();

    // Original name
    variants.push(name.to_string());

    // Convert spaces and underscores to camelCase
    let camel_case = transform_property_name(name);
    if !variants.contains(&camel_case) {
        variants.push(camel_case);
    }

    // Convert to snake_case
    let words: Vec<&str> = name
        .split(&[' ', '_'][..])
        .filter(|s| !s.is_empty())
        .collect();
    if words.len() > 1 {
        let snake_case = words.join("_").to_lowercase();
        if !variants.contains(&snake_case) {
            variants.push(snake_case);
        }
    }

    // Convert to space-separated
    if words.len() > 1 {
        let space_separated = words.join(" ").to_lowercase();
        if !variants.contains(&space_separated) {
            variants.push(space_separated);
        }
    }

    variants
}

/// Check if two names match using fuzzy matching (camelCase, snake_case, spaces)
pub fn names_match(name1: &str, name2: &str) -> bool {
    if name1 == name2 {
        return true;
    }

    let variants1 = normalize_name(name1);
    let variants2 = normalize_name(name2);

    for v1 in &variants1 {
        for v2 in &variants2 {
            if v1.eq_ignore_ascii_case(v2) {
                return true;
            }
        }
    }

    false
}
