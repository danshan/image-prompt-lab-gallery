use crate::{PromptId, PromptVersionId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptDocumentKind {
    Draft,
    Template,
}

impl PromptDocumentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Template => "template",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "draft" => Some(Self::Draft),
            "template" => Some(Self::Template),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptDocumentStatus {
    Active,
    Archived,
}

impl PromptDocumentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptTemplateVariable {
    pub name: String,
    pub label: Option<String>,
    pub required: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptDocumentSummary {
    pub id: PromptId,
    pub name: String,
    pub kind: PromptDocumentKind,
    pub status: PromptDocumentStatus,
    pub draft_body: String,
    pub draft_negative_prompt: Option<String>,
    pub draft_style_prompt: Option<String>,
    pub variables: Vec<PromptTemplateVariable>,
    pub parameter_preset_json: String,
    pub notes: Option<String>,
    pub latest_version_id: Option<PromptVersionId>,
    pub latest_version_number: Option<u32>,
    pub latest_version_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptVersionSummary {
    pub id: PromptVersionId,
    pub prompt_id: PromptId,
    pub version_number: u32,
    pub version_name: String,
    pub body: String,
    pub negative_prompt: Option<String>,
    pub style_prompt: Option<String>,
    pub variables: Vec<PromptTemplateVariable>,
    pub parameter_preset_json: String,
    pub notes: Option<String>,
    pub created_at: String,
}
