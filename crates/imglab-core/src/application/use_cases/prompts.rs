use crate::application::ports::PromptRepository;
use crate::domain::prompt::{render_prompt_template, PromptTemplateVariable};
use crate::{
    CreatePromptDocumentRequest, DomainError, DomainResult, ListPromptDocumentsRequest,
    ListPromptOutputHistoryRequest, ListPromptVersionsRequest, LoadPromptVersionRequest,
    PromptDocumentView, PromptOutputHistoryItem, PromptVersionView, RenderPromptRunRequest,
    RenderPromptRunResult, SaveGenerationPromptAsPromptRequest, SavePromptAsPromptRequest,
    SavePromptVersionRequest, UpdatePromptDraftRequest,
};
use serde_json::Value;

pub struct PromptWorkspaceUseCase<R> {
    repository: R,
}

impl<R> PromptWorkspaceUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> PromptWorkspaceUseCase<R>
where
    R: PromptRepository,
{
    pub fn create_prompt_document(
        &self,
        request: CreatePromptDocumentRequest,
    ) -> DomainResult<PromptDocumentView> {
        self.repository.create_prompt_document(request)
    }

    pub fn update_prompt_draft(
        &self,
        request: UpdatePromptDraftRequest,
    ) -> DomainResult<PromptDocumentView> {
        self.repository.update_prompt_draft(request)
    }

    pub fn save_prompt_version(
        &self,
        request: SavePromptVersionRequest,
    ) -> DomainResult<PromptVersionView> {
        self.repository.save_prompt_version(request)
    }

    pub fn list_prompt_documents(
        &self,
        request: ListPromptDocumentsRequest,
    ) -> DomainResult<Vec<PromptDocumentView>> {
        self.repository.list_prompt_documents(request)
    }

    pub fn list_prompt_versions(
        &self,
        request: ListPromptVersionsRequest,
    ) -> DomainResult<Vec<PromptVersionView>> {
        self.repository.list_prompt_versions(request)
    }

    pub fn list_prompt_output_history(
        &self,
        request: ListPromptOutputHistoryRequest,
    ) -> DomainResult<Vec<PromptOutputHistoryItem>> {
        self.repository.list_prompt_output_history(request)
    }

    pub fn save_as_prompt(
        &self,
        request: SavePromptAsPromptRequest,
    ) -> DomainResult<PromptDocumentView> {
        self.repository
            .create_prompt_document(CreatePromptDocumentRequest {
                library_path: request.library_path,
                name: request.name,
                draft_body: request.prompt,
                draft_negative_prompt: request.negative_prompt,
                draft_style_prompt: None,
                variables_schema_json: "[]".to_string(),
                default_values_json: "{}".to_string(),
                parameter_preset_json: "{}".to_string(),
                notes: request.notes,
            })
    }

    pub fn save_generation_prompt_as_prompt(
        &self,
        request: SaveGenerationPromptAsPromptRequest,
    ) -> DomainResult<PromptVersionView> {
        self.repository.save_generation_prompt_as_prompt(request)
    }

    pub fn render_prompt_run(
        &self,
        request: RenderPromptRunRequest,
    ) -> DomainResult<RenderPromptRunResult> {
        let values = parse_json_object("values_json", &request.values_json)?;
        let version = self
            .repository
            .load_prompt_version(LoadPromptVersionRequest {
                library_path: request.library_path,
                prompt_version_id: request.prompt_version_id,
            })?;
        let variables =
            parse_prompt_variables(&version.variables_schema_json, &version.default_values_json)?;

        let rendered_body = render_prompt_template(&version.body, &variables, &values)?;
        let rendered_style = version
            .style_prompt
            .as_deref()
            .map(|style| render_prompt_template(style, &variables, &values))
            .transpose()?
            .filter(|style| !style.trim().is_empty());
        let rendered_prompt = match rendered_style {
            Some(style) if !rendered_body.trim().is_empty() => {
                format!("{rendered_body}\n\n{style}")
            }
            Some(style) => style,
            None => rendered_body,
        };
        let rendered_negative_prompt = version
            .negative_prompt
            .as_deref()
            .map(|negative| render_prompt_template(negative, &variables, &values))
            .transpose()?
            .filter(|negative| !negative.trim().is_empty());

        Ok(RenderPromptRunResult {
            prompt_version_id: version.id,
            prompt_id: version.prompt_id,
            version_number: version.version_number,
            version_name: version.version_name,
            rendered_prompt,
            rendered_negative_prompt,
            values_json: request.values_json,
            parameter_preset_json: version.parameter_preset_json,
        })
    }
}

fn parse_prompt_variables(
    variables_schema_json: &str,
    default_values_json: &str,
) -> DomainResult<Vec<PromptTemplateVariable>> {
    let schema = parse_json_value("variables_schema_json", variables_schema_json)?;
    let default_values = parse_json_object("default_values_json", default_values_json)?;

    match schema {
        Value::Array(items) => variables_from_schema_array(&items, &default_values),
        Value::Object(fields) => {
            if let Some(variables) = fields.get("variables") {
                let items = variables.as_array().ok_or_else(|| {
                    DomainError::InvalidGenerationParameters {
                        message: "variables_schema_json.variables must be an array".to_string(),
                    }
                })?;
                variables_from_schema_array(items, &default_values)
            } else {
                fields
                    .iter()
                    .map(|(name, config)| {
                        variable_from_object_schema(name, config, &default_values)
                    })
                    .collect()
            }
        }
        _ => Err(DomainError::InvalidGenerationParameters {
            message: "variables_schema_json must be an array or object".to_string(),
        }),
    }
}

fn variables_from_schema_array(
    items: &[Value],
    default_values: &Value,
) -> DomainResult<Vec<PromptTemplateVariable>> {
    items
        .iter()
        .map(|item| variable_from_schema_item(item, default_values))
        .collect()
}

fn variable_from_schema_item(
    item: &Value,
    default_values: &Value,
) -> DomainResult<PromptTemplateVariable> {
    let object = item
        .as_object()
        .ok_or_else(|| DomainError::InvalidGenerationParameters {
            message: "variables_schema_json entries must be objects".to_string(),
        })?;
    let name = object
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| DomainError::InvalidGenerationParameters {
            message: "prompt variable name is required".to_string(),
        })?
        .trim()
        .to_string();

    Ok(PromptTemplateVariable {
        label: object
            .get("label")
            .and_then(Value::as_str)
            .map(str::to_string),
        required: object
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        default_value: prompt_default_value(&name, schema_default_value(object), default_values),
        name,
    })
}

fn variable_from_object_schema(
    name: &str,
    config: &Value,
    default_values: &Value,
) -> DomainResult<PromptTemplateVariable> {
    if name.trim().is_empty() {
        return Err(DomainError::InvalidGenerationParameters {
            message: "prompt variable name is required".to_string(),
        });
    }

    let label = config
        .as_object()
        .and_then(|object| object.get("label"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let required = config
        .as_object()
        .and_then(|object| object.get("required"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let schema_default = config.as_object().and_then(schema_default_value);

    Ok(PromptTemplateVariable {
        name: name.trim().to_string(),
        label,
        required,
        default_value: prompt_default_value(name, schema_default, default_values),
    })
}

fn schema_default_value(object: &serde_json::Map<String, Value>) -> Option<&Value> {
    object
        .get("defaultValue")
        .or_else(|| object.get("default_value"))
}

fn prompt_default_value(
    name: &str,
    schema_default: Option<&Value>,
    default_values: &Value,
) -> Option<String> {
    default_values
        .get(name)
        .and_then(prompt_value_to_string)
        .or_else(|| schema_default.and_then(prompt_value_to_string))
}

fn prompt_value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(value) => Some(value.clone()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        _ => Some(value.to_string()),
    }
}

fn parse_json_object(field: &str, value: &str) -> DomainResult<Value> {
    let value = parse_json_value(field, value)?;
    if value.is_object() {
        Ok(value)
    } else {
        Err(DomainError::InvalidGenerationParameters {
            message: format!("{field} must be a JSON object"),
        })
    }
}

fn parse_json_value(field: &str, value: &str) -> DomainResult<Value> {
    serde_json::from_str(value).map_err(|error| DomainError::InvalidGenerationParameters {
        message: format!("{field} must be valid JSON: {error}"),
    })
}
