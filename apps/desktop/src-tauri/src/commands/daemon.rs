use crate::*;

#[tauri::command]
pub(crate) fn daemon_health(state: tauri::State<'_, DesktopState>) -> Result<bool, CommandError> {
    ensure_daemon_client(&state).map(|_| true)
}

#[tauri::command]
pub(crate) fn automation_daemon_status() -> AutomationDaemonStatusView {
    crate::automation_daemon::automation_daemon_status()
}

#[tauri::command]
pub(crate) fn start_automation_daemon() -> Result<AutomationDaemonStatusView, CommandError> {
    crate::automation_daemon::install_automation_daemon()
}

#[tauri::command]
pub(crate) fn stop_automation_daemon() -> Result<AutomationDaemonStatusView, CommandError> {
    crate::automation_daemon::uninstall_automation_daemon()
}

#[tauri::command]
pub(crate) fn restart_automation_daemon() -> Result<AutomationDaemonStatusView, CommandError> {
    crate::automation_daemon::restart_automation_daemon()
}

#[tauri::command]
pub(crate) fn repair_automation_daemon() -> Result<AutomationDaemonStatusView, CommandError> {
    crate::automation_daemon::repair_automation_daemon()
}

#[tauri::command]
pub(crate) fn set_library_automation_enabled(
    input: LibraryAutomationInput,
) -> Result<LibraryView, CommandError> {
    desktop_app()
        .schedules()
        .set_library_automation_enabled(&LibraryId(input.library_id), input.enabled)
        .map(library_view)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) fn enqueue_generation_tasks(
    input: EnqueueGenerationTasksInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<Vec<DaemonTaskView>, CommandError> {
    let client = ensure_daemon_client(&state)?;
    let library_id = client.open_library(&input.library_path)?;
    let tasks = input
        .tasks
        .into_iter()
        .map(generation_draft_to_daemon_task)
        .collect::<Result<Vec<_>, _>>()?;
    client
        .batch_create_tasks(BatchCreateTasksInput { library_id, tasks })
        .map(|tasks| tasks.into_iter().map(daemon_task_view).collect())
}

#[tauri::command]
pub(crate) fn list_daemon_tasks(
    input: DaemonTaskQueryInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<Vec<DaemonTaskView>, CommandError> {
    let client = ensure_daemon_client(&state)?;
    let library_id = client.open_library(&input.library_path)?;
    client
        .list_tasks(&library_id)
        .map(|tasks| tasks.into_iter().map(daemon_task_view).collect())
}

#[tauri::command]
pub(crate) fn get_daemon_task_detail(
    input: DaemonTaskActionInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<DaemonTaskDetailView, CommandError> {
    let client = ensure_daemon_client(&state)?;
    let detail = client.get_task(&input.task_id)?;
    let tail = client.tail_task_log(&input.task_id)?;
    Ok(daemon_task_detail_view(
        detail,
        tail.content,
        tail.truncated,
    ))
}

#[tauri::command]
pub(crate) fn reorder_daemon_tasks(
    input: ReorderDaemonTasksInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<(), CommandError> {
    let client = ensure_daemon_client(&state)?;
    let library_id = client.open_library(&input.library_path)?;
    client.reorder_tasks(library_id, input.task_ids)
}

#[tauri::command]
pub(crate) fn cancel_daemon_task(
    input: DaemonTaskActionInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<DaemonTaskView, CommandError> {
    ensure_daemon_client(&state)?
        .cancel_task(&input.task_id)
        .map(daemon_task_view)
}

#[tauri::command]
pub(crate) fn retry_daemon_task(
    input: DaemonTaskActionInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<DaemonTaskView, CommandError> {
    ensure_daemon_client(&state)?
        .retry_task(&input.task_id)
        .map(daemon_task_view)
}

#[tauri::command]
pub(crate) fn duplicate_daemon_task(
    input: DaemonTaskActionInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<DaemonTaskView, CommandError> {
    ensure_daemon_client(&state)?
        .duplicate_task(&input.task_id)
        .map(daemon_task_view)
}

#[tauri::command]
pub(crate) fn create_scheduled_generation_job(
    input: DaemonScheduleMutationInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<DaemonScheduledGenerationJob, CommandError> {
    let client = ensure_daemon_client(&state)?;
    let library_id = client.open_library(&input.library_path)?;
    client.create_schedule(schedule_mutation_to_daemon_input(input, library_id))
}

#[tauri::command]
pub(crate) fn update_scheduled_generation_job(
    input: DaemonScheduleMutationInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<DaemonScheduledGenerationJob, CommandError> {
    let client = ensure_daemon_client(&state)?;
    let library_id = client.open_library(&input.library_path)?;
    let job_id = input.job_id.clone().ok_or_else(|| CommandError {
        code: "InvalidScheduleJob".to_string(),
        message: "schedule job id is required for update".to_string(),
        recoverable: true,
    })?;
    client.update_schedule(
        &job_id,
        schedule_mutation_to_daemon_input(input, library_id),
    )
}

#[tauri::command]
pub(crate) fn list_scheduled_generation_jobs(
    input: DaemonScheduleQueryInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<Vec<DaemonScheduledGenerationJob>, CommandError> {
    let client = ensure_daemon_client(&state)?;
    let library_id = client.open_library(&input.library_path)?;
    client.list_schedules(&library_id)
}

#[tauri::command]
pub(crate) fn enable_scheduled_generation_job(
    input: DaemonScheduleActionInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<DaemonScheduledGenerationJob, CommandError> {
    ensure_daemon_client(&state)?.enable_schedule(&input.job_id)
}

#[tauri::command]
pub(crate) fn disable_scheduled_generation_job(
    input: DaemonScheduleActionInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<DaemonScheduledGenerationJob, CommandError> {
    ensure_daemon_client(&state)?.disable_schedule(&input.job_id)
}

#[tauri::command]
pub(crate) fn delete_scheduled_generation_job(
    input: DaemonScheduleActionInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<(), CommandError> {
    ensure_daemon_client(&state)?.delete_schedule(&input.job_id)
}

#[tauri::command]
pub(crate) fn run_scheduled_generation_now(
    input: DaemonScheduleActionInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<DaemonScheduledGenerationRun, CommandError> {
    ensure_daemon_client(&state)?.run_schedule_now(&input.job_id)
}

#[tauri::command]
pub(crate) fn list_scheduled_generation_runs(
    input: DaemonScheduleActionInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<Vec<DaemonScheduledGenerationRun>, CommandError> {
    ensure_daemon_client(&state)?.list_schedule_runs(&input.job_id)
}

#[tauri::command]
pub(crate) fn get_scheduled_generation_run(
    input: DaemonScheduleRunActionInput,
    state: tauri::State<'_, DesktopState>,
) -> Result<DaemonScheduledGenerationRun, CommandError> {
    ensure_daemon_client(&state)?.get_schedule_run(&input.job_id, &input.run_id)
}

fn schedule_mutation_to_daemon_input(
    input: DaemonScheduleMutationInput,
    library_id: String,
) -> DaemonScheduleInput {
    DaemonScheduleInput {
        library_id,
        name: input.name,
        prompt_mode: input.prompt_mode,
        fixed_prompt: input.fixed_prompt,
        negative_prompt: input.negative_prompt,
        base_prompt: input.base_prompt,
        dynamic_prompt: input.dynamic_prompt,
        prompt_expander_provider: input.prompt_expander_provider,
        prompt_expander_model: input.prompt_expander_model,
        image_provider: input.image_provider,
        image_model: input.image_model,
        parameters: input.parameters,
        schedule_rule: DaemonScheduleRuleInput {
            kind: input.schedule_rule.kind,
            minutes: input.schedule_rule.minutes,
            hours: input.schedule_rule.hours,
            timezone_id: input.schedule_rule.timezone_id,
            local_time_hh_mm: input.schedule_rule.local_time_hh_mm,
        },
        target_album_id: input.target_album_id,
        tags: input.tags,
        next_run_at: input.next_run_at,
    }
}

pub(crate) fn generation_draft_to_daemon_task(
    input: GenerationTaskDraftInput,
) -> Result<DaemonTaskInput, CommandError> {
    let parameters = input
        .parameters_json
        .as_deref()
        .map(serde_json::from_str::<serde_json::Value>)
        .transpose()
        .map_err(|error| CommandError {
            code: "InvalidGenerationParameters".to_string(),
            message: format!("invalid generation parameters JSON: {error}"),
            recoverable: true,
        })?;
    Ok(DaemonTaskInput {
        task_type: input
            .task_type
            .unwrap_or_else(|| "image_generation".to_string()),
        provider: Some(input.provider),
        operation: Some(
            input
                .operation
                .unwrap_or_else(|| "text_to_image".to_string()),
        ),
        priority: input.priority,
        concurrency_group: None,
        max_attempts: input.max_attempts,
        input: input.input.unwrap_or_else(|| {
            serde_json::json!({
                "prompt": input.prompt,
                "negativePrompt": input.negative_prompt,
                "promptVersionId": input.prompt_version_id,
                "model": input.model,
                "valuesJson": input.values_json,
                "inputFile": input.input_file,
                "inputVersionId": input.input_version_id,
                "parametersJson": parameters,
            })
        }),
    })
}

pub(crate) fn execute_generation(
    input: GenerateImageInput,
    log_path: Option<PathBuf>,
) -> Result<Vec<VersionView>, CommandError> {
    let prepared = prepare_generation_request(GenerationRequestInput {
        library_path: input.library_path,
        provider: input.provider,
        prompt: input.prompt,
        negative_prompt: input.negative_prompt,
        model: None,
        operation: None,
        input_file: input.input_file,
        input_version_id: input.input_version_id.map(imglab_core::AssetVersionId),
        prompt_version_id: None,
        parameters_json: input.parameters_json,
    })?;

    match prepared.provider.as_str() {
        "codex" | "codex-cli" => run_generation(
            codex_provider(&prepared.request.library_path, log_path),
            prepared.request,
        ),
        "fake" => run_generation(
            imglab_core::FakeImageProvider::success("fake"),
            prepared.request,
        ),
        _ => unreachable!("provider is normalized before dispatch"),
    }
}

pub(crate) fn codex_provider(
    library_path: &PathBuf,
    log_path: Option<PathBuf>,
) -> CodexCliImageProvider {
    let provider = CodexCliImageProvider::new("codex", library_path);
    match log_path {
        Some(path) => provider.with_log_path(path),
        None => provider,
    }
}
