// src/model.rs
use chrono::NaiveDate;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOperator {
    GreaterThanOrEqual,
    EqualTo,
    SameAs,
    LaterThan,
    GreaterThan,
    In,
}

impl fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparisonOperator::GreaterThanOrEqual => write!(f, "is greater than or equal to"),
            ComparisonOperator::EqualTo => write!(f, "is equal to"),
            ComparisonOperator::SameAs => write!(f, "is the same as"),
            ComparisonOperator::LaterThan => write!(f, "is later than"),
            ComparisonOperator::GreaterThan => write!(f, "is greater than"),
            ComparisonOperator::In => write!(f, "is in"),
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
    pub selector: String,
    pub outcome: String,
    pub conditions: Vec<Condition>,
}

impl Rule {
    pub fn new(selector: String, outcome: String) -> Self {
        Rule {
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
}
