use super::*;

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
