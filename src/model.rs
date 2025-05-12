// src/model.rs
use chrono::NaiveDate;
use std::collections::HashMap;
use std::fmt;

// src/model.rs (partial update)
#[derive(Debug, Clone, PartialEq)]
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


#[derive(Debug, Clone, PartialEq)]
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
        property: String,
        operator: ComparisonOperator,
        value: RuleValue,
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
    pub outcome: String,
    pub conditions: Vec<Condition>,
}

impl Rule {
    pub fn new(label: Option<String>, selector: String, outcome: String) -> Self {
        Rule {
            label,
            selector,
            outcome,
            conditions: Vec::new(),
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
}

impl RuleSet {
    pub fn new() -> Self {
        RuleSet {
            rules: Vec::new(),
            rule_map: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        let index = self.rules.len();
        self.rule_map.insert(rule.outcome.clone(), index);
        self.rules.push(rule);
    }

    pub fn get_rule(&self, outcome: &str) -> Option<&Rule> {
        self.rule_map.get(outcome).map(|&index| &self.rules[index])
    }

    pub fn find_rule_by_description(&self, description: &str) -> Option<&Rule> {
        for rule in &self.rules {
            let rule_desc = format!("passes {}", rule.outcome);
            if rule_desc == description {
                return Some(rule);
            }
        }
        None
    }

    pub fn find_matching_rule(&self, selector: &str, description: &str) -> Option<&Rule> {
        // First try exact match on outcome
        if let Some(rule) = self.get_rule(description) {
            return Some(rule);
        }

        // Try to find a rule where the selector matches and there's semantic similarity
        let mut best_match = None;
        let mut best_score = 0;

        for rule in &self.rules {
            if rule.selector == selector {
                // Calculate a similarity score
                let score = self.calculate_similarity(&rule.outcome, description);
                if score > best_score {
                    best_score = score;
                    best_match = Some(rule);
                }
            }
        }

        // If we found a decent match, return it
        if best_score > 0 {
            return best_match;
        }

        // If no match found, look for any rule with just the selector
        for rule in &self.rules {
            if rule.selector == selector {
                return Some(rule);
            }
        }

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

        // Count matching words
        let mut score = 0;
        for word1 in &words1 {
            if words2.contains(word1) {
                score += 1;
            }
        }

        score
    }
}
