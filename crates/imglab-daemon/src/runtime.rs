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
    pub schema_version: u32,
    pub provider_capabilities: Vec<ProviderCapabilityView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCapabilityView {
    pub provider: String,
    pub supported_operations: Vec<String>,
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
    pub input_file: Option<PathBuf>,
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

