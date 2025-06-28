mod lib;

use crate::runner::model::{ComparisonOperator, RuleValue, SourcePosition};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct RuleSetTrace {
    pub(crate) execution: Vec<RuleTrace>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RuleTrace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub selector: SelectorTrace,
    pub outcome: OutcomeTrace,
    pub conditions: Vec<ConditionTrace>,
    pub result: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum ConditionTrace {
    Comparison(ComparisonTrace),
    RuleReference(RuleReferenceTrace),
}

#[derive(Debug, Serialize, Clone)]
pub struct ComparisonTrace {
    pub selector: SelectorTrace,
    pub property: PropertyTrace,
    pub operator: ComparisonOperator,
    pub value: ValueTrace,
    pub evaluation_details: Option<ComparisonEvaluationTrace>,
    pub result: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct SelectorTrace {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos: Option<SourcePosition>,
}

#[derive(Debug, Serialize, Clone)]
pub struct OutcomeTrace {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos: Option<SourcePosition>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PropertyTrace {
    pub value: serde_json::Value,
    pub path: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ValueTrace {
    pub value: serde_json::Value,
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos: Option<SourcePosition>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ComparisonEvaluationTrace {
    pub left_value: TypedValue,
    pub right_value: TypedValue,
    pub comparison_result: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct TypedValue {
    pub value: serde_json::Value,
    #[serde(rename = "type")]
    pub value_type: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct RuleReferenceTrace {
    pub selector: SelectorTrace,
    pub rule_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referenced_rule_outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_check: Option<PropertyCheckTrace>,
    pub result: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct PropertyCheckTrace {
    pub property_name: String,
    pub property_value: serde_json::Value,
}

// Helper functions to convert RuleValue to TypedValue and ValueTrace
impl From<&RuleValue> for TypedValue {
    fn from(rule_value: &RuleValue) -> Self {
        match rule_value {
            RuleValue::Number(n) => TypedValue {
                value: serde_json::json!(n),
                value_type: "number".to_string(),
            },
            RuleValue::String(s) => TypedValue {
                value: serde_json::json!(s),
                value_type: "string".to_string(),
            },
            RuleValue::Date(d) => TypedValue {
                value: serde_json::json!(d.format("%Y-%m-%d").to_string()),
                value_type: "date".to_string(),
            },
            RuleValue::Boolean(b) => TypedValue {
                value: serde_json::json!(b),
                value_type: "boolean".to_string(),
            },
            RuleValue::Duration(d) => TypedValue {
                value: serde_json::json!(d.to_string()),
                value_type: "duration".to_string(),
            },
            RuleValue::List(items) => {
                let json_items: Vec<serde_json::Value> = items
                    .iter()
                    .map(|item| match item {
                        RuleValue::Number(n) => serde_json::json!(n),
                        RuleValue::String(s) => serde_json::json!(s),
                        RuleValue::Date(d) => serde_json::json!(d.format("%Y-%m-%d").to_string()),
                        RuleValue::Boolean(b) => serde_json::json!(b),
                        RuleValue::Duration(d) => serde_json::json!(d.to_string()),
                        RuleValue::List(_) => serde_json::json!(null), // nested lists not shown in example
                    })
                    .collect();
                TypedValue {
                    value: serde_json::json!(json_items),
                    value_type: "list".to_string(),
                }
            }
        }
    }
}

impl RuleValue {
    pub fn to_value_trace(&self, pos: Option<SourcePosition>) -> ValueTrace {
        let typed_value = TypedValue::from(self);
        ValueTrace {
            value: typed_value.value,
            value_type: typed_value.value_type,
            pos,
        }
    }
}