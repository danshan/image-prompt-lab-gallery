use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::{DomainError, DomainResult};

use super::PromptTemplateVariable;

pub fn next_prompt_version_number(current_max_version_number: Option<u32>) -> u32 {
    current_max_version_number.unwrap_or(0) + 1
}

pub fn prompt_version_name(version_number: u32) -> String {
    format!("v{version_number}")
}

pub fn render_prompt_template(
    body: &str,
    variables: &[PromptTemplateVariable],
    values: &Value,
) -> DomainResult<String> {
    let declared = variables
        .iter()
        .map(|variable| (variable.name.as_str(), variable))
        .collect::<BTreeMap<_, _>>();
    let referenced = referenced_variables(body);

    for name in &referenced {
        if !declared.contains_key(name.as_str()) {
            return Err(DomainError::InvalidGenerationParameters {
                message: format!("prompt variable `{name}` is not declared"),
            });
        }
    }

    let mut resolved = BTreeMap::new();
    for variable in variables {
        let runtime_value = values
            .get(&variable.name)
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string);
        let value = runtime_value.or_else(|| variable.default_value.clone());

        if variable.required && value.as_ref().is_none_or(|value| value.trim().is_empty()) {
            return Err(DomainError::InvalidGenerationParameters {
                message: format!("prompt variable `{}` is required", variable.name),
            });
        }

        if let Some(value) = value {
            resolved.insert(variable.name.as_str(), value);
        }
    }

    render_referenced_placeholders(body, &resolved)
}

fn referenced_variables(body: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut remaining = body;

    while let Some(start) = remaining.find("{{") {
        let after_start = &remaining[start + 2..];
        let Some(end) = after_start.find("}}") else {
            break;
        };
        let name = after_start[..end].trim();
        if !name.is_empty() {
            names.insert(name.to_string());
        }
        remaining = &after_start[end + 2..];
    }

    names
}

fn render_referenced_placeholders(
    body: &str,
    resolved: &BTreeMap<&str, String>,
) -> DomainResult<String> {
    let mut rendered = String::with_capacity(body.len());
    let mut remaining = body;

    while let Some(start) = remaining.find("{{") {
        rendered.push_str(&remaining[..start]);
        let after_start = &remaining[start + 2..];
        let Some(end) = after_start.find("}}") else {
            rendered.push_str(&remaining[start..]);
            return Ok(rendered);
        };

        let raw_name = &after_start[..end];
        let name = raw_name.trim();
        if let Some(value) = resolved.get(name) {
            rendered.push_str(value);
        } else {
            rendered.push_str("{{");
            rendered.push_str(raw_name);
            rendered.push_str("}}");
        }
        remaining = &after_start[end + 2..];
    }

    rendered.push_str(remaining);
    Ok(rendered)
}
