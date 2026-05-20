use super::{database_error, LocalLibraryService};
use crate::{
    DaemonStatusView, DiagnosticsOverviewView, DomainResult, GenerationOperation, LibraryService,
    ProviderHealthSummaryView, StudioOverviewView, StudioTaskSummaryView, TaskService, TaskStatus,
};
use std::path::Path;

impl LocalLibraryService {
    pub(super) fn studio_diagnostics_overview(
        &self,
        root_path: &Path,
    ) -> DomainResult<StudioOverviewView> {
        Self::validate_layout(root_path)?;
        let manifest = Self::read_manifest(root_path)?;
        let library = Self::summary_from_manifest(root_path, &manifest, false);
        let status = self.library_status(root_path)?;
        let (registered_library_count, missing_library_count) = self.registry_counts()?;
        let review_pending_count = count_pending_reviews(root_path)?;
        let task_summary = summarize_tasks(&self.list_tasks(root_path)?);

        Ok(StudioOverviewView {
            library,
            status,
            registered_library_count,
            missing_library_count,
            review_pending_count,
            task_summary,
            provider_health: default_provider_health(),
        })
    }

    pub(super) fn library_diagnostics_overview(
        &self,
        root_path: &Path,
    ) -> DomainResult<DiagnosticsOverviewView> {
        Self::validate_layout(root_path)?;
        let status = self.library_status(root_path)?;
        let (library_count, missing_library_count) = self.registry_counts()?;
        Ok(DiagnosticsOverviewView {
            provider_health: default_provider_health(),
            daemon_status: DaemonStatusView {
                state: "not_checked".to_string(),
                recoverable_error: None,
            },
            library_status: status,
            library_count,
            missing_library_count,
        })
    }
}

fn count_pending_reviews(root_path: &Path) -> DomainResult<u32> {
    let connection = LocalLibraryService::open_library_database(root_path)?;
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM metadata_suggestions WHERE status = 'pending_review'",
            [],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    Ok(count as u32)
}

fn summarize_tasks(tasks: &[crate::TaskSummary]) -> StudioTaskSummaryView {
    let mut queued_count = 0;
    let mut running_count = 0;
    let mut retry_waiting_count = 0;
    let mut failed_count = 0;

    for task in tasks {
        match task.status {
            TaskStatus::Queued => queued_count += 1,
            TaskStatus::Running | TaskStatus::CancelRequested => running_count += 1,
            TaskStatus::RetryWaiting => retry_waiting_count += 1,
            TaskStatus::FailedRetryable
            | TaskStatus::FailedFinal
            | TaskStatus::InterruptedFinal => {
                failed_count += 1;
            }
            TaskStatus::InterruptedRetryable => retry_waiting_count += 1,
            TaskStatus::Canceled | TaskStatus::Completed => {}
        }
    }

    StudioTaskSummaryView {
        active_count: queued_count + running_count + retry_waiting_count,
        queued_count,
        running_count,
        retry_waiting_count,
        failed_count,
    }
}

fn default_provider_health() -> Vec<ProviderHealthSummaryView> {
    vec![
        ProviderHealthSummaryView {
            provider: "fake".to_string(),
            display_name: "Fake".to_string(),
            availability: "available".to_string(),
            credential_state: "not_required".to_string(),
            supported_operations: vec![GenerationOperation::TextToImage],
            recoverable_error: None,
        },
        ProviderHealthSummaryView {
            provider: "codex-cli".to_string(),
            display_name: "Codex CLI".to_string(),
            availability: "not_checked".to_string(),
            credential_state: "external".to_string(),
            supported_operations: vec![
                GenerationOperation::TextToImage,
                GenerationOperation::ImageToImage,
            ],
            recoverable_error: None,
        },
        ProviderHealthSummaryView {
            provider: "grok".to_string(),
            display_name: "Grok".to_string(),
            availability: "not_configured".to_string(),
            credential_state: "missing".to_string(),
            supported_operations: vec![GenerationOperation::TextToImage],
            recoverable_error: Some("native provider client is deferred".to_string()),
        },
    ]
}
