use crate::domain::shared::{AlbumId, AssetId, AssetVersionId, GenerationEventId, TaskId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulePromptMode {
    Fixed,
    Dynamic,
}

impl SchedulePromptMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixed => "fixed",
            Self::Dynamic => "dynamic",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "fixed" => Some(Self::Fixed),
            "dynamic" => Some(Self::Dynamic),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduledGenerationJobStatus {
    Active,
    Paused,
    Disabled,
    Deleted,
}

impl ScheduledGenerationJobStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Disabled => "disabled",
            Self::Deleted => "deleted",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "paused" => Some(Self::Paused),
            "disabled" => Some(Self::Disabled),
            "deleted" => Some(Self::Deleted),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduledGenerationRunStatus {
    Pending,
    ExpandingPrompt,
    TaskQueued,
    TaskRunning,
    PostProcessing,
    Completed,
    Skipped,
    Failed,
}

impl ScheduledGenerationRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::ExpandingPrompt => "expanding_prompt",
            Self::TaskQueued => "task_queued",
            Self::TaskRunning => "task_running",
            Self::PostProcessing => "post_processing",
            Self::Completed => "completed",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "expanding_prompt" => Some(Self::ExpandingPrompt),
            "task_queued" => Some(Self::TaskQueued),
            "task_running" => Some(Self::TaskRunning),
            "post_processing" => Some(Self::PostProcessing),
            "completed" => Some(Self::Completed),
            "skipped" => Some(Self::Skipped),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleOverlapPolicy {
    Skip,
}

impl ScheduleOverlapPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Skip => "skip",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "skip" => Some(Self::Skip),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleMissedRunPolicy {
    NoCatchUp,
}

impl ScheduleMissedRunPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoCatchUp => "no_catch_up",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "no_catch_up" => Some(Self::NoCatchUp),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleRule {
    IntervalMinutes(u32),
    IntervalHours(u32),
    DailyTime {
        timezone_id: String,
        local_time_hh_mm: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledGenerationJob {
    pub id: String,
    pub library_id: String,
    pub name: String,
    pub status: ScheduledGenerationJobStatus,
    pub prompt_mode: SchedulePromptMode,
    pub fixed_prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub base_prompt: Option<String>,
    pub dynamic_prompt: Option<String>,
    pub prompt_expander_provider: Option<String>,
    pub prompt_expander_model: Option<String>,
    pub image_provider: String,
    pub image_model: String,
    pub parameters_json: String,
    pub schedule_rule: ScheduleRule,
    pub target_album_id: AlbumId,
    pub tags: Vec<String>,
    pub overlap_policy: ScheduleOverlapPolicy,
    pub missed_run_policy: ScheduleMissedRunPolicy,
    pub last_run_at: Option<String>,
    pub next_run_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledGenerationRun {
    pub id: String,
    pub job_id: String,
    pub library_id: String,
    pub status: ScheduledGenerationRunStatus,
    pub scheduled_for: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub skip_reason: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub expanded_prompt: Option<String>,
    pub prompt_expansion_provider_metadata_json: Option<String>,
    pub image_task_id: Option<TaskId>,
    pub output_asset_count: u32,
    pub tagged_asset_count: u32,
    pub album_added_asset_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledGenerationRunOutput {
    pub run_id: String,
    pub asset_id: AssetId,
    pub asset_version_id: Option<AssetVersionId>,
    pub generation_event_id: Option<GenerationEventId>,
    pub album_added: bool,
    pub tags_applied: Vec<String>,
}
