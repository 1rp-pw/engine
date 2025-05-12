// src/parser.rs
use crate::error::RuleError;
use crate::model::{Condition, ComparisonOperator, RuleSet, RuleValue};
use chrono::NaiveDate;
use pest::Parser;
use pest_derive::Parser;
use pest::iterators::{Pair, Pairs};

#[derive(Parser)]
#[grammar = "src/grammer.pest"]
pub struct RuleParser;

pub fn parse_rules(input: &str) -> Result<RuleSet, RuleError> {
    let pairs = RuleParser::parse(Rule::rule_set, input)
        .map_err(|e| RuleError::ParseError(e.to_string()))?;

    let mut rule_set = RuleSet::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::rule_set => {
                for rule_pair in pair.into_inner() {
                    if rule_pair.as_rule() == Rule::rule {
                        let rule = parse_rule(rule_pair)?;
                        rule_set.add_rule(rule);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(rule_set)
}

fn parse_rule(pair: Pair<Rule>) -> Result<crate::model::Rule, RuleError> {
    let mut inner_pairs = pair.into_inner();

    let selector = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing selector".to_string()))?
        .as_str().trim().to_string();

    let outcome = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing outcome".to_string()))?
        .as_str().trim().to_string();

    let mut rule = crate::model::Rule::new(selector, outcome);

    // Parse conditions
    for condition_pair in inner_pairs {
        if condition_pair.as_rule() == Rule::condition {
            let condition = parse_condition(condition_pair)?;
            rule.add_condition(condition);
        }
    }

    Ok(rule)
}

fn parse_condition(pair: Pair<Rule>) -> Result<Condition, RuleError> {
    let inner_pair = pair.into_inner().next()
        .ok_or_else(|| RuleError::ParseError("Empty condition".to_string()))?;

    match inner_pair.as_rule() {
        Rule::property_condition => parse_property_condition(inner_pair),
        Rule::rule_reference => parse_rule_reference(inner_pair),
        _ => Err(RuleError::ParseError(format!("Unknown condition type: {:?}", inner_pair.as_rule())))
    }
}

fn parse_property_condition(pair: Pair<Rule>) -> Result<Condition, RuleError> {
    let mut inner_pairs = pair.into_inner();

    let property = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing property".to_string()))?
        .into_inner().next()
        .ok_or_else(|| RuleError::ParseError("Empty property".to_string()))?
        .as_str().to_string();

    let selector = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing selector".to_string()))?
        .as_str().to_string();

    let predicate = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing predicate".to_string()))?;

    let (operator, value) = parse_predicate(predicate)?;

    Ok(Condition::Comparison {
        selector,
        property,
        operator,
        value,
    })
}

fn parse_rule_reference(pair: Pair<Rule>) -> Result<Condition, RuleError> {
    let mut inner_pairs = pair.into_inner();

    let selector = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing selector in rule reference".to_string()))?
        .as_str().to_string();

    let rule_name = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing rule name in reference".to_string()))?
        .as_str().trim_matches('"').to_string();

    Ok(Condition::RuleReference {
        selector,
        rule_name,
    })
}

fn parse_predicate(pair: Pair<Rule>) -> Result<(ComparisonOperator, RuleValue), RuleError> {
    let mut inner_pairs = pair.into_inner();

    let op_pair = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing operator".to_string()))?;

    let operator = match op_pair.as_str() {
        "is greater than or equal to" => ComparisonOperator::GreaterThanOrEqual,
        "is equal to" => ComparisonOperator::EqualTo,
        "is the same as" => ComparisonOperator::SameAs,
        "is later than" => ComparisonOperator::LaterThan,
        "is greater than" => ComparisonOperator::GreaterThan,
        "is in" => ComparisonOperator::In,
        _ => return Err(RuleError::ParseError(format!("Unknown operator: {}", op_pair.as_str())))
    };

    let value_pair = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing value".to_string()))?;

    let value = if operator == ComparisonOperator::In {
        parse_list_value(value_pair)?
    } else {
        parse_value(value_pair)?
    };

    Ok((operator, value))
}

fn parse_list_value(pair: Pair<Rule>) -> Result<RuleValue, RuleError> {
    let inner_pairs = pair.into_inner();
    let mut values = Vec::new();

    for value_pair in inner_pairs {
        values.push(parse_value(value_pair)?);
    }

    Ok(RuleValue::List(values))
}

fn parse_value(pair: Pair<Rule>) -> Result<RuleValue, RuleError> {
    let inner_pair = pair.into_inner().next()
        .ok_or_else(|| RuleError::ParseError("Empty value".to_string()))?;

    match inner_pair.as_rule() {
        Rule::number => {
            let num = inner_pair.as_str().parse::<f64>()
                .map_err(|e| RuleError::ParseError(format!("Invalid number: {}", e)))?;
            Ok(RuleValue::Number(num))
        },
        Rule::string_literal => {
            let s = inner_pair.as_str().trim_matches('"').to_string();
            Ok(RuleValue::String(s))
        },
        Rule::date_literal => {
            let date_str = inner_pair.as_str();
            let date_part = &date_str[5..date_str.len()-1]; // Remove date() wrapper
            let date = NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
                .map_err(|e| RuleError::ParseError(format!("Invalid date: {}", e)))?;
            Ok(RuleValue::Date(date))
        },
        Rule::boolean => {
            let b = inner_pair.as_str() == "true";
            Ok(RuleValue::Boolean(b))
        },
        _ => Err(RuleError::ParseError(format!("Unknown value type: {:?}", inner_pair.as_rule())))
    }
}
