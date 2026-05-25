use crate::executors::*;
use crate::routes::{io_error, serialization_error};
use crate::runtime::*;
use crate::*;

pub(crate) fn execute_task(
    state: &mut DaemonState,
    library_path: &Path,
    task_id: &TaskId,
    retry_policy: &RetryPolicy,
) -> DomainResult<TaskSummary> {
    let Some(claimed_task) = state.tasks().claim_queued_task(library_path, task_id)? else {
        return Ok(state.tasks().get_task_detail(library_path, task_id)?.task);
    };
    fs::create_dir_all(&state.log_root).map_err(|error| io_error(&state.log_root, error))?;
    let log_path = state.log_root.join(format!(
        "imglab-task-{}-attempt-{}.log",
        task_id.0,
        claimed_task.attempt_count + 1
    ));
    fs::write(&log_path, format!("starting task {}\n", task_id.0))
        .map_err(|error| io_error(&log_path, error))?;

    let attempt = state
        .tasks()
        .append_task_attempt(imglab_core::AppendTaskAttemptRequest {
            library_path: library_path.to_path_buf(),
            task_id: task_id.clone(),
            status: "running".to_string(),
            log_path: Some(log_path.clone()),
        })?;
    state
        .tasks()
        .append_task_event(imglab_core::AppendTaskEventRequest {
            library_path: library_path.to_path_buf(),
            task_id: task_id.clone(),
            event_type: "attempt_started".to_string(),
            message: Some(format!("Attempt {} started", attempt.attempt_number)),
            payload_json: Some(format!("{{\"attempt_id\":\"{}\"}}", attempt.id.0)),
        })?;

    match execute_task_body(state, library_path, task_id, &claimed_task, &log_path) {
        Ok(()) => complete_successful_attempt(state, library_path, task_id, &attempt.id, &log_path),
        Err(error) => {
            if cancel_marker_path(&log_path).exists()
                || state
                    .tasks()
                    .get_task_detail(library_path, task_id)?
                    .task
                    .status
                    == TaskStatus::CancelRequested
            {
                complete_canceled_attempt(state, library_path, task_id, &attempt.id, &log_path)
            } else {
                complete_failed_attempt(
                    state,
                    library_path,
                    task_id,
                    &attempt.id,
                    &log_path,
                    error,
                    retry_policy,
                )
            }
        }
    }
}

pub(crate) fn execute_task_body(
    state: &mut DaemonState,
    library_path: &Path,
    task_id: &TaskId,
    task: &TaskSummary,
    log_path: &Path,
) -> DomainResult<()> {
    match task.task_type {
        TaskType::ImageGeneration => {
            execute_image_generation_task(state, library_path, task_id, task, log_path)
        }
        TaskType::MetadataFieldGeneration => {
            execute_metadata_field_task(state, library_path, task_id, task, log_path)
        }
        TaskType::MetadataSuggestionGeneration => {
            execute_metadata_suggestion_task(state, library_path, task_id, task, log_path)
        }
    }
}

pub(crate) fn execute_image_generation_task(
    state: &mut DaemonState,
    library_path: &Path,
    task_id: &TaskId,
    task: &TaskSummary,
    log_path: &Path,
) -> DomainResult<()> {
    let input: ImageGenerationTaskInput =
        serde_json::from_str(&task.input_json).map_err(serialization_error)?;
    let provider = task.provider.as_deref().unwrap_or("fake");
    append_log(log_path, &format!("provider: {provider}\n"))?;
    let fake_mode = input.fake_mode.clone();
    let prepared = prepare_generation_request(GenerationRequestInput {
        library_path: library_path.to_path_buf(),
        provider: provider.to_string(),
        prompt: input.prompt,
        negative_prompt: input.negative_prompt,
        model: input.model,
        operation: task.operation,
        input_file: input.input_file,
        input_version_id: input.input_version_id.map(imglab_core::AssetVersionId),
        prompt_version_id: input.prompt_version_id.map(imglab_core::PromptVersionId),
        parameters_json: input
            .parameters_json
            .or(input.parameters)
            .map(|value| value.to_string()),
    })?;

    let versions = match prepared.provider.as_str() {
        "fake" => {
            if fake_mode.as_deref() == Some("slow_success") {
                wait_for_fake_slow_task(state, library_path, task_id, log_path)?;
            }
            let fake = match fake_mode.as_deref() {
                Some("timeout") => imglab_core::FakeImageProvider::timeout("fake"),
                Some("failure") => {
                    imglab_core::FakeImageProvider::failure("fake", "manual retry failure")
                }
                Some("invalid_parameters") => imglab_core::FakeImageProvider::invalid_parameters(
                    "fake",
                    "invalid fake parameters",
                ),
                _ => imglab_core::FakeImageProvider::success("fake"),
            };
            imglab_core::infrastructure::composition::sqlite_application(
                state.registry_path.clone(),
                fake,
            )
            .generation()
            .execute(prepared.request)?
        }
        "codex" | "codex-cli" => {
            let codex = CodexCliImageProvider::new("codex", library_path)
                .with_log_path(log_path)
                .with_cancel_path(cancel_marker_path(log_path));
            imglab_core::infrastructure::composition::sqlite_application(
                state.registry_path.clone(),
                codex,
            )
            .generation()
            .execute(prepared.request)?
        }
        other => {
            return Err(DomainError::UnsupportedProvider {
                provider: other.to_string(),
            });
        }
    };

    for version in versions {
        let source_reference = state
            .gallery()
            .get_asset_detail(library_path, &version.asset_id, Some(&version.id))
            .ok()
            .and_then(|detail| detail.source_reference);
        state
            .tasks()
            .append_task_output(imglab_core::AppendTaskOutputRequest {
                library_path: library_path.to_path_buf(),
                task_id: task_id.clone(),
                output_type: TaskOutputType::Asset,
                target_id: version.asset_id.0.clone(),
                payload_json: Some(
                    serde_json::json!({
                        "role": "output",
                        "versionNumber": version.version_number,
                        "versionName": version.version_name
                    })
                    .to_string(),
                ),
            })?;
        state
            .tasks()
            .append_task_output(imglab_core::AppendTaskOutputRequest {
                library_path: library_path.to_path_buf(),
                task_id: task_id.clone(),
                output_type: TaskOutputType::AssetVersion,
                target_id: version.id.0.clone(),
                payload_json: Some(
                    serde_json::json!({
                        "role": "output",
                        "versionNumber": version.version_number,
                        "versionName": version.version_name
                    })
                    .to_string(),
                ),
            })?;
        if let Some(event_id) = version.generation_event_id {
            state
                .tasks()
                .append_task_output(imglab_core::AppendTaskOutputRequest {
                    library_path: library_path.to_path_buf(),
                    task_id: task_id.clone(),
                    output_type: TaskOutputType::GenerationEvent,
                    target_id: event_id.0,
                    payload_json: None,
                })?;
        }
        if let Some(reference) = source_reference {
            state
                .tasks()
                .append_task_output(imglab_core::AppendTaskOutputRequest {
                    library_path: library_path.to_path_buf(),
                    task_id: task_id.clone(),
                    output_type: TaskOutputType::Asset,
                    target_id: reference.asset_id.0.clone(),
                    payload_json: Some(
                        serde_json::json!({
                            "role": "reference",
                            "versionId": reference.version_id.0,
                            "versionNumber": reference.version_number,
                            "versionName": reference.version_name
                        })
                        .to_string(),
                    ),
                })?;
            state
                .tasks()
                .append_task_output(imglab_core::AppendTaskOutputRequest {
                    library_path: library_path.to_path_buf(),
                    task_id: task_id.clone(),
                    output_type: TaskOutputType::AssetVersion,
                    target_id: reference.version_id.0,
                    payload_json: Some(
                        serde_json::json!({
                            "role": "reference",
                            "assetId": reference.asset_id.0,
                            "versionNumber": reference.version_number,
                            "versionName": reference.version_name
                        })
                        .to_string(),
                    ),
                })?;
        }
    }
    append_log(log_path, "task completed\n")
}

pub(crate) fn execute_metadata_field_task(
    state: &mut DaemonState,
    library_path: &Path,
    task_id: &TaskId,
    task: &TaskSummary,
    log_path: &Path,
) -> DomainResult<()> {
    let input: MetadataFieldTaskInput =
        serde_json::from_str(&task.input_json).map_err(serialization_error)?;
    append_log(
        log_path,
        &format!("metadata field generation: {}\n", input.field),
    )?;
    let value = generated_metadata_field_value(&input.field, &input.context)?;
    state
        .tasks()
        .append_task_output(imglab_core::AppendTaskOutputRequest {
            library_path: library_path.to_path_buf(),
            task_id: task_id.clone(),
            output_type: TaskOutputType::MetadataFieldResult,
            target_id: input.suggestion_id,
            payload_json: Some(
                serde_json::json!({
                    "field": input.field,
                    "value": value,
                    "assetId": input.asset_id,
                    "baseRevision": input.base_revision,
                })
                .to_string(),
            ),
        })?;
    append_log(log_path, "metadata field task completed\n")
}

pub(crate) fn execute_metadata_suggestion_task(
    state: &mut DaemonState,
    library_path: &Path,
    task_id: &TaskId,
    task: &TaskSummary,
    log_path: &Path,
) -> DomainResult<()> {
    let input: MetadataSuggestionTaskInput =
        serde_json::from_str(&task.input_json).map_err(serialization_error)?;
    append_log(log_path, "metadata suggestion generation\n")?;
    let title = generated_metadata_field_value("title", &input.context)?;
    let description = generated_metadata_field_value("description", &input.context)?;
    let schema_prompt = generated_metadata_field_value("schemaPrompt", &input.context)?;
    let suggestion = state.create_metadata_suggestion(CreateMetadataSuggestionRequest {
        library_path: library_path.to_path_buf(),
        asset_id: AssetId(input.asset_id.clone()),
        source: "daemon_metadata_task".to_string(),
        suggested_title: Some(title),
        suggested_description: Some(description),
        suggested_schema_prompt: Some(schema_prompt),
        suggested_tags: input.context.tags,
        suggested_category: input.context.category,
        confidence_json: "{}".to_string(),
    })?;
    state
        .tasks()
        .append_task_output(imglab_core::AppendTaskOutputRequest {
            library_path: library_path.to_path_buf(),
            task_id: task_id.clone(),
            output_type: TaskOutputType::MetadataSuggestion,
            target_id: suggestion.id.0,
            payload_json: Some(
                serde_json::json!({
                    "sourceSuggestionId": input.suggestion_id,
                    "assetId": input.asset_id,
                    "baseRevision": input.base_revision,
                })
                .to_string(),
            ),
        })?;
    append_log(log_path, "metadata suggestion task completed\n")
}

pub(crate) fn generated_metadata_field_value(
    field: &str,
    context: &MetadataTaskContext,
) -> DomainResult<String> {
    let source = context
        .source_prompt
        .as_deref()
        .or(context.title.as_deref())
        .unwrap_or("generated asset");
    match field {
        "title" => Ok(format!("Generated title for {source}")),
        "description" => Ok(format!(
            "Generated description from {}",
            context
                .description
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or(source)
        )),
        "schemaPrompt" => {
            let value = context
                .schema_prompt
                .clone()
                .filter(|text| serde_json::from_str::<Value>(text).is_ok())
                .unwrap_or_else(|| {
                    serde_json::json!({
                        "OUTPUT": {
                            "title": context.title,
                            "category": context.category,
                            "tags": context.tags,
                        }
                    })
                    .to_string()
                });
            serde_json::from_str::<Value>(&value).map_err(|error| {
                DomainError::InvalidGenerationParameters {
                    message: format!("generated schema prompt is not valid JSON: {error}"),
                }
            })?;
            Ok(value)
        }
        other => Err(DomainError::InvalidGenerationParameters {
            message: format!("unsupported metadata field: {other}"),
        }),
    }
}

pub(crate) fn wait_for_fake_slow_task(
    state: &DaemonState,
    library_path: &Path,
    task_id: &TaskId,
    log_path: &Path,
) -> DomainResult<()> {
    for _ in 0..50 {
        if cancel_marker_path(log_path).exists()
            || state
                .tasks()
                .get_task_detail(library_path, task_id)?
                .task
                .status
                == TaskStatus::CancelRequested
        {
            return Err(DomainError::GenerationFailed {
                provider: "fake".to_string(),
                message: "task canceled by user".to_string(),
            });
        }
        thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}

pub(crate) fn complete_successful_attempt(
    state: &mut DaemonState,
    library_path: &Path,
    task_id: &TaskId,
    attempt_id: &imglab_core::TaskAttemptId,
    log_path: &Path,
) -> DomainResult<TaskSummary> {
    state
        .tasks()
        .complete_task_attempt(imglab_core::CompleteTaskAttemptRequest {
            library_path: library_path.to_path_buf(),
            attempt_id: attempt_id.clone(),
            status: "completed".to_string(),
            exit_code: Some(0),
            error_code: None,
            error_message: None,
            error_classification: None,
        })?;
    let current = state.tasks().get_task_detail(library_path, task_id)?.task;
    let resolution = imglab_core::domain::task::resolve_successful_attempt(current.status);
    if let (Some(event_type), Some(message)) =
        (resolution.extra_event_type, resolution.extra_event_message)
    {
        state
            .tasks()
            .append_task_event(imglab_core::AppendTaskEventRequest {
                library_path: library_path.to_path_buf(),
                task_id: task_id.clone(),
                event_type: event_type.to_string(),
                message: Some(message.to_string()),
                payload_json: None,
            })?;
    }
    append_log(log_path, "attempt completed\n")?;
    state
        .tasks()
        .append_task_event(imglab_core::AppendTaskEventRequest {
            library_path: library_path.to_path_buf(),
            task_id: task_id.clone(),
            event_type: "attempt_completed".to_string(),
            message: Some("Attempt completed".to_string()),
            payload_json: None,
        })?;
    state.tasks().update_task_status(UpdateTaskStatusRequest {
        library_path: library_path.to_path_buf(),
        task_id: task_id.clone(),
        status: resolution.task_status,
        next_retry_at: None,
        last_error_code: None,
        last_error_message: None,
        error_classification: None,
        wait_reason: None,
    })
}

pub(crate) fn complete_canceled_attempt(
    state: &mut DaemonState,
    library_path: &Path,
    task_id: &TaskId,
    attempt_id: &imglab_core::TaskAttemptId,
    log_path: &Path,
) -> DomainResult<TaskSummary> {
    append_log(log_path, "attempt canceled\n")?;
    state
        .tasks()
        .complete_task_attempt(imglab_core::CompleteTaskAttemptRequest {
            library_path: library_path.to_path_buf(),
            attempt_id: attempt_id.clone(),
            status: "canceled".to_string(),
            exit_code: None,
            error_code: Some("Canceled".to_string()),
            error_message: Some("Task canceled by user".to_string()),
            error_classification: Some(TaskErrorClassification::Cancel),
        })?;
    state
        .tasks()
        .append_task_event(imglab_core::AppendTaskEventRequest {
            library_path: library_path.to_path_buf(),
            task_id: task_id.clone(),
            event_type: "attempt_canceled".to_string(),
            message: Some("Attempt canceled by user request".to_string()),
            payload_json: None,
        })?;
    state.tasks().update_task_status(UpdateTaskStatusRequest {
        library_path: library_path.to_path_buf(),
        task_id: task_id.clone(),
        status: imglab_core::domain::task::resolve_canceled_attempt_status(),
        next_retry_at: None,
        last_error_code: Some("Canceled".to_string()),
        last_error_message: Some("Task canceled by user".to_string()),
        error_classification: Some(TaskErrorClassification::Cancel),
        wait_reason: None,
    })
}

pub(crate) fn complete_failed_attempt(
    state: &mut DaemonState,
    library_path: &Path,
    task_id: &TaskId,
    attempt_id: &imglab_core::TaskAttemptId,
    log_path: &Path,
    error: DomainError,
    retry_policy: &RetryPolicy,
) -> DomainResult<TaskSummary> {
    let classification = classify_task_error(&error);
    let detail = state.tasks().get_task_detail(library_path, task_id)?;
    let should_retry = retry_policy.should_auto_retry(classification, detail.task.attempt_count);
    let resolution =
        imglab_core::domain::task::resolve_failed_attempt(classification, should_retry);
    let next_retry_at = should_retry.then(|| {
        unix_timestamp_string(retry_policy.backoff_delay_seconds(detail.task.attempt_count))
    });
    append_log(log_path, &format!("attempt failed: {error}\n"))?;
    state
        .tasks()
        .complete_task_attempt(imglab_core::CompleteTaskAttemptRequest {
            library_path: library_path.to_path_buf(),
            attempt_id: attempt_id.clone(),
            status: "failed".to_string(),
            exit_code: Some(1),
            error_code: Some(error.code().to_string()),
            error_message: Some(error.to_string()),
            error_classification: Some(classification),
        })?;
    state
        .tasks()
        .append_task_event(imglab_core::AppendTaskEventRequest {
            library_path: library_path.to_path_buf(),
            task_id: task_id.clone(),
            event_type: resolution.event_type.to_string(),
            message: Some(error.to_string()),
            payload_json: next_retry_at
                .as_ref()
                .map(|value| format!("{{\"next_retry_at\":\"{value}\"}}")),
        })?;
    state.tasks().update_task_status(UpdateTaskStatusRequest {
        library_path: library_path.to_path_buf(),
        task_id: task_id.clone(),
        status: resolution.task_status,
        next_retry_at,
        last_error_code: Some(error.code().to_string()),
        last_error_message: Some(error.to_string()),
        error_classification: Some(classification),
        wait_reason: None,
    })
}
