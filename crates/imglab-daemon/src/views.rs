use crate::routes::{io_error, serialization_error};
use crate::runtime::*;
use crate::*;

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

pub(crate) fn summary_views(tasks: Vec<TaskSummary>) -> Vec<TaskSummaryView> {
    tasks.into_iter().map(TaskSummaryView::from).collect()
}

impl TryFrom<CreateTaskInputView> for CreateTaskInput {
    type Error = DomainError;

    fn try_from(value: CreateTaskInputView) -> Result<Self, Self::Error> {
        let task_type = TaskType::parse(&value.task_type).ok_or_else(|| {
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

pub(crate) fn parse_generation_operation(value: &str) -> DomainResult<GenerationOperation> {
    match value {
        "text_to_image" => Ok(GenerationOperation::TextToImage),
        "image_to_image" => Ok(GenerationOperation::ImageToImage),
        _ => Err(DomainError::InvalidGenerationParameters {
            message: format!("unsupported generation operation: {value}"),
        }),
    }
}

pub(crate) fn generation_operation_as_str(value: GenerationOperation) -> &'static str {
    match value {
        GenerationOperation::TextToImage => "text_to_image",
        GenerationOperation::ImageToImage => "image_to_image",
    }
}

pub(crate) fn parse_optional_json(value: Option<String>) -> Option<Value> {
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
