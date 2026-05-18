use imglab_core::{
    classify_task_error, evaluate_scheduler, AssetId, BatchCreateTasksRequest,
    CreateMetadataSuggestionRequest, CreateTaskInput, DomainError, DomainResult,
    GenerateImageRequest, GenerationOperation, GenerationParameters, GenerationService, LibraryId,
    LibraryService, LibrarySummary, LocalGenerationService, LocalLibraryService,
    MetadataReviewService, ReorderQueuedTasksRequest, RetryPolicy, TaskAttempt, TaskDetail,
    TaskErrorClassification, TaskEvent, TaskId, TaskOutput, TaskOutputType, TaskSchedulerConfig,
    TaskService, TaskStatus, TaskSummary, TaskType, UpdateTaskStatusRequest,
};
use imglab_provider_codex::CodexCliImageProvider;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const API_VERSION: &str = "v1";
pub const HEALTH_PATH: &str = "/v1/health";
pub const CAPABILITIES_PATH: &str = "/v1/capabilities";
const LIBRARY_OPEN_PATH: &str = "/v1/libraries/open";
const TASKS_PATH: &str = "/v1/tasks";
const TASKS_BATCH_PATH: &str = "/v1/tasks/batch";
const TASKS_REORDER_PATH: &str = "/v1/tasks/reorder";
const MAX_LOG_TAIL_BYTES: usize = 64 * 1024;
pub const DEFAULT_SCHEDULER_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeFile {
    pub api_version: String,
    pub pid: u32,
    pub port: u16,
    pub token_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonConfig {
    pub bind_addr: SocketAddr,
    pub runtime_path: PathBuf,
    pub token_path: PathBuf,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthView {
    pub status: String,
    pub api_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitiesView {
    pub api_version: String,
    pub task_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status_code: u16,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct DaemonState {
    service: LocalLibraryService,
    opened_libraries: BTreeMap<String, PathBuf>,
    recovered_libraries: BTreeSet<String>,
    log_root: PathBuf,
}

pub type SharedDaemonState = Arc<Mutex<DaemonState>>;

impl DaemonState {
    pub fn new(registry_path: impl Into<PathBuf>, log_root: impl Into<PathBuf>) -> Self {
        Self {
            service: LocalLibraryService::new(registry_path),
            opened_libraries: BTreeMap::new(),
            recovered_libraries: BTreeSet::new(),
            log_root: log_root.into(),
        }
    }

    fn open_library(&mut self, root_path: &Path) -> DomainResult<LibrarySummary> {
        let library = self.service.open_library(root_path)?;
        let should_recover = !self.recovered_libraries.contains(&library.id.0);
        self.opened_libraries
            .insert(library.id.0.clone(), library.root_path.clone());
        if should_recover {
            recover_open_libraries(self, &RetryPolicy::default())?;
            self.recovered_libraries.insert(library.id.0.clone());
        }
        Ok(library)
    }

    fn library_path(&self, library_id: &str) -> DomainResult<PathBuf> {
        self.opened_libraries
            .get(library_id)
            .cloned()
            .ok_or_else(|| DomainError::InvalidGenerationParameters {
                message: format!("library is not open: {library_id}"),
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageGenerationTaskInput {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub model: Option<String>,
    pub parameters: Option<Value>,
    pub parameters_json: Option<Value>,
    pub input_version_id: Option<String>,
    pub fake_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetadataFieldTaskInput {
    suggestion_id: String,
    asset_id: String,
    field: String,
    base_revision: Option<String>,
    context: MetadataTaskContext,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetadataSuggestionTaskInput {
    suggestion_id: String,
    asset_id: String,
    base_revision: Option<String>,
    context: MetadataTaskContext,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetadataTaskContext {
    title: Option<String>,
    description: Option<String>,
    schema_prompt: Option<String>,
    tags: Vec<String>,
    category: Option<String>,
    source_prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenLibraryInput {
    library_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTaskInputView {
    task_type: String,
    provider: Option<String>,
    operation: Option<String>,
    priority: Option<i64>,
    concurrency_group: Option<String>,
    max_attempts: Option<u32>,
    input: Option<Value>,
    input_json: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchCreateTasksInput {
    library_id: String,
    tasks: Vec<CreateTaskInputView>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SingleCreateTaskInput {
    library_id: String,
    #[serde(flatten)]
    task: CreateTaskInputView,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReorderTasksInput {
    library_id: String,
    task_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LibraryView {
    id: String,
    name: String,
    root_path: PathBuf,
    schema_version: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskSummaryView {
    id: String,
    library_id: String,
    task_type: String,
    status: String,
    queue_position: i64,
    priority: i64,
    provider: Option<String>,
    operation: Option<String>,
    concurrency_group: Option<String>,
    attempt_count: u32,
    max_attempts: u32,
    next_retry_at: Option<String>,
    input: Value,
    created_at: String,
    updated_at: String,
    last_error_code: Option<String>,
    last_error_message: Option<String>,
    error_classification: Option<String>,
    wait_reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskAttemptView {
    id: String,
    task_id: String,
    attempt_number: u32,
    status: String,
    started_at: String,
    completed_at: Option<String>,
    log_path: Option<PathBuf>,
    exit_code: Option<i32>,
    error_code: Option<String>,
    error_message: Option<String>,
    error_classification: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskEventView {
    id: String,
    task_id: String,
    event_type: String,
    message: Option<String>,
    payload: Option<Value>,
    created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskOutputView {
    id: String,
    task_id: String,
    output_type: String,
    target_id: String,
    payload: Option<Value>,
    created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskDetailView {
    task: TaskSummaryView,
    attempts: Vec<TaskAttemptView>,
    events: Vec<TaskEventView>,
    outputs: Vec<TaskOutputView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LogTailView {
    content: String,
    truncated: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiErrorView {
    code: String,
    message: String,
    recoverable: bool,
}

pub fn health_view() -> HealthView {
    HealthView {
        status: "ok".to_string(),
        api_version: API_VERSION.to_string(),
    }
}

pub fn capabilities_view() -> CapabilitiesView {
    CapabilitiesView {
        api_version: API_VERSION.to_string(),
        task_types: vec![
            TaskType::ImageGeneration.as_str().to_string(),
            TaskType::MetadataFieldGeneration.as_str().to_string(),
            TaskType::MetadataSuggestionGeneration.as_str().to_string(),
        ],
    }
}

pub fn write_runtime_file(path: &Path, runtime: &RuntimeFile) -> DomainResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
    }
    let content = serde_json::to_string_pretty(runtime).map_err(serialization_error)?;
    fs::write(path, content).map_err(|error| io_error(path, error))
}

pub fn read_runtime_file(path: &Path) -> DomainResult<RuntimeFile> {
    let content = fs::read_to_string(path).map_err(|error| io_error(path, error))?;
    serde_json::from_str(&content).map_err(serialization_error)
}

pub fn write_token_file(path: &Path, token: &str) -> DomainResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
    }
    fs::write(path, token).map_err(|error| io_error(path, error))
}

pub fn generate_session_token() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{:x}-{}", nanos, std::process::id())
}

pub fn bind_loopback_listener(addr: SocketAddr) -> DomainResult<TcpListener> {
    if !is_loopback_addr(addr) {
        return Err(DomainError::InvalidGenerationParameters {
            message: format!("daemon must bind to loopback address: {addr}"),
        });
    }
    TcpListener::bind(addr).map_err(|error| DomainError::Io {
        path: addr.to_string(),
        message: error.to_string(),
    })
}

pub fn is_loopback_addr(addr: SocketAddr) -> bool {
    match addr.ip() {
        IpAddr::V4(ip) => ip.is_loopback(),
        IpAddr::V6(ip) => ip.is_loopback(),
    }
}

pub fn handle_http_request(request: &str, token: &str) -> HttpResponse {
    let registry_path = std::env::temp_dir().join("imglab-daemon-stateless-registry.sqlite");
    let log_root = std::env::temp_dir().join("imglab-daemon-logs");
    let mut state = DaemonState::new(registry_path, log_root);
    handle_http_request_with_state(request, token, &mut state)
}

pub fn handle_http_request_with_state(
    request: &str,
    token: &str,
    state: &mut DaemonState,
) -> HttpResponse {
    let Some(request_line) = request.lines().next() else {
        return error_response(
            400,
            ApiErrorView {
                code: "bad_request".to_string(),
                message: "bad request".to_string(),
                recoverable: false,
            },
        );
    };
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or_default();
    let (path, query) = split_target(target);

    if !request_has_valid_token(request, token) {
        return error_response(
            401,
            ApiErrorView {
                code: "unauthorized".to_string(),
                message: "unauthorized".to_string(),
                recoverable: false,
            },
        );
    }

    match route_request(method, path, query, request_body(request), state) {
        Ok(Some(response)) => response,
        Ok(None) => match (method, path) {
            ("GET", HEALTH_PATH) => json_response(200, &health_view()),
            ("GET", CAPABILITIES_PATH) => json_response(200, &capabilities_view()),
            _ => error_response(
                404,
                ApiErrorView {
                    code: "not_found".to_string(),
                    message: "not found".to_string(),
                    recoverable: false,
                },
            ),
        },
        Err(error) => domain_error_response(error),
    }
}

pub fn handle_http_request_with_shared_state(
    request: &str,
    token: &str,
    state: &SharedDaemonState,
) -> HttpResponse {
    match state.lock() {
        Ok(mut guard) => handle_http_request_with_state(request, token, &mut guard),
        Err(_) => error_response(
            500,
            ApiErrorView {
                code: "StateLockPoisoned".to_string(),
                message: "daemon state lock is poisoned".to_string(),
                recoverable: false,
            },
        ),
    }
}

pub fn serve_one(listener: &TcpListener, token: &str) -> DomainResult<()> {
    let registry_path = std::env::temp_dir().join("imglab-daemon-stateless-registry.sqlite");
    let log_root = std::env::temp_dir().join("imglab-daemon-logs");
    let mut state = DaemonState::new(registry_path, log_root);
    serve_one_with_state(listener, token, &mut state)
}

pub fn serve_forever(
    listener: &TcpListener,
    token: &str,
    state: &mut DaemonState,
) -> DomainResult<()> {
    loop {
        serve_one_with_state(listener, token, state)?;
    }
}

pub fn serve_forever_shared(
    listener: &TcpListener,
    token: &str,
    state: SharedDaemonState,
) -> DomainResult<()> {
    loop {
        serve_one_with_shared_state(listener, token, &state)?;
    }
}

pub fn serve_one_with_state(
    listener: &TcpListener,
    token: &str,
    state: &mut DaemonState,
) -> DomainResult<()> {
    let (mut stream, _) = listener.accept().map_err(|error| DomainError::Io {
        path: listener
            .local_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
        message: error.to_string(),
    })?;
    handle_stream(&mut stream, token, state)
}

pub fn serve_one_with_shared_state(
    listener: &TcpListener,
    token: &str,
    state: &SharedDaemonState,
) -> DomainResult<()> {
    let (mut stream, _) = listener.accept().map_err(|error| DomainError::Io {
        path: listener
            .local_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
        message: error.to_string(),
    })?;
    handle_shared_stream(&mut stream, token, state)
}

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
    let mut snapshot = state
        .lock()
        .map_err(|_| DomainError::ConcurrentWriteConflict {
            message: "daemon state lock is poisoned".to_string(),
        })?
        .clone();
    run_scheduler_tick(&mut snapshot, config, retry_policy)
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

fn execute_task(
    state: &mut DaemonState,
    library_path: &Path,
    task_id: &TaskId,
    retry_policy: &RetryPolicy,
) -> DomainResult<TaskSummary> {
    let detail = state.service.get_task_detail(library_path, task_id)?;
    if detail.task.status != TaskStatus::Queued {
        return Ok(detail.task);
    }
    fs::create_dir_all(&state.log_root).map_err(|error| io_error(&state.log_root, error))?;
    let log_path = state.log_root.join(format!(
        "imglab-task-{}-attempt-{}.log",
        task_id.0,
        detail.task.attempt_count + 1
    ));
    fs::write(&log_path, format!("starting task {}\n", task_id.0))
        .map_err(|error| io_error(&log_path, error))?;

    state.service.update_task_status(UpdateTaskStatusRequest {
        library_path: library_path.to_path_buf(),
        task_id: task_id.clone(),
        status: TaskStatus::Running,
        next_retry_at: None,
        last_error_code: None,
        last_error_message: None,
        error_classification: None,
        wait_reason: None,
    })?;
    let attempt = state
        .service
        .append_task_attempt(imglab_core::AppendTaskAttemptRequest {
            library_path: library_path.to_path_buf(),
            task_id: task_id.clone(),
            status: "running".to_string(),
            log_path: Some(log_path.clone()),
        })?;
    state
        .service
        .append_task_event(imglab_core::AppendTaskEventRequest {
            library_path: library_path.to_path_buf(),
            task_id: task_id.clone(),
            event_type: "attempt_started".to_string(),
            message: Some(format!("Attempt {} started", attempt.attempt_number)),
            payload_json: Some(format!("{{\"attempt_id\":\"{}\"}}", attempt.id.0)),
        })?;

    match execute_task_body(state, library_path, task_id, &detail.task, &log_path) {
        Ok(()) => complete_successful_attempt(state, library_path, task_id, &attempt.id, &log_path),
        Err(error) => {
            if cancel_marker_path(&log_path).exists()
                || state
                    .service
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

fn execute_task_body(
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

fn execute_image_generation_task(
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
    let request = GenerateImageRequest {
        library_path: library_path.to_path_buf(),
        parameters: GenerationParameters {
            library_path: Some(library_path.to_path_buf()),
            provider: provider.to_string(),
            model: input
                .model
                .clone()
                .unwrap_or_else(|| default_model_label(provider).to_string()),
            prompt: input.prompt,
            negative_prompt: input.negative_prompt,
            operation: task.operation.unwrap_or(GenerationOperation::TextToImage),
            input_version_id: input.input_version_id.map(imglab_core::AssetVersionId),
            parameters_json: input
                .parameters_json
                .or(input.parameters)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "{}".to_string()),
        },
        input_bytes: None,
    };

    let versions = match provider {
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
            LocalGenerationService::new(fake).generate(request)?
        }
        "codex" | "codex-cli" => {
            let codex = CodexCliImageProvider::new("codex", library_path)
                .with_log_path(log_path)
                .with_cancel_path(cancel_marker_path(log_path));
            LocalGenerationService::new(codex).generate(request)?
        }
        other => {
            return Err(DomainError::UnsupportedProvider {
                provider: other.to_string(),
            });
        }
    };

    for version in versions {
        state
            .service
            .append_task_output(imglab_core::AppendTaskOutputRequest {
                library_path: library_path.to_path_buf(),
                task_id: task_id.clone(),
                output_type: TaskOutputType::Asset,
                target_id: version.asset_id.0.clone(),
                payload_json: None,
            })?;
        state
            .service
            .append_task_output(imglab_core::AppendTaskOutputRequest {
                library_path: library_path.to_path_buf(),
                task_id: task_id.clone(),
                output_type: TaskOutputType::AssetVersion,
                target_id: version.id.0.clone(),
                payload_json: None,
            })?;
        if let Some(event_id) = version.generation_event_id {
            state
                .service
                .append_task_output(imglab_core::AppendTaskOutputRequest {
                    library_path: library_path.to_path_buf(),
                    task_id: task_id.clone(),
                    output_type: TaskOutputType::GenerationEvent,
                    target_id: event_id.0,
                    payload_json: None,
                })?;
        }
    }
    append_log(log_path, "task completed\n")
}

fn execute_metadata_field_task(
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
        .service
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

fn execute_metadata_suggestion_task(
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
    let suggestion = state
        .service
        .create_suggestion(CreateMetadataSuggestionRequest {
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
        .service
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

fn generated_metadata_field_value(
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

fn wait_for_fake_slow_task(
    state: &DaemonState,
    library_path: &Path,
    task_id: &TaskId,
    log_path: &Path,
) -> DomainResult<()> {
    for _ in 0..50 {
        if cancel_marker_path(log_path).exists()
            || state
                .service
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

fn complete_successful_attempt(
    state: &mut DaemonState,
    library_path: &Path,
    task_id: &TaskId,
    attempt_id: &imglab_core::TaskAttemptId,
    log_path: &Path,
) -> DomainResult<TaskSummary> {
    state
        .service
        .complete_task_attempt(imglab_core::CompleteTaskAttemptRequest {
            library_path: library_path.to_path_buf(),
            attempt_id: attempt_id.clone(),
            status: "completed".to_string(),
            exit_code: Some(0),
            error_code: None,
            error_message: None,
            error_classification: None,
        })?;
    let current = state.service.get_task_detail(library_path, task_id)?.task;
    let completed_status = if current.status == TaskStatus::CancelRequested {
        state
            .service
            .append_task_event(imglab_core::AppendTaskEventRequest {
                library_path: library_path.to_path_buf(),
                task_id: task_id.clone(),
                event_type: "completed_after_cancel_requested".to_string(),
                message: Some("Task completed after cancel was requested".to_string()),
                payload_json: None,
            })?;
        TaskStatus::Completed
    } else {
        TaskStatus::Completed
    };
    append_log(log_path, "attempt completed\n")?;
    state
        .service
        .append_task_event(imglab_core::AppendTaskEventRequest {
            library_path: library_path.to_path_buf(),
            task_id: task_id.clone(),
            event_type: "attempt_completed".to_string(),
            message: Some("Attempt completed".to_string()),
            payload_json: None,
        })?;
    state.service.update_task_status(UpdateTaskStatusRequest {
        library_path: library_path.to_path_buf(),
        task_id: task_id.clone(),
        status: completed_status,
        next_retry_at: None,
        last_error_code: None,
        last_error_message: None,
        error_classification: None,
        wait_reason: None,
    })
}

fn complete_canceled_attempt(
    state: &mut DaemonState,
    library_path: &Path,
    task_id: &TaskId,
    attempt_id: &imglab_core::TaskAttemptId,
    log_path: &Path,
) -> DomainResult<TaskSummary> {
    append_log(log_path, "attempt canceled\n")?;
    state
        .service
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
        .service
        .append_task_event(imglab_core::AppendTaskEventRequest {
            library_path: library_path.to_path_buf(),
            task_id: task_id.clone(),
            event_type: "attempt_canceled".to_string(),
            message: Some("Attempt canceled by user request".to_string()),
            payload_json: None,
        })?;
    state.service.update_task_status(UpdateTaskStatusRequest {
        library_path: library_path.to_path_buf(),
        task_id: task_id.clone(),
        status: TaskStatus::Canceled,
        next_retry_at: None,
        last_error_code: Some("Canceled".to_string()),
        last_error_message: Some("Task canceled by user".to_string()),
        error_classification: Some(TaskErrorClassification::Cancel),
        wait_reason: None,
    })
}

fn complete_failed_attempt(
    state: &mut DaemonState,
    library_path: &Path,
    task_id: &TaskId,
    attempt_id: &imglab_core::TaskAttemptId,
    log_path: &Path,
    error: DomainError,
    retry_policy: &RetryPolicy,
) -> DomainResult<TaskSummary> {
    let classification = classify_task_error(&error);
    let detail = state.service.get_task_detail(library_path, task_id)?;
    let should_retry = retry_policy.should_auto_retry(classification, detail.task.attempt_count);
    let status = if should_retry {
        TaskStatus::RetryWaiting
    } else if classification == TaskErrorClassification::RetryableManual {
        TaskStatus::FailedRetryable
    } else {
        TaskStatus::FailedFinal
    };
    let next_retry_at = should_retry.then(|| {
        unix_timestamp_string(retry_policy.backoff_delay_seconds(detail.task.attempt_count))
    });
    append_log(log_path, &format!("attempt failed: {error}\n"))?;
    state
        .service
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
        .service
        .append_task_event(imglab_core::AppendTaskEventRequest {
            library_path: library_path.to_path_buf(),
            task_id: task_id.clone(),
            event_type: if should_retry {
                "retry_scheduled".to_string()
            } else {
                "attempt_failed".to_string()
            },
            message: Some(error.to_string()),
            payload_json: next_retry_at
                .as_ref()
                .map(|value| format!("{{\"next_retry_at\":\"{value}\"}}")),
        })?;
    state.service.update_task_status(UpdateTaskStatusRequest {
        library_path: library_path.to_path_buf(),
        task_id: task_id.clone(),
        status,
        next_retry_at,
        last_error_code: Some(error.code().to_string()),
        last_error_message: Some(error.to_string()),
        error_classification: Some(classification),
        wait_reason: None,
    })
}

fn append_log(path: &Path, line: &str) -> DomainResult<()> {
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(line.as_bytes()))
        .map_err(|error| io_error(path, error))
}

fn cancel_marker_path(log_path: &Path) -> PathBuf {
    let mut marker = log_path.to_path_buf();
    marker.set_extension("cancel");
    marker
}

fn default_model_label(provider: &str) -> &'static str {
    match provider {
        "fake" => "fake-image",
        "codex" | "codex-cli" => "codex-cli-imagegen",
        _ => "unknown",
    }
}

fn unix_timestamp_string(add_seconds: u64) -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() + add_seconds)
        .unwrap_or(add_seconds);
    seconds.to_string()
}

fn handle_stream(stream: &mut TcpStream, token: &str, state: &mut DaemonState) -> DomainResult<()> {
    let mut buffer = [0_u8; 8192];
    let read = stream.read(&mut buffer).map_err(|error| DomainError::Io {
        path: "daemon-http-stream".to_string(),
        message: error.to_string(),
    })?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let response = handle_http_request_with_state(&request, token, state);
    stream
        .write_all(http_response_bytes(&response).as_bytes())
        .map_err(|error| DomainError::Io {
            path: "daemon-http-stream".to_string(),
            message: error.to_string(),
        })
}

fn handle_shared_stream(
    stream: &mut TcpStream,
    token: &str,
    state: &SharedDaemonState,
) -> DomainResult<()> {
    let mut buffer = [0_u8; 8192];
    let read = stream.read(&mut buffer).map_err(|error| DomainError::Io {
        path: "daemon-http-stream".to_string(),
        message: error.to_string(),
    })?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let response = handle_http_request_with_shared_state(&request, token, state);
    stream
        .write_all(http_response_bytes(&response).as_bytes())
        .map_err(|error| DomainError::Io {
            path: "daemon-http-stream".to_string(),
            message: error.to_string(),
        })
}

fn route_request(
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

fn route_task_member(
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

fn find_task_detail(state: &DaemonState, task_id: &TaskId) -> DomainResult<(PathBuf, TaskDetail)> {
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

fn request_task_cancel(
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

fn tail_task_log(detail: &TaskDetail, log_root: &Path) -> DomainResult<LogTailView> {
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

fn ensure_app_owned_log_path(log_path: &Path, log_root: &Path) -> DomainResult<()> {
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

fn parse_json_body<T: for<'de> Deserialize<'de>>(body: &str) -> DomainResult<T> {
    serde_json::from_str(body).map_err(serialization_error)
}

fn split_target(target: &str) -> (&str, Option<&str>) {
    match target.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (target, None),
    }
}

fn request_body(request: &str) -> &str {
    request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("")
}

fn query_value(query: &str, name: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == name).then(|| decode_query_component(value))
    })
}

fn decode_query_component(value: &str) -> String {
    value.replace('+', " ")
}

fn request_has_valid_token(request: &str, token: &str) -> bool {
    request.lines().any(|line| {
        let lower = line.to_ascii_lowercase();
        lower == format!("authorization: bearer {}", token.to_ascii_lowercase())
            || lower == format!("x-imglab-token: {}", token.to_ascii_lowercase())
    })
}

fn json_response<T: Serialize>(status_code: u16, value: &T) -> HttpResponse {
    match serde_json::to_string(value) {
        Ok(body) => response(status_code, &body),
        Err(error) => response(
            500,
            &format!(
                "{{\"error\":\"{}\"}}",
                escape_json_string(&error.to_string())
            ),
        ),
    }
}

fn error_response(status_code: u16, error: ApiErrorView) -> HttpResponse {
    json_response(status_code, &error)
}

fn domain_error_response(error: DomainError) -> HttpResponse {
    let status_code = match error {
        DomainError::InvalidTaskReference { .. } => 404,
        DomainError::InvalidGenerationParameters { .. } | DomainError::Serialization { .. } => 400,
        _ => 500,
    };
    error_response(
        status_code,
        ApiErrorView {
            code: error.code().to_string(),
            message: error.to_string(),
            recoverable: error.recoverable(),
        },
    )
}

fn response(status_code: u16, body: &str) -> HttpResponse {
    HttpResponse {
        status_code,
        body: body.to_string(),
    }
}

fn http_response_bytes(response: &HttpResponse) -> String {
    let status_text = match response.status_code {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        _ => "Internal Server Error",
    };
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.status_code,
        status_text,
        response.body.len(),
        response.body
    )
}

fn escape_json_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn io_error(path: &Path, error: std::io::Error) -> DomainError {
    DomainError::Io {
        path: path.display().to_string(),
        message: error.to_string(),
    }
}

fn serialization_error(error: serde_json::Error) -> DomainError {
    DomainError::Serialization {
        message: error.to_string(),
    }
}

fn summary_views(tasks: Vec<TaskSummary>) -> Vec<TaskSummaryView> {
    tasks.into_iter().map(TaskSummaryView::from).collect()
}

impl TryFrom<CreateTaskInputView> for CreateTaskInput {
    type Error = DomainError;

    fn try_from(value: CreateTaskInputView) -> Result<Self, Self::Error> {
        let task_type = TaskType::from_str(&value.task_type).ok_or_else(|| {
            DomainError::InvalidGenerationParameters {
                message: format!("unsupported task type: {}", value.task_type),
            }
        })?;
        let operation = value
            .operation
            .as_deref()
            .map(parse_generation_operation)
            .transpose()?;
        let input = value.input_json.or(value.input).unwrap_or(Value::Null);
        Ok(Self {
            task_type,
            provider: value.provider,
            operation,
            priority: value.priority.unwrap_or_default(),
            concurrency_group: value.concurrency_group,
            max_attempts: value.max_attempts.unwrap_or(3),
            input_json: serde_json::to_string(&input).map_err(serialization_error)?,
        })
    }
}

fn parse_generation_operation(value: &str) -> DomainResult<GenerationOperation> {
    match value {
        "text_to_image" => Ok(GenerationOperation::TextToImage),
        "image_to_image" => Ok(GenerationOperation::ImageToImage),
        _ => Err(DomainError::InvalidGenerationParameters {
            message: format!("unsupported generation operation: {value}"),
        }),
    }
}

fn generation_operation_as_str(value: GenerationOperation) -> &'static str {
    match value {
        GenerationOperation::TextToImage => "text_to_image",
        GenerationOperation::ImageToImage => "image_to_image",
    }
}

fn parse_optional_json(value: Option<String>) -> Option<Value> {
    value.and_then(|text| serde_json::from_str(&text).ok())
}

impl From<LibrarySummary> for LibraryView {
    fn from(value: LibrarySummary) -> Self {
        Self {
            id: value.id.0,
            name: value.name,
            root_path: value.root_path,
            schema_version: value.schema_version,
        }
    }
}

impl From<TaskSummary> for TaskSummaryView {
    fn from(value: TaskSummary) -> Self {
        Self {
            id: value.id.0,
            library_id: value.library_id.0,
            task_type: value.task_type.as_str().to_string(),
            status: value.status.as_str().to_string(),
            queue_position: value.queue_position,
            priority: value.priority,
            provider: value.provider,
            operation: value
                .operation
                .map(generation_operation_as_str)
                .map(str::to_string),
            concurrency_group: value.concurrency_group,
            attempt_count: value.attempt_count,
            max_attempts: value.max_attempts,
            next_retry_at: value.next_retry_at,
            input: serde_json::from_str(&value.input_json).unwrap_or(Value::Null),
            created_at: value.created_at,
            updated_at: value.updated_at,
            last_error_code: value.last_error_code,
            last_error_message: value.last_error_message,
            error_classification: value
                .error_classification
                .map(TaskErrorClassification::as_str)
                .map(str::to_string),
            wait_reason: value.wait_reason,
        }
    }
}

impl From<TaskAttempt> for TaskAttemptView {
    fn from(value: TaskAttempt) -> Self {
        Self {
            id: value.id.0,
            task_id: value.task_id.0,
            attempt_number: value.attempt_number,
            status: value.status,
            started_at: value.started_at,
            completed_at: value.completed_at,
            log_path: value.log_path,
            exit_code: value.exit_code,
            error_code: value.error_code,
            error_message: value.error_message,
            error_classification: value
                .error_classification
                .map(TaskErrorClassification::as_str)
                .map(str::to_string),
        }
    }
}

impl From<TaskEvent> for TaskEventView {
    fn from(value: TaskEvent) -> Self {
        Self {
            id: value.id.0,
            task_id: value.task_id.0,
            event_type: value.event_type,
            message: value.message,
            payload: parse_optional_json(value.payload_json),
            created_at: value.created_at,
        }
    }
}

impl From<TaskOutput> for TaskOutputView {
    fn from(value: TaskOutput) -> Self {
        Self {
            id: value.id.0,
            task_id: value.task_id.0,
            output_type: value.output_type.as_str().to_string(),
            target_id: value.target_id,
            payload: parse_optional_json(value.payload_json),
            created_at: value.created_at,
        }
    }
}

impl From<TaskDetail> for TaskDetailView {
    fn from(value: TaskDetail) -> Self {
        Self {
            task: TaskSummaryView::from(value.task),
            attempts: value
                .attempts
                .into_iter()
                .map(TaskAttemptView::from)
                .collect(),
            events: value.events.into_iter().map(TaskEventView::from).collect(),
            outputs: value
                .outputs
                .into_iter()
                .map(TaskOutputView::from)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use imglab_core::{
        AppendTaskAttemptRequest, CreateLibraryRequest, CreateTaskInput, TaskService,
    };
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
        let reorder_response =
            handle_http_request_with_state(&reorder_request, "secret", &mut state);
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
        assert!(serde_json::from_str::<Value>(
            payload["value"].as_str().expect("schema prompt value")
        )
        .is_ok());

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
}
