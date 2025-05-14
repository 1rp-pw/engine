use crate::runner::error::RuleError;
use chrono::NaiveDate;
use pest::Parser;
use pest_derive::Parser;
use pest::iterators::Pair;
use crate::runner::model::{ComparisonOperator, RuleSet, RuleValue, Condition, SourcePosition};

#[derive(Parser)]
#[grammar = "pests/grammer.pest"]
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
                        rule_set.add_rule(rule)
                    }
                }
            }
            _ => {}
        }
    }

    crate::runner::utils::find_global_rule(&rule_set.rules)?;

    Ok(rule_set)
}

fn parse_rule(pair: Pair<Rule>) -> Result<crate::runner::model::Rule, RuleError> {
    let span = pair.as_span();
    let line = span.start_pos().line_col().0;
    let start = span.start();
    let end = span.end();
    let position = Some(SourcePosition {
        line,
        start,
        end,
    });

    let mut inner_pairs = pair.into_inner();

    // Parse rule header (which includes label and selector)
    let header_pair = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing rule header".to_string()))?;

    let mut label = None;
    let mut selector = String::new();
    let mut selector_pos = Some(SourcePosition {
        line: 0,
        start: 0,
        end: 0,
    });

    for header_part in header_pair.into_inner() {
        match header_part.as_rule() {
            Rule::label => {
                // Remove the trailing ". " from the label
                let label_text = header_part.as_str();
                label = Some(label_text[..label_text.len()-2].to_string());
            },
            Rule::object_selector => {
                // Extract the selector from between the double asterisks
                let span = header_part.as_span();
                selector_pos = Some(SourcePosition {
                    line: span.start_pos().line_col().0,
                    start: span.start(),
                    end: span.end(),
                });
                let selector_text = header_part.as_str();
                selector = selector_text[2..selector_text.len()-2].to_string();
            },
            _ => {}
        }
    }

    if selector.is_empty() {
        return Err(RuleError::ParseError("Missing selector in rule".to_string()));
    }

    // Parse the outcome
    let outcome_pair = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing outcome".to_string()))?;

    // Extract the outcome text, ignoring the verb
    let outcome_text = outcome_pair.into_inner().nth(1)
        .ok_or_else(|| RuleError::ParseError("Missing outcome text".to_string()))?
        .as_str().trim().to_string();

    let mut rule = crate::runner::model::Rule::new(label, selector, outcome_text);
    rule.position = position;
    rule.selector_pos = selector_pos;

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

    let property_pair = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing property".to_string()))?;

    // Extract the property name from between the double underscores
    let property_text = property_pair.as_str();
    let property_name = property_text[2..property_text.len()-2].to_string();

    // Transform property name with spaces to camelCase
    let property = crate::runner::utils::transform_property_name(&property_name);

    let object_selector_pair = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing object selector".to_string()))?;

    // Extract the selector from between the double asterisks
    let selector_text = object_selector_pair.as_str();
    let selector = selector_text[2..selector_text.len()-2].to_string();

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

    let object_selector_pair = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing object selector in rule reference".to_string()))?;

    // Extract the selector from between the double asterisks
    let selector_text = object_selector_pair.as_str();
    let selector = selector_text[2..selector_text.len()-2].to_string();

    // Skip the verb
    inner_pairs.next();

    // The reference object might be optional
    let mut rule_name = String::new();
    for part in inner_pairs {
        rule_name.push_str(part.as_str().trim());
    }

    let rule_name = if rule_name.is_empty() {
        "requirement".to_string()
    } else {
        rule_name
    };

    Ok(Condition::RuleReference {
        selector,
        rule_name,
    })
}

fn parse_predicate(pair: Pair<Rule>) -> Result<(ComparisonOperator, RuleValue), RuleError> {
    let inner_pairs = pair.into_inner().collect::<Vec<_>>();

    if inner_pairs.len() < 2 {
        return Err(RuleError::ParseError("Predicate must have an operator and a value".to_string()));
    }

    let op_pair = &inner_pairs[0];
    let operator = match op_pair.as_str() {
        "is greater than or equal to" => ComparisonOperator::GreaterThanOrEqual,
        "is less than or equal to" => ComparisonOperator::LessThanOrEqual,
        "is equal to" => ComparisonOperator::EqualTo,
        "is not equal to" => ComparisonOperator::NotEqualTo,
        "is the same as" => ComparisonOperator::SameAs,
        "is not the same as" => ComparisonOperator::NotSameAs,
        "is later than" => ComparisonOperator::LaterThan,
        "is earlier than" => ComparisonOperator::EarlierThan,
        "is greater than" => ComparisonOperator::GreaterThan,
        "is less than" => ComparisonOperator::LessThan,
        "is in" => ComparisonOperator::In,
        "is not in" => ComparisonOperator::NotIn,
        "contains" => ComparisonOperator::Contains,
        _ => return Err(RuleError::ParseError(format!("Unknown operator: {}", op_pair.as_str())))
    };

    let value_pair = &inner_pairs[1];
    //println!("Value pair rule: {:?}, text: {}", value_pair.as_rule(), value_pair.as_str());

    let value = if operator == ComparisonOperator::In || operator == ComparisonOperator::NotIn {
        parse_list_value(value_pair.clone())?
    } else {
        parse_value(value_pair.clone())?
    };

    println!("Parsed value: {:?}", value);

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
    match pair.as_rule() {
        Rule::value => {
            // If we got a value container, get the first child
            let inner = pair.into_inner().next()
                .ok_or_else(|| RuleError::ParseError("Empty value".to_string()))?;
            parse_value(inner)
        },
        Rule::number => {
            let num = pair.as_str().parse::<f64>()
                .map_err(|e| RuleError::ParseError(format!("Invalid number: {}", e)))?;
            Ok(RuleValue::Number(num))
        },
        Rule::string_literal => {
            let s = pair.as_str().trim_matches('"').to_string();
            Ok(RuleValue::String(s))
        },
        Rule::date_literal => {
            let date_str = pair.as_str();
            println!("Parsing date literal: {}", date_str);

            // Check if it's in date() format or plain format
            let date_part = if date_str.starts_with("date(") && date_str.ends_with(")") {
                // Extract from date() wrapper
                &date_str[5..date_str.len()-1]
            } else {
                // It's already in YYYY-MM-DD format
                date_str
            };

            println!("Extracted date part: {}", date_part);

            let date = NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
                .map_err(|e| RuleError::ParseError(format!("Invalid date: {}", e)))?;

            println!("Parsed date: {}", date);
            Ok(RuleValue::Date(date))
        },
        Rule::boolean => {
            let b = pair.as_str() == "true";
            Ok(RuleValue::Boolean(b))
        },
        _ => Err(RuleError::ParseError(format!("Unknown value type: {:?}", pair.as_rule())))
    }
}