use crate::application::ports::ScheduleRepository;
use crate::{
    CreateScheduledGenerationJobRequest, CreateScheduledGenerationRunRequest, DomainResult,
    LibraryId, LibrarySummary, ScheduledGenerationJobId, ScheduledGenerationJobStatus,
    ScheduledGenerationJobView, ScheduledGenerationRunView, UpdateScheduledGenerationJobRequest,
    UpdateScheduledGenerationRunRequest, UpsertScheduledGenerationRunOutputRequest,
};
use std::path::Path;

pub struct ScheduleUseCase<R> {
    repository: R,
}

impl<R> ScheduleUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> ScheduleUseCase<R>
where
    R: ScheduleRepository,
{
    pub fn create_job(
        &self,
        request: CreateScheduledGenerationJobRequest,
    ) -> DomainResult<ScheduledGenerationJobView> {
        self.repository.create_scheduled_generation_job(request)
    }

    pub fn update_job(
        &self,
        request: UpdateScheduledGenerationJobRequest,
    ) -> DomainResult<ScheduledGenerationJobView> {
        self.repository.update_scheduled_generation_job(request)
    }

    pub fn list_jobs(&self, library_path: &Path) -> DomainResult<Vec<ScheduledGenerationJobView>> {
        self.repository.list_scheduled_generation_jobs(library_path)
    }

    pub fn list_due_jobs(
        &self,
        library_path: &Path,
        now: &str,
    ) -> DomainResult<Vec<ScheduledGenerationJobView>> {
        self.repository
            .list_due_scheduled_generation_jobs(library_path, now)
    }

    pub fn set_job_status(
        &self,
        library_path: &Path,
        job_id: &ScheduledGenerationJobId,
        status: ScheduledGenerationJobStatus,
    ) -> DomainResult<ScheduledGenerationJobView> {
        self.repository
            .set_scheduled_generation_job_status(library_path, job_id, status)
    }

    pub fn delete_job(
        &self,
        library_path: &Path,
        job_id: &ScheduledGenerationJobId,
    ) -> DomainResult<()> {
        self.repository
            .delete_scheduled_generation_job(library_path, job_id)
    }

    pub fn create_run(
        &self,
        request: CreateScheduledGenerationRunRequest,
    ) -> DomainResult<ScheduledGenerationRunView> {
        self.repository.create_scheduled_generation_run(request)
    }

    pub fn update_run(
        &self,
        request: UpdateScheduledGenerationRunRequest,
    ) -> DomainResult<ScheduledGenerationRunView> {
        self.repository.update_scheduled_generation_run(request)
    }

    pub fn list_runs(
        &self,
        library_path: &Path,
        job_id: &ScheduledGenerationJobId,
    ) -> DomainResult<Vec<ScheduledGenerationRunView>> {
        self.repository
            .list_scheduled_generation_runs(library_path, job_id)
    }

    pub fn upsert_run_output(
        &self,
        request: UpsertScheduledGenerationRunOutputRequest,
    ) -> DomainResult<crate::ScheduledGenerationRunOutputView> {
        self.repository
            .upsert_scheduled_generation_run_output(request)
    }

    pub fn set_library_automation_enabled(
        &self,
        library_id: &LibraryId,
        enabled: bool,
    ) -> DomainResult<LibrarySummary> {
        self.repository
            .set_library_automation_enabled(library_id, enabled)
    }
}
