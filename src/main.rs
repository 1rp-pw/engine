mod runner;

use std::collections::HashMap;
use std::env;
use runner::parser::parse_rules;
use runner::evaluator::evaluate_rule_set;
use runner::trace::RuleSetTrace;
use flags_rs::{Auth, Client};
use serde_json::Value;
use serde::{Deserialize, Serialize};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::post,
    Router,
};

#[derive(Deserialize)]
struct RuleDataPackage {
    rule: String,
    data: Value,
}

#[derive(Serialize, Debug)]
struct EvaluationResponse {
    result: bool,
    #[serde(skip_serializing_if="Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    trace: Option<RuleSetTrace>,
    #[serde(skip_serializing_if="Option::is_none")]
    labels: Option<HashMap<String, bool>>,
    rule: Vec<String>,
    data: Value,
}

#[derive(Clone)]
struct AppState {
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

    let state = AppState {
        flags_client,
    };

    let app = Router::new()
        .route("/", post(handle_run))
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    let addr = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    println!("Listening on http://0.0.0.0:{}", port);
    axum::serve(addr, app).await.unwrap();
}

async fn handle_run(
    State(state): State<AppState>,
    Json(package): Json<RuleDataPackage>
) -> (StatusCode, Json<EvaluationResponse>) {
    if !state.flags_client.is("run_v1").enabled().await {
        return (StatusCode::NOT_IMPLEMENTED, Json(EvaluationResponse{
            result: false,
            error: None,
            trace: None,
            labels: None,
            rule: Vec::new(),
            data: Value::Null,
        }));
    }

    match parse_rules(&package.rule) {
        Ok(rule_set) => match evaluate_rule_set(&rule_set, &package.data) {
            Ok((results, trace)) => {
                //eprintln!("rules: {:?}", rule_set);

                let mut labels = HashMap::new();
                for rule_trace in &trace.execution {
                    if let Some(label) = &rule_trace.label {
                        labels.insert(label.to_string(), rule_trace.result);
                    }
                }

                let result = results.values().next().cloned().unwrap_or(false);
                let rule = package.rule.lines().map(String::from).collect();
                let response = EvaluationResponse {
                    result,
                    error: None,
                    trace: Some(trace),
                    labels: if labels.is_empty() {
                        None
                    } else {
                        Some(labels)
                    },
                    rule,
                    data: package.data.clone(),
                };
                //eprintln!("response: {:?}", response);
                (StatusCode::OK, Json(response))
            }
            Err(error) => {
                let rule = package.rule.lines().map(String::from).collect();
                let response = EvaluationResponse {
                    result: false,
                    error: Some(error.to_string()),
                    trace: None,
                    labels: None,
                    rule,
                    data: package.data.clone(),
                };
                (StatusCode::BAD_REQUEST, Json(response))
            }
        },
        Err(error) => {
            let rule = package.rule.lines().map(String::from).collect();
            
            let response = EvaluationResponse {
                result: false,
                error: Some(error.to_string()),
                trace: None,
                labels: None,
                rule,
                data: package.data.clone(),
            };
            (StatusCode::BAD_REQUEST, Json(response))
        }
    }
}
