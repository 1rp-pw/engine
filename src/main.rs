mod runner;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use flags_rs::{Auth, Client};
use runner::evaluator::evaluate_rule_set_with_trace;
use runner::parser::parse_rules;
use runner::trace::RuleSetTrace;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::env;

#[derive(Deserialize)]
struct RuleDataPackage {
    rule: String,
    data: Value,
}

#[derive(Serialize, Debug)]
struct EvaluationResponse {
    result: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace: Option<RuleSetTrace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<HashMap<String, bool>>,
    rule: Vec<String>,
    data: Value,
}

#[derive(Clone)]
struct AppState {
    #[allow(dead_code)]
    flags_client: Client,
}

#[tokio::main]
async fn main() {
    let flags_client = Client::builder()
        .with_memory_cache()
        .with_auth(Auth {
            environment_id: env::var("FF_ENV_ID").unwrap(),
            agent_id: env::var("FF_AGENT_ID").unwrap(),
            project_id: env::var("FF_PROJECT_ID").unwrap(),
        })
        .build();

    let state = AppState { flags_client };

    let app = Router::new()
        .route("/", post(handle_run))
        .route("/health", get(health_check))
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    let addr = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    println!("Listening on http://0.0.0.0:{}", port);
    axum::serve(addr, app).await.unwrap();
}

async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "healthy",
            "service": "policy-engine"
        })),
    )
}

async fn handle_run(
    State(_state): State<AppState>,
    Json(package): Json<RuleDataPackage>,
) -> (StatusCode, Json<EvaluationResponse>) {
    match parse_rules(&package.rule) {
        Ok(rule_set) => {
            let evaluation_result = evaluate_rule_set_with_trace(&rule_set, &package.data);

            // Extract labels from trace if available
            let mut labels = HashMap::new();
            if let Some(trace) = &evaluation_result.trace {
                for rule_trace in &trace.execution {
                    if let Some(label) = &rule_trace.label {
                        labels.insert(label.to_string(), rule_trace.result);
                    }
                }
            }

            let rule = package.rule.lines().map(String::from).collect();

            match evaluation_result.result {
                Ok(results) => {
                    // Find the global rule to get its outcome
                    let global_rule = match crate::runner::utils::find_global_rule(&rule_set.rules)
                    {
                        Ok(rule) => rule,
                        Err(_) => {
                            // If no global rule found, fall back to first result
                            let result = results.values().next().cloned().unwrap_or(false);
                            let response = EvaluationResponse {
                                result,
                                error: None,
                                trace: evaluation_result.trace,
                                labels: if labels.is_empty() {
                                    None
                                } else {
                                    Some(labels)
                                },
                                rule,
                                data: package.data.clone(),
                            };
                            return (StatusCode::OK, Json(response));
                        }
                    };

                    // Get the result for the global rule's outcome
                    let result = results.get(&global_rule.outcome).cloned().unwrap_or(false);
                    let response = EvaluationResponse {
                        result,
                        error: None,
                        trace: evaluation_result.trace,
                        labels: if labels.is_empty() {
                            None
                        } else {
                            Some(labels)
                        },
                        rule,
                        data: package.data.clone(),
                    };
                    (StatusCode::OK, Json(response))
                }
                Err(error) => {
                    // KEY IMPROVEMENT: Now we include trace even on errors!
                    // This allows API users to see where the error occurred in the evaluation process
                    // without having to look through logs.
                    let response = EvaluationResponse {
                        result: false,
                        error: Some(error.to_string()),
                        trace: evaluation_result.trace, // This preserves the evaluation trace even on failure!
                        labels: if labels.is_empty() {
                            None
                        } else {
                            Some(labels)
                        },
                        rule,
                        data: package.data.clone(),
                    };
                    (StatusCode::BAD_REQUEST, Json(response))
                }
            }
        }
        Err(parse_error) => {
            let rule = package.rule.lines().map(String::from).collect();

            // Even for parse errors, create a basic trace showing what we attempted to parse
            let parse_trace = create_parse_error_trace(&parse_error, &package.rule);

            let response = EvaluationResponse {
                result: false,
                error: Some(parse_error.to_string()),
                trace: Some(parse_trace), // Always include trace, even for parse errors!
                labels: None,
                rule,
                data: package.data.clone(),
            };
            (StatusCode::BAD_REQUEST, Json(response))
        }
    }
}

/// Creates a trace showing parse error information
fn create_parse_error_trace(
    parse_error: &runner::error::RuleError,
    rule_text: &str,
) -> RuleSetTrace {
    use runner::trace::*;

    // Extract line information from parse error if possible
    let error_line = extract_line_from_parse_error(&parse_error.to_string());
    let error_location = find_error_location(rule_text, error_line);

    // Create a synthetic rule trace showing where parsing failed
    let parse_trace = RuleTrace {
        label: Some("Parse Error".to_string()),
        selector: SelectorTrace {
            value: "parser".to_string(),
            pos: error_location.clone(),
        },
        outcome: OutcomeTrace {
            value: "parse_failed".to_string(),
            pos: error_location,
        },
        conditions: vec![ConditionTrace::Comparison(ComparisonTrace {
            selector: SelectorTrace {
                value: "rule_syntax".to_string(),
                pos: None,
            },
            property: PropertyTrace {
                value: serde_json::json!({
                    "error_type": "parse_error",
                    "failed_at_line": error_line,
                    "rule_length": rule_text.lines().count()
                }),
                path: format!("$.rule_syntax.line_{}", error_line.unwrap_or(0)),
            },
            operator: runner::model::ComparisonOperator::EqualTo,
            value: ValueTrace {
                value: serde_json::json!("invalid"),
                value_type: "string".to_string(),
                pos: None,
            },
            evaluation_details: Some(ComparisonEvaluationTrace {
                left_value: TypedValue {
                    value: serde_json::json!("invalid_syntax"),
                    value_type: "parse_error".to_string(),
                },
                right_value: TypedValue {
                    value: serde_json::json!("valid_syntax"),
                    value_type: "expectation".to_string(),
                },
                comparison_result: false,
            }),
            result: false,
        })],
        result: false,
    };

    RuleSetTrace {
        execution: vec![parse_trace],
    }
}

/// Extract line number from parse error message
fn extract_line_from_parse_error(error_msg: &str) -> Option<usize> {
    // Parse error messages typically contain line numbers like "--> 13:5"
    if let Some(arrow_pos) = error_msg.find("-->") {
        let after_arrow = &error_msg[arrow_pos + 3..];
        if let Some(colon_pos) = after_arrow.find(':') {
            let line_part = &after_arrow[..colon_pos].trim();
            line_part.parse::<usize>().ok()
        } else {
            None
        }
    } else {
        None
    }
}

/// Find the source position of the error in the rule text
fn find_error_location(
    rule_text: &str,
    error_line: Option<usize>,
) -> Option<runner::model::SourcePosition> {
    if let Some(line_num) = error_line {
        let lines: Vec<&str> = rule_text.lines().collect();
        if line_num > 0 && line_num <= lines.len() {
            let line_content = lines[line_num - 1]; // Convert to 0-based index
            Some(runner::model::SourcePosition {
                line: line_num,
                start: 0,
                end: line_content.len(),
            })
        } else {
            None
        }
    } else {
        None
    }
}
