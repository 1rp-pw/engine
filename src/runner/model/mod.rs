mod lib;

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
            ComparisonOperator::GreaterThanOrEqual => write!(f, "is at least"),
            
            ComparisonOperator::LessThanOrEqual => write!(f, "is less than or equal to"),
            ComparisonOperator::LessThanOrEqual => write!(f, "is now more than"),
            
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

#[derive(Debug, Clone)]
pub enum Condition {
    Comparison(ComparisonCondition),
    RuleReference(RuleReferenceCondition),
}

// Keep original structure but add support for property chains
#[derive(Debug, Clone)]
pub struct ComparisonCondition {
    pub selector: PositionedValue<String>,
    pub property: PositionedValue<String>,
    pub operator: ComparisonOperator,
    pub value: PositionedValue<RuleValue>,
    // Add optional property chain for complex access patterns
    pub property_chain: Option<Vec<PropertyChainElement>>,
    // Add support for cross-object comparisons
    pub left_property_path: Option<PropertyPath>,
    pub right_property_path: Option<PropertyPath>,
}

#[derive(Debug, Clone)]
pub struct PropertyPath {
    pub properties: Vec<String>,
    pub selector: String,
}

// Simple enum for property chain elements
#[derive(Debug, Clone)]
pub enum PropertyChainElement {
    Property(String),
    Selector(String),
}

#[derive(Debug, Clone)]
pub struct RuleReferenceCondition {
    pub selector: PositionedValue<String>,
    pub rule_name: PositionedValue<String>,
}

#[derive(Debug, Clone)]
pub struct PositionedValue<T> {
    pub value: T,
    pub pos: Option<SourcePosition>,
}

impl<T> PositionedValue<T> {
    pub fn new(value: T) -> Self {
        Self { value, pos: None }
    }

    pub fn with_position(value: T, pos: Option<SourcePosition>) -> Self {
        Self { value, pos }
    }
}

// For convenience, you might want to add From implementations:
impl From<String> for PositionedValue<String> {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<RuleValue> for PositionedValue<RuleValue> {
    fn from(value: RuleValue) -> Self {
        Self::new(value)
    }
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
    pub(crate) rule_map: HashMap<String, usize>,
    pub(crate) label_map: HashMap<String, usize>,
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
}

#[derive(Debug, Serialize, Clone)]
#[derive(PartialEq)]
pub struct SourcePosition {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}