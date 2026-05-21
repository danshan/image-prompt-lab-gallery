use super::*;

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
        .service()
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
        .service()
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
