use super::CommandError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const API_VERSION: &str = "v1";
const HEALTH_PATH: &str = "/v1/health";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
#[allow(dead_code)]
const SIDECAR_START_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonRuntimeFile {
    pub api_version: String,
    #[allow(dead_code)]
    pub pid: u32,
    pub port: u16,
    pub token_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DaemonClient {
    port: u16,
    token: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DaemonSidecar {
    pub client: DaemonClient,
    pub child: Child,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonTaskInput {
    pub task_type: String,
    pub provider: Option<String>,
    pub operation: Option<String>,
    pub priority: Option<i64>,
    pub concurrency_group: Option<String>,
    pub max_attempts: Option<u32>,
    pub input: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCreateTasksInput {
    pub library_id: String,
    pub tasks: Vec<DaemonTaskInput>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonTask {
    pub id: String,
    pub library_id: String,
    pub task_type: String,
    pub status: String,
    pub queue_position: i64,
    pub priority: i64,
    pub provider: Option<String>,
    pub operation: Option<String>,
    pub concurrency_group: Option<String>,
    pub attempt_count: u32,
    pub max_attempts: u32,
    pub next_retry_at: Option<String>,
    pub input: Value,
    pub created_at: String,
    pub updated_at: String,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
    pub error_classification: Option<String>,
    pub wait_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonTaskAttempt {
    pub id: String,
    pub task_id: String,
    pub attempt_number: u32,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub log_path: Option<PathBuf>,
    pub exit_code: Option<i32>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub error_classification: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonTaskEvent {
    pub id: String,
    pub task_id: String,
    pub event_type: String,
    pub message: Option<String>,
    pub payload: Option<Value>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonTaskOutput {
    pub id: String,
    pub task_id: String,
    pub output_type: String,
    pub target_id: String,
    pub payload: Option<Value>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonTaskDetail {
    pub task: DaemonTask,
    pub attempts: Vec<DaemonTaskAttempt>,
    pub events: Vec<DaemonTaskEvent>,
    pub outputs: Vec<DaemonTaskOutput>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonLogTail {
    pub content: String,
    pub truncated: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DaemonErrorBody {
    code: String,
    message: String,
    recoverable: bool,
}

#[derive(Debug)]
struct HttpResponse {
    status_code: u16,
    body: String,
}

impl DaemonClient {
    pub fn from_runtime_file(path: &Path) -> Result<Self, CommandError> {
        let runtime: DaemonRuntimeFile =
            serde_json::from_str(&fs::read_to_string(path).map_err(|error| CommandError {
                code: "DaemonRuntimeUnavailable".to_string(),
                message: format!("failed to read daemon runtime file: {error}"),
                recoverable: true,
            })?)
            .map_err(|error| CommandError {
                code: "DaemonRuntimeInvalid".to_string(),
                message: format!("failed to parse daemon runtime file: {error}"),
                recoverable: true,
            })?;
        if runtime.api_version != API_VERSION {
            return Err(CommandError {
                code: "DaemonVersionMismatch".to_string(),
                message: format!(
                    "daemon API version mismatch: expected {API_VERSION}, found {}",
                    runtime.api_version
                ),
                recoverable: true,
            });
        }
        let token = fs::read_to_string(&runtime.token_path).map_err(|error| CommandError {
            code: "DaemonTokenUnavailable".to_string(),
            message: format!("failed to read daemon token file: {error}"),
            recoverable: true,
        })?;
        Ok(Self {
            port: runtime.port,
            token,
        })
    }

    pub fn health(&self) -> Result<(), CommandError> {
        self.request("GET", HEALTH_PATH, None).map(|_| ())
    }

    pub fn open_library(&self, library_path: &Path) -> Result<String, CommandError> {
        let body = serde_json::json!({ "libraryPath": library_path });
        let response = self.request("POST", "/v1/libraries/open", Some(body))?;
        let value: Value = serde_json::from_str(&response).map_err(daemon_parse_error)?;
        value["id"]
            .as_str()
            .map(ToString::to_string)
            .ok_or_else(|| CommandError {
                code: "DaemonResponseInvalid".to_string(),
                message: "daemon open library response did not include id".to_string(),
                recoverable: true,
            })
    }

    pub fn batch_create_tasks(
        &self,
        input: BatchCreateTasksInput,
    ) -> Result<Vec<DaemonTask>, CommandError> {
        let body = serde_json::to_value(input).map_err(daemon_parse_error)?;
        let response = self.request("POST", "/v1/tasks/batch", Some(body))?;
        serde_json::from_str(&response).map_err(daemon_parse_error)
    }

    pub fn list_tasks(&self, library_id: &str) -> Result<Vec<DaemonTask>, CommandError> {
        let response = self.request("GET", &format!("/v1/tasks?library_id={library_id}"), None)?;
        serde_json::from_str(&response).map_err(daemon_parse_error)
    }

    pub fn get_task(&self, task_id: &str) -> Result<DaemonTaskDetail, CommandError> {
        let response = self.request("GET", &format!("/v1/tasks/{task_id}"), None)?;
        serde_json::from_str(&response).map_err(daemon_parse_error)
    }

    pub fn reorder_tasks(
        &self,
        library_id: String,
        task_ids: Vec<String>,
    ) -> Result<(), CommandError> {
        let body = serde_json::json!({ "libraryId": library_id, "taskIds": task_ids });
        self.request("POST", "/v1/tasks/reorder", Some(body))
            .map(|_| ())
    }

    pub fn cancel_task(&self, task_id: &str) -> Result<DaemonTask, CommandError> {
        let response = self.request("POST", &format!("/v1/tasks/{task_id}/cancel"), None)?;
        serde_json::from_str(&response).map_err(daemon_parse_error)
    }

    pub fn retry_task(&self, task_id: &str) -> Result<DaemonTask, CommandError> {
        let response = self.request("POST", &format!("/v1/tasks/{task_id}/retry"), None)?;
        serde_json::from_str(&response).map_err(daemon_parse_error)
    }

    pub fn duplicate_task(&self, task_id: &str) -> Result<DaemonTask, CommandError> {
        let response = self.request("POST", &format!("/v1/tasks/{task_id}/duplicate"), None)?;
        serde_json::from_str(&response).map_err(daemon_parse_error)
    }

    pub fn tail_task_log(&self, task_id: &str) -> Result<DaemonLogTail, CommandError> {
        let response = self.request("GET", &format!("/v1/tasks/{task_id}/logs/tail"), None)?;
        serde_json::from_str(&response).map_err(daemon_parse_error)
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> Result<String, CommandError> {
        let request = build_http_request(method, path, &self.token, body)?;
        let mut stream =
            TcpStream::connect(("127.0.0.1", self.port)).map_err(|error| CommandError {
                code: "DaemonUnavailable".to_string(),
                message: format!("failed to connect daemon: {error}"),
                recoverable: true,
            })?;
        stream
            .set_read_timeout(Some(REQUEST_TIMEOUT))
            .map_err(daemon_io_error)?;
        stream
            .set_write_timeout(Some(REQUEST_TIMEOUT))
            .map_err(daemon_io_error)?;
        stream
            .write_all(request.as_bytes())
            .map_err(daemon_io_error)?;
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(daemon_io_error)?;
        let parsed = parse_http_response(&response)?;
        if parsed.status_code >= 400 {
            return Err(map_daemon_error(parsed));
        }
        Ok(parsed.body)
    }
}

pub fn discover_daemon(runtime_path: &Path) -> Result<Option<DaemonClient>, CommandError> {
    if !runtime_path.exists() {
        return Ok(None);
    }
    let client = DaemonClient::from_runtime_file(runtime_path)?;
    client.health()?;
    Ok(Some(client))
}

#[allow(dead_code)]
pub fn start_daemon_sidecar(
    daemon_bin: &Path,
    runtime_dir: &Path,
) -> Result<DaemonSidecar, CommandError> {
    fs::create_dir_all(runtime_dir).map_err(daemon_io_error)?;
    let runtime_path = runtime_dir.join("runtime.json");
    let mut child = Command::new(daemon_bin)
        .env("IMGLAB_DAEMON_RUNTIME_DIR", runtime_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| CommandError {
            code: "DaemonStartFailed".to_string(),
            message: format!("failed to start daemon sidecar: {error}"),
            recoverable: true,
        })?;
    let started_at = Instant::now();
    while started_at.elapsed() < SIDECAR_START_TIMEOUT {
        if let Some(client) = discover_daemon(&runtime_path)? {
            return Ok(DaemonSidecar { client, child });
        }
        thread::sleep(Duration::from_millis(100));
    }
    let _ = child.kill();
    let _ = child.wait();
    Err(CommandError {
        code: "DaemonStartTimeout".to_string(),
        message: "daemon sidecar did not become healthy before timeout".to_string(),
        recoverable: true,
    })
}

fn build_http_request(
    method: &str,
    path: &str,
    token: &str,
    body: Option<Value>,
) -> Result<String, CommandError> {
    let body = body
        .map(|value| serde_json::to_string(&value))
        .transpose()
        .map_err(daemon_parse_error)?
        .unwrap_or_default();
    Ok(format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    ))
}

fn parse_http_response(raw: &str) -> Result<HttpResponse, CommandError> {
    let (head, body) = raw.split_once("\r\n\r\n").ok_or_else(|| CommandError {
        code: "DaemonResponseInvalid".to_string(),
        message: "daemon response did not contain HTTP header separator".to_string(),
        recoverable: true,
    })?;
    let status_line = head.lines().next().ok_or_else(|| CommandError {
        code: "DaemonResponseInvalid".to_string(),
        message: "daemon response did not contain status line".to_string(),
        recoverable: true,
    })?;
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| CommandError {
            code: "DaemonResponseInvalid".to_string(),
            message: "daemon response status line did not contain status code".to_string(),
            recoverable: true,
        })?
        .parse::<u16>()
        .map_err(|error| CommandError {
            code: "DaemonResponseInvalid".to_string(),
            message: format!("daemon response status code was invalid: {error}"),
            recoverable: true,
        })?;
    Ok(HttpResponse {
        status_code,
        body: body.to_string(),
    })
}

fn map_daemon_error(response: HttpResponse) -> CommandError {
    serde_json::from_str::<DaemonErrorBody>(&response.body)
        .map(|error| CommandError {
            code: error.code,
            message: error.message,
            recoverable: error.recoverable,
        })
        .unwrap_or_else(|_| CommandError {
            code: "DaemonRequestFailed".to_string(),
            message: format!("daemon request failed with HTTP {}", response.status_code),
            recoverable: true,
        })
}

fn daemon_io_error(error: std::io::Error) -> CommandError {
    CommandError {
        code: "DaemonIo".to_string(),
        message: format!("daemon io error: {error}"),
        recoverable: true,
    }
}

fn daemon_parse_error(error: serde_json::Error) -> CommandError {
    CommandError {
        code: "DaemonResponseInvalid".to_string(),
        message: format!("daemon JSON response was invalid: {error}"),
        recoverable: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_daemon_error_response() {
        let response = parse_http_response(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\r\n{\"code\":\"InvalidTaskReference\",\"message\":\"missing\",\"recoverable\":true}",
        )
        .expect("parse response");
        let error = map_daemon_error(response);
        assert_eq!(error.code, "InvalidTaskReference");
        assert_eq!(error.message, "missing");
        assert!(error.recoverable);
    }

    #[test]
    fn builds_authorized_json_request() {
        let request = build_http_request(
            "POST",
            "/v1/tasks/batch",
            "secret",
            Some(serde_json::json!({"libraryId":"library"})),
        )
        .expect("build request");
        assert!(request.contains("Authorization: Bearer secret"));
        assert!(request.contains("Content-Length: 23"));
        assert!(request.ends_with("{\"libraryId\":\"library\"}"));
    }

    #[test]
    fn missing_runtime_file_is_not_a_daemon() {
        let path = std::env::temp_dir().join("imglab-missing-runtime-file.json");
        let client = discover_daemon(&path).expect("discovery");
        assert!(client.is_none());
    }
}
