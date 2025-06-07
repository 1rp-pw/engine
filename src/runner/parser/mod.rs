mod lib;

use crate::runner::error::RuleError;
use chrono::NaiveDate;
use pest::error::InputLocation::Pos;
use pest::Parser;
use pest_derive::Parser;
use pest::iterators::Pair;
use crate::runner::model::{ComparisonOperator, RuleSet, RuleValue, Condition, SourcePosition, ConditionOperator, ComparisonCondition, PositionedValue, RuleReferenceCondition, PropertyPath};

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
    let (line, _) = span.start_pos().line_col();
    let start = span.start();
    let end = span.end();
    let position = Some(SourcePosition { line, start, end });

    let mut inner_pairs = pair.into_inner();

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
                let (l, start_col) = span.start_pos().line_col();
                let (_, end_col) = span.end_pos().line_col();
                selector_pos = Some(SourcePosition {
                    line: l,
                    start: start_col,
                    end: end_col,
                });
                let s = header_part.as_str();
                selector = s[2..s.len() - 2].to_string();
            }
            _ => {}
        }
    }

    if selector.is_empty() {
        return Err(RuleError::ParseError("Missing selector in rule".to_string()));
    }

    let outcome_pair = inner_pairs
        .next()
        .ok_or_else(|| RuleError::ParseError("Missing outcome".to_string()))?;

    let mut oi = outcome_pair.into_inner();
    let first = oi
        .next()
        .ok_or_else(|| RuleError::ParseError("Empty outcome".to_string()))?
        .as_str()
        .trim()
        .to_string();
    let outcome_text = if let Some(second) = oi.next() {
        second.as_str().trim().to_string()
    } else {
        first
    };

    let mut rule = crate::runner::model::Rule::new(label.clone(), selector.clone(), outcome_text.clone());
    rule.position = position;
    rule.selector_pos = selector_pos;

    let remaining_pairs: Vec<_> = inner_pairs.collect();
    let mut i = 0;
    while i < remaining_pairs.len() {
        if remaining_pairs[i].as_rule() == Rule::condition {
            let cond = parse_condition(remaining_pairs[i].clone())?;

            let op = if rule.conditions.is_empty() {
                None
            } else {
                let mut found: Option<ConditionOperator> = None;
                for j in (0..i).rev() {
                    if remaining_pairs[j].as_rule() == Rule::condition_operator {
                        found = Some(parse_condition_operator(remaining_pairs[j].clone())?);
                        break;
                    }
                }
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
        Rule::property_condition => {
            let comparison = parse_property_condition(inner_pair)?;
            Ok(Condition::Comparison(comparison))
        },
        Rule::rule_reference => {
            let reference = parse_rule_reference(inner_pair)?;
            Ok(Condition::RuleReference(reference))
        },
        Rule::label_reference => {
            let reference = parse_label_reference(inner_pair)?;
            Ok(Condition::RuleReference(reference))
        },
        _ => Err(RuleError::ParseError(format!("Unknown condition type: {:?}", inner_pair.as_rule())))
    }
}

fn parse_label_reference(pair: Pair<Rule>) -> Result<RuleReferenceCondition, RuleError> {
    let mut inner_parts = pair.into_inner();
    let label_name_pair = inner_parts.next()
        .ok_or_else(|| RuleError::ParseError("Missing label name".to_string()))?;

    let span = label_name_pair.as_span();
    let (line, start_col) = span.start_pos().line_col();
    let (_, end_col) = span.end_pos().line_col();
    let pos = Some(SourcePosition {
        line,
        start: start_col,
        end: end_col,
    });

    let label_name = PositionedValue::with_position(
        label_name_pair.as_str().to_string(),
        pos
    );

    Ok(RuleReferenceCondition {
        selector: PositionedValue::new(String::new()),
        rule_name: label_name,
    })
}

fn parse_property_condition(pair: Pair<Rule>) -> Result<ComparisonCondition, RuleError> {
    let mut inner_pairs = pair.into_inner();

    // Parse the left side - could be property_access or length_expr
    let left_access_pair = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing property access".to_string()))?;

    // Check what type of left side we have
    match left_access_pair.as_rule() {
        Rule::length_expr => {
            // Handle length expression
            parse_length_condition(left_access_pair, inner_pairs)
        }
        Rule::property_access => {
            // Handle regular property access (existing logic)
            parse_regular_property_condition(left_access_pair, inner_pairs)
        }
        _ => Err(RuleError::ParseError("Expected property access or length expression".to_string()))
    }
}

fn parse_length_condition(
    length_expr_pair: Pair<Rule>,
    mut remaining_pairs: pest::iterators::Pairs<Rule>
) -> Result<ComparisonCondition, RuleError> {
    // Parse the length expression to get the property path
    let property_path = parse_length_expression(length_expr_pair)?;

    // Parse the predicate
    let predicate_pair = remaining_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing predicate after length expression".to_string()))?;

    let mut predicate_inner = predicate_pair.into_inner();

    // Parse operator
    let operator_pair = predicate_inner.next()
        .ok_or_else(|| RuleError::ParseError("Missing operator".to_string()))?;

    let operator = match operator_pair.as_rule() {
        Rule::comparison_operator => {
            match operator_pair.as_str() {
                "is greater than or equal to" => ComparisonOperator::GreaterThanOrEqual,
                "is at least" => ComparisonOperator::GreaterThanOrEqual,
                "is less than or equal to" => ComparisonOperator::LessThanOrEqual,
                "is no more than" => ComparisonOperator::LessThanOrEqual,
                "is equal to" => ComparisonOperator::EqualTo,
                "is the same as" => ComparisonOperator::EqualTo,
                "is not equal to" => ComparisonOperator::NotEqualTo,
                "is not the same as" => ComparisonOperator::NotEqualTo,
                "is greater than" => ComparisonOperator::GreaterThan,
                "is less than" => ComparisonOperator::LessThan,
                _ => return Err(RuleError::ParseError(format!("Unsupported operator for length comparison: {}", operator_pair.as_str())))
            }
        }
        _ => return Err(RuleError::ParseError("Length comparisons require comparison operators".to_string()))
    };

    // Parse right operand (should be a number for length comparisons)
    let right_pair = predicate_inner.next()
        .ok_or_else(|| RuleError::ParseError("Missing right operand".to_string()))?;

    let right_value = match right_pair.as_rule() {
        Rule::value => {
            let value_span = right_pair.as_span();
            let (value_line, start_col) = value_span.start_pos().line_col();
            let (_, end_col) = value_span.end_pos().line_col();
            let val_pos = Some(SourcePosition{
                line: value_line,
                start: start_col,
                end: end_col,
            });
            PositionedValue::with_position(parse_value(right_pair)?, val_pos)
        }
        _ => return Err(RuleError::ParseError("Length comparisons require a numeric value".to_string()))
    };

    Ok(ComparisonCondition {
        selector: PositionedValue::new(property_path.selector.clone()),
        property: PositionedValue::new("__length__".to_string()), // Special marker for length
        operator,
        value: right_value,
        property_chain: None,
        left_property_path: Some(property_path),
        right_property_path: None,
    })
}

fn parse_length_expression(pair: Pair<Rule>) -> Result<PropertyPath, RuleError> {
    let mut inner_pairs = pair.into_inner();

    let property_access_pair = inner_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing property access".to_string()))?;

    let mut path = parse_property_access(property_access_pair)?;
    path.properties.push("__length__".to_string());

    Ok(path)
}

fn parse_comparison_operator(pair: Pair<Rule>) -> Result<ComparisonOperator, RuleError> {
    match pair.as_str() {
        "is greater than or equal to" => Ok(ComparisonOperator::GreaterThanOrEqual),
        "is at least" => Ok(ComparisonOperator::GreaterThanOrEqual),

        "is less than or equal to" => Ok(ComparisonOperator::LessThanOrEqual),
        "is no more than" => Ok(ComparisonOperator::LessThanOrEqual),

        "is equal to" => Ok(ComparisonOperator::EqualTo),
        "is the same as" => Ok(ComparisonOperator::EqualTo),

        "is not equal to" => Ok(ComparisonOperator::NotEqualTo),
        "is not the same as" => Ok(ComparisonOperator::NotEqualTo),

        "is later than" => Ok(ComparisonOperator::LaterThan),

        "is earlier than" => Ok(ComparisonOperator::EarlierThan),

        "is greater than" => Ok(ComparisonOperator::GreaterThan),
        "is less than" => Ok(ComparisonOperator::LessThan),

        "is in" => Ok(ComparisonOperator::In),
        "is not in" => Ok(ComparisonOperator::NotIn),
        "contains" => Ok(ComparisonOperator::Contains),
        _ => Err(RuleError::ParseError(format!("Unknown operator: {}", pair.as_str())))
    }
}

fn parse_regular_property_condition(
    property_access_pair: Pair<Rule>,
    mut remaining_pairs: pest::iterators::Pairs<Rule>
) -> Result<ComparisonCondition, RuleError> {
    // This is the EXISTING logic from the original parse_property_condition function
    let left_path = parse_property_access(property_access_pair)?;

    // Parse the predicate
    let predicate_pair = remaining_pairs.next()
        .ok_or_else(|| RuleError::ParseError("Missing predicate".to_string()))?;

    let mut predicate_inner = predicate_pair.into_inner();

    // Parse operator
    let operator_pair = predicate_inner.next()
        .ok_or_else(|| RuleError::ParseError("Missing operator".to_string()))?;

    let operator = match operator_pair.as_rule() {
        Rule::comparison_operator => {
            match operator_pair.as_str() {
                "is greater than or equal to" => ComparisonOperator::GreaterThanOrEqual,
                "is at least" => ComparisonOperator::GreaterThanOrEqual,

                "is less than or equal to" => ComparisonOperator::LessThanOrEqual,
                "is no more than" => ComparisonOperator::LessThanOrEqual,

                "is equal to" => ComparisonOperator::EqualTo,
                "is the same as" => ComparisonOperator::EqualTo,

                "is not equal to" => ComparisonOperator::NotEqualTo,
                "is not the same as" => ComparisonOperator::NotEqualTo,

                "is later than" => ComparisonOperator::LaterThan,
                "is earlier than" => ComparisonOperator::EarlierThan,

                "is greater than" => ComparisonOperator::GreaterThan,
                "is less than" => ComparisonOperator::LessThan,

                "is in" => ComparisonOperator::In,
                "is not in" => ComparisonOperator::NotIn,
                "contains" => ComparisonOperator::Contains,
                _ => return Err(RuleError::ParseError(format!("Unknown operator: {}", operator_pair.as_str())))
            }
        }
        Rule::list_operator => {
            match operator_pair.as_str() {
                "is in" => ComparisonOperator::In,
                "is not in" => ComparisonOperator::NotIn,
                _ => return Err(RuleError::ParseError(format!("Unknown list operator: {}", operator_pair.as_str())))
            }
        }
        _ => return Err(RuleError::ParseError("Expected operator".to_string()))
    };

    // Parse right operand
    let right_pair = predicate_inner.next()
        .ok_or_else(|| RuleError::ParseError("Missing right operand".to_string()))?;
    let rp = right_pair.clone();

    let (right_value, right_property_path) = match right_pair.as_rule() {
        Rule::property_access => {
            let right_path = parse_property_access(right_pair)?;
            let value_span = rp.as_span();
            let (value_line, start_col) = value_span.start_pos().line_col();
            let (_, end_col) = value_span.end_pos().line_col();
            let val_pos = Some(SourcePosition{
                line: value_line,
                start: start_col,
                end: end_col,
            });
            let property_path_string = format!("$.{}.{}",
                                               right_path.selector,
                                               right_path.properties.join(".")
            );
            (
                PositionedValue::with_position(RuleValue::String(property_path_string), val_pos),
                Some(right_path)
            )
        }
        Rule::list_value => {
            let value_span = right_pair.as_span();
            let (value_line, start_col) = value_span.start_pos().line_col();
            let (_, end_col) = value_span.end_pos().line_col();
            let val_pos = Some(SourcePosition{
                line: value_line,
                start: start_col,
                end: end_col,
            });
            (
                PositionedValue::with_position(parse_list_value(right_pair)?, val_pos),
                None
            )
        }
        Rule::value => {
            let value_span = right_pair.as_span();
            let (value_line, start_col) = value_span.start_pos().line_col();
            let (_, end_col) = value_span.end_pos().line_col();
            let val_pos = Some(SourcePosition{
                line: value_line,
                start: start_col,
                end: end_col,
            });
            (
                PositionedValue::with_position(parse_value(right_pair)?, val_pos),
                None
            )
        }
        _ => return Err(RuleError::ParseError(format!("Unknown right operand type: {:?}", right_pair.as_rule())))
    };

    Ok(ComparisonCondition {
        selector: PositionedValue::new(left_path.selector.clone()),
        property: PositionedValue::new(left_path.properties.last().unwrap_or(&String::new()).clone()),
        operator,
        value: right_value,
        property_chain: None,
        left_property_path: Some(left_path),
        right_property_path,
    })
}

fn parse_property_access(pair: Pair<Rule>) -> Result<crate::runner::model::PropertyPath, RuleError> {
    let mut properties = Vec::new();
    let mut selector = String::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::property => {
                let property_text = inner.as_str();
                let property_name = property_text[2..property_text.len()-2].to_string();
                properties.push(property_name);
            }
            Rule::object_selector => {
                let selector_text = inner.as_str();
                selector = selector_text[2..selector_text.len()-2].to_string();
            }
            _ => {}
        }
    }

    Ok(crate::runner::model::PropertyPath {
        properties,
        selector,
    })
}

fn parse_rule_reference(pair: Pair<Rule>) -> Result<RuleReferenceCondition, RuleError> {
    let mut selector = None;
    let mut rule_name = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::object_selector => {
                let span = inner.as_span();
                let (line, start_col) = span.start_pos().line_col();
                let(_, end_col) = span.end_pos().line_col();
                let pos = Some(SourcePosition {
                    line,
                    start: start_col,
                    end: end_col,
                });
                let s = inner.as_str();
                let name = s[2..s.len()-2].to_string();
                selector = Some(PositionedValue::with_position(name, pos));
            }
            Rule::reference_name => {
                let span = inner.as_span();
                let (line, start_col) = span.start_pos().line_col();
                let (_, end_col) = span.end_pos().line_col();
                let pos = Some(SourcePosition {
                    line,
                    start: start_col,
                    end: end_col,
                });
                let name = inner.as_str().trim().to_string();
                rule_name = Some(PositionedValue::with_position(name, pos));
            }
            _ => {}
        }
    }

    let selector = selector
        .ok_or_else(|| RuleError::ParseError("Missing selector in rule‐ref".into()))?;
    let rule_name = rule_name
        .unwrap_or_else(|| PositionedValue::new("requirement".to_string()));

    Ok(RuleReferenceCondition {
        selector,
        rule_name,
    })
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