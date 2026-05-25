use crate::executors::*;
use crate::routes::*;
use crate::runtime::*;
use crate::scheduler::execute_task;
use crate::views::*;
use crate::*;
use imglab_core::application::ports::PromptExpansionProvider;

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

pub fn spawn_schedule_loop(state: SharedDaemonState, interval: Duration) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        if let Err(error) = run_schedule_loop_iteration(&state) {
            eprintln!("schedule tick failed: {error}");
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

pub fn run_schedule_loop_iteration(
    state: &SharedDaemonState,
) -> DomainResult<Vec<ScheduledGenerationRunView>> {
    let mut guard = state
        .lock()
        .map_err(|_| DomainError::ConcurrentWriteConflict {
            message: "daemon state lock is poisoned".to_string(),
        })?;
    recover_automation_enabled_libraries(&mut guard)?;
    let mut snapshot = guard.clone();
    drop(guard);
    run_schedule_tick(&mut snapshot)
}

pub fn run_schedule_tick(state: &mut DaemonState) -> DomainResult<Vec<ScheduledGenerationRunView>> {
    let mut changed_runs = Vec::new();
    let now = unix_timestamp_millis_string(0);
    let now_ms = parse_millis(&now)?;
    for library_path in state.opened_libraries.values().cloned().collect::<Vec<_>>() {
        let jobs = state.schedules().list_jobs(&library_path)?;
        for job in &jobs {
            changed_runs.extend(reconcile_schedule_runs_for_job(state, &library_path, job)?);
        }
        for job in state.schedules().list_due_jobs(&library_path, &now)? {
            let active_run_exists = state
                .schedules()
                .list_runs(&library_path, &job.id)?
                .into_iter()
                .any(|run| schedule_run_is_active(run.status));
            let next_run_at_ms = parse_millis(&job.next_run_at)?;
            match imglab_core::domain::schedule::resolve_due_schedule(
                now_ms as u64,
                next_run_at_ms as u64,
                active_run_exists,
                job.missed_run_policy,
                job.overlap_policy,
            ) {
                imglab_core::domain::schedule::DueScheduleAction::Run => {
                    changed_runs.push(create_scheduled_run_and_task(
                        state,
                        &library_path,
                        &job,
                        job.next_run_at.clone(),
                    )?);
                    update_job_next_run(state, &library_path, &job, now_ms)?;
                }
                imglab_core::domain::schedule::DueScheduleAction::Skip { reason } => {
                    changed_runs.push(create_skipped_schedule_run(
                        state,
                        &library_path,
                        &job,
                        job.next_run_at.clone(),
                        reason,
                    )?);
                    update_job_next_run(state, &library_path, &job, now_ms)?;
                }
                imglab_core::domain::schedule::DueScheduleAction::MissedNoCatchUp {
                    diagnostic,
                } => {
                    changed_runs.push(create_skipped_schedule_run(
                        state,
                        &library_path,
                        &job,
                        job.next_run_at.clone(),
                        diagnostic,
                    )?);
                    update_job_next_run(state, &library_path, &job, now_ms)?;
                }
                imglab_core::domain::schedule::DueScheduleAction::NotDue => {}
            }
        }
    }
    Ok(changed_runs)
}

pub(crate) fn daemon_has_runnable_work(state: &DaemonState) -> DomainResult<bool> {
    let now = unix_timestamp_string(0);
    for library_path in state.opened_libraries.values() {
        for task in state.tasks().list_tasks(library_path)? {
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
        for task in state.tasks().list_tasks(&library_path)? {
            match task.status {
                TaskStatus::Queued => {}
                TaskStatus::RetryWaiting => {
                    if task
                        .next_retry_at
                        .as_deref()
                        .is_none_or(|next_retry_at| next_retry_at <= now.as_str())
                    {
                        state
                            .tasks()
                            .append_task_event(imglab_core::AppendTaskEventRequest {
                                library_path: library_path.clone(),
                                task_id: task.id.clone(),
                                event_type: "recovery_retry_ready".to_string(),
                                message: Some(
                                    "Retry backoff elapsed during daemon downtime".to_string(),
                                ),
                                payload_json: None,
                            })?;
                        recovered.push(state.tasks().update_task_status(
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
                    let detail = state.tasks().get_task_detail(&library_path, &task.id)?;
                    if !detail.outputs.is_empty() {
                        state
                            .tasks()
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
                        recovered
                            .push(state.tasks().update_task_status(UpdateTaskStatusRequest {
                            library_path: library_path.clone(),
                            task_id: task.id.clone(),
                            status:
                                imglab_core::domain::task::resolve_recovered_running_with_outputs(),
                            next_retry_at: None,
                            last_error_code: None,
                            last_error_message: None,
                            error_classification: None,
                            wait_reason: None,
                        })?);
                    } else {
                        let resolution = imglab_core::domain::task::resolve_interrupted_recovery(
                            task.attempt_count,
                            retry_policy.max_attempts,
                        );
                        state
                            .tasks()
                            .append_task_event(imglab_core::AppendTaskEventRequest {
                                library_path: library_path.clone(),
                                task_id: task.id.clone(),
                                event_type: "recovery_interrupted".to_string(),
                                message: Some("Task was running when daemon stopped".to_string()),
                                payload_json: Some(format!(
                                    "{{\"retryable\":{}}}",
                                    resolution.retryable
                                )),
                            })?;
                        recovered.push(state.tasks().update_task_status(
                            UpdateTaskStatusRequest {
                                library_path: library_path.clone(),
                                task_id: task.id.clone(),
                                status: resolution.task_status,
                                next_retry_at: None,
                                last_error_code: Some("DaemonInterrupted".to_string()),
                                last_error_message: Some(
                                    "Task was running when daemon stopped".to_string(),
                                ),
                                error_classification: Some(resolution.error_classification),
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
        for task in state.tasks().list_tasks(library_path)? {
            task_libraries.insert(task.id.0.clone(), library_path.clone());
            all_tasks.push(task);
        }
    }

    let now = unix_timestamp_string(0);
    let decision = evaluate_scheduler(&all_tasks, config, &now);
    for wait_reason in decision.wait_reasons {
        if let Some(library_path) = task_libraries.get(&wait_reason.task_id.0) {
            let detail = state
                .tasks()
                .get_task_detail(library_path, &wait_reason.task_id)?;
            state.tasks().update_task_status(UpdateTaskStatusRequest {
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
            let tasks = state.tasks().create_tasks(BatchCreateTasksRequest {
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
            let created = state.tasks().create_tasks(BatchCreateTasksRequest {
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
            let tasks = state.tasks().list_tasks(&library_path)?;
            Ok(Some(json_response(200, &summary_views(tasks))))
        }
        ("POST", SCHEDULES_PATH) => {
            let input: CreateScheduleInput = parse_json_body(body)?;
            let library_path = state.library_path(&input.library_id)?;
            let schedule_rule = parse_schedule_rule(input.schedule_rule)?;
            let next_run_at = input.next_run_at.unwrap_or_else(|| {
                imglab_core::domain::schedule::next_run_after(
                    &schedule_rule,
                    parse_millis(&unix_timestamp_millis_string(0)).unwrap_or_default() as u64,
                    None,
                )
                .unwrap_or_else(|| unix_timestamp_millis_string(0))
            });
            let job =
                state
                    .schedules()
                    .create_job(imglab_core::CreateScheduledGenerationJobRequest {
                        library_path,
                        library_id: LibraryId(input.library_id),
                        name: input.name,
                        prompt_mode: parse_prompt_mode(&input.prompt_mode)?,
                        fixed_prompt: input.fixed_prompt,
                        negative_prompt: input.negative_prompt,
                        base_prompt: input.base_prompt,
                        dynamic_prompt: input.dynamic_prompt,
                        prompt_expander_provider: input.prompt_expander_provider,
                        prompt_expander_model: input.prompt_expander_model,
                        image_provider: input.image_provider,
                        image_model: input.image_model,
                        parameters_json: parameters_json(input.parameters, input.parameters_json)?,
                        schedule_rule,
                        target_album_id: imglab_core::AlbumId(input.target_album_id),
                        tags: input.tags,
                        next_run_at,
                    })?;
            Ok(Some(json_response(
                200,
                &ScheduledGenerationJobViewDto::from(job),
            )))
        }
        ("GET", SCHEDULES_PATH) => {
            let library_id = query_value(query.unwrap_or_default(), "library_id")
                .or_else(|| query_value(query.unwrap_or_default(), "libraryId"))
                .ok_or_else(|| DomainError::InvalidGenerationParameters {
                    message: "library_id query parameter is required".to_string(),
                })?;
            let library_path = state.library_path(&library_id)?;
            let jobs = state
                .schedules()
                .list_jobs(&library_path)?
                .into_iter()
                .map(ScheduledGenerationJobViewDto::from)
                .collect::<Vec<_>>();
            Ok(Some(json_response(200, &jobs)))
        }
        ("POST", TASKS_REORDER_PATH) => {
            let input: ReorderTasksInput = parse_json_body(body)?;
            let library_path = state.library_path(&input.library_id)?;
            state
                .tasks()
                .reorder_queued_tasks(ReorderQueuedTasksRequest {
                    library_path,
                    task_ids: input.task_ids.into_iter().map(TaskId).collect(),
                })?;
            Ok(Some(json_response(
                200,
                &serde_json::json!({"status":"ok"}),
            )))
        }
        _ => {
            if let Some(response) = route_schedule_member(method, path, query, body, state)? {
                Ok(Some(response))
            } else {
                route_task_member(method, path, state)
            }
        }
    }
}

pub(crate) fn route_schedule_member(
    method: &str,
    path: &str,
    query: Option<&str>,
    body: &str,
    state: &mut DaemonState,
) -> DomainResult<Option<HttpResponse>> {
    let Some(rest) = path.strip_prefix("/v1/schedules/") else {
        return Ok(None);
    };
    let segments = rest.split('/').collect::<Vec<_>>();
    if segments.is_empty() || segments[0].is_empty() {
        return Ok(None);
    }
    let job_id = ScheduledGenerationJobId(segments[0].to_string());
    let (library_path, job) = find_schedule_job(state, &job_id)?;

    match (method, segments.as_slice()) {
        ("GET", [_]) => Ok(Some(json_response(
            200,
            &ScheduledGenerationJobViewDto::from(job),
        ))),
        ("PUT", [_]) => {
            let input: UpdateScheduleInput = parse_json_body(body)?;
            let library_path = state.library_path(&input.library_id)?;
            let schedule_rule = parse_schedule_rule(input.schedule_rule)?;
            let next_run_at = input.next_run_at.unwrap_or_else(|| {
                imglab_core::domain::schedule::next_run_after(
                    &schedule_rule,
                    parse_millis(&unix_timestamp_millis_string(0)).unwrap_or_default() as u64,
                    None,
                )
                .unwrap_or_else(|| unix_timestamp_millis_string(0))
            });
            let updated = state
                .schedules()
                .update_job(UpdateScheduledGenerationJobRequest {
                    library_path,
                    job_id,
                    name: input.name,
                    prompt_mode: parse_prompt_mode(&input.prompt_mode)?,
                    fixed_prompt: input.fixed_prompt,
                    negative_prompt: input.negative_prompt,
                    base_prompt: input.base_prompt,
                    dynamic_prompt: input.dynamic_prompt,
                    prompt_expander_provider: input.prompt_expander_provider,
                    prompt_expander_model: input.prompt_expander_model,
                    image_provider: input.image_provider,
                    image_model: input.image_model,
                    parameters_json: parameters_json(input.parameters, input.parameters_json)?,
                    schedule_rule,
                    target_album_id: imglab_core::AlbumId(input.target_album_id),
                    tags: input.tags,
                    next_run_at,
                })?;
            Ok(Some(json_response(
                200,
                &ScheduledGenerationJobViewDto::from(updated),
            )))
        }
        ("DELETE", [_]) => {
            state.schedules().delete_job(&library_path, &job_id)?;
            Ok(Some(json_response(
                200,
                &serde_json::json!({ "deleted": true }),
            )))
        }
        ("POST", [_, "enable"]) => {
            let updated = state.schedules().set_job_status(
                &library_path,
                &job_id,
                ScheduledGenerationJobStatus::Active,
            )?;
            Ok(Some(json_response(
                200,
                &ScheduledGenerationJobViewDto::from(updated),
            )))
        }
        ("POST", [_, "disable"]) | ("POST", [_, "pause"]) => {
            let updated = state.schedules().set_job_status(
                &library_path,
                &job_id,
                ScheduledGenerationJobStatus::Paused,
            )?;
            Ok(Some(json_response(
                200,
                &ScheduledGenerationJobViewDto::from(updated),
            )))
        }
        ("POST", [_, "run-now"]) => {
            let run = create_scheduled_run_and_task(
                state,
                &library_path,
                &job,
                unix_timestamp_millis_string(0),
            )?;
            Ok(Some(json_response(
                200,
                &ScheduledGenerationRunViewDto::from(run),
            )))
        }
        ("GET", [_, "runs"]) => {
            let _library_id = query_value(query.unwrap_or_default(), "library_id")
                .or_else(|| query_value(query.unwrap_or_default(), "libraryId"));
            let runs = state
                .schedules()
                .list_runs(&library_path, &job_id)?
                .into_iter()
                .map(ScheduledGenerationRunViewDto::from)
                .collect::<Vec<_>>();
            Ok(Some(json_response(200, &runs)))
        }
        ("GET", [_, "runs", run_id]) => {
            let run = state
                .schedules()
                .list_runs(&library_path, &job_id)?
                .into_iter()
                .find(|run| run.id.0 == *run_id)
                .ok_or_else(|| DomainError::InvalidGenerationParameters {
                    message: format!("scheduled generation run not found: {run_id}"),
                })?;
            Ok(Some(json_response(
                200,
                &ScheduledGenerationRunViewDto::from(run),
            )))
        }
        _ => Ok(None),
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
            let task = state.tasks().retry_task(&library_path, &task_id)?;
            Ok(Some(json_response(200, &TaskSummaryView::from(task))))
        }
        ("POST", [_, "duplicate"]) => {
            let task = state.tasks().duplicate_task(&library_path, &task_id)?;
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
        match state.tasks().get_task_detail(library_path, task_id) {
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
    let resolution = imglab_core::domain::task::resolve_cancel_request(detail.task.status);
    if resolution.write_cancel_marker {
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
        .tasks()
        .append_task_event(imglab_core::AppendTaskEventRequest {
            library_path: library_path.clone(),
            task_id: task_id.clone(),
            event_type: "cancel_requested".to_string(),
            message: Some("Cancel requested by client".to_string()),
            payload_json: None,
        })?;
    state.tasks().update_task_status(UpdateTaskStatusRequest {
        library_path,
        task_id,
        status: resolution.task_status,
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
    let bytes = match fs::read(log_path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(LogTailView {
                content: format!(
                    "task log file is no longer available: {}",
                    log_path.display()
                ),
                truncated: false,
            });
        }
        Err(error) => return Err(io_error(log_path, error)),
    };
    let truncated = bytes.len() > MAX_LOG_TAIL_BYTES;
    let start = bytes.len().saturating_sub(MAX_LOG_TAIL_BYTES);
    Ok(LogTailView {
        content: String::from_utf8_lossy(&bytes[start..]).to_string(),
        truncated,
    })
}

pub(crate) fn ensure_app_owned_log_path(log_path: &Path, log_root: &Path) -> DomainResult<()> {
    let canonical_root = log_root
        .canonicalize()
        .map_err(|error| io_error(log_root, error))?;
    let canonical_log = match log_path.canonicalize() {
        Ok(path) => path,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let Some(parent) = log_path.parent() else {
                return Err(io_error(log_path, error));
            };
            let canonical_parent = parent
                .canonicalize()
                .map_err(|parent_error| io_error(parent, parent_error))?;
            if !canonical_parent.starts_with(&canonical_root) {
                return Err(DomainError::InvalidGenerationParameters {
                    message: "task log path is outside app-owned log root".to_string(),
                });
            }
            return Ok(());
        }
        Err(error) => return Err(io_error(log_path, error)),
    };
    if !canonical_log.starts_with(&canonical_root) {
        return Err(DomainError::InvalidGenerationParameters {
            message: "task log path is outside app-owned log root".to_string(),
        });
    }
    Ok(())
}

fn recover_automation_enabled_libraries(state: &mut DaemonState) -> DomainResult<()> {
    let libraries = state.library_lifecycle().list_libraries(false)?;
    for library in libraries {
        if library.automation_enabled && !state.opened_libraries.contains_key(&library.id.0) {
            state.open_library(&library.root_path)?;
        }
    }
    Ok(())
}

fn find_schedule_job(
    state: &DaemonState,
    job_id: &ScheduledGenerationJobId,
) -> DomainResult<(PathBuf, ScheduledGenerationJobView)> {
    for library_path in state.opened_libraries.values() {
        for job in state.schedules().list_jobs(library_path)? {
            if &job.id == job_id {
                return Ok((library_path.clone(), job));
            }
        }
    }
    Err(DomainError::InvalidGenerationParameters {
        message: format!("scheduled generation job not found: {}", job_id.0),
    })
}

fn create_scheduled_run_and_task(
    state: &mut DaemonState,
    library_path: &Path,
    job: &ScheduledGenerationJobView,
    scheduled_for: String,
) -> DomainResult<ScheduledGenerationRunView> {
    let run = state
        .schedules()
        .create_run(CreateScheduledGenerationRunRequest {
            library_path: library_path.to_path_buf(),
            job_id: job.id.clone(),
            library_id: job.library_id.clone(),
            scheduled_for,
        })?;
    let started_at = Some(unix_timestamp_millis_string(0));
    let prompt = match job.prompt_mode {
        SchedulePromptMode::Fixed => match job.fixed_prompt.clone() {
            Some(prompt) => prompt,
            None => {
                return fail_schedule_run(
                    state,
                    library_path,
                    run,
                    "InvalidSchedulePrompt",
                    "fixed scheduled generation job requires fixed prompt",
                );
            }
        },
        SchedulePromptMode::Dynamic => {
            let Some(base_prompt) = job.base_prompt.clone() else {
                return fail_schedule_run(
                    state,
                    library_path,
                    run,
                    "InvalidSchedulePrompt",
                    "dynamic scheduled generation job requires base prompt",
                );
            };
            let Some(dynamic_prompt) = job.dynamic_prompt.clone() else {
                return fail_schedule_run(
                    state,
                    library_path,
                    run,
                    "InvalidSchedulePrompt",
                    "dynamic scheduled generation job requires dynamic prompt",
                );
            };
            let expansion = match expand_schedule_prompt(
                state,
                library_path,
                job,
                &run,
                base_prompt,
                dynamic_prompt,
            ) {
                Ok(expansion) => expansion,
                Err(error) => {
                    return fail_schedule_run(
                        state,
                        library_path,
                        run,
                        "PromptExpansionFailed",
                        &error.to_string(),
                    );
                }
            };
            state
                .schedules()
                .update_run(UpdateScheduledGenerationRunRequest {
                    library_path: library_path.to_path_buf(),
                    run_id: run.id.clone(),
                    status: ScheduledGenerationRunStatus::ExpandingPrompt,
                    started_at: started_at.clone(),
                    completed_at: None,
                    skip_reason: None,
                    error_code: None,
                    error_message: None,
                    expanded_prompt: Some(expansion.expanded_prompt.clone()),
                    prompt_expansion_provider_metadata_json: Some(expansion.provider_metadata_json),
                    image_task_id: None,
                })?;
            expansion.expanded_prompt
        }
    };
    let task_input = serde_json::json!({
        "prompt": prompt,
        "negativePrompt": job.negative_prompt,
        "promptVersionId": null,
        "model": job.image_model,
        "parametersJson": serde_json::from_str::<Value>(&job.parameters_json).unwrap_or(Value::Null),
        "inputFile": null,
        "inputVersionId": null,
        "fakeMode": null,
        "schedule": {
            "jobId": job.id.0.clone(),
            "jobName": job.name.clone(),
            "runId": run.id.0.clone(),
            "scheduledFor": run.scheduled_for.clone(),
            "promptMode": job.prompt_mode.as_str(),
        },
    });
    let created = state.tasks().create_tasks(BatchCreateTasksRequest {
        library_path: library_path.to_path_buf(),
        library_id: job.library_id.clone(),
        tasks: vec![CreateTaskInput {
            task_type: TaskType::ImageGeneration,
            provider: Some(job.image_provider.clone()),
            operation: Some(GenerationOperation::TextToImage),
            priority: 0,
            concurrency_group: Some(format!("schedule:{}", job.id.0)),
            max_attempts: 3,
            input_json: task_input.to_string(),
        }],
    })?;
    let task = created
        .into_iter()
        .next()
        .ok_or_else(|| DomainError::Database {
            message: "scheduled generation did not create an image task".to_string(),
        })?;
    state
        .schedules()
        .update_run(UpdateScheduledGenerationRunRequest {
            library_path: library_path.to_path_buf(),
            run_id: run.id,
            status: ScheduledGenerationRunStatus::TaskQueued,
            started_at,
            completed_at: None,
            skip_reason: None,
            error_code: None,
            error_message: None,
            expanded_prompt: Some(prompt),
            prompt_expansion_provider_metadata_json: None,
            image_task_id: Some(task.id),
        })
}

fn expand_schedule_prompt(
    state: &DaemonState,
    library_path: &Path,
    job: &ScheduledGenerationJobView,
    run: &ScheduledGenerationRunView,
    base_prompt: String,
    dynamic_prompt: String,
) -> DomainResult<imglab_core::PromptExpansionResult> {
    let provider = job
        .prompt_expander_provider
        .clone()
        .unwrap_or_else(|| "fake".to_string());
    let request = PromptExpansionRequest {
        provider: provider.clone(),
        model: job.prompt_expander_model.clone(),
        base_prompt,
        dynamic_prompt,
        context_json: Some(
            serde_json::json!({
                "scheduleJobId": job.id.0,
                "scheduleRunId": run.id.0,
                "libraryId": job.library_id.0,
            })
            .to_string(),
        ),
    };
    match provider.as_str() {
        "fake" => imglab_core::FakeImageProvider::success("fake").expand_prompt(&request),
        "codex" | "codex-cli" => {
            let log_path = state
                .log_root
                .join("prompt-expansion")
                .join(format!("schedule-{}-{}.log", job.id.0, run.id.0));
            CodexCliImageProvider::new("codex", library_path)
                .with_log_path(log_path)
                .expand_prompt(&request)
        }
        other => Err(DomainError::InvalidGenerationParameters {
            message: format!("unsupported prompt expansion provider: {other}"),
        }),
    }
}

fn fail_schedule_run(
    state: &mut DaemonState,
    library_path: &Path,
    run: ScheduledGenerationRunView,
    error_code: &str,
    error_message: &str,
) -> DomainResult<ScheduledGenerationRunView> {
    state
        .schedules()
        .update_run(UpdateScheduledGenerationRunRequest {
            library_path: library_path.to_path_buf(),
            run_id: run.id,
            status: ScheduledGenerationRunStatus::Failed,
            started_at: Some(unix_timestamp_millis_string(0)),
            completed_at: Some(unix_timestamp_millis_string(0)),
            skip_reason: None,
            error_code: Some(error_code.to_string()),
            error_message: Some(error_message.to_string()),
            expanded_prompt: None,
            prompt_expansion_provider_metadata_json: None,
            image_task_id: None,
        })
}

fn create_skipped_schedule_run(
    state: &mut DaemonState,
    library_path: &Path,
    job: &ScheduledGenerationJobView,
    scheduled_for: String,
    reason: &str,
) -> DomainResult<ScheduledGenerationRunView> {
    let run = state
        .schedules()
        .create_run(CreateScheduledGenerationRunRequest {
            library_path: library_path.to_path_buf(),
            job_id: job.id.clone(),
            library_id: job.library_id.clone(),
            scheduled_for,
        })?;
    state
        .schedules()
        .update_run(UpdateScheduledGenerationRunRequest {
            library_path: library_path.to_path_buf(),
            run_id: run.id,
            status: ScheduledGenerationRunStatus::Skipped,
            started_at: None,
            completed_at: Some(unix_timestamp_millis_string(0)),
            skip_reason: Some(reason.to_string()),
            error_code: None,
            error_message: None,
            expanded_prompt: None,
            prompt_expansion_provider_metadata_json: None,
            image_task_id: None,
        })
}

fn update_job_next_run(
    state: &mut DaemonState,
    library_path: &Path,
    job: &ScheduledGenerationJobView,
    now_ms: u128,
) -> DomainResult<ScheduledGenerationJobView> {
    let next_run_at = imglab_core::domain::schedule::next_run_after(
        &job.schedule_rule,
        now_ms as u64,
        job.next_run_at.parse::<u64>().ok(),
    )
    .ok_or_else(|| DomainError::InvalidGenerationParameters {
        message: "scheduled generation job could not compute next run".to_string(),
    })?;
    state
        .schedules()
        .update_job(UpdateScheduledGenerationJobRequest {
            library_path: library_path.to_path_buf(),
            job_id: job.id.clone(),
            name: job.name.clone(),
            prompt_mode: job.prompt_mode,
            fixed_prompt: job.fixed_prompt.clone(),
            negative_prompt: job.negative_prompt.clone(),
            base_prompt: job.base_prompt.clone(),
            dynamic_prompt: job.dynamic_prompt.clone(),
            prompt_expander_provider: job.prompt_expander_provider.clone(),
            prompt_expander_model: job.prompt_expander_model.clone(),
            image_provider: job.image_provider.clone(),
            image_model: job.image_model.clone(),
            parameters_json: job.parameters_json.clone(),
            schedule_rule: job.schedule_rule.clone(),
            target_album_id: job.target_album_id.clone(),
            tags: job.tags.clone(),
            next_run_at,
        })
}

fn reconcile_schedule_runs_for_job(
    state: &mut DaemonState,
    library_path: &Path,
    job: &ScheduledGenerationJobView,
) -> DomainResult<Vec<ScheduledGenerationRunView>> {
    let mut changed = Vec::new();
    for run in state.schedules().list_runs(library_path, &job.id)? {
        if matches!(
            run.status,
            ScheduledGenerationRunStatus::Pending | ScheduledGenerationRunStatus::ExpandingPrompt
        ) && run.image_task_id.is_none()
        {
            changed.push(
                state
                    .schedules()
                    .update_run(UpdateScheduledGenerationRunRequest {
                    library_path: library_path.to_path_buf(),
                    run_id: run.id.clone(),
                    status: ScheduledGenerationRunStatus::Failed,
                    started_at: run
                        .started_at
                        .clone()
                        .or_else(|| Some(unix_timestamp_millis_string(0))),
                    completed_at: Some(unix_timestamp_millis_string(0)),
                    skip_reason: None,
                    error_code: Some("DaemonInterrupted".to_string()),
                    error_message: Some(
                        "Scheduled generation run was interrupted before an image task was queued"
                            .to_string(),
                    ),
                    expanded_prompt: run.expanded_prompt.clone(),
                    prompt_expansion_provider_metadata_json: run
                        .prompt_expansion_provider_metadata_json
                        .clone(),
                    image_task_id: None,
                })?,
            );
            continue;
        }
        if !matches!(
            run.status,
            ScheduledGenerationRunStatus::TaskQueued | ScheduledGenerationRunStatus::TaskRunning
        ) {
            continue;
        }
        let Some(task_id) = run.image_task_id.clone() else {
            continue;
        };
        let detail = state.tasks().get_task_detail(library_path, &task_id)?;
        match detail.task.status {
            TaskStatus::Queued | TaskStatus::RetryWaiting => {}
            TaskStatus::Running | TaskStatus::CancelRequested => {
                changed.push(state.schedules().update_run(
                    UpdateScheduledGenerationRunRequest {
                        library_path: library_path.to_path_buf(),
                        run_id: run.id.clone(),
                        status: ScheduledGenerationRunStatus::TaskRunning,
                        started_at: run.started_at.clone(),
                        completed_at: None,
                        skip_reason: None,
                        error_code: None,
                        error_message: None,
                        expanded_prompt: run.expanded_prompt.clone(),
                        prompt_expansion_provider_metadata_json:
                            run.prompt_expansion_provider_metadata_json.clone(),
                        image_task_id: Some(task_id),
                    },
                )?);
            }
            TaskStatus::Completed => {
                changed.push(post_process_completed_schedule_run(
                    state,
                    library_path,
                    job,
                    run,
                    detail,
                )?);
            }
            TaskStatus::FailedRetryable
            | TaskStatus::FailedFinal
            | TaskStatus::Canceled
            | TaskStatus::InterruptedRetryable
            | TaskStatus::InterruptedFinal => {
                changed.push(state.schedules().update_run(
                    UpdateScheduledGenerationRunRequest {
                        library_path: library_path.to_path_buf(),
                        run_id: run.id.clone(),
                        status: ScheduledGenerationRunStatus::Failed,
                        started_at: run.started_at.clone(),
                        completed_at: Some(unix_timestamp_millis_string(0)),
                        skip_reason: None,
                        error_code: detail.task.last_error_code,
                        error_message: detail.task.last_error_message,
                        expanded_prompt: run.expanded_prompt.clone(),
                        prompt_expansion_provider_metadata_json:
                            run.prompt_expansion_provider_metadata_json.clone(),
                        image_task_id: Some(task_id),
                    },
                )?);
            }
        }
    }
    Ok(changed)
}

fn post_process_completed_schedule_run(
    state: &mut DaemonState,
    library_path: &Path,
    job: &ScheduledGenerationJobView,
    run: ScheduledGenerationRunView,
    detail: TaskDetail,
) -> DomainResult<ScheduledGenerationRunView> {
    let output_links = detail
        .output_links
        .into_iter()
        .filter_map(|link| {
            link.asset_id
                .map(|asset_id| (asset_id, link.version_id, link.generation_event_id))
        })
        .collect::<Vec<_>>();
    let asset_ids = output_links
        .iter()
        .map(|(asset_id, _, _)| asset_id.clone())
        .collect::<Vec<_>>();
    if !asset_ids.is_empty() {
        state
            .albums()
            .batch_add_assets(BatchAddAssetsToAlbumRequest {
                album_id: job.target_album_id.clone(),
                asset_ids: asset_ids.clone(),
            })?;
    }
    let mut tagged_count = 0_u32;
    for asset_id in &asset_ids {
        for tag in &job.tags {
            state.assets().add_tag(imglab_core::AddAssetTagRequest {
                library_path: library_path.to_path_buf(),
                asset_id: asset_id.clone(),
                tag: tag.clone(),
            })?;
        }
        if !job.tags.is_empty() {
            tagged_count += 1;
        }
    }
    for (asset_id, version_id, generation_event_id) in output_links {
        state
            .schedules()
            .upsert_run_output(UpsertScheduledGenerationRunOutputRequest {
                library_path: library_path.to_path_buf(),
                run_id: run.id.clone(),
                asset_id,
                asset_version_id: version_id,
                generation_event_id,
                album_added: true,
                tags_applied: job.tags.clone(),
            })?;
    }
    state
        .schedules()
        .update_run(UpdateScheduledGenerationRunRequest {
            library_path: library_path.to_path_buf(),
            run_id: run.id.clone(),
            status: ScheduledGenerationRunStatus::Completed,
            started_at: run.started_at,
            completed_at: Some(unix_timestamp_millis_string(0)),
            skip_reason: None,
            error_code: None,
            error_message: None,
            expanded_prompt: run.expanded_prompt,
            prompt_expansion_provider_metadata_json: run.prompt_expansion_provider_metadata_json,
            image_task_id: run.image_task_id,
        })?;
    let mut completed = state
        .schedules()
        .list_runs(library_path, &job.id)?
        .into_iter()
        .find(|candidate| candidate.id == run.id)
        .ok_or_else(|| DomainError::InvalidGenerationParameters {
            message: format!("scheduled generation run not found: {}", run.id.0),
        })?;
    completed.output_asset_count = asset_ids.len() as u32;
    completed.album_added_asset_count = asset_ids.len() as u32;
    completed.tagged_asset_count = tagged_count;
    Ok(completed)
}

fn schedule_run_is_active(status: ScheduledGenerationRunStatus) -> bool {
    matches!(
        status,
        ScheduledGenerationRunStatus::Pending
            | ScheduledGenerationRunStatus::ExpandingPrompt
            | ScheduledGenerationRunStatus::TaskQueued
            | ScheduledGenerationRunStatus::TaskRunning
            | ScheduledGenerationRunStatus::PostProcessing
    )
}

fn parse_millis(value: &str) -> DomainResult<u128> {
    value
        .parse::<u128>()
        .map_err(|_| DomainError::InvalidGenerationParameters {
            message: format!("invalid millisecond timestamp: {value}"),
        })
}
