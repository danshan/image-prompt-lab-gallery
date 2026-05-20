use super::*;

#[test]
fn scheduler_tick_executes_fake_image_task_and_links_outputs() {
    let mut state = test_state("worker-success");
    let library_id = create_open_library(&mut state, "worker-success-library");
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

    let library_path = state.library_path(&library_id).expect("library path");
    let detail = state
        .service
        .get_task_detail(&library_path, &TaskId(task_id.clone()))
        .expect("detail");
    assert_eq!(detail.attempts[0].status, "completed");
    assert!(detail
        .outputs
        .iter()
        .any(|output| output.output_type == TaskOutputType::AssetVersion));
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
        .service
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
        .service
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
        .service
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
        .service
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
fn shared_scheduler_iteration_returns_no_work_without_open_libraries() {
    let state = test_state("scheduler-no-open-library");
    let shared = Arc::new(Mutex::new(state));

    let executed = run_scheduler_loop_iteration(
        &shared,
        &TaskSchedulerConfig::default(),
        &RetryPolicy::default(),
    )
    .expect("scheduler iteration");

    assert!(executed.is_none());
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

    let executed = run_scheduler_loop_iteration(
        &shared,
        &TaskSchedulerConfig::default(),
        &RetryPolicy::default(),
    )
    .expect("scheduler iteration")
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
        run_scheduler_loop_iteration(
            &worker_state,
            &TaskSchedulerConfig::default(),
            &RetryPolicy::default(),
        )
        .expect("scheduler iteration")
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
