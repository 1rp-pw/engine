use serde::Serialize;
use crate::runner::model::{ComparisonOperator, RuleValue};

#[derive(Debug, Serialize)]
pub struct RuleSetTrace {
    pub execution: Vec<RuleTrace>,
}

#[derive(Debug, Serialize)]
pub struct RuleTrace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub selector: String,
    pub outcome: String,
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
    pub property: String,
    pub operator: ComparisonOperator,
    pub value: RuleValue,
    pub evaluation_details: Option<ComparisonEvaluationTrace>,
    pub result: bool,
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
    pub result: bool,
}