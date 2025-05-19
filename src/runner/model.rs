use chrono::NaiveDate;
use std::collections::HashMap;
use std::fmt;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ComparisonOperator {
    GreaterThanOrEqual,
    LessThanOrEqual,
    EqualTo,
    NotEqualTo,
    SameAs,
    NotSameAs,
    LaterThan,
    EarlierThan,
    GreaterThan,
    LessThan,
    In,
    NotIn,
    Contains,
}

impl fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparisonOperator::GreaterThanOrEqual => write!(f, "is greater than or equal to"),
            ComparisonOperator::LessThanOrEqual => write!(f, "is less than or equal to"),
            ComparisonOperator::EqualTo => write!(f, "is equal to"),
            ComparisonOperator::NotEqualTo => write!(f, "is not equal to"),
            ComparisonOperator::SameAs => write!(f, "is the same as"),
            ComparisonOperator::NotSameAs => write!(f, "is not the same as"),
            ComparisonOperator::LaterThan => write!(f, "is later than"),
            ComparisonOperator::EarlierThan => write!(f, "is earlier than"),
            ComparisonOperator::GreaterThan => write!(f, "is greater than"),
            ComparisonOperator::LessThan => write!(f, "is less than"),
            ComparisonOperator::In => write!(f, "is in"),
            ComparisonOperator::NotIn => write!(f, "is not in"),
            ComparisonOperator::Contains => write!(f, "contains"),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum RuleValue {
    Number(f64),
    String(String),
    Date(NaiveDate),
    Boolean(bool),
    List(Vec<RuleValue>),
}

impl fmt::Display for RuleValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleValue::Number(n) => write!(f, "{}", n),
            RuleValue::String(s) => write!(f, "\"{}\"", s),
            RuleValue::Date(d) => write!(f, "date({})", d.format("%Y-%m-%d")),
            RuleValue::Boolean(b) => write!(f, "{}", b),
            RuleValue::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Condition {
    Comparison {
        selector: String,
        selector_pos: SourcePosition,
        property: String,
        property_pos: SourcePosition,
        operator: ComparisonOperator,
        value: RuleValue,
        value_pos: SourcePosition,
    },
    RuleReference {
        selector: String,
        rule_name: String,
    },
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub label: Option<String>,
    pub selector: String,
    pub selector_pos: Option<SourcePosition>,
    pub outcome: String,
    pub outcome_pos: Option<SourcePosition>,
    pub conditions: Vec<Condition>,
    pub position: Option<SourcePosition>,
}

impl Rule {
    pub fn new(label: Option<String>, selector: String, outcome: String) -> Self {
        Rule {
            label,
            selector,
            selector_pos: None,
            outcome,
            outcome_pos: None,
            conditions: Vec::new(),
            position: None,
        }
    }

    pub fn add_condition(&mut self, condition: Condition) {
        self.conditions.push(condition);
    }
}

#[derive(Debug, Default)]
pub struct RuleSet {
    pub rules: Vec<Rule>,
    rule_map: HashMap<String, usize>,
    label_map: HashMap<String, usize>
}

impl RuleSet {
    pub fn new() -> Self {
        RuleSet {
            rules: Vec::new(),
            rule_map: HashMap::new(),
            label_map: HashMap::new()
        }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        let index = self.rules.len();
        self.rule_map.insert(rule.outcome.clone(), index);
        if let Some(label) = &rule.label {
            self.label_map.insert(label.clone(), index);
        }
        self.rules.push(rule);
    }

    pub fn get_rule(&self, outcome: &str) -> Option<&Rule> {
        self.rule_map.get(outcome).map(|&index| &self.rules[index])
    }
    
    pub fn get_rule_by_label(&self, label: &str) -> Option<&Rule> {
        self.label_map.get(label).map(|&index| &self.rules[index])
    }

    // pub fn find_matching_rule(&self, selector: &str, description: &str) -> Option<&Rule> {
    //     // First try exact match on outcome
    //     if let Some(rule) = self.get_rule(description) {
    //         println!("  Found exact match: {}", rule.outcome);
    //         return Some(rule);
    //     }
    // 
    //     // Try to find a rule where the selector matches and there's semantic similarity
    //     let mut best_match = None;
    //     let mut best_score = 0;
    // 
    //     for rule in &self.rules {
    //         if rule.selector == selector {
    //             // Calculate a similarity score
    //             let score = self.calculate_similarity(&rule.outcome, description);
    //             println!("  Comparing with rule '{}', score: {}", rule.outcome, score);
    //             if score > best_score {
    //                 best_score = score;
    //                 best_match = Some(rule);
    //             }
    //         }
    //     }
    // 
    //     // If we found a decent match, return it
    //     if best_score > 0 {
    //         let best_rule = best_match.unwrap();
    //         let common_words = ["the", "a", "an", "and", "or", "of", "for", "to", "in", "on", "at", "by", "with"];
    // 
    //         // Split and filter in one pass, collecting owned strings
    //         let desc_words: Vec<String> = description.to_lowercase().split_whitespace()
    //             .filter(|w| !common_words.contains(w))
    //             .map(String::from)
    //             .collect();
    //         let rule_words: Vec<String> = best_rule.outcome.to_lowercase().split_whitespace()
    //             .filter(|w| !common_words.contains(w))
    //             .map(String::from)
    //             .collect();
    // 
    //         // Check if there's at least one significant word in common
    //         let mut has_significant_match = false;
    //         for word in &desc_words {
    //             if rule_words.contains(word) {
    //                 has_significant_match = true;
    //                 break;
    //             }
    //         }
    // 
    //         if has_significant_match {
    //             println!("  Found best match: {} with score {}", best_rule.outcome, best_score);
    //             return Some(best_rule);
    //         }
    //     }
    //     println!("  No match found");
    //     None
    // }
    pub fn find_matching_rule(&self, selector: &str, description: &str) -> Option<&Rule> {
        //println!("Finding rule for selector: '{}', description: '{}'", selector, description);

        // First try exact match on outcome
        if let Some(rule) = self.get_rule(description) {
            //println!("  Found exact match: {}", rule.outcome);
            return Some(rule);
        }

        // For rule references, require an exact substring match
        // This is more strict than similarity scoring
        for rule in &self.rules {
            if rule.selector == selector {
                // Check if the rule's outcome contains the description as a substring
                // or vice versa
                if rule.outcome.contains(description) || description.contains(&rule.outcome) {
                    //println!("  Found substring match: {}", rule.outcome);
                    return Some(rule);
                }
            }
        }

        // If we get here, no match was found
        //println!("  No match found for '{}'", description);
        None
    }

    // Calculate a simple similarity score between two strings
    fn calculate_similarity(&self, s1: &str, s2: &str) -> usize {
        // Convert to lowercase for comparison
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();

        // Split into words
        let words1: Vec<&str> = s1_lower.split_whitespace().collect();
        let words2: Vec<&str> = s2_lower.split_whitespace().collect();

        // Common words to ignore
        let common_words = ["the", "a", "an", "and", "or", "of", "for", "to", "in", "on", "at", "by", "with"];

        // Count matching significant words
        let mut score = 0;
        for word1 in &words1 {
            if !common_words.contains(word1) && words2.contains(word1) {
                score += 1;
            }
        }

        // If the score is too low relative to the number of significant words, return 0
        let significant_words1 = words1.iter().filter(|w| !common_words.contains(w)).count();
        let significant_words2 = words2.iter().filter(|w| !common_words.contains(w)).count();
        let min_significant = significant_words1.min(significant_words2);

        // Require at least 50% match of significant words
        if min_significant > 0 && score * 2 < min_significant {
            return 0;
        }

        score
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct SourcePosition {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}
