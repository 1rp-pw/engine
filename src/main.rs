mod runner;

use std::collections::HashMap;
use runner::parser::parse_rules;
use runner::evaluator::evaluate_rule_set;
use runner::model::{Condition, RuleSet};
use runner::trace::RuleSetTrace;

use serde_json::Value;
use serde::{Deserialize, Serialize};
use axum::{
    extract::Json,
    http::StatusCode,
    routing::post,
    Router,
};

#[derive(Deserialize)]
struct RuleDataPackage {
    rule: String,
    data: Value,
}

#[derive(Serialize)]
struct EvaluationResponse {
    result: bool,
    #[serde(skip_serializing_if="Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    trace: Option<RuleSetTrace>,
    #[serde(skip_serializing_if="Option::is_none")]
    labels: Option<HashMap<String, bool>>,
    text: Vec<String>,
    data: Value,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/run", post(handle_evaluation));
    
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    let addr = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    println!("Listening on http://0.0.0.0:{}", port);
    axum::serve(addr, app).await.unwrap();
}

async fn handle_evaluation(Json(package): Json<RuleDataPackage>) -> (StatusCode, Json<EvaluationResponse>) {
    match parse_rules(&package.rule) {
        Ok(rule_set) => match evaluate_rule_set(&rule_set, &package.data) {
            Ok((results, trace)) => {
                print_rules(&rule_set);
                
                let mut labels = HashMap::new();
                for rule_trace in &trace.execution {
                    if let Some(label) = &rule_trace.label {
                        labels.insert(label.to_string(), rule_trace.result);
                    }
                }

                let result = results.values().next().cloned().unwrap_or(false);
                let text = package.rule.lines().map(String::from).collect();
                let response = EvaluationResponse {
                    result,
                    error: None,
                    trace: Some(trace),
                    labels: if labels.is_empty() { 
                        None
                    } else {
                        Some(labels)
                    },
                    text,
                    data: package.data.clone(),
                };
                (StatusCode::OK, Json(response))
            }
            Err(error) => {
                let response = EvaluationResponse {
                    result: false,
                    error: Some(error.to_string()),
                    trace: None,
                    labels: None,
                    text: vec![],
                    data: Default::default(),
                };
                (StatusCode::BAD_REQUEST, Json(response))
            }
        },
        Err(error) => {
            let response = EvaluationResponse {
                result: false,
                error: Some(error.to_string()),
                trace: None,
                labels: None,
                text: vec![],
                data: Default::default(),
            };
            (StatusCode::BAD_REQUEST, Json(response))
        }
    }
}

fn print_rules(rule_set: &RuleSet) {
    for (i, rule) in rule_set.rules.iter().enumerate() {
        if let Some(label) = &rule.label {
            println!("  {}. [{}] **{}** gets {}", i + 1, label, rule.selector, rule.outcome);
        } else {
            println!("  {}. **{}** gets {}", i + 1, rule.selector, rule.outcome);
        }

        // Print conditions for each rule
        for (j, condition) in rule.conditions.iter().enumerate() {
            match condition {
                Condition::Comparison { selector, property, operator, value } => {
                    println!("     Comparison Condition {}: the __{}__ of the **{}** {} {}",
                             j + 1, property, selector, operator, value);
                },
                Condition::RuleReference { selector, rule_name } => {
                    println!("     Rule Condition {}: the **{}** passes {}",
                             j + 1, selector, rule_name);
                },
            }
        }
    }
}
