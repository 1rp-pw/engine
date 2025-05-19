use serde::Serialize;
use crate::runner::model::{ComparisonOperator, RuleValue, SourcePosition};

#[derive(Debug, Serialize)]
pub struct RuleSetTrace {
    pub(crate) execution: Vec<RuleTrace>,
}



#[derive(Debug, Serialize)]
pub struct RuleTrace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub selector: String,
    pub selector_pos: Option<SourcePosition>,
    pub outcome: String,
    pub outcome_pos: Option<SourcePosition>,
    pub conditions: Vec<ConditionTrace>,
    pub result: bool,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ConditionTrace {
    Comparison(ComparisonTrace),
    RuleReference(RuleReferenceTrace),
}

#[derive(Debug, Serialize)]
pub struct ComparisonTrace {
    pub selector: String,
    pub selector_pos: SourcePosition,
    pub property: String,
    pub property_pos: SourcePosition,
    pub operator: ComparisonOperator,
    pub value: RuleValue,
    pub value_pos: SourcePosition,
    pub evaluation_details: Option<ComparisonEvaluationTrace>,
}

#[derive(Debug, Serialize)]
pub struct ComparisonEvaluationTrace {
    pub left_value: RuleValue,
    pub right_value: RuleValue,
    pub comparison_result: bool,
}

#[derive(Debug, Serialize)]
pub struct RuleReferenceTrace {
    pub selector: String,
    pub rule_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referenced_rule_outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_check: Option<PropertyCheckTrace>,
    pub result: bool,
}

#[derive(Debug, Serialize)]
pub struct PropertyCheckTrace {
    pub property_name: String,
    pub property_value: serde_json::Value,
}