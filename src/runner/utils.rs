use crate::runner::error::RuleError;
use crate::runner::model::{Condition, Rule};

fn find_referenced_outcomes(rules: &[Rule]) -> std::collections::HashSet<String> {
    let mut referenced = std::collections::HashSet::new();
    for rule in rules {
        for cond in &rule.conditions {
            if let Condition::RuleReference { selector: _, rule_name } = cond {
                for other_rule in rules {
                    if other_rule.outcome.contains(rule_name) || rule_name.contains(&other_rule.outcome) {
                        referenced.insert(other_rule.outcome.clone());
                    }
                }
            }
        }
    }
    referenced
}

pub fn find_global_rule<'a>(rules: &'a [Rule]) -> Result<&'a Rule, RuleError> {
    if rules.len() == 1 {
        return Ok(&rules[0])
    }
    
    let referenced = find_referenced_outcomes(rules);
    let globals: Vec<&Rule> = rules
        .iter()
        .filter(|r| !referenced.contains(&r.outcome))
        .collect();

    match globals.len() {
        1 => Ok(globals[0]),
        0 => Err(RuleError::ParseError("No global rule found".to_string())),
        _ => Err(RuleError::ParseError("Multiple global rules found".to_string())),
    }
}