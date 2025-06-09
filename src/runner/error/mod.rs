mod lib;

use thiserror::Error;
use crate::runner::trace::{RuleSetTrace, RuleTrace, ConditionTrace};

#[derive(Error, Debug)]
pub enum RuleError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Evaluation error: {0}")]
    EvaluationError(String),

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Enhanced evaluation result that includes traces even on failure
#[derive(Debug)]
pub struct EvaluationResult<T> {
    pub result: Result<T, RuleError>,
    pub trace: Option<RuleSetTrace>,
}

impl<T> EvaluationResult<T> {
    pub fn success(value: T, trace: RuleSetTrace) -> Self {
        Self {
            result: Ok(value),
            trace: Some(trace),
        }
    }
    
    pub fn failure(error: RuleError, trace: Option<RuleSetTrace>) -> Self {
        Self {
            result: Err(error),
            trace,
        }
    }
    
    pub fn is_success(&self) -> bool {
        self.result.is_ok()
    }
    
    pub fn is_failure(&self) -> bool {
        self.result.is_err()
    }
    
    pub fn unwrap(self) -> T {
        self.result.unwrap()
    }
    
    pub fn unwrap_trace(self) -> RuleSetTrace {
        self.trace.expect("Expected trace to be present")
    }
}

/// Partial rule trace for building traces during evaluation
#[derive(Debug, Clone)]
pub struct PartialRuleTrace {
    pub label: Option<String>,
    pub selector: String,
    pub selector_pos: Option<crate::runner::model::SourcePosition>,
    pub outcome: String,
    pub outcome_pos: Option<crate::runner::model::SourcePosition>,
    pub conditions: Vec<ConditionTrace>,
    pub result: Option<bool>,
    pub error: Option<String>,
}

impl PartialRuleTrace {
    pub fn new(
        label: Option<String>,
        selector: String,
        selector_pos: Option<crate::runner::model::SourcePosition>,
        outcome: String,
        outcome_pos: Option<crate::runner::model::SourcePosition>,
    ) -> Self {
        Self {
            label,
            selector,
            selector_pos,
            outcome,
            outcome_pos,
            conditions: Vec::new(),
            result: None,
            error: None,
        }
    }
    
    pub fn add_condition(&mut self, condition_trace: ConditionTrace) {
        self.conditions.push(condition_trace);
    }
    
    pub fn set_result(&mut self, result: bool) {
        self.result = Some(result);
    }
    
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.result = Some(false);
    }
    
    pub fn to_rule_trace(self) -> RuleTrace {
        RuleTrace {
            label: self.label,
            selector: crate::runner::trace::SelectorTrace {
                value: self.selector,
                pos: self.selector_pos,
            },
            outcome: crate::runner::trace::OutcomeTrace {
                value: self.outcome,
                pos: self.outcome_pos,
            },
            conditions: self.conditions,
            result: self.result.unwrap_or(false),
        }
    }
}
