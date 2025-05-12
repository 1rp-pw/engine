// src/main.rs
mod error;
mod model;
mod parser;
mod evaluator;

use error::RuleError;
use parser::parse_rules;
use evaluator::evaluate_rule_set;
use serde_json::Value;
use std::fs;
use crate::model::Condition;

fn main() -> Result<(), RuleError> {
    let rules_file = "examples/driving_test.rules";
    let json_file = "examples/person.json";

    // Read the rules file
    let rule_text = fs::read_to_string(rules_file)?;

    // Parse the rules
    let rule_set = parse_rules(&rule_text)?;
    println!("Successfully parsed {} rules from {}", rule_set.rules.len(), rules_file);

    // Read the JSON file
    let json_text = fs::read_to_string(json_file)?;
    let json: Value = serde_json::from_str(&json_text)?;

    // Evaluate rules against JSON
    let results = evaluate_rule_set(&rule_set, &json)?;

    // Print results
    println!("\nRules:");
    for (i, rule) in rule_set.rules.iter().enumerate() {
        if let Some(label) = &rule.label {
            println!("  {}. [{}] **{}** gets {}", i+1, label, rule.selector, rule.outcome);
        } else {
            println!("  {}. **{}** gets {}", i+1, rule.selector, rule.outcome);
        }

        // Print conditions for each rule
        for (j, condition) in rule.conditions.iter().enumerate() {
            match condition {
                Condition::Comparison { selector, property, operator, value } => {
                    println!("     Condition {}: the __{}__ of the **{}** {} {}",
                             j+1, property, selector, operator, value);
                },
                Condition::RuleReference { selector, rule_name } => {
                    println!("     Condition {}: the **{}** passes {}",
                             j+1, selector, rule_name);
                },
            }
        }
    }

    Ok(())
}
