use super::*;
use imglab_core::{AppendTaskAttemptRequest, CreateLibraryRequest, CreateTaskInput, TaskService};
use std::net::{Ipv4Addr, SocketAddrV4};

fn test_root(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("imglab-daemon-{name}-{}", generate_session_token()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove test root");
    }
    root
}

fn test_state(name: &str) -> DaemonState {
    let root = test_root(name);
    fs::create_dir_all(root.join("logs")).expect("create log root");
    DaemonState::new(root.join("registry.sqlite"), root.join("logs"))
}

fn json_request(method: &str, path: &str, body: Value) -> String {
    format!(
        "{method} {path} HTTP/1.1\r\nAuthorization: Bearer secret\r\nContent-Type: application/json\r\n\r\n{}",
        serde_json::to_string(&body).expect("serialize body")
    )
}

fn auth_get(path: &str) -> String {
    format!("GET {path} HTTP/1.1\r\nAuthorization: Bearer secret\r\n\r\n")
}

fn json_value(response: &HttpResponse) -> Value {
    serde_json::from_str(&response.body).expect("parse response body")
}

fn create_open_library(state: &mut DaemonState, name: &str) -> String {
    let library_root = test_root(name).join("library");
    let library = state
        .service
        .create_library(CreateLibraryRequest {
            root_path: library_root.clone(),
            name: name.to_string(),
        })
        .expect("create library");
    let request = json_request(
        "POST",
        LIBRARY_OPEN_PATH,
        serde_json::json!({ "libraryPath": library_root }),
    );
    let response = handle_http_request_with_state(&request, "secret", state);
    assert_eq!(response.status_code, 200);
    library.id.0
}

#[test]
fn runtime_file_round_trips() {
    let root = test_root("runtime");
    let runtime_path = root.join("daemon.json");
    let runtime = RuntimeFile {
        api_version: API_VERSION.to_string(),
        pid: 42,
        port: 4317,
        token_path: root.join("token"),
    };

    write_runtime_file(&runtime_path, &runtime).expect("write runtime");
    assert_eq!(
        read_runtime_file(&runtime_path).expect("read runtime"),
        runtime
    );
}

#[test]
fn health_and_capabilities_require_token() {
    let unauthorized = handle_http_request("GET /v1/health HTTP/1.1\r\n\r\n", "secret");
    assert_eq!(unauthorized.status_code, 401);

    let health = handle_http_request(
        "GET /v1/health HTTP/1.1\r\nAuthorization: Bearer secret\r\n\r\n",
        "secret",
    );
    assert_eq!(health.status_code, 200);
    assert!(health.body.contains("\"apiVersion\":\"v1\""));
    assert!(health.body.contains("\"schemaVersion\":6"));
    assert!(health.body.contains("\"provider\":\"codex-cli\""));
    assert!(health.body.contains("\"image_to_image\""));

    let capabilities = handle_http_request(
        "GET /v1/capabilities HTTP/1.1\r\nX-ImgLab-Token: secret\r\n\r\n",
        "secret",
    );
    assert_eq!(capabilities.status_code, 200);
    assert!(capabilities.body.contains("image_generation"));
}

#[test]
fn loopback_guard_rejects_non_loopback_addresses() {
    let non_loopback = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0));
    let error = bind_loopback_listener(non_loopback).expect_err("reject non loopback");
    assert!(matches!(
        error,
        DomainError::InvalidGenerationParameters { .. }
    ));

    let loopback = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));
    assert!(is_loopback_addr(loopback));
}

#[test]
fn task_api_creates_lists_reorders_and_loads_detail() {
    let mut state = test_state("task-api");
    let library_id = create_open_library(&mut state, "task-api-library");
    let create_request = json_request(
        "POST",
        TASKS_BATCH_PATH,
        serde_json::json!({
            "libraryId": library_id,
            "tasks": [
                {
                    "taskType": "image_generation",
                    "provider": "fake",
                    "operation": "text_to_image",
                    "priority": 1,
                    "input": { "prompt": "first prompt\nwith multiple lines" }
                },
                {
                    "taskType": "image_generation",
                    "provider": "fake",
                    "operation": "text_to_image",
                    "priority": 1,
                    "input": { "prompt": "second prompt" }
                }
            ]
        }),
    );
    let create_response = handle_http_request_with_state(&create_request, "secret", &mut state);
    assert_eq!(create_response.status_code, 200);
    let created = json_value(&create_response);
    let first_id = created[0]["id"].as_str().expect("first id").to_string();
    let second_id = created[1]["id"].as_str().expect("second id").to_string();

    let list_response = handle_http_request_with_state(
        &auth_get(&format!("{TASKS_PATH}?library_id={library_id}")),
        "secret",
        &mut state,
    );
    assert_eq!(list_response.status_code, 200);
    let listed = json_value(&list_response);
    assert_eq!(listed.as_array().expect("task array").len(), 2);

    let reorder_request = json_request(
        "POST",
        TASKS_REORDER_PATH,
        serde_json::json!({
            "libraryId": library_id,
            "taskIds": [second_id, first_id]
        }),
    );
    let reorder_response = handle_http_request_with_state(&reorder_request, "secret", &mut state);
    assert_eq!(reorder_response.status_code, 200);

    let detail_response = handle_http_request_with_state(
        &auth_get(&format!("{TASKS_PATH}/{first_id}")),
        "secret",
        &mut state,
    );
    assert_eq!(detail_response.status_code, 200);
    let detail = json_value(&detail_response);
    assert_eq!(detail["task"]["id"], first_id);
    assert_eq!(
        detail["task"]["input"]["prompt"],
        "first prompt\nwith multiple lines"
    );
    assert_eq!(detail["events"][0]["eventType"].as_str(), Some("submitted"));
}

#[test]
fn task_actions_cancel_retry_duplicate_and_events() {
    let mut state = test_state("task-actions");
    let library_id = create_open_library(&mut state, "task-actions-library");
    let create_response = handle_http_request_with_state(
        &json_request(
            "POST",
            TASKS_PATH,
            serde_json::json!({
                "libraryId": library_id,
                "taskType": "image_generation",
                "provider": "fake",
                "operation": "text_to_image",
                "input": { "prompt": "retry me" }
            }),
        ),
        "secret",
        &mut state,
    );
    assert_eq!(create_response.status_code, 200);
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
    assert_eq!(json_value(&cancel_response)["status"], "canceled");

    let retry_response = handle_http_request_with_state(
        &json_request(
            "POST",
            &format!("{TASKS_PATH}/{task_id}/retry"),
            serde_json::json!({}),
        ),
        "secret",
        &mut state,
    );
    assert_eq!(retry_response.status_code, 200);
    assert_eq!(json_value(&retry_response)["status"], "queued");

    let duplicate_response = handle_http_request_with_state(
        &json_request(
            "POST",
            &format!("{TASKS_PATH}/{task_id}/duplicate"),
            serde_json::json!({}),
        ),
        "secret",
        &mut state,
    );
    assert_eq!(duplicate_response.status_code, 200);
    assert_ne!(json_value(&duplicate_response)["id"], task_id);

    let events_response = handle_http_request_with_state(
        &auth_get(&format!("{TASKS_PATH}/{task_id}/events")),
        "secret",
        &mut state,
    );
    assert_eq!(events_response.status_code, 200);
    let events = json_value(&events_response);
    assert!(events
        .as_array()
        .expect("events")
        .iter()
        .any(|event| event["eventType"] == "manual_retry"));
}

#[test]
fn task_api_maps_errors_and_rejects_unowned_log_paths() {
    let mut state = test_state("task-errors");
    let library_id = create_open_library(&mut state, "task-errors-library");
    let library_path = state.library_path(&library_id).expect("library path");
    let created = state
        .service
        .create_tasks(BatchCreateTasksRequest {
            library_path: library_path.clone(),
            library_id: LibraryId(library_id),
            tasks: vec![CreateTaskInput {
                task_type: TaskType::ImageGeneration,
                provider: Some("fake".to_string()),
                operation: Some(GenerationOperation::TextToImage),
                priority: 0,
                concurrency_group: None,
                max_attempts: 3,
                input_json: "{\"prompt\":\"log test\"}".to_string(),
            }],
        })
        .expect("create task");
    let task_id = created[0].id.clone();
    let outside_log = test_root("outside-log").join("attempt.log");
    fs::create_dir_all(outside_log.parent().expect("outside log parent"))
        .expect("create outside log dir");
    fs::write(&outside_log, "secret log").expect("write outside log");
    state
        .service
        .append_task_attempt(AppendTaskAttemptRequest {
            library_path,
            task_id: task_id.clone(),
            status: "running".to_string(),
            log_path: Some(outside_log),
        })
        .expect("append attempt");

    let unauthorized = handle_http_request_with_state(
        &format!("GET {TASKS_PATH}/{} HTTP/1.1\r\n\r\n", task_id.0),
        "secret",
        &mut state,
    );
    assert_eq!(unauthorized.status_code, 401);

    let missing = handle_http_request_with_state(
        &auth_get(&format!("{TASKS_PATH}/missing-task")),
        "secret",
        &mut state,
    );
    assert_eq!(missing.status_code, 404);
    assert_eq!(json_value(&missing)["code"], "InvalidTaskReference");

    let log_response = handle_http_request_with_state(
        &auth_get(&format!("{TASKS_PATH}/{}/logs/tail", task_id.0)),
        "secret",
        &mut state,
    );
    assert_eq!(log_response.status_code, 400);
    assert!(json_value(&log_response)["message"]
        .as_str()
        .expect("error message")
        .contains("outside app-owned log root"));
}

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

#[test]
fn recovery_restores_retry_waiting_and_running_tasks() {
    let mut state = test_state("recovery");
    let library_id = create_open_library(&mut state, "recovery-library");
    let library_path = state.library_path(&library_id).expect("library path");
    let created = state
        .service
        .create_tasks(BatchCreateTasksRequest {
            library_path: library_path.clone(),
            library_id: LibraryId(library_id),
            tasks: vec![
                CreateTaskInput {
                    task_type: TaskType::ImageGeneration,
                    provider: Some("fake".to_string()),
                    operation: Some(GenerationOperation::TextToImage),
                    priority: 0,
                    concurrency_group: None,
                    max_attempts: 3,
                    input_json: "{\"prompt\":\"retry expired\"}".to_string(),
                },
                CreateTaskInput {
                    task_type: TaskType::ImageGeneration,
                    provider: Some("fake".to_string()),
                    operation: Some(GenerationOperation::TextToImage),
                    priority: 0,
                    concurrency_group: None,
                    max_attempts: 3,
                    input_json: "{\"prompt\":\"retry future\"}".to_string(),
                },
                CreateTaskInput {
                    task_type: TaskType::ImageGeneration,
                    provider: Some("fake".to_string()),
                    operation: Some(GenerationOperation::TextToImage),
                    priority: 0,
                    concurrency_group: None,
                    max_attempts: 3,
                    input_json: "{\"prompt\":\"running no output\"}".to_string(),
                },
                CreateTaskInput {
                    task_type: TaskType::ImageGeneration,
                    provider: Some("fake".to_string()),
                    operation: Some(GenerationOperation::TextToImage),
                    priority: 0,
                    concurrency_group: None,
                    max_attempts: 3,
                    input_json: "{\"prompt\":\"running with output\"}".to_string(),
                },
            ],
        })
        .expect("create tasks");
    let expired = created[0].id.clone();
    let future = created[1].id.clone();
    let interrupted = created[2].id.clone();
    let committed = created[3].id.clone();

    state
        .service
        .update_task_status(UpdateTaskStatusRequest {
            library_path: library_path.clone(),
            task_id: expired.clone(),
            status: TaskStatus::RetryWaiting,
            next_retry_at: Some("0".to_string()),
            last_error_code: Some("ProviderUnavailable".to_string()),
            last_error_message: Some("temporary outage".to_string()),
            error_classification: Some(TaskErrorClassification::Transient),
            wait_reason: None,
        })
        .expect("set expired retry");
    state
        .service
        .update_task_status(UpdateTaskStatusRequest {
            library_path: library_path.clone(),
            task_id: future.clone(),
            status: TaskStatus::RetryWaiting,
            next_retry_at: Some("9999999999".to_string()),
            last_error_code: Some("ProviderUnavailable".to_string()),
            last_error_message: Some("temporary outage".to_string()),
            error_classification: Some(TaskErrorClassification::Transient),
            wait_reason: None,
        })
        .expect("set future retry");
    for task_id in [&interrupted, &committed] {
        state
            .service
            .append_task_attempt(imglab_core::AppendTaskAttemptRequest {
                library_path: library_path.clone(),
                task_id: task_id.clone(),
                status: "running".to_string(),
                log_path: None,
            })
            .expect("append attempt");
        state
            .service
            .update_task_status(UpdateTaskStatusRequest {
                library_path: library_path.clone(),
                task_id: task_id.clone(),
                status: TaskStatus::Running,
                next_retry_at: None,
                last_error_code: None,
                last_error_message: None,
                error_classification: None,
                wait_reason: None,
            })
            .expect("set running");
    }
    state
        .service
        .append_task_output(imglab_core::AppendTaskOutputRequest {
            library_path: library_path.clone(),
            task_id: committed.clone(),
            output_type: TaskOutputType::AssetVersion,
            target_id: "committed-version".to_string(),
            payload_json: None,
        })
        .expect("append output");

    let recovered =
        recover_open_libraries(&mut state, &RetryPolicy::default()).expect("recover tasks");
    assert_eq!(recovered.len(), 3);
    assert_eq!(
        state
            .service
            .get_task_detail(&library_path, &expired)
            .expect("expired detail")
            .task
            .status,
        TaskStatus::Queued
    );
    assert_eq!(
        state
            .service
            .get_task_detail(&library_path, &future)
            .expect("future detail")
            .task
            .status,
        TaskStatus::RetryWaiting
    );
    assert_eq!(
        state
            .service
            .get_task_detail(&library_path, &interrupted)
            .expect("interrupted detail")
            .task
            .status,
        TaskStatus::InterruptedRetryable
    );
    let committed_detail = state
        .service
        .get_task_detail(&library_path, &committed)
        .expect("committed detail");
    assert_eq!(committed_detail.task.status, TaskStatus::Completed);
    assert_eq!(committed_detail.outputs.len(), 1);
    assert!(committed_detail
        .events
        .iter()
        .any(|event| event.event_type == "recovery_reconciled_completed"));
}

#[test]
fn reopening_library_does_not_interrupt_live_running_tasks() {
    let mut state = test_state("reopen-running");
    let library_id = create_open_library(&mut state, "reopen-running-library");
    let library_path = state.library_path(&library_id).expect("library path");
    let created = state
        .service
        .create_tasks(BatchCreateTasksRequest {
            library_path: library_path.clone(),
            library_id: LibraryId(library_id),
            tasks: vec![CreateTaskInput {
                task_type: TaskType::ImageGeneration,
                provider: Some("fake".to_string()),
                operation: Some(GenerationOperation::TextToImage),
                priority: 0,
                concurrency_group: None,
                max_attempts: 3,
                input_json: "{\"prompt\":\"still running\"}".to_string(),
            }],
        })
        .expect("create task");
    let task_id = created[0].id.clone();
    state
        .service
        .append_task_attempt(imglab_core::AppendTaskAttemptRequest {
            library_path: library_path.clone(),
            task_id: task_id.clone(),
            status: "running".to_string(),
            log_path: None,
        })
        .expect("append attempt");
    state
        .service
        .update_task_status(UpdateTaskStatusRequest {
            library_path: library_path.clone(),
            task_id: task_id.clone(),
            status: TaskStatus::Running,
            next_retry_at: None,
            last_error_code: None,
            last_error_message: None,
            error_classification: None,
            wait_reason: None,
        })
        .expect("set running");

    state.open_library(&library_path).expect("reopen library");

    let detail = state
        .service
        .get_task_detail(&library_path, &task_id)
        .expect("detail");
    assert_eq!(detail.task.status, TaskStatus::Running);
    assert!(!detail
        .events
        .iter()
        .any(|event| event.event_type == "recovery_interrupted"));
}
