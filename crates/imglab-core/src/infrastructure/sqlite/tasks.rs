use crate::application::ports::TaskRepository;
use crate::library::LocalLibraryService;
use crate::{
    AppendTaskAttemptRequest, AppendTaskEventRequest, AppendTaskOutputRequest,
    BatchCreateTasksRequest, CompleteTaskAttemptRequest, DomainResult, ReorderQueuedTasksRequest,
    TaskAttempt, TaskDetail, TaskEvent, TaskId, TaskOutput, TaskOutputType, TaskService,
    TaskSummary, UpdateTaskStatusRequest,
};
use std::path::Path;

impl TaskRepository for LocalLibraryService {
    fn create_tasks(&self, request: BatchCreateTasksRequest) -> DomainResult<Vec<TaskSummary>> {
        TaskService::create_tasks(self, request)
    }

    fn list_tasks(&self, library_path: &Path) -> DomainResult<Vec<TaskSummary>> {
        TaskService::list_tasks(self, library_path)
    }

    fn get_task_detail(&self, library_path: &Path, task_id: &TaskId) -> DomainResult<TaskDetail> {
        TaskService::get_task_detail(self, library_path, task_id)
    }

    fn claim_queued_task(
        &self,
        library_path: &Path,
        task_id: &TaskId,
    ) -> DomainResult<Option<TaskSummary>> {
        TaskService::claim_queued_task(self, library_path, task_id)
    }

    fn update_task_status(&self, request: UpdateTaskStatusRequest) -> DomainResult<TaskSummary> {
        TaskService::update_task_status(self, request)
    }

    fn append_task_event(&self, request: AppendTaskEventRequest) -> DomainResult<TaskEvent> {
        TaskService::append_task_event(self, request)
    }

    fn append_task_attempt(&self, request: AppendTaskAttemptRequest) -> DomainResult<TaskAttempt> {
        TaskService::append_task_attempt(self, request)
    }

    fn complete_task_attempt(
        &self,
        request: CompleteTaskAttemptRequest,
    ) -> DomainResult<TaskAttempt> {
        TaskService::complete_task_attempt(self, request)
    }

    fn append_task_output(&self, request: AppendTaskOutputRequest) -> DomainResult<TaskOutput> {
        TaskService::append_task_output(self, request)
    }

    fn has_task_output(
        &self,
        library_path: &Path,
        task_id: &TaskId,
        output_type: TaskOutputType,
        target_id: &str,
    ) -> DomainResult<bool> {
        TaskService::has_task_output(self, library_path, task_id, output_type, target_id)
    }

    fn reorder_queued_tasks(&self, request: ReorderQueuedTasksRequest) -> DomainResult<()> {
        TaskService::reorder_queued_tasks(self, request)
    }

    fn retry_task(&self, library_path: &Path, task_id: &TaskId) -> DomainResult<TaskSummary> {
        TaskService::retry_task(self, library_path, task_id)
    }

    fn duplicate_task(&self, library_path: &Path, task_id: &TaskId) -> DomainResult<TaskSummary> {
        TaskService::duplicate_task(self, library_path, task_id)
    }
}
