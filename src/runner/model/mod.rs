mod lib;

use chrono::NaiveDate;
use std::collections::HashMap;
use std::fmt;
use serde::Serialize;
use std::borrow::Cow;
use std::sync::RwLock;

// String constants to avoid allocations
pub mod constants {
    pub const LENGTH_OF_MARKER: &str = "__length_of__";
    pub const NUMBER_OF_MARKER: &str = "__number_of__";
    pub const EMPTY_STRING: &str = "";
}

// Caching system for performance optimization
#[derive(Debug, Default)]
pub struct PerformanceCache {
    // Cache for rule fuzzy matching to avoid O(n) searches
    pub rule_fuzzy_matches: RwLock<HashMap<String, Option<String>>>,

    // Cache for JSON property lookups to avoid repeated case-insensitive searches
    #[allow(dead_code)]
    pub json_property_lookups: RwLock<HashMap<(String, String), Option<String>>>,
    
    // Cache for selector transformations (e.g., "user profile" -> "userProfile")
    #[allow(dead_code)]
    pub selector_transformations: RwLock<HashMap<String, String>>,
    
    // Cache for effective selector resolutions
    #[allow(dead_code)]
    pub effective_selectors: RwLock<HashMap<String, Option<String>>>,
    
    // Cache for property name transformations
    #[allow(dead_code)]
    pub property_transformations: RwLock<HashMap<String, String>>,
}

impl PerformanceCache {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn clear(&self) {
        if let Ok(mut cache) = self.rule_fuzzy_matches.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.json_property_lookups.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.selector_transformations.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.effective_selectors.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.property_transformations.write() {
            cache.clear();
        }
    }
    
    // Get cache statistics for monitoring
    #[allow(dead_code)]
    pub fn get_stats(&self) -> CacheStats {
        let rule_count = self.rule_fuzzy_matches.read()
            .map(|cache| cache.len())
            .unwrap_or(0);
        let json_count = self.json_property_lookups.read()
            .map(|cache| cache.len())
            .unwrap_or(0);
        let selector_count = self.selector_transformations.read()
            .map(|cache| cache.len())
            .unwrap_or(0);
        let effective_count = self.effective_selectors.read()
            .map(|cache| cache.len())
            .unwrap_or(0);
        let property_count = self.property_transformations.read()
            .map(|cache| cache.len())
            .unwrap_or(0);
            
        CacheStats {
            rule_fuzzy_matches: rule_count,
            json_property_lookups: json_count,
            selector_transformations: selector_count,
            effective_selectors: effective_count,
            property_transformations: property_count,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    #[allow(dead_code)]
    pub rule_fuzzy_matches: usize,
    #[allow(dead_code)]
    pub json_property_lookups: usize,
    #[allow(dead_code)]
    pub selector_transformations: usize,
    #[allow(dead_code)]
    pub effective_selectors: usize,
    #[allow(dead_code)]
    pub property_transformations: usize,
}

/// Efficient string type that avoids allocations for common strings
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EfficientString(Cow<'static, str>);

impl EfficientString {
    pub fn from_static(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }
    
    pub fn from_string(s: String) -> Self {
        Self(Cow::Owned(s))
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[allow(dead_code)]
    pub fn into_string(self) -> String {
        self.0.into_owned()
    }
}

impl fmt::Display for EfficientString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&'static str> for EfficientString {
    fn from(s: &'static str) -> Self {
        Self::from_static(s)
    }
}

impl From<String> for EfficientString {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ComparisonOperator {
    GreaterThanOrEqual,
    LessThanOrEqual,
    EqualTo,
    ExactlyEqualTo,
    NotEqualTo,
    LaterThan,
    EarlierThan,
    GreaterThan,
    LessThan,
    In,
    NotIn,
    Contains,
    IsEmpty,
    IsNotEmpty,
    Within,
}

impl fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparisonOperator::GreaterThanOrEqual => write!(f, "is greater than or equal to"),

            ComparisonOperator::LessThanOrEqual => write!(f, "is less than or equal to"),

            ComparisonOperator::EqualTo => write!(f, "is equal to"),
            ComparisonOperator::ExactlyEqualTo => write!(f, "is exactly equal to"),
            ComparisonOperator::NotEqualTo => write!(f, "is not equal to"),

            ComparisonOperator::LaterThan => write!(f, "is later than"),
            ComparisonOperator::EarlierThan => write!(f, "is earlier than"),

            ComparisonOperator::GreaterThan => write!(f, "is greater than"),
            ComparisonOperator::LessThan => write!(f, "is less than"),

            ComparisonOperator::In => write!(f, "is in"),
            ComparisonOperator::NotIn => write!(f, "is not in"),
            ComparisonOperator::Contains => write!(f, "contains"),
            ComparisonOperator::IsEmpty => write!(f, "is empty"),
            ComparisonOperator::IsNotEmpty => write!(f, "is not empty"),
            ComparisonOperator::Within => write!(f, "is within"),
        }
    }
}

impl ComparisonOperator {
    #[allow(dead_code)]
    pub fn all_representations(&self) -> Vec<&'static str> {
        match self {
            ComparisonOperator::GreaterThanOrEqual => vec![
                "is greater than or equal to",
                "is at least"
            ],
            ComparisonOperator::LessThanOrEqual => vec![
                "is less than or equal to",
                "is no more than"
            ],
            ComparisonOperator::EqualTo => vec![
                "is equal to",
                "is the same as"
            ],
            ComparisonOperator::ExactlyEqualTo => vec![
                "is exactly equal to"
            ],
            ComparisonOperator::NotEqualTo => vec![
                "is not equal to",
                "is not the same as"
            ],
            ComparisonOperator::LaterThan => vec!["is later than"],
            ComparisonOperator::EarlierThan => vec!["is earlier than"],
            ComparisonOperator::GreaterThan => vec!["is greater than"],
            ComparisonOperator::LessThan => vec!["is less than"],
            ComparisonOperator::In => vec!["is in"],
            ComparisonOperator::NotIn => vec!["is not in"],
            ComparisonOperator::Contains => vec!["contains"],
            ComparisonOperator::IsEmpty => vec!["is empty"],
            ComparisonOperator::IsNotEmpty => vec!["is not empty"],
            ComparisonOperator::Within => vec!["is within"],
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
    Duration(Duration),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Duration {
    pub amount: f64,
    pub unit: TimeUnit,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
    Years,
    Decades,
    Centuries,
}

impl Duration {
    pub fn new(amount: f64, unit: TimeUnit) -> Self {
        Self { amount, unit }
    }
    
    /// Convert to seconds for comparison purposes
    pub fn to_seconds(&self) -> f64 {
        match self.unit {
            TimeUnit::Seconds => self.amount,
            TimeUnit::Minutes => self.amount * 60.0,
            TimeUnit::Hours => self.amount * 3600.0,
            TimeUnit::Days => self.amount * 86400.0,
            TimeUnit::Weeks => self.amount * 604800.0,
            TimeUnit::Months => self.amount * 2629746.0, // Average month in seconds
            TimeUnit::Years => self.amount * 31556952.0, // Average year in seconds
            TimeUnit::Decades => self.amount * 315569520.0,
            TimeUnit::Centuries => self.amount * 3155695200.0,
        }
    }
    
    /// Auto-reduce to appropriate unit
    pub fn normalize(self) -> Self {
        let seconds = self.to_seconds();
        
        // If less than a day, reduce to seconds for precision
        if seconds < 86400.0 {
            return Duration::new(seconds, TimeUnit::Seconds);
        }
        
        // Otherwise, reduce to days
        Duration::new(seconds / 86400.0, TimeUnit::Days)
    }
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
            RuleValue::Duration(d) => write!(f, "{}", d),
        }
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unit_str = match self.unit {
            TimeUnit::Seconds => if self.amount == 1.0 { "second" } else { "seconds" },
            TimeUnit::Minutes => if self.amount == 1.0 { "minute" } else { "minutes" },
            TimeUnit::Hours => if self.amount == 1.0 { "hour" } else { "hours" },
            TimeUnit::Days => if self.amount == 1.0 { "day" } else { "days" },
            TimeUnit::Weeks => if self.amount == 1.0 { "week" } else { "weeks" },
            TimeUnit::Months => if self.amount == 1.0 { "month" } else { "months" },
            TimeUnit::Years => if self.amount == 1.0 { "year" } else { "years" },
            TimeUnit::Decades => if self.amount == 1.0 { "decade" } else { "decades" },
            TimeUnit::Centuries => if self.amount == 1.0 { "century" } else { "centuries" },
        };
        
        if self.amount.fract() == 0.0 {
            write!(f, "{} {}", self.amount as i64, unit_str)
        } else {
            write!(f, "{} {}", self.amount, unit_str)
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
    #[allow(dead_code)]
    Property(String),
    #[allow(dead_code)]
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

// Specialized constructors for common string values to avoid allocations
impl PositionedValue<String> {
    pub fn from_static(value: &'static str) -> Self {
        Self { value: value.to_string(), pos: None }
    }

    #[allow(dead_code)]
    pub fn from_static_with_pos(value: &'static str, pos: Option<SourcePosition>) -> Self {
        Self { value: value.to_string(), pos }
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
    pub cache: PerformanceCache,
    // Maps custom object selectors to actual JSON paths
    // e.g., "driver" -> "person"
    #[allow(dead_code)]
    pub selector_mappings: HashMap<String, String>,
}

impl RuleSet {
    #[allow(dead_code)]
    pub fn new() -> Self {
        RuleSet {
            rules: Vec::new(),
            rule_map: HashMap::new(),
            label_map: HashMap::new(),
            cache: PerformanceCache::new(),
            selector_mappings: HashMap::new(),
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        RuleSet {
            rules: Vec::with_capacity(capacity),
            rule_map: HashMap::with_capacity(capacity),
            label_map: HashMap::with_capacity(capacity),
            cache: PerformanceCache::new(),
            selector_mappings: HashMap::new(),
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

    #[allow(dead_code)]
    pub fn add_rules(&mut self, rules: Vec<Rule>) {
        // Reserve capacity to avoid reallocations
        let new_capacity = self.rules.len() + rules.len();
        self.rules.reserve(new_capacity);
        self.rule_map.reserve(rules.len());
        self.label_map.reserve(rules.len());
        
        for rule in rules {
            self.add_rule(rule);
        }
    }

    pub fn get_rule(&self, outcome: &str) -> Option<&Rule> {
        self.rule_map.get(outcome).map(|&index| &self.rules[index])
    }

    pub fn get_rule_by_label(&self, label: &str) -> Option<&Rule> {
        self.label_map.get(label).map(|&index| &self.rules[index])
    }

    /// Add a mapping from a custom selector to an actual JSON path
    /// e.g., map_selector("driver", "person") allows **driver** to reference the "person" object
    #[allow(dead_code)]
    pub fn map_selector(&mut self, custom_selector: &str, actual_path: &str) {
        self.selector_mappings.insert(custom_selector.to_string(), actual_path.to_string());
    }

    /// Get the actual JSON path for a selector, applying mappings if they exist
    #[allow(dead_code)]
    pub fn resolve_selector(&self, selector: &str) -> String {
        self.selector_mappings.get(selector).cloned().unwrap_or_else(|| selector.to_string())
    }
}

#[derive(Debug, Serialize, Clone)]
#[derive(PartialEq)]
pub struct SourcePosition {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}