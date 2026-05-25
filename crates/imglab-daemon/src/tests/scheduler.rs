use super::*;
use crate::executors::unix_timestamp_millis_string;

#[test]
fn scheduler_tick_executes_fake_image_task_and_links_outputs() {
    let mut state = test_state("worker-success");
    let library_id = create_open_library(&mut state, "worker-success-library");
    let library_path = state.library_path(&library_id).expect("library path");
    let create_response = handle_http_request_with_state(
        &json_request(
            "POST",
            TASKS_PATH,
            serde_json::json!({
                "libraryId": library_id,
                "taskType": "image_generation",
                "provider": "fake",
                "operation": "text_to_image",
                "input": { "prompt": "make a worker image" }
            }),
        ),
        "secret",
        &mut state,
    );
    let task_id = json_value(&create_response)[0]["id"]
        .as_str()
        .expect("task id")
        .to_string();

    let task = run_scheduler_tick(
        &mut state,
        &TaskSchedulerConfig::default(),
        &RetryPolicy::default(),
    )
    .expect("run tick")
    .expect("executed task");
    assert_eq!(task.id.0, task_id);
    assert_eq!(task.status, TaskStatus::Completed);

    let detail = state
        .tasks()
        .get_task_detail(&library_path, &TaskId(task_id.clone()))
        .expect("detail");
    assert_eq!(detail.attempts[0].status, "completed");
    assert!(detail
        .outputs
        .iter()
        .any(|output| output.output_type == TaskOutputType::AssetVersion));
    let generation_event = output_generation_event(&state, &library_path, &detail);
    assert!(generation_event.prompt_version_id.is_none());
    assert!(detail
        .events
        .iter()
        .any(|event| event.event_type == "attempt_completed"));

    let log_response = handle_http_request_with_state(
        &auth_get(&format!("{TASKS_PATH}/{task_id}/logs/tail")),
        "secret",
        &mut state,
    );
    assert_eq!(log_response.status_code, 200);
    assert!(json_value(&log_response)["content"]
        .as_str()
        .expect("log content")
        .contains("task completed"));
}

#[test]
fn schedule_api_run_now_queues_task_and_reconciles_completion() {
    let mut state = test_state("schedule-run-now");
    let library_id = create_open_library(&mut state, "schedule-run-now-library");
    let library_path = state.library_path(&library_id).expect("library path");
    let album = state
        .app
        .albums()
        .create_manual_album_in_library(&library_path, "Scheduled outputs")
        .expect("create album");
    let create_response = handle_http_request_with_state(
        &json_request(
            "POST",
            SCHEDULES_PATH,
            serde_json::json!({
                "libraryId": library_id,
                "name": "Daily fixed image",
                "promptMode": "fixed",
                "fixedPrompt": "make a scheduled image",
                "negativePrompt": null,
                "imageProvider": "fake",
                "imageModel": "fake",
                "parameters": {},
                "scheduleRule": {
                    "kind": "interval_minutes",
                    "minutes": 5
                },
                "targetAlbumId": album.id.0,
                "tags": ["scheduled", "fake"],
                "nextRunAt": "0"
            }),
        ),
        "secret",
        &mut state,
    );
    assert_eq!(create_response.status_code, 200);
    let job_id = json_value(&create_response)["id"]
        .as_str()
        .expect("job id")
        .to_string();

    let run_response = handle_http_request_with_state(
        &json_request(
            "POST",
            &format!("{SCHEDULES_PATH}/{job_id}/run-now"),
            serde_json::json!({}),
        ),
        "secret",
        &mut state,
    );
    assert_eq!(run_response.status_code, 200);
    let run = json_value(&run_response);
    assert_eq!(run["status"], "task_queued");
    let task_id = run["imageTaskId"].as_str().expect("task id").to_string();
    let library_path = state.library_path(&library_id).expect("library path");
    let detail = state
        .tasks()
        .get_task_detail(&library_path, &TaskId(task_id.clone()))
        .expect("task detail");
    let task_input: serde_json::Value =
        serde_json::from_str(&detail.task.input_json).expect("task input json");
    assert_eq!(task_input["schedule"]["jobId"], job_id);
    assert_eq!(task_input["schedule"]["runId"], run["id"]);

    let executed = run_scheduler_tick(
        &mut state,
        &TaskSchedulerConfig::default(),
        &RetryPolicy::default(),
    )
    .expect("run task scheduler")
    .expect("executed task");
    assert_eq!(executed.id.0, task_id);
    assert_eq!(executed.status, TaskStatus::Completed);

    let changed = run_schedule_tick(&mut state).expect("reconcile schedule");
    let completed_run = changed
        .iter()
        .find(|run| {
            run.image_task_id.as_ref() == Some(&TaskId(task_id.clone()))
                && run.status == ScheduledGenerationRunStatus::Completed
                && run.output_asset_count == 1
                && run.album_added_asset_count == 1
                && run.tagged_asset_count == 1
        })
        .expect("completed run");
    let album_items = state
        .gallery()
        .query_gallery(
            &library_path,
            imglab_core::GalleryQuery {
                album_id: Some(album.id.clone()),
                sort: imglab_core::GallerySort::AlbumOrder,
                ..Default::default()
            },
        )
        .expect("album gallery");
    assert_eq!(album_items.len(), 1);
    assert!(album_items[0].tags.contains(&"scheduled".to_string()));
    assert!(album_items[0].tags.contains(&"fake".to_string()));

    let unchanged = run_schedule_tick(&mut state).expect("reconcile again");
    assert!(unchanged.is_empty());
    let runs = state
        .schedules()
        .list_runs(
            &library_path,
            &imglab_core::ScheduledGenerationJobId(job_id),
        )
        .expect("runs");
    let persisted_run = runs
        .iter()
        .find(|run| run.id == completed_run.id)
        .expect("persisted run");
    assert_eq!(persisted_run.output_asset_count, 1);
    assert_eq!(persisted_run.album_added_asset_count, 1);
    assert_eq!(persisted_run.tagged_asset_count, 1);
}

#[test]
fn dynamic_schedule_api_expands_prompt_before_queuing_task() {
    let mut state = test_state("schedule-dynamic-run-now");
    let library_id = create_open_library(&mut state, "schedule-dynamic-run-now-library");
    let library_path = state.library_path(&library_id).expect("library path");
    let album = state
        .app
        .albums()
        .create_manual_album_in_library(&library_path, "Dynamic scheduled outputs")
        .expect("create album");
    let create_response = handle_http_request_with_state(
        &json_request(
            "POST",
            SCHEDULES_PATH,
            serde_json::json!({
                "libraryId": library_id,
                "name": "Dynamic image",
                "promptMode": "dynamic",
                "basePrompt": "make a studio image",
                "dynamicPrompt": "add one seasonal detail",
                "promptExpanderProvider": "fake",
                "promptExpanderModel": "fake",
                "imageProvider": "fake",
                "imageModel": "fake",
                "parameters": {},
                "scheduleRule": {
                    "kind": "interval_minutes",
                    "minutes": 5
                },
                "targetAlbumId": album.id.0,
                "tags": ["dynamic"],
                "nextRunAt": "0"
            }),
        ),
        "secret",
        &mut state,
    );
    assert_eq!(create_response.status_code, 200);
    let job_id = json_value(&create_response)["id"]
        .as_str()
        .expect("job id")
        .to_string();

    let run_response = handle_http_request_with_state(
        &json_request(
            "POST",
            &format!("{SCHEDULES_PATH}/{job_id}/run-now"),
            serde_json::json!({}),
        ),
        "secret",
        &mut state,
    );
    assert_eq!(run_response.status_code, 200);
    let run = json_value(&run_response);
    assert_eq!(run["status"], "task_queued");
    assert!(run["expandedPrompt"]
        .as_str()
        .expect("expanded prompt")
        .contains("add one seasonal detail"));
    let task_id = run["imageTaskId"].as_str().expect("task id").to_string();
    let detail = state
        .tasks()
        .get_task_detail(&library_path, &TaskId(task_id))
        .expect("task detail");
    let task_input: serde_json::Value =
        serde_json::from_str(&detail.task.input_json).expect("task input json");
    assert_eq!(task_input["prompt"], run["expandedPrompt"]);
}

#[test]
fn schedule_loop_recovers_automation_enabled_libraries() {
    let state = test_state("schedule-auto-recover");
    let library_root = test_root("schedule-auto-recover-library").join("library");
    let library = state
        .library_lifecycle()
        .create_library(CreateLibraryRequest {
            root_path: library_root.clone(),
            name: "Recoverable".to_string(),
        })
        .expect("create library");
    let album = state
        .app
        .albums()
        .create_manual_album_in_library(&library_root, "Scheduled outputs")
        .expect("create album");
    state
        .schedules()
        .set_library_automation_enabled(&library.id, true)
        .expect("enable automation");
    state
        .schedules()
        .create_job(imglab_core::CreateScheduledGenerationJobRequest {
            library_path: library_root.clone(),
            library_id: library.id.clone(),
            name: "Recovered job".to_string(),
            prompt_mode: SchedulePromptMode::Fixed,
            fixed_prompt: Some("make a recovered scheduled image".to_string()),
            negative_prompt: None,
            base_prompt: None,
            dynamic_prompt: None,
            prompt_expander_provider: None,
            prompt_expander_model: None,
            image_provider: "fake".to_string(),
            image_model: "fake".to_string(),
            parameters_json: "{}".to_string(),
            schedule_rule: imglab_core::ScheduleRule::IntervalMinutes(5),
            target_album_id: album.id,
            tags: vec!["scheduled".to_string()],
            next_run_at: unix_timestamp_millis_string(0),
        })
        .expect("create schedule");
    assert!(state.opened_libraries.is_empty());

    let shared = Arc::new(Mutex::new(state));
    let changed = run_schedule_loop_iteration(&shared).expect("run schedule loop");
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].status, ScheduledGenerationRunStatus::TaskQueued);
    let guard = shared.lock().expect("lock state");
    assert!(guard.opened_libraries.contains_key(&library.id.0));
}

#[test]
fn dynamic_schedule_failure_does_not_create_image_task() {
    let mut state = test_state("schedule-dynamic-failure");
    let library_id = create_open_library(&mut state, "schedule-dynamic-failure-library");
    let library_path = state.library_path(&library_id).expect("library path");
    let album = state
        .app
        .albums()
        .create_manual_album_in_library(&library_path, "Scheduled outputs")
        .expect("create album");
    let create_response = handle_http_request_with_state(
        &json_request(
            "POST",
            SCHEDULES_PATH,
            serde_json::json!({
                "libraryId": library_id,
                "name": "Invalid dynamic image",
                "promptMode": "dynamic",
                "basePrompt": "base prompt",
                "dynamicPrompt": "",
                "promptExpanderProvider": "fake",
                "promptExpanderModel": "fake",
                "imageProvider": "fake",
                "imageModel": "fake",
                "parameters": {},
                "scheduleRule": {
                    "kind": "interval_minutes",
                    "minutes": 5
                },
                "targetAlbumId": album.id.0,
                "tags": [],
                "nextRunAt": unix_timestamp_millis_string(0)
            }),
        ),
        "secret",
        &mut state,
    );
    assert_eq!(create_response.status_code, 200);

    let changed = run_schedule_tick(&mut state).expect("run schedule");
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].status, ScheduledGenerationRunStatus::Failed);
    assert!(changed[0].image_task_id.is_none());
    assert!(state
        .tasks()
        .list_tasks(&library_path)
        .expect("list tasks")
        .is_empty());
}

#[test]
fn due_schedule_skips_when_previous_run_is_active() {
    let mut state = test_state("schedule-overlap-skip");
    let library_id = create_open_library(&mut state, "schedule-overlap-skip-library");
    let library_path = state.library_path(&library_id).expect("library path");
    let album = state
        .app
        .albums()
        .create_manual_album_in_library(&library_path, "Scheduled outputs")
        .expect("create album");
    let job = state
        .schedules()
        .create_job(imglab_core::CreateScheduledGenerationJobRequest {
            library_path: library_path.clone(),
            library_id: LibraryId(library_id.clone()),
            name: "Overlap skip".to_string(),
            prompt_mode: SchedulePromptMode::Fixed,
            fixed_prompt: Some("make an image".to_string()),
            negative_prompt: None,
            base_prompt: None,
            dynamic_prompt: None,
            prompt_expander_provider: None,
            prompt_expander_model: None,
            image_provider: "fake".to_string(),
            image_model: "fake".to_string(),
            parameters_json: "{}".to_string(),
            schedule_rule: imglab_core::ScheduleRule::IntervalMinutes(5),
            target_album_id: album.id,
            tags: vec![],
            next_run_at: unix_timestamp_millis_string(0),
        })
        .expect("create job");
    let task = state
        .tasks()
        .create_tasks(BatchCreateTasksRequest {
            library_path: library_path.clone(),
            library_id: LibraryId(library_id.clone()),
            tasks: vec![CreateTaskInput {
                task_type: TaskType::ImageGeneration,
                provider: Some("fake".to_string()),
                operation: Some(GenerationOperation::TextToImage),
                priority: 0,
                concurrency_group: Some(format!("schedule:{}", job.id.0)),
                max_attempts: 3,
                input_json: serde_json::json!({ "prompt": "already queued" }).to_string(),
            }],
        })
        .expect("create queued task")
        .into_iter()
        .next()
        .expect("queued task");
    let run = state
        .schedules()
        .create_run(CreateScheduledGenerationRunRequest {
            library_path: library_path.clone(),
            job_id: job.id.clone(),
            library_id: LibraryId(library_id.clone()),
            scheduled_for: job.next_run_at.clone(),
        })
        .expect("create active run");
    state
        .schedules()
        .update_run(UpdateScheduledGenerationRunRequest {
            library_path: library_path.clone(),
            run_id: run.id,
            status: ScheduledGenerationRunStatus::TaskQueued,
            started_at: Some(unix_timestamp_millis_string(0)),
            completed_at: None,
            skip_reason: None,
            error_code: None,
            error_message: None,
            expanded_prompt: Some("already queued".to_string()),
            prompt_expansion_provider_metadata_json: None,
            image_task_id: Some(task.id),
        })
        .expect("mark run task queued");

    let changed = run_schedule_tick(&mut state).expect("run schedule");
    assert!(changed.iter().any(|run| {
        run.status == ScheduledGenerationRunStatus::Skipped
            && run.skip_reason.as_deref() == Some("previous_run_active")
    }));
}

#[test]
fn schedule_tick_recovers_interrupted_prompt_expansion_run() {
    let mut state = test_state("schedule-interrupted-expansion");
    let library_id = create_open_library(&mut state, "schedule-interrupted-expansion-library");
    let library_path = state.library_path(&library_id).expect("library path");
    let album = state
        .app
        .albums()
        .create_manual_album_in_library(&library_path, "Scheduled outputs")
        .expect("create album");
    let job = state
        .schedules()
        .create_job(imglab_core::CreateScheduledGenerationJobRequest {
            library_path: library_path.clone(),
            library_id: LibraryId(library_id.clone()),
            name: "Interrupted dynamic image".to_string(),
            prompt_mode: SchedulePromptMode::Dynamic,
            fixed_prompt: None,
            negative_prompt: None,
            base_prompt: Some("make an image".to_string()),
            dynamic_prompt: Some("add a cinematic detail".to_string()),
            prompt_expander_provider: Some("fake".to_string()),
            prompt_expander_model: Some("fake".to_string()),
            image_provider: "fake".to_string(),
            image_model: "fake".to_string(),
            parameters_json: "{}".to_string(),
            schedule_rule: imglab_core::ScheduleRule::IntervalMinutes(5),
            target_album_id: album.id,
            tags: vec![],
            next_run_at: unix_timestamp_millis_string(0),
        })
        .expect("create job");
    state
        .schedules()
        .create_run(CreateScheduledGenerationRunRequest {
            library_path: library_path.clone(),
            job_id: job.id,
            library_id: LibraryId(library_id),
            scheduled_for: job.next_run_at,
        })
        .expect("create interrupted run");

    let changed = run_schedule_tick(&mut state).expect("run schedule");

    assert!(changed.iter().any(|run| {
        run.status == ScheduledGenerationRunStatus::Failed
            && run.error_code.as_deref() == Some("DaemonInterrupted")
            && run.image_task_id.is_none()
    }));
    assert!(changed
        .iter()
        .any(|run| run.status == ScheduledGenerationRunStatus::TaskQueued));
}

#[test]
fn missed_schedule_uses_no_catch_up_policy() {
    let mut state = test_state("schedule-missed-no-catch-up");
    let library_id = create_open_library(&mut state, "schedule-missed-no-catch-up-library");
    let library_path = state.library_path(&library_id).expect("library path");
    let album = state
        .app
        .albums()
        .create_manual_album_in_library(&library_path, "Scheduled outputs")
        .expect("create album");
    state
        .schedules()
        .create_job(imglab_core::CreateScheduledGenerationJobRequest {
            library_path: library_path.clone(),
            library_id: LibraryId(library_id),
            name: "Missed job".to_string(),
            prompt_mode: SchedulePromptMode::Fixed,
            fixed_prompt: Some("make an image".to_string()),
            negative_prompt: None,
            base_prompt: None,
            dynamic_prompt: None,
            prompt_expander_provider: None,
            prompt_expander_model: None,
            image_provider: "fake".to_string(),
            image_model: "fake".to_string(),
            parameters_json: "{}".to_string(),
            schedule_rule: imglab_core::ScheduleRule::IntervalHours(1),
            target_album_id: album.id,
            tags: vec![],
            next_run_at: "0".to_string(),
        })
        .expect("create job");

    let changed = run_schedule_tick(&mut state).expect("run schedule");
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].status, ScheduledGenerationRunStatus::Skipped);
    assert_eq!(
        changed[0].skip_reason.as_deref(),
        Some("missed_no_catch_up")
    );
    assert!(state
        .tasks()
        .list_tasks(&library_path)
        .expect("list tasks")
        .is_empty());
}

#[test]
fn scheduler_tick_preserves_prompt_version_id_on_image_generation_event() {
    let mut state = test_state("worker-prompt-version");
    let library_id = create_open_library(&mut state, "worker-prompt-version-library");
    let library_path = state.library_path(&library_id).expect("library path");
    let prompt = state
        .app
        .prompts()
        .create_prompt_document(CreatePromptDocumentRequest {
            library_path: library_path.clone(),
            name: "Daemon prompt".to_string(),
            draft_body: "rendered prompt snapshot".to_string(),
            draft_negative_prompt: Some("rendered negative prompt snapshot".to_string()),
            draft_style_prompt: None,
            variables_schema_json: r#"{"variables":[]}"#.to_string(),
            default_values_json: "{}".to_string(),
            parameter_preset_json: "{}".to_string(),
            notes: None,
        })
        .expect("create prompt");
    let prompt_version = state
        .app
        .prompts()
        .save_prompt_version(SavePromptVersionRequest {
            library_path: library_path.clone(),
            prompt_id: prompt.id.0,
        })
        .expect("save prompt version");
    let prompt_version_id = prompt_version.id.0;
    let create_response = handle_http_request_with_state(
        &json_request(
            "POST",
            TASKS_PATH,
            serde_json::json!({
                "libraryId": library_id,
                "taskType": "image_generation",
                "provider": "fake",
                "operation": "text_to_image",
                "input": {
                    "prompt": "rendered prompt snapshot",
                    "negativePrompt": "rendered negative prompt snapshot",
                    "promptVersionId": prompt_version_id.clone()
                }
            }),
        ),
        "secret",
        &mut state,
    );
    let task_id = json_value(&create_response)[0]["id"]
        .as_str()
        .expect("task id")
        .to_string();

    let task = run_scheduler_tick(
        &mut state,
        &TaskSchedulerConfig::default(),
        &RetryPolicy::default(),
    )
    .expect("run tick")
    .expect("executed task");
    assert_eq!(task.status, TaskStatus::Completed);

    let detail = state
        .tasks()
        .get_task_detail(&library_path, &TaskId(task_id))
        .expect("detail");
    let generation_event = output_generation_event(&state, &library_path, &detail);
    assert_eq!(
        generation_event
            .prompt_version_id
            .as_ref()
            .map(|id| id.0.as_str()),
        Some(prompt_version_id.as_str())
    );
    assert_eq!(generation_event.prompt, "rendered prompt snapshot");
}

fn output_generation_event(
    state: &DaemonState,
    library_path: &Path,
    detail: &TaskDetail,
) -> imglab_core::GenerationEventSummary {
    let asset_id = detail
        .outputs
        .iter()
        .find(|output| output.output_type == TaskOutputType::Asset)
        .expect("asset output")
        .target_id
        .clone();
    let version_id = detail
        .outputs
        .iter()
        .find(|output| output.output_type == TaskOutputType::AssetVersion)
        .expect("asset version output")
        .target_id
        .clone();
    let generated_asset = state
        .gallery()
        .get_asset_detail(
            library_path,
            &imglab_core::AssetId(asset_id),
            Some(&imglab_core::AssetVersionId(version_id)),
        )
        .expect("generated asset detail");
    generated_asset.lineage[0]
        .generation_event
        .as_ref()
        .expect("generation event")
        .clone()
}

#[test]
fn scheduler_tick_executes_metadata_field_task_and_records_output() {
    let mut state = test_state("metadata-field-worker");
    let library_id = create_open_library(&mut state, "metadata-field-worker-library");
    let create_response = handle_http_request_with_state(
        &json_request(
            "POST",
            TASKS_PATH,
            serde_json::json!({
                "libraryId": library_id,
                "taskType": "metadata_field_generation",
                "provider": "fake",
                "input": {
                    "suggestionId": "suggestion-1",
                    "assetId": "asset-1",
                    "field": "schemaPrompt",
                    "baseRevision": "revision-1",
                    "context": {
                        "title": "Draft title",
                        "description": "Draft description",
                        "schemaPrompt": "{\"OUTPUT\":{\"mood\":\"calm\"}}",
                        "tags": ["portrait", "studio"],
                        "category": "reference",
                        "sourcePrompt": "a calm studio portrait"
                    }
                }
            }),
        ),
        "secret",
        &mut state,
    );
    let task_id = json_value(&create_response)[0]["id"]
        .as_str()
        .expect("task id")
        .to_string();

    let task = run_scheduler_tick(
        &mut state,
        &TaskSchedulerConfig::default(),
        &RetryPolicy::default(),
    )
    .expect("run tick")
    .expect("executed task");
    assert_eq!(task.status, TaskStatus::Completed);

    let library_path = state.library_path(&library_id).expect("library path");
    let detail = state
        .tasks()
        .get_task_detail(&library_path, &TaskId(task_id.clone()))
        .expect("detail");
    let output = detail
        .outputs
        .iter()
        .find(|output| output.output_type == TaskOutputType::MetadataFieldResult)
        .expect("metadata field output");
    assert_eq!(output.target_id, "suggestion-1");
    let payload: Value = serde_json::from_str(
        output
            .payload_json
            .as_deref()
            .expect("metadata field payload"),
    )
    .expect("parse payload");
    assert_eq!(payload["field"], "schemaPrompt");
    assert_eq!(payload["assetId"], "asset-1");
    assert_eq!(payload["baseRevision"], "revision-1");
    assert!(
        serde_json::from_str::<Value>(payload["value"].as_str().expect("schema prompt value"))
            .is_ok()
    );

    let log_response = handle_http_request_with_state(
        &auth_get(&format!("{TASKS_PATH}/{task_id}/logs/tail")),
        "secret",
        &mut state,
    );
    assert_eq!(log_response.status_code, 200);
    assert!(json_value(&log_response)["content"]
        .as_str()
        .expect("log content")
        .contains("metadata field task completed"));
}

#[test]
fn scheduler_tick_executes_metadata_suggestion_task_for_generated_asset() {
    let mut state = test_state("metadata-suggestion-worker");
    let library_id = create_open_library(&mut state, "metadata-suggestion-worker-library");
    let image_response = handle_http_request_with_state(
        &json_request(
            "POST",
            TASKS_PATH,
            serde_json::json!({
                "libraryId": library_id,
                "taskType": "image_generation",
                "provider": "fake",
                "operation": "text_to_image",
                "input": { "prompt": "image for metadata suggestion" }
            }),
        ),
        "secret",
        &mut state,
    );
    let image_task_id = json_value(&image_response)[0]["id"]
        .as_str()
        .expect("image task id")
        .to_string();
    run_scheduler_tick(
        &mut state,
        &TaskSchedulerConfig::default(),
        &RetryPolicy::default(),
    )
    .expect("run image task")
    .expect("executed image task");

    let library_path = state.library_path(&library_id).expect("library path");
    let image_detail = state
        .tasks()
        .get_task_detail(&library_path, &TaskId(image_task_id))
        .expect("image detail");
    let asset_id = image_detail
        .outputs
        .iter()
        .find(|output| output.output_type == TaskOutputType::Asset)
        .expect("asset output")
        .target_id
        .clone();

    let suggestion_response = handle_http_request_with_state(
        &json_request(
            "POST",
            TASKS_PATH,
            serde_json::json!({
                "libraryId": library_id,
                "taskType": "metadata_suggestion_generation",
                "provider": "fake",
                "input": {
                    "suggestionId": "source-suggestion-1",
                    "assetId": asset_id,
                    "baseRevision": "revision-2",
                    "context": {
                        "title": "Existing title",
                        "description": "Existing description",
                        "schemaPrompt": "",
                        "tags": ["metadata"],
                        "category": "generated",
                        "sourcePrompt": "image for metadata suggestion"
                    }
                }
            }),
        ),
        "secret",
        &mut state,
    );
    let suggestion_task_id = json_value(&suggestion_response)[0]["id"]
        .as_str()
        .expect("suggestion task id")
        .to_string();

    let task = run_scheduler_tick(
        &mut state,
        &TaskSchedulerConfig::default(),
        &RetryPolicy::default(),
    )
    .expect("run suggestion task")
    .expect("executed suggestion task");
    assert_eq!(task.status, TaskStatus::Completed);

    let detail = state
        .tasks()
        .get_task_detail(&library_path, &TaskId(suggestion_task_id.clone()))
        .expect("suggestion detail");
    let output = detail
        .outputs
        .iter()
        .find(|output| output.output_type == TaskOutputType::MetadataSuggestion)
        .expect("metadata suggestion output");
    let payload: Value = serde_json::from_str(
        output
            .payload_json
            .as_deref()
            .expect("metadata suggestion payload"),
    )
    .expect("parse payload");
    assert_eq!(payload["sourceSuggestionId"], "source-suggestion-1");
    assert_eq!(payload["assetId"], asset_id);
    assert_eq!(payload["baseRevision"], "revision-2");

    let log_response = handle_http_request_with_state(
        &auth_get(&format!("{TASKS_PATH}/{suggestion_task_id}/logs/tail")),
        "secret",
        &mut state,
    );
    assert_eq!(log_response.status_code, 200);
    assert!(json_value(&log_response)["content"]
        .as_str()
        .expect("log content")
        .contains("metadata suggestion task completed"));
}

#[test]
fn scheduler_tick_marks_transient_failure_retry_waiting() {
    let mut state = test_state("worker-retry");
    let library_id = create_open_library(&mut state, "worker-retry-library");
    let create_response = handle_http_request_with_state(
        &json_request(
            "POST",
            TASKS_PATH,
            serde_json::json!({
                "libraryId": library_id,
                "taskType": "image_generation",
                "provider": "fake",
                "operation": "text_to_image",
                "input": {
                    "prompt": "this will time out",
                    "fakeMode": "timeout"
                }
            }),
        ),
        "secret",
        &mut state,
    );
    let task_id = json_value(&create_response)[0]["id"]
        .as_str()
        .expect("task id")
        .to_string();

    let task = run_scheduler_tick(
        &mut state,
        &TaskSchedulerConfig::default(),
        &RetryPolicy::default(),
    )
    .expect("run tick")
    .expect("executed task");
    assert_eq!(task.status, TaskStatus::RetryWaiting);
    assert!(task.next_retry_at.is_some());
    assert_eq!(
        task.error_classification,
        Some(TaskErrorClassification::Transient)
    );

    let library_path = state.library_path(&library_id).expect("library path");
    let detail = state
        .tasks()
        .get_task_detail(&library_path, &TaskId(task_id))
        .expect("detail");
    assert_eq!(detail.attempts[0].status, "failed");
    assert!(detail
        .events
        .iter()
        .any(|event| event.event_type == "retry_scheduled"));
}

#[test]
fn scheduler_tick_skips_canceled_tasks() {
    let mut state = test_state("worker-cancel");
    let library_id = create_open_library(&mut state, "worker-cancel-library");
    let create_response = handle_http_request_with_state(
        &json_request(
            "POST",
            TASKS_PATH,
            serde_json::json!({
                "libraryId": library_id,
                "taskType": "image_generation",
                "provider": "fake",
                "operation": "text_to_image",
                "input": { "prompt": "cancel me" }
            }),
        ),
        "secret",
        &mut state,
    );
    let task_id = json_value(&create_response)[0]["id"]
        .as_str()
        .expect("task id")
        .to_string();
    let cancel_response = handle_http_request_with_state(
        &json_request(
            "POST",
            &format!("{TASKS_PATH}/{task_id}/cancel"),
            serde_json::json!({}),
        ),
        "secret",
        &mut state,
    );
    assert_eq!(cancel_response.status_code, 200);

    let executed = run_scheduler_tick(
        &mut state,
        &TaskSchedulerConfig::default(),
        &RetryPolicy::default(),
    )
    .expect("run tick");
    assert!(executed.is_none());
}

#[test]
fn scheduler_tick_executes_multiple_tasks_up_to_global_limit() {
    let mut state = test_state("worker-multiple");
    let library_id = create_open_library(&mut state, "worker-multiple-library");
    let library_path = state.library_path(&library_id).expect("library path");
    let create_response = handle_http_request_with_state(
        &json_request(
            "POST",
            TASKS_BATCH_PATH,
            serde_json::json!({
                "libraryId": library_id,
                "tasks": [
                    {
                        "taskType": "image_generation",
                        "provider": "fake",
                        "operation": "text_to_image",
                        "input": { "prompt": "first" }
                    },
                    {
                        "taskType": "image_generation",
                        "provider": "fake",
                        "operation": "text_to_image",
                        "input": { "prompt": "second" }
                    },
                    {
                        "taskType": "image_generation",
                        "provider": "fake",
                        "operation": "text_to_image",
                        "input": { "prompt": "third" }
                    }
                ]
            }),
        ),
        "secret",
        &mut state,
    );
    assert_eq!(create_response.status_code, 200);
    let config = TaskSchedulerConfig {
        global_concurrency_limit: 2,
        ..Default::default()
    };

    let executed =
        run_scheduler_tick_batch(&mut state, &config, &RetryPolicy::default()).expect("run tick");

    assert_eq!(executed.len(), 2);
    assert!(executed
        .iter()
        .all(|task| task.status == TaskStatus::Completed));
    let tasks = state.tasks().list_tasks(&library_path).expect("tasks");
    let queued = tasks
        .iter()
        .find(|task| task.status == TaskStatus::Queued)
        .expect("queued task");
    assert_eq!(
        queued.wait_reason.as_deref(),
        Some("Waiting for global concurrency slot")
    );
}

#[test]
fn scheduler_tick_keeps_provider_slot_wait_reason_when_global_slots_remain() {
    let mut state = test_state("provider-slot");
    let library_id = create_open_library(&mut state, "provider-slot-library");
    let library_path = state.library_path(&library_id).expect("library path");
    let tasks = state
        .tasks()
        .create_tasks(BatchCreateTasksRequest {
            library_path: library_path.clone(),
            library_id: LibraryId(library_id),
            tasks: vec![
                CreateTaskInput {
                    task_type: TaskType::ImageGeneration,
                    provider: Some("codex-cli".to_string()),
                    operation: Some(GenerationOperation::TextToImage),
                    priority: 0,
                    concurrency_group: Some("codex-cli".to_string()),
                    max_attempts: 3,
                    input_json: "{\"prompt\":\"running\"}".to_string(),
                },
                CreateTaskInput {
                    task_type: TaskType::ImageGeneration,
                    provider: Some("codex-cli".to_string()),
                    operation: Some(GenerationOperation::TextToImage),
                    priority: 0,
                    concurrency_group: Some("codex-cli".to_string()),
                    max_attempts: 3,
                    input_json: "{\"prompt\":\"queued\"}".to_string(),
                },
            ],
        })
        .expect("create tasks");
    state
        .tasks()
        .update_task_status(UpdateTaskStatusRequest {
            library_path: library_path.clone(),
            task_id: tasks[0].id.clone(),
            status: TaskStatus::Running,
            next_retry_at: None,
            last_error_code: None,
            last_error_message: None,
            error_classification: None,
            wait_reason: None,
        })
        .expect("mark running");
    let config = TaskSchedulerConfig {
        global_concurrency_limit: 2,
        ..Default::default()
    };

    let executed =
        run_scheduler_tick_batch(&mut state, &config, &RetryPolicy::default()).expect("run tick");

    assert!(executed.is_empty());
    let queued = state
        .tasks()
        .get_task_detail(&library_path, &tasks[1].id)
        .expect("detail")
        .task;
    assert_eq!(
        queued.wait_reason.as_deref(),
        Some("Waiting for codex-cli slot")
    );
}

#[test]
fn scheduler_tick_does_not_start_new_tasks_when_running_exceeds_lowered_limit() {
    let mut state = test_state("lowered-limit");
    let library_id = create_open_library(&mut state, "lowered-limit-library");
    let library_path = state.library_path(&library_id).expect("library path");
    let tasks = state
        .tasks()
        .create_tasks(BatchCreateTasksRequest {
            library_path: library_path.clone(),
            library_id: LibraryId(library_id),
            tasks: vec![
                CreateTaskInput {
                    task_type: TaskType::ImageGeneration,
                    provider: Some("fake".to_string()),
                    operation: Some(GenerationOperation::TextToImage),
                    priority: 0,
                    concurrency_group: Some("fake".to_string()),
                    max_attempts: 3,
                    input_json: "{\"prompt\":\"running\"}".to_string(),
                },
                CreateTaskInput {
                    task_type: TaskType::ImageGeneration,
                    provider: Some("fake".to_string()),
                    operation: Some(GenerationOperation::TextToImage),
                    priority: 0,
                    concurrency_group: Some("fake".to_string()),
                    max_attempts: 3,
                    input_json: "{\"prompt\":\"queued\"}".to_string(),
                },
            ],
        })
        .expect("create tasks");
    state
        .tasks()
        .update_task_status(UpdateTaskStatusRequest {
            library_path: library_path.clone(),
            task_id: tasks[0].id.clone(),
            status: TaskStatus::Running,
            next_retry_at: None,
            last_error_code: None,
            last_error_message: None,
            error_classification: None,
            wait_reason: None,
        })
        .expect("mark running");
    let config = TaskSchedulerConfig {
        global_concurrency_limit: 1,
        ..Default::default()
    };

    let executed =
        run_scheduler_tick_batch(&mut state, &config, &RetryPolicy::default()).expect("run tick");

    assert!(executed.is_empty());
    let queued = state
        .tasks()
        .get_task_detail(&library_path, &tasks[1].id)
        .expect("detail")
        .task;
    assert_eq!(
        queued.wait_reason.as_deref(),
        Some("Waiting for global concurrency slot")
    );
}

#[test]
fn shared_scheduler_iteration_returns_no_work_without_open_libraries() {
    let state = test_state("scheduler-no-open-library");
    let shared = Arc::new(Mutex::new(state));

    let executed = run_scheduler_loop_iteration(&shared, &RetryPolicy::default())
        .expect("scheduler iteration");

    assert!(executed.is_empty());
}

#[test]
fn shared_scheduler_iteration_executes_without_holding_api_state() {
    let mut state = test_state("scheduler-loop");
    let library_id = create_open_library(&mut state, "scheduler-loop-library");
    let create_response = handle_http_request_with_state(
        &json_request(
            "POST",
            TASKS_PATH,
            serde_json::json!({
                "libraryId": library_id,
                "taskType": "image_generation",
                "provider": "fake",
                "operation": "text_to_image",
                "input": { "prompt": "scheduler loop task" }
            }),
        ),
        "secret",
        &mut state,
    );
    let task_id = json_value(&create_response)[0]["id"]
        .as_str()
        .expect("task id")
        .to_string();
    let shared = Arc::new(Mutex::new(state));

    let executed = run_scheduler_loop_iteration(&shared, &RetryPolicy::default())
        .expect("scheduler iteration")
        .into_iter()
        .next()
        .expect("executed task");
    assert_eq!(executed.status, TaskStatus::Completed);

    let detail_response = handle_http_request_with_shared_state(
        &auth_get(&format!("{TASKS_PATH}/{task_id}")),
        "secret",
        &shared,
    );
    assert_eq!(detail_response.status_code, 200);
    assert_eq!(json_value(&detail_response)["task"]["status"], "completed");
}

#[test]
fn running_task_cancel_requests_best_effort_stop() {
    let mut state = test_state("running-cancel");
    let library_id = create_open_library(&mut state, "running-cancel-library");
    let create_response = handle_http_request_with_state(
        &json_request(
            "POST",
            TASKS_PATH,
            serde_json::json!({
                "libraryId": library_id,
                "taskType": "image_generation",
                "provider": "fake",
                "operation": "text_to_image",
                "input": {
                    "prompt": "slow task",
                    "fakeMode": "slow_success"
                }
            }),
        ),
        "secret",
        &mut state,
    );
    let task_id = json_value(&create_response)[0]["id"]
        .as_str()
        .expect("task id")
        .to_string();
    let shared = Arc::new(Mutex::new(state));
    let worker_state = Arc::clone(&shared);
    let worker = thread::spawn(move || {
        run_scheduler_loop_iteration(&worker_state, &RetryPolicy::default())
            .expect("scheduler iteration")
            .into_iter()
            .next()
            .expect("executed task")
    });

    thread::sleep(Duration::from_millis(80));
    let cancel_response = handle_http_request_with_shared_state(
        &json_request(
            "POST",
            &format!("{TASKS_PATH}/{task_id}/cancel"),
            serde_json::json!({}),
        ),
        "secret",
        &shared,
    );
    assert_eq!(cancel_response.status_code, 200);
    assert_eq!(json_value(&cancel_response)["status"], "cancel_requested");

    let executed = worker.join().expect("join worker");
    assert_eq!(executed.status, TaskStatus::Canceled);

    let detail_response = handle_http_request_with_shared_state(
        &auth_get(&format!("{TASKS_PATH}/{task_id}")),
        "secret",
        &shared,
    );
    assert_eq!(detail_response.status_code, 200);
    let detail = json_value(&detail_response);
    assert_eq!(detail["task"]["status"], "canceled");
    assert_eq!(detail["attempts"][0]["status"], "canceled");
    assert!(detail["events"]
        .as_array()
        .expect("events")
        .iter()
        .any(|event| event["eventType"] == "attempt_canceled"));
}
