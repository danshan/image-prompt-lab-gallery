pub use crate::dto::{TaskAttempt, TaskDetail, TaskEvent, TaskOutput, TaskSummary};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    ImageGeneration,
    MetadataFieldGeneration,
    MetadataSuggestionGeneration,
}

impl TaskType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ImageGeneration => "image_generation",
            Self::MetadataFieldGeneration => "metadata_field_generation",
            Self::MetadataSuggestionGeneration => "metadata_suggestion_generation",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "image_generation" => Some(Self::ImageGeneration),
            "metadata_field_generation" => Some(Self::MetadataFieldGeneration),
            "metadata_suggestion_generation" => Some(Self::MetadataSuggestionGeneration),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Queued,
    Running,
    RetryWaiting,
    FailedRetryable,
    FailedFinal,
    CancelRequested,
    Canceled,
    Completed,
    InterruptedRetryable,
    InterruptedFinal,
}

impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::RetryWaiting => "retry_waiting",
            Self::FailedRetryable => "failed_retryable",
            Self::FailedFinal => "failed_final",
            Self::CancelRequested => "cancel_requested",
            Self::Canceled => "canceled",
            Self::Completed => "completed",
            Self::InterruptedRetryable => "interrupted_retryable",
            Self::InterruptedFinal => "interrupted_final",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "queued" => Some(Self::Queued),
            "running" => Some(Self::Running),
            "retry_waiting" => Some(Self::RetryWaiting),
            "failed_retryable" => Some(Self::FailedRetryable),
            "failed_final" => Some(Self::FailedFinal),
            "cancel_requested" => Some(Self::CancelRequested),
            "canceled" => Some(Self::Canceled),
            "completed" => Some(Self::Completed),
            "interrupted_retryable" => Some(Self::InterruptedRetryable),
            "interrupted_final" => Some(Self::InterruptedFinal),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskErrorClassification {
    Transient,
    RetryableManual,
    Final,
    Cancel,
    Conflict,
}

impl TaskErrorClassification {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Transient => "transient",
            Self::RetryableManual => "retryable_manual",
            Self::Final => "final",
            Self::Cancel => "cancel",
            Self::Conflict => "conflict",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "transient" => Some(Self::Transient),
            "retryable_manual" => Some(Self::RetryableManual),
            "final" => Some(Self::Final),
            "cancel" => Some(Self::Cancel),
            "conflict" => Some(Self::Conflict),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskOutputType {
    Asset,
    AssetVersion,
    GenerationEvent,
    MetadataSuggestion,
    MetadataFieldResult,
}

impl TaskOutputType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asset => "asset",
            Self::AssetVersion => "asset_version",
            Self::GenerationEvent => "generation_event",
            Self::MetadataSuggestion => "metadata_suggestion",
            Self::MetadataFieldResult => "metadata_field_result",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "asset" => Some(Self::Asset),
            "asset_version" => Some(Self::AssetVersion),
            "generation_event" => Some(Self::GenerationEvent),
            "metadata_suggestion" => Some(Self::MetadataSuggestion),
            "metadata_field_result" => Some(Self::MetadataFieldResult),
            _ => None,
        }
    }
}
