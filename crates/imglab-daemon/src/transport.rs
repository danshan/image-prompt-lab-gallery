use crate::executors::*;
use crate::routes::*;
use crate::runtime::*;
use crate::scheduler::execute_task;
use crate::views::*;
use crate::*;

pub fn spawn_scheduler_loop(
    state: SharedDaemonState,
    config: TaskSchedulerConfig,
    retry_policy: RetryPolicy,
    interval: Duration,
) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        if let Err(error) = run_scheduler_loop_iteration(&state, &config, &retry_policy) {
            eprintln!("scheduler tick failed: {error}");
        }
        thread::sleep(interval);
    })
}

pub fn run_scheduler_loop_iteration(
    state: &SharedDaemonState,
    config: &TaskSchedulerConfig,
    retry_policy: &RetryPolicy,
) -> DomainResult<Option<TaskSummary>> {
    let guard = state
        .lock()
        .map_err(|_| DomainError::ConcurrentWriteConflict {
            message: "daemon state lock is poisoned".to_string(),
        })?;
    if guard.opened_libraries.is_empty() || !daemon_has_runnable_work(&guard)? {
        return Ok(None);
    }
    let mut snapshot = guard.clone();
    drop(guard);
    run_scheduler_tick(&mut snapshot, config, retry_policy)
}

pub(crate) fn daemon_has_runnable_work(state: &DaemonState) -> DomainResult<bool> {
    let now = unix_timestamp_string(0);
    for library_path in state.opened_libraries.values() {
        for task in state.service.list_tasks(library_path)? {
            match task.status {
                TaskStatus::Queued => return Ok(true),
                TaskStatus::RetryWaiting
                    if task
                        .next_retry_at
                        .as_deref()
                        .is_none_or(|next_retry_at| next_retry_at <= now.as_str()) =>
                {
                    return Ok(true);
                }
                _ => {}
            }
        }
    }
    Ok(false)
}

pub fn recover_open_libraries(
    state: &mut DaemonState,
    retry_policy: &RetryPolicy,
) -> DomainResult<Vec<TaskSummary>> {
    let mut recovered = Vec::new();
    let now = unix_timestamp_string(0);
    for library_path in state.opened_libraries.values().cloned().collect::<Vec<_>>() {
        for task in state.service.list_tasks(&library_path)? {
            match task.status {
                TaskStatus::Queued => {}
                TaskStatus::RetryWaiting => {
                    if task
                        .next_retry_at
                        .as_deref()
                        .is_none_or(|next_retry_at| next_retry_at <= now.as_str())
                    {
                        state
                            .service
                            .append_task_event(imglab_core::AppendTaskEventRequest {
                                library_path: library_path.clone(),
                                task_id: task.id.clone(),
                                event_type: "recovery_retry_ready".to_string(),
                                message: Some(
                                    "Retry backoff elapsed during daemon downtime".to_string(),
                                ),
                                payload_json: None,
                            })?;
                        recovered.push(state.service.update_task_status(
                            UpdateTaskStatusRequest {
                                library_path: library_path.clone(),
                                task_id: task.id.clone(),
                                status: TaskStatus::Queued,
                                next_retry_at: None,
                                last_error_code: task.last_error_code,
                                last_error_message: task.last_error_message,
                                error_classification: task.error_classification,
                                wait_reason: None,
                            },
                        )?);
                    }
                }
                TaskStatus::Running | TaskStatus::CancelRequested => {
                    let detail = state.service.get_task_detail(&library_path, &task.id)?;
                    if !detail.outputs.is_empty() {
                        state
                            .service
                            .append_task_event(imglab_core::AppendTaskEventRequest {
                                library_path: library_path.clone(),
                                task_id: task.id.clone(),
                                event_type: "recovery_reconciled_completed".to_string(),
                                message: Some(
                                    "Task had committed output links before daemon recovery"
                                        .to_string(),
                                ),
                                payload_json: None,
                            })?;
                        recovered.push(state.service.update_task_status(
                            UpdateTaskStatusRequest {
                                library_path: library_path.clone(),
                                task_id: task.id.clone(),
                                status: TaskStatus::Completed,
                                next_retry_at: None,
                                last_error_code: None,
                                last_error_message: None,
                                error_classification: None,
                                wait_reason: None,
                            },
                        )?);
                    } else {
                        let retryable = task.attempt_count < retry_policy.max_attempts;
                        let status = if retryable {
                            TaskStatus::InterruptedRetryable
                        } else {
                            TaskStatus::InterruptedFinal
                        };
                        state
                            .service
                            .append_task_event(imglab_core::AppendTaskEventRequest {
                                library_path: library_path.clone(),
                                task_id: task.id.clone(),
                                event_type: "recovery_interrupted".to_string(),
                                message: Some("Task was running when daemon stopped".to_string()),
                                payload_json: Some(format!("{{\"retryable\":{retryable}}}")),
                            })?;
                        recovered.push(state.service.update_task_status(
                            UpdateTaskStatusRequest {
                                library_path: library_path.clone(),
                                task_id: task.id.clone(),
                                status,
                                next_retry_at: None,
                                last_error_code: Some("DaemonInterrupted".to_string()),
                                last_error_message: Some(
                                    "Task was running when daemon stopped".to_string(),
                                ),
                                error_classification: Some(if retryable {
                                    TaskErrorClassification::Transient
                                } else {
                                    TaskErrorClassification::Final
                                }),
                                wait_reason: None,
                            },
                        )?);
                    }
                }
                TaskStatus::FailedRetryable
                | TaskStatus::FailedFinal
                | TaskStatus::Canceled
                | TaskStatus::Completed
                | TaskStatus::InterruptedRetryable
                | TaskStatus::InterruptedFinal => {}
            }
        }
    }
    Ok(recovered)
}

pub fn run_scheduler_tick(
    state: &mut DaemonState,
    config: &TaskSchedulerConfig,
    retry_policy: &RetryPolicy,
) -> DomainResult<Option<TaskSummary>> {
    let mut all_tasks = Vec::new();
    let mut task_libraries = BTreeMap::new();
    for library_path in state.opened_libraries.values() {
        for task in state.service.list_tasks(library_path)? {
            task_libraries.insert(task.id.0.clone(), library_path.clone());
            all_tasks.push(task);
        }
    }

    let now = unix_timestamp_string(0);
    let decision = evaluate_scheduler(&all_tasks, config, &now);
    for wait_reason in decision.wait_reasons {
        if let Some(library_path) = task_libraries.get(&wait_reason.task_id.0) {
            let detail = state
                .service
                .get_task_detail(library_path, &wait_reason.task_id)?;
            state.service.update_task_status(UpdateTaskStatusRequest {
                library_path: library_path.clone(),
                task_id: wait_reason.task_id,
                status: detail.task.status,
                next_retry_at: detail.task.next_retry_at,
                last_error_code: detail.task.last_error_code,
                last_error_message: detail.task.last_error_message,
                error_classification: detail.task.error_classification,
                wait_reason: Some(wait_reason.reason),
            })?;
        }
    }

    let Some(task_id) = decision.selected_task_id else {
        return Ok(None);
    };
    let library_path = task_libraries.get(&task_id.0).cloned().ok_or_else(|| {
        DomainError::InvalidTaskReference {
            id: task_id.0.clone(),
        }
    })?;
    execute_task(state, &library_path, &task_id, retry_policy).map(Some)
}

pub(crate) fn route_request(
    method: &str,
    path: &str,
    query: Option<&str>,
    body: &str,
    state: &mut DaemonState,
) -> DomainResult<Option<HttpResponse>> {
    match (method, path) {
        ("POST", LIBRARY_OPEN_PATH) => {
            let input: OpenLibraryInput = parse_json_body(body)?;
            let library = state.open_library(&input.library_path)?;
            Ok(Some(json_response(200, &LibraryView::from(library))))
        }
        ("POST", TASKS_PATH) => {
            let input: SingleCreateTaskInput = parse_json_body(body)?;
            let library_path = state.library_path(&input.library_id)?;
            let tasks = state.service.create_tasks(BatchCreateTasksRequest {
                library_path,
                library_id: LibraryId(input.library_id),
                tasks: vec![input.task.try_into()?],
            })?;
            Ok(Some(json_response(200, &summary_views(tasks))))
        }
        ("POST", TASKS_BATCH_PATH) => {
            let input: BatchCreateTasksInput = parse_json_body(body)?;
            let library_path = state.library_path(&input.library_id)?;
            let tasks = input
                .tasks
                .into_iter()
                .map(CreateTaskInput::try_from)
                .collect::<DomainResult<Vec<_>>>()?;
            let created = state.service.create_tasks(BatchCreateTasksRequest {
                library_path,
                library_id: LibraryId(input.library_id),
                tasks,
            })?;
            Ok(Some(json_response(200, &summary_views(created))))
        }
        ("GET", TASKS_PATH) => {
            let library_id = query_value(query.unwrap_or_default(), "library_id")
                .or_else(|| query_value(query.unwrap_or_default(), "libraryId"))
                .ok_or_else(|| DomainError::InvalidGenerationParameters {
                    message: "library_id query parameter is required".to_string(),
                })?;
            let library_path = state.library_path(&library_id)?;
            let tasks = state.service.list_tasks(&library_path)?;
            Ok(Some(json_response(200, &summary_views(tasks))))
        }
        ("POST", TASKS_REORDER_PATH) => {
            let input: ReorderTasksInput = parse_json_body(body)?;
            let library_path = state.library_path(&input.library_id)?;
            state
                .service
                .reorder_queued_tasks(ReorderQueuedTasksRequest {
                    library_path,
                    task_ids: input.task_ids.into_iter().map(TaskId).collect(),
                })?;
            Ok(Some(json_response(
                200,
                &serde_json::json!({"status":"ok"}),
            )))
        }
        _ => route_task_member(method, path, state),
    }
}

pub(crate) fn route_task_member(
    method: &str,
    path: &str,
    state: &mut DaemonState,
) -> DomainResult<Option<HttpResponse>> {
    let Some(rest) = path.strip_prefix("/v1/tasks/") else {
        return Ok(None);
    };
    let segments = rest.split('/').collect::<Vec<_>>();
    if segments.is_empty() || segments[0].is_empty() {
        return Ok(None);
    }
    let task_id = TaskId(segments[0].to_string());
    let (library_path, detail) = find_task_detail(state, &task_id)?;

    match (method, segments.as_slice()) {
        ("GET", [_]) => Ok(Some(json_response(200, &TaskDetailView::from(detail)))),
        ("POST", [_, "cancel"]) => {
            let task = request_task_cancel(state, library_path, task_id, detail)?;
            Ok(Some(json_response(200, &TaskSummaryView::from(task))))
        }
        ("POST", [_, "retry"]) => {
            let task = state.service.retry_task(&library_path, &task_id)?;
            Ok(Some(json_response(200, &TaskSummaryView::from(task))))
        }
        ("POST", [_, "duplicate"]) => {
            let task = state.service.duplicate_task(&library_path, &task_id)?;
            Ok(Some(json_response(200, &TaskSummaryView::from(task))))
        }
        ("GET", [_, "events"]) => Ok(Some(json_response(
            200,
            &detail
                .events
                .into_iter()
                .map(TaskEventView::from)
                .collect::<Vec<_>>(),
        ))),
        ("GET", [_, "logs", "tail"]) => {
            let tail = tail_task_log(&detail, &state.log_root)?;
            Ok(Some(json_response(200, &tail)))
        }
        _ => Ok(None),
    }
}

pub(crate) fn find_task_detail(
    state: &DaemonState,
    task_id: &TaskId,
) -> DomainResult<(PathBuf, TaskDetail)> {
    for library_path in state.opened_libraries.values() {
        match state.service.get_task_detail(library_path, task_id) {
            Ok(detail) => return Ok((library_path.clone(), detail)),
            Err(DomainError::InvalidTaskReference { .. }) => {}
            Err(error) => return Err(error),
        }
    }
    Err(DomainError::InvalidTaskReference {
        id: task_id.0.clone(),
    })
}

pub(crate) fn request_task_cancel(
    state: &mut DaemonState,
    library_path: PathBuf,
    task_id: TaskId,
    detail: TaskDetail,
) -> DomainResult<TaskSummary> {
    let next_status = match detail.task.status {
        TaskStatus::Running | TaskStatus::CancelRequested => TaskStatus::CancelRequested,
        TaskStatus::Queued | TaskStatus::RetryWaiting => TaskStatus::Canceled,
        status => status,
    };
    if next_status == TaskStatus::CancelRequested {
        if let Some(log_path) = detail
            .attempts
            .iter()
            .rev()
            .find_map(|attempt| attempt.log_path.as_ref())
        {
            let marker = cancel_marker_path(log_path);
            fs::write(&marker, "cancel requested\n").map_err(|error| io_error(&marker, error))?;
        }
    }
    state
        .service
        .append_task_event(imglab_core::AppendTaskEventRequest {
            library_path: library_path.clone(),
            task_id: task_id.clone(),
            event_type: "cancel_requested".to_string(),
            message: Some("Cancel requested by client".to_string()),
            payload_json: None,
        })?;
    state.service.update_task_status(UpdateTaskStatusRequest {
        library_path,
        task_id,
        status: next_status,
        next_retry_at: None,
        last_error_code: None,
        last_error_message: None,
        error_classification: Some(TaskErrorClassification::Cancel),
        wait_reason: None,
    })
}

pub(crate) fn tail_task_log(detail: &TaskDetail, log_root: &Path) -> DomainResult<LogTailView> {
    let Some(log_path) = detail
        .attempts
        .iter()
        .rev()
        .find_map(|attempt| attempt.log_path.as_ref())
    else {
        return Ok(LogTailView {
            content: String::new(),
            truncated: false,
        });
    };
    ensure_app_owned_log_path(log_path, log_root)?;
    let bytes = fs::read(log_path).map_err(|error| io_error(log_path, error))?;
    let truncated = bytes.len() > MAX_LOG_TAIL_BYTES;
    let start = bytes.len().saturating_sub(MAX_LOG_TAIL_BYTES);
    Ok(LogTailView {
        content: String::from_utf8_lossy(&bytes[start..]).to_string(),
        truncated,
    })
}

pub(crate) fn ensure_app_owned_log_path(log_path: &Path, log_root: &Path) -> DomainResult<()> {
    let canonical_log = log_path
        .canonicalize()
        .map_err(|error| io_error(log_path, error))?;
    let canonical_root = log_root
        .canonicalize()
        .map_err(|error| io_error(log_root, error))?;
    if !canonical_log.starts_with(&canonical_root) {
        return Err(DomainError::InvalidGenerationParameters {
            message: "task log path is outside app-owned log root".to_string(),
        });
    }
    Ok(())
}
