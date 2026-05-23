use crate::transport::recover_open_libraries;
use crate::*;

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

pub struct DaemonState {
    pub(crate) registry_path: PathBuf,
    pub(crate) app: imglab_core::infrastructure::composition::SqliteImgLabApplication<
        imglab_core::FakeImageProvider,
    >,
    pub(crate) opened_libraries: BTreeMap<String, PathBuf>,
    pub(crate) recovered_libraries: BTreeSet<String>,
    pub(crate) log_root: PathBuf,
}

pub type SharedDaemonState = Arc<Mutex<DaemonState>>;

impl DaemonState {
    pub fn new(registry_path: impl Into<PathBuf>, log_root: impl Into<PathBuf>) -> Self {
        let registry_path = registry_path.into();
        Self {
            registry_path: registry_path.clone(),
            app: imglab_core::infrastructure::composition::sqlite_application(
                registry_path,
                imglab_core::FakeImageProvider::success("fake"),
            ),
            opened_libraries: BTreeMap::new(),
            recovered_libraries: BTreeSet::new(),
            log_root: log_root.into(),
        }
    }

    pub(crate) fn service(&self) -> &LocalLibraryService {
        self.app.library()
    }

    pub(crate) fn library_lifecycle(
        &self,
    ) -> &imglab_core::application::use_cases::library::LibraryUseCase<LocalLibraryService> {
        self.app.library_lifecycle()
    }

    pub(crate) fn tasks(
        &self,
    ) -> &imglab_core::application::use_cases::tasks::TaskUseCase<LocalLibraryService> {
        self.app.tasks()
    }

    pub(crate) fn gallery(
        &self,
    ) -> &imglab_core::application::use_cases::albums::QueryGalleryUseCase<LocalLibraryService>
    {
        self.app.gallery()
    }

    pub(crate) fn create_metadata_suggestion(
        &self,
        request: CreateMetadataSuggestionRequest,
    ) -> DomainResult<imglab_core::MetadataSuggestion> {
        self.app.metadata_review().create_suggestion(request)
    }

    pub(crate) fn open_library(&mut self, root_path: &Path) -> DomainResult<LibrarySummary> {
        let library = self.library_lifecycle().open_library(root_path)?;
        let should_recover = !self.recovered_libraries.contains(&library.id.0);
        self.opened_libraries
            .insert(library.id.0.clone(), library.root_path.clone());
        if should_recover {
            recover_open_libraries(self, &RetryPolicy::default())?;
            self.recovered_libraries.insert(library.id.0.clone());
        }
        Ok(library)
    }

    pub(crate) fn library_path(&self, library_id: &str) -> DomainResult<PathBuf> {
        self.opened_libraries
            .get(library_id)
            .cloned()
            .ok_or_else(|| DomainError::InvalidGenerationParameters {
                message: format!("library is not open: {library_id}"),
            })
    }
}

impl Clone for DaemonState {
    fn clone(&self) -> Self {
        Self {
            registry_path: self.registry_path.clone(),
            app: imglab_core::infrastructure::composition::sqlite_application(
                self.registry_path.clone(),
                imglab_core::FakeImageProvider::success("fake"),
            ),
            opened_libraries: self.opened_libraries.clone(),
            recovered_libraries: self.recovered_libraries.clone(),
            log_root: self.log_root.clone(),
        }
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
pub(crate) struct MetadataFieldTaskInput {
    pub(crate) suggestion_id: String,
    pub(crate) asset_id: String,
    pub(crate) field: String,
    pub(crate) base_revision: Option<String>,
    pub(crate) context: MetadataTaskContext,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MetadataSuggestionTaskInput {
    pub(crate) suggestion_id: String,
    pub(crate) asset_id: String,
    pub(crate) base_revision: Option<String>,
    pub(crate) context: MetadataTaskContext,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MetadataTaskContext {
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) schema_prompt: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) category: Option<String>,
    pub(crate) source_prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenLibraryInput {
    pub(crate) library_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateTaskInputView {
    pub(crate) task_type: String,
    pub(crate) provider: Option<String>,
    pub(crate) operation: Option<String>,
    pub(crate) priority: Option<i64>,
    pub(crate) concurrency_group: Option<String>,
    pub(crate) max_attempts: Option<u32>,
    pub(crate) input: Option<Value>,
    pub(crate) input_json: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BatchCreateTasksInput {
    pub(crate) library_id: String,
    pub(crate) tasks: Vec<CreateTaskInputView>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SingleCreateTaskInput {
    pub(crate) library_id: String,
    #[serde(flatten)]
    pub(crate) task: CreateTaskInputView,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReorderTasksInput {
    pub(crate) library_id: String,
    pub(crate) task_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LibraryView {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) root_path: PathBuf,
    pub(crate) schema_version: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TaskSummaryView {
    pub(crate) id: String,
    pub(crate) library_id: String,
    pub(crate) task_type: String,
    pub(crate) status: String,
    pub(crate) queue_position: i64,
    pub(crate) priority: i64,
    pub(crate) provider: Option<String>,
    pub(crate) operation: Option<String>,
    pub(crate) concurrency_group: Option<String>,
    pub(crate) attempt_count: u32,
    pub(crate) max_attempts: u32,
    pub(crate) next_retry_at: Option<String>,
    pub(crate) input: Value,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
    pub(crate) last_error_code: Option<String>,
    pub(crate) last_error_message: Option<String>,
    pub(crate) error_classification: Option<String>,
    pub(crate) wait_reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TaskAttemptView {
    pub(crate) id: String,
    pub(crate) task_id: String,
    pub(crate) attempt_number: u32,
    pub(crate) status: String,
    pub(crate) started_at: String,
    pub(crate) completed_at: Option<String>,
    pub(crate) log_path: Option<PathBuf>,
    pub(crate) exit_code: Option<i32>,
    pub(crate) error_code: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) error_classification: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TaskEventView {
    pub(crate) id: String,
    pub(crate) task_id: String,
    pub(crate) event_type: String,
    pub(crate) message: Option<String>,
    pub(crate) payload: Option<Value>,
    pub(crate) created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TaskOutputView {
    pub(crate) id: String,
    pub(crate) task_id: String,
    pub(crate) output_type: String,
    pub(crate) target_id: String,
    pub(crate) payload: Option<Value>,
    pub(crate) created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TaskDetailView {
    pub(crate) task: TaskSummaryView,
    pub(crate) attempts: Vec<TaskAttemptView>,
    pub(crate) events: Vec<TaskEventView>,
    pub(crate) outputs: Vec<TaskOutputView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LogTailView {
    pub(crate) content: String,
    pub(crate) truncated: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApiErrorView {
    pub(crate) code: String,
    pub(crate) message: String,
    pub(crate) recoverable: bool,
}
