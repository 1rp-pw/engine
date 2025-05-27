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
pub enum ConditionOperator {
    And,
    Or,
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

// #[derive(Debug, Clone)]
// pub struct Source {
//     pub value: String,
//     pub pos: SourcePosition,
// }
//
// #[derive(Debug, Clone)]
// pub struct RuleSource {
//     pub value: RuleValue,
//     pub pos: SourcePosition,
// }

#[derive(Debug, Clone)]
pub enum Condition {
    Comparison {
        selector: String,
        selector_pos: Option<SourcePosition>,
        property: String,
        property_pos: Option<SourcePosition>,
        operator: ComparisonOperator,
        value: RuleValue,
        value_pos: Option<SourcePosition>,
    },
    RuleReference {
        selector: String,
        rule_name: String,
    },
}

#[derive(Debug, Clone)]
pub struct ConditionGroup {
    pub condition: Condition,
    pub operator: Option<ConditionOperator>, // None for the first condition, Some for subsequent ones
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub label: Option<String>,
    pub selector: String,
    pub selector_pos: Option<SourcePosition>,
    pub outcome: String,
    pub conditions: Vec<ConditionGroup>, // Changed from Vec<Condition>
    pub position: Option<SourcePosition>,
}

impl Rule {
    pub fn new(label: Option<String>, selector: String, outcome: String) -> Self {
        Rule {
            label,
            selector,
            selector_pos: None,
            outcome,
            conditions: Vec::new(),
            position: None,
        }
    }

    pub fn add_condition(&mut self, condition: Condition, operator: Option<ConditionOperator>) {
        self.conditions.push(ConditionGroup {
            condition,
            operator,
        });
    }
}

#[derive(Debug, Default)]
pub struct RuleSet {
    pub rules: Vec<Rule>,
    rule_map: HashMap<String, usize>,
    label_map: HashMap<String, usize>,
}

impl RuleSet {
    pub fn new() -> Self {
        RuleSet {
            rules: Vec::new(),
            rule_map: HashMap::new(),
            label_map: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        let index = self.rules.len();
        if let Some(label) = &rule.label {
            self.label_map.insert(label.clone(), index);
        }

        self.rule_map.insert(rule.outcome.clone(), index);
        self.rules.push(rule);
    }

    pub fn get_rule(&self, outcome: &str) -> Option<&Rule> {
        self.rule_map.get(outcome).map(|&index| &self.rules[index])
    }

    pub fn get_rule_by_label(&self, label: &str) -> Option<&Rule> {
        self.label_map.get(label).map(|&index| &self.rules[index])
    }

//     pub fn find_matching_rule(&self, selector: &str, description: &str) -> Option<&Rule> {
//         // First try exact outcome match
//         if let Some(rule) = self.get_rule(description) {
//             return Some(rule);
//         }
//
//         // Then try exact label match
//         if let Some(rule) = self.get_rule_by_label(description) {
//             return Some(rule);
//         }
//
//         // Finally try partial matching
//         for rule in &self.rules {
//             if rule.selector == selector {
//                 if rule.outcome.contains(description) || description.contains(&rule.outcome) {
//                     return Some(rule);
//                 }
//             }
//         }
//
//         None
//     }
}

#[derive(Debug, Serialize, Clone)]
pub struct SourcePosition {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}