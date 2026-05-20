use crate::{DomainError, TaskErrorClassification, TaskId, TaskStatus, TaskSummary};
use std::collections::BTreeMap;

pub const DEFAULT_GLOBAL_CONCURRENCY_LIMIT: usize = 2;
pub const DEFAULT_CODEX_CLI_CONCURRENCY_LIMIT: usize = 1;
pub const DEFAULT_FAKE_CONCURRENCY_LIMIT: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskSchedulerConfig {
    pub global_concurrency_limit: usize,
    pub provider_concurrency_limits: BTreeMap<String, usize>,
}

impl Default for TaskSchedulerConfig {
    fn default() -> Self {
        let mut provider_concurrency_limits = BTreeMap::new();
        provider_concurrency_limits
            .insert("codex-cli".to_string(), DEFAULT_CODEX_CLI_CONCURRENCY_LIMIT);
        provider_concurrency_limits.insert("fake".to_string(), DEFAULT_FAKE_CONCURRENCY_LIMIT);
        Self {
            global_concurrency_limit: DEFAULT_GLOBAL_CONCURRENCY_LIMIT,
            provider_concurrency_limits,
        }
    }
}

impl TaskSchedulerConfig {
    pub fn provider_limit(&self, provider: &str) -> usize {
        self.provider_concurrency_limits
            .get(provider)
            .copied()
            .unwrap_or(self.global_concurrency_limit)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskWaitReason {
    pub task_id: TaskId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskSchedulerDecision {
    pub selected_task_id: Option<TaskId>,
    pub wait_reasons: Vec<TaskWaitReason>,
}

pub fn evaluate_scheduler(
    tasks: &[TaskSummary],
    config: &TaskSchedulerConfig,
    now: &str,
) -> TaskSchedulerDecision {
    let running_tasks: Vec<&TaskSummary> = tasks
        .iter()
        .filter(|task| {
            matches!(
                task.status,
                TaskStatus::Running | TaskStatus::CancelRequested
            )
        })
        .collect();
    let running_count = running_tasks.len();
    let mut running_by_provider = BTreeMap::<String, usize>::new();
    for task in running_tasks {
        if let Some(provider) = &task.provider {
            *running_by_provider.entry(provider.clone()).or_default() += 1;
        }
    }

    let mut wait_reasons = Vec::new();
    let mut queued_tasks: Vec<&TaskSummary> = tasks
        .iter()
        .filter(|task| task.status == TaskStatus::Queued)
        .collect();
    queued_tasks.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then(left.queue_position.cmp(&right.queue_position))
            .then(left.created_at.cmp(&right.created_at))
    });

    let mut selected_task_id = None;
    for task in queued_tasks {
        if running_count >= config.global_concurrency_limit {
            wait_reasons.push(TaskWaitReason {
                task_id: task.id.clone(),
                reason: "Waiting for global concurrency slot".to_string(),
            });
            continue;
        }
        if let Some(provider) = &task.provider {
            let running_for_provider = running_by_provider.get(provider).copied().unwrap_or(0);
            if running_for_provider >= config.provider_limit(provider) {
                wait_reasons.push(TaskWaitReason {
                    task_id: task.id.clone(),
                    reason: format!("Waiting for {provider} slot"),
                });
                continue;
            }
        }
        if selected_task_id.is_none() {
            selected_task_id = Some(task.id.clone());
        }
    }

    for task in tasks
        .iter()
        .filter(|task| task.status == TaskStatus::RetryWaiting)
    {
        if let Some(next_retry_at) = &task.next_retry_at {
            if next_retry_at.as_str() > now {
                wait_reasons.push(TaskWaitReason {
                    task_id: task.id.clone(),
                    reason: "Waiting until retry backoff expires".to_string(),
                });
            }
        }
    }

    TaskSchedulerDecision {
        selected_task_id,
        wait_reasons,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_seconds: Vec<u64>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_seconds: vec![30, 120, 300],
        }
    }
}

impl RetryPolicy {
    pub fn should_auto_retry(
        &self,
        classification: TaskErrorClassification,
        attempt_count: u32,
    ) -> bool {
        classification == TaskErrorClassification::Transient && attempt_count < self.max_attempts
    }

    pub fn backoff_delay_seconds(&self, attempt_count: u32) -> u64 {
        let index = attempt_count.saturating_sub(1) as usize;
        self.backoff_seconds
            .get(index)
            .copied()
            .or_else(|| self.backoff_seconds.last().copied())
            .unwrap_or(30)
    }
}

pub fn classify_task_error(error: &DomainError) -> TaskErrorClassification {
    match error {
        DomainError::GenerationFailed { message, .. } => {
            let lower = message.to_ascii_lowercase();
            if lower.contains("timeout")
                || lower.contains("rate limit")
                || lower.contains("temporar")
                || lower.contains("network")
            {
                TaskErrorClassification::Transient
            } else {
                TaskErrorClassification::RetryableManual
            }
        }
        DomainError::ProviderUnavailable { .. } | DomainError::Io { .. } => {
            TaskErrorClassification::Transient
        }
        DomainError::CredentialMissing { .. } => TaskErrorClassification::RetryableManual,
        DomainError::ConcurrentWriteConflict { .. } => TaskErrorClassification::Conflict,
        DomainError::UnsupportedProvider { .. }
        | DomainError::UnsupportedProviderCapability { .. }
        | DomainError::InvalidGenerationParameters { .. }
        | DomainError::InvalidAssetReference { .. }
        | DomainError::InvalidTaskReference { .. }
        | DomainError::InvalidSmartAlbumQuery { .. }
        | DomainError::InvalidGalleryQuery { .. }
        | DomainError::InvalidLibraryBackup { .. }
        | DomainError::InvalidLibraryAlias { .. }
        | DomainError::ImportDestinationNotEmpty { .. }
        | DomainError::SchemaMismatch { .. }
        | DomainError::FileIntegrityMismatch { .. }
        | DomainError::LibraryNotFound { .. }
        | DomainError::ZipIoError { .. }
        | DomainError::Database { .. }
        | DomainError::Serialization { .. } => TaskErrorClassification::Final,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LibraryId, TaskType};

    fn task(
        id: &str,
        status: TaskStatus,
        provider: Option<&str>,
        priority: i64,
        queue_position: i64,
    ) -> TaskSummary {
        TaskSummary {
            id: TaskId(id.to_string()),
            library_id: LibraryId("library".to_string()),
            task_type: TaskType::ImageGeneration,
            status,
            queue_position,
            priority,
            provider: provider.map(ToString::to_string),
            operation: None,
            concurrency_group: provider.map(ToString::to_string),
            attempt_count: 0,
            max_attempts: 3,
            next_retry_at: None,
            input_json: "{}".to_string(),
            created_at: format!("2026-01-01T00:00:0{queue_position}Z"),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            last_error_code: None,
            last_error_message: None,
            error_classification: None,
            wait_reason: None,
        }
    }

    #[test]
    fn scheduler_prefers_priority_then_queue_position() {
        let tasks = vec![
            task("low", TaskStatus::Queued, Some("fake"), 0, 1),
            task("high", TaskStatus::Queued, Some("fake"), 10, 2),
            task("next", TaskStatus::Queued, Some("fake"), 10, 1),
        ];

        let decision = evaluate_scheduler(
            &tasks,
            &TaskSchedulerConfig::default(),
            "2026-01-01T00:00:00Z",
        );

        assert_eq!(decision.selected_task_id, Some(TaskId("next".to_string())));
    }

    #[test]
    fn scheduler_reports_global_and_provider_wait_reasons() {
        let mut config = TaskSchedulerConfig {
            global_concurrency_limit: 1,
            ..Default::default()
        };
        let global_full = vec![
            task("running", TaskStatus::Running, Some("fake"), 0, 1),
            task("queued", TaskStatus::Queued, Some("fake"), 0, 2),
        ];
        let decision = evaluate_scheduler(&global_full, &config, "2026-01-01T00:00:00Z");
        assert_eq!(decision.selected_task_id, None);
        assert_eq!(
            decision.wait_reasons[0].reason,
            "Waiting for global concurrency slot"
        );

        config.global_concurrency_limit = 2;
        config
            .provider_concurrency_limits
            .insert("codex-cli".to_string(), 1);
        let provider_full = vec![
            task("running", TaskStatus::Running, Some("codex-cli"), 0, 1),
            task("queued", TaskStatus::Queued, Some("codex-cli"), 0, 2),
        ];
        let decision = evaluate_scheduler(&provider_full, &config, "2026-01-01T00:00:00Z");
        assert_eq!(decision.selected_task_id, None);
        assert_eq!(
            decision.wait_reasons[0].reason,
            "Waiting for codex-cli slot"
        );
    }

    #[test]
    fn retry_policy_only_auto_retries_transient_before_limit() {
        let policy = RetryPolicy::default();

        assert!(policy.should_auto_retry(TaskErrorClassification::Transient, 1));
        assert!(!policy.should_auto_retry(TaskErrorClassification::Transient, 3));
        assert!(!policy.should_auto_retry(TaskErrorClassification::Final, 1));
        assert_eq!(policy.backoff_delay_seconds(1), 30);
        assert_eq!(policy.backoff_delay_seconds(2), 120);
        assert_eq!(policy.backoff_delay_seconds(99), 300);
    }

    #[test]
    fn classifies_generation_failures_by_message() {
        assert_eq!(
            classify_task_error(&DomainError::GenerationFailed {
                provider: "fake".to_string(),
                message: "temporary network timeout".to_string(),
            }),
            TaskErrorClassification::Transient
        );
        assert_eq!(
            classify_task_error(&DomainError::UnsupportedProviderCapability {
                provider: "fake".to_string(),
                capability: "image_to_image".to_string(),
            }),
            TaskErrorClassification::Final
        );
    }
}
