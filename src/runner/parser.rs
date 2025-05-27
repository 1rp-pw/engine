use crate::runner::error::RuleError;
use chrono::NaiveDate;
use pest::Parser;
use pest_derive::Parser;
use pest::iterators::Pair;
use crate::runner::model::{ComparisonOperator, RuleSet, RuleValue, Condition, SourcePosition, ConditionOperator};

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
    // capture source position for the whole rule
    let span = pair.as_span();
    let (line, _) = span.start_pos().line_col();
    let start = span.start();
    let end   = span.end();
    let position = Some(SourcePosition { line, start, end });

    let mut inner_pairs = pair.into_inner();

    // 1) parse the header: optional label + selector
    let header_pair = inner_pairs
        .next()
        .ok_or_else(|| RuleError::ParseError("Missing rule header".to_string()))?;

    let mut label: Option<String> = None;
    let mut selector = String::new();
    let mut selector_pos: Option<SourcePosition> = None;

    for header_part in header_pair.into_inner() {
        match header_part.as_rule() {
            Rule::label => {
                let txt = header_part.as_str().strip_suffix(". ").unwrap_or_else(|| header_part.as_str());
                label = Some(txt.to_string());
            }
            Rule::object_selector => {
                let span = header_part.as_span();
                let (l, _) = span.start_pos().line_col();
                selector_pos = Some(SourcePosition {
                    line: l,
                    start: span.start(),
                    end: span.end(),
                });
                // strip the `**` markup
                let s = header_part.as_str();
                selector = s[2..s.len() - 2].to_string();
            }
            _ => {}
        }
    }

    if selector.is_empty() {
        return Err(RuleError::ParseError("Missing selector in rule".to_string()));
    }

    // 2) parse the (now optional‐verb) outcome
    let outcome_pair = inner_pairs
        .next()
        .ok_or_else(|| RuleError::ParseError("Missing outcome".to_string()))?;

    // flatten its children: [ maybe verb , outcome_text ]
    let mut oi = outcome_pair.into_inner();
    let first = oi
        .next()
        .ok_or_else(|| RuleError::ParseError("Empty outcome".to_string()))?
        .as_str()
        .trim()
        .to_string();
    // if a second piece exists, that's the real outcome; otherwise first *is* the outcome
    let outcome_text = if let Some(second) = oi.next() {
        second.as_str().trim().to_string()
    } else {
        first
    };

    // 3) build the Rule
    let mut rule =
        crate::runner::model::Rule::new(label.clone(), selector.clone(), outcome_text.clone());
    rule.position = position;
    rule.selector_pos = selector_pos;

    // 4) now parse all following conditions (with their and/or operators)
    let remaining_pairs: Vec<_> = inner_pairs.collect();
    let mut i = 0;
    while i < remaining_pairs.len() {
        if remaining_pairs[i].as_rule() == Rule::condition {
            // parse the condition itself
            let cond = parse_condition(remaining_pairs[i].clone())?;

            // decide if it has an operator (None for the very first)
            let op = if rule.conditions.is_empty() {
                None
            } else {
                // look backwards for the nearest condition_operator
                let mut found: Option<ConditionOperator> = None;
                for j in (0..i).rev() {
                    if remaining_pairs[j].as_rule() == Rule::condition_operator {
                        found = Some(parse_condition_operator(
                            remaining_pairs[j].clone(),
                        )?);
                        break;
                    }
                }
                // default to AND if none was written
                found.or(Some(ConditionOperator::And))
            };

            rule.add_condition(cond, op);
        }
        i += 1;
    }

    Ok(rule)
}

fn parse_condition_operator(pair: Pair<Rule>) -> Result<ConditionOperator, RuleError> {
    match pair.as_str() {
        "and" => Ok(ConditionOperator::And),
        "or" => Ok(ConditionOperator::Or),
        _ => Err(RuleError::ParseError(format!("Unknown condition operator: {}", pair.as_str())))
    }
}

fn parse_condition(pair: Pair<Rule>) -> Result<Condition, RuleError> {
    let inner_pair = pair.into_inner().next()
        .ok_or_else(|| RuleError::ParseError("Empty condition".to_string()))?;

    match inner_pair.as_rule() {
        Rule::property_condition => parse_property_condition(inner_pair),
        Rule::rule_reference => parse_rule_reference(inner_pair),
        Rule::label_reference => parse_label_reference(inner_pair),
        _ => Err(RuleError::ParseError(format!("Unknown condition type: {:?}", inner_pair.as_rule())))
    }
}

fn parse_label_reference(pair: Pair<Rule>) -> Result<Condition, RuleError> {
    let mut inner_parts = pair.into_inner();
    let label_name_pair = inner_parts.next()
        .ok_or_else(|| RuleError::ParseError("Missing label name".to_string()))?;

    let label_name = label_name_pair.as_str().to_string();

    Ok(Condition::RuleReference {
        selector: String::new(),
        rule_name: label_name,
    })
}

fn parse_property_condition(pair: Pair<Rule>) -> Result<Condition, RuleError> {
    let mut inner_pairs = pair.into_inner();

    let property_pair = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing property".to_string()))?;
    let property_text = property_pair.as_str();
    let property_name = property_text[2..property_text.len()-2].to_string();
    let property = crate::runner::utils::transform_property_name(&property_name);
    let property_span = property_pair.as_span();
    let (property_line_start, property_word_start) = property_span.start_pos().line_col();
    let (_, property_word_end) = property_span.end_pos().line_col();
    let property_pos = Some(SourcePosition {
        line: property_line_start,
        start: property_word_start,
        end: property_word_end,
    });

    let object_selector_pair = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing object selector".to_string()))?;

    let selector_text = object_selector_pair.as_str();
    let selector = selector_text[2..selector_text.len()-2].to_string();
    let selector_span = object_selector_pair.as_span();
    let (selector_line_start, selector_word_start) = selector_span.start_pos().line_col();
    let (_, selector_word_end) = selector_span.end_pos().line_col();
    let selector_pos = Some(SourcePosition{
        line: selector_line_start,
        start: selector_word_start,
        end: selector_word_end,
    });

    let predicate = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing predicate".to_string()))?;

    let (operator, value, value_pos) = parse_predicate(predicate)?;

    Ok(Condition::Comparison {
        selector,
        selector_pos,
        property,
        property_pos,
        operator,
        value,
        value_pos: Some(value_pos),
    })
}

fn parse_rule_reference(pair: Pair<Rule>) -> Result<Condition, RuleError> {
    let mut selector = None;
    let mut rule_name = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::object_selector => {
                let s = inner.as_str();
                selector = Some(s[2..s.len()-2].to_string());
            }
            Rule::reference_name => {
                rule_name = Some(inner.as_str().trim().to_string());
            }
            _ => {
                // this will catch the optional "the" literal — just ignore it
            }
        }
    }

    let selector = selector
        .ok_or_else(|| RuleError::ParseError("Missing selector in rule‐ref".into()))?;
    // default name if none captured
    let rule_name = rule_name.unwrap_or_else(|| "requirement".into());

    Ok(Condition::RuleReference { selector, rule_name })
}

fn parse_predicate(pair: Pair<Rule>) -> Result<(ComparisonOperator, RuleValue, SourcePosition), RuleError> {
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

    let value_span = value_pair.as_span();
    let (value_line_start, value_word_start) = value_span.start_pos().line_col();
    let (_, value_word_end) = value_span.end_pos().line_col();
    let val_pos = SourcePosition{
        line: value_line_start,
        start: value_word_start,
        end: value_word_end,
    };

    Ok((operator, value, val_pos))
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

            let date_part = if date_str.starts_with("date(") && date_str.ends_with(")") {
                &date_str[5..date_str.len()-1]
            } else {
                date_str
            };


            let date = NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
                .map_err(|e| RuleError::ParseError(format!("Invalid date: {}", e)))?;

            Ok(RuleValue::Date(date))
        },
        Rule::boolean => {
            let b = pair.as_str() == "true";
            Ok(RuleValue::Boolean(b))
        },
        _ => Err(RuleError::ParseError(format!("Unknown value type: {:?}", pair.as_rule())))
    }
}