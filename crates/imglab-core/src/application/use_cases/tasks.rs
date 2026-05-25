use crate::application::ports::TaskRepository;
use crate::{
    AppendTaskAttemptRequest, AppendTaskEventRequest, AppendTaskOutputRequest,
    BatchCreateTasksRequest, CompleteTaskAttemptRequest, DomainResult, ReorderQueuedTasksRequest,
    TaskAttempt, TaskDetail, TaskEvent, TaskId, TaskOutput, TaskOutputType, TaskSummary,
    UpdateTaskStatusRequest,
};
use std::path::Path;

pub struct TaskUseCase<R> {
    repository: R,
}

impl<R> TaskUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> TaskUseCase<R>
where
    R: TaskRepository,
{
    pub fn create_tasks(&self, request: BatchCreateTasksRequest) -> DomainResult<Vec<TaskSummary>> {
        self.repository.create_tasks(request)
    }

    pub fn list_tasks(&self, library_path: &Path) -> DomainResult<Vec<TaskSummary>> {
        self.repository.list_tasks(library_path)
    }

    pub fn get_task_detail(
        &self,
        library_path: &Path,
        task_id: &TaskId,
    ) -> DomainResult<TaskDetail> {
        self.repository.get_task_detail(library_path, task_id)
    }

    pub fn claim_queued_task(
        &self,
        library_path: &Path,
        task_id: &TaskId,
    ) -> DomainResult<Option<TaskSummary>> {
        self.repository.claim_queued_task(library_path, task_id)
    }

    pub fn update_task_status(
        &self,
        request: UpdateTaskStatusRequest,
    ) -> DomainResult<TaskSummary> {
        self.repository.update_task_status(request)
    }

    pub fn append_task_event(&self, request: AppendTaskEventRequest) -> DomainResult<TaskEvent> {
        self.repository.append_task_event(request)
    }

    pub fn append_task_attempt(
        &self,
        request: AppendTaskAttemptRequest,
    ) -> DomainResult<TaskAttempt> {
        self.repository.append_task_attempt(request)
    }

    pub fn complete_task_attempt(
        &self,
        request: CompleteTaskAttemptRequest,
    ) -> DomainResult<TaskAttempt> {
        self.repository.complete_task_attempt(request)
    }

    pub fn append_task_output(&self, request: AppendTaskOutputRequest) -> DomainResult<TaskOutput> {
        self.repository.append_task_output(request)
    }

    pub fn has_task_output(
        &self,
        library_path: &Path,
        task_id: &TaskId,
        output_type: TaskOutputType,
        target_id: &str,
    ) -> DomainResult<bool> {
        self.repository
            .has_task_output(library_path, task_id, output_type, target_id)
    }

    pub fn reorder_queued_tasks(&self, request: ReorderQueuedTasksRequest) -> DomainResult<()> {
        self.repository.reorder_queued_tasks(request)
    }

    pub fn retry_task(&self, library_path: &Path, task_id: &TaskId) -> DomainResult<TaskSummary> {
        self.repository.retry_task(library_path, task_id)
    }

    pub fn duplicate_task(
        &self,
        library_path: &Path,
        task_id: &TaskId,
    ) -> DomainResult<TaskSummary> {
        self.repository.duplicate_task(library_path, task_id)
    }
}
