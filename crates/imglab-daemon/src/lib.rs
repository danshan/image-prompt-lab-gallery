use imglab_core::{
    classify_task_error, evaluate_scheduler, prepare_generation_request, AssetId,
    BatchCreateTasksRequest, CreateMetadataSuggestionRequest, CreateTaskInput, DomainError,
    DomainResult, GalleryReadService, GenerationOperation, GenerationRequestInput,
    GenerationService, ImageProvider, LibraryId, LibraryService, LibrarySummary,
    LocalGenerationService, LocalLibraryService, MetadataReviewService, ReorderQueuedTasksRequest,
    RetryPolicy, TaskAttempt, TaskDetail, TaskErrorClassification, TaskEvent, TaskId, TaskOutput,
    TaskOutputType, TaskSchedulerConfig, TaskService, TaskStatus, TaskSummary, TaskType,
    UpdateTaskStatusRequest, CURRENT_SCHEMA_VERSION,
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

mod executors;
mod logs;
mod routes;
mod runtime;
mod runtime_io;
mod scheduler;
mod task_dto;
mod transport;
mod views;

pub use runtime::{
    CapabilitiesView, DaemonConfig, DaemonState, HealthView, HttpResponse,
    ImageGenerationTaskInput, ProviderCapabilityView, RuntimeFile, SharedDaemonState,
};
pub use runtime_io::{
    handle_http_request, handle_http_request_with_shared_state, handle_http_request_with_state,
    serve_forever, serve_forever_shared, serve_one, serve_one_with_shared_state,
    serve_one_with_state,
};
pub use task_dto::{capabilities_view, health_view};
pub use transport::{
    recover_open_libraries, run_scheduler_loop_iteration, run_scheduler_tick, spawn_scheduler_loop,
};
pub use views::{
    bind_loopback_listener, generate_session_token, is_loopback_addr, read_runtime_file,
    write_runtime_file, write_token_file,
};

#[cfg(test)]
mod tests;
