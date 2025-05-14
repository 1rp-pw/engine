mod runner;
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
                let result = results.values().next().cloned().unwrap_or(false);
                let response = EvaluationResponse {
                    result,
                    error: None,
                    trace: Some(trace),
                };
                (StatusCode::OK, Json(response))
            }
            Err(error) => {
                let response = EvaluationResponse {
                    result: false,
                    error: Some(error.to_string()),
                    trace: None,
                };
                (StatusCode::BAD_REQUEST, Json(response))
            }
        },
        Err(error) => {
            let response = EvaluationResponse {
                result: false,
                error: Some(error.to_string()),
                trace: None,
            };
            (StatusCode::BAD_REQUEST, Json(response))
        }
    }
}
// let rule_text: String;
//     let json: Value;
//
//     let mut buffer = String::new();
//     io::stdin().read_to_string(&mut buffer)?;
//
//     let package: RuleDataPackage = serde_json::from_str(&buffer)
//         .map_err(|err| RuleError::ParseError(err.to_string()))?;
//
//     rule_text = package.rule;
//     json = package.data;
//
//     let rule_set = parse_rules(&rule_text)?;
//     let results = evaluate_rule_set(&rule_set, &json)?;
//     for (selector, outcome) in results.iter() {
//         println!("{}: {}", selector, outcome);
//     }
//
//     print_rules(&rule_set);
//
//     Ok(())
// }

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
                    println!("     Condition {}: the __{}__ of the **{}** {} {}",
                             j + 1, property, selector, operator, value);
                },
                Condition::RuleReference { selector, rule_name } => {
                    println!("     Condition {}: the **{}** passes {}",
                             j + 1, selector, rule_name);
                },
            }
        }
    }
}
