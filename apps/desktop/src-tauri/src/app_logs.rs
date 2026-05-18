use crate::CommandError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

const LOG_PREVIEW_BYTES: usize = 800;
const LOG_CONTENT_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppLogView {
    pub path: PathBuf,
    pub kind: String,
    pub modified_at: String,
    pub size_bytes: u64,
    pub preview: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadAppLogInput {
    pub path: PathBuf,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppLogContentView {
    pub path: PathBuf,
    pub content: String,
    pub truncated: bool,
}

pub fn list_app_logs() -> Result<Vec<AppLogView>, CommandError> {
    let mut logs = Vec::new();
    for directory in app_log_roots() {
        if directory.exists() {
            logs.extend(list_app_logs_in_dir(&directory)?);
        }
    }
    logs.sort_by(|left, right| right.modified_at.cmp(&left.modified_at));
    Ok(logs)
}

pub fn read_app_log(path: &Path) -> Result<AppLogContentView, CommandError> {
    if !app_log_roots()
        .into_iter()
        .any(|root| is_allowed_app_log_path(&root, path))
    {
        return Err(invalid_log_path(path));
    }
    read_app_log_content(path)
}

fn app_log_roots() -> Vec<PathBuf> {
    let mut roots = vec![std::env::temp_dir()];
    let daemon_runtime_dir = std::env::var_os("IMGLAB_DAEMON_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("imglab-desktop-daemon"));
    roots.push(daemon_runtime_dir.join("logs"));
    roots.push(std::env::temp_dir().join("imglab-daemon-logs"));
    roots
}

fn list_app_logs_in_dir(directory: &Path) -> Result<Vec<AppLogView>, CommandError> {
    let entries = fs::read_dir(directory).map_err(|error| CommandError {
        code: "Io".to_string(),
        message: format!(
            "failed to read log directory {}: {error}",
            directory.display()
        ),
        recoverable: true,
    })?;

    let mut logs = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| CommandError {
            code: "Io".to_string(),
            message: format!("failed to read log directory entry: {error}"),
            recoverable: true,
        })?;
        let path = entry.path();
        let Some(kind) = classify_app_log(&path) else {
            continue;
        };
        let metadata = fs::symlink_metadata(&path).map_err(|error| CommandError {
            code: "Io".to_string(),
            message: format!("failed to read log metadata {}: {error}", path.display()),
            recoverable: true,
        })?;
        if !metadata.is_file() {
            continue;
        }
        if !is_allowed_app_log_path(directory, &path) {
            continue;
        }
        logs.push(AppLogView {
            path: path.clone(),
            kind: kind.to_string(),
            modified_at: modified_at_string(&metadata),
            size_bytes: metadata.len(),
            preview: read_preview(&path, LOG_PREVIEW_BYTES)?,
        });
    }

    logs.sort_by(|left, right| right.modified_at.cmp(&left.modified_at));
    Ok(logs)
}

fn read_app_log_content(path: &Path) -> Result<AppLogContentView, CommandError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| CommandError {
        code: "Io".to_string(),
        message: format!("failed to read log metadata {}: {error}", path.display()),
        recoverable: true,
    })?;
    if !metadata.is_file() {
        return Err(invalid_log_path(path));
    }
    let mut file = fs::File::open(path).map_err(|error| CommandError {
        code: "Io".to_string(),
        message: format!("failed to open log {}: {error}", path.display()),
        recoverable: true,
    })?;
    let size = file
        .metadata()
        .map_err(|error| CommandError {
            code: "Io".to_string(),
            message: format!("failed to read log metadata {}: {error}", path.display()),
            recoverable: true,
        })?
        .len();
    let truncated = size > LOG_CONTENT_BYTES;
    if truncated {
        file.seek(SeekFrom::Start(size.saturating_sub(LOG_CONTENT_BYTES)))
            .map_err(|error| CommandError {
                code: "Io".to_string(),
                message: format!("failed to seek log {}: {error}", path.display()),
                recoverable: true,
            })?;
    }
    let mut content = Vec::new();
    file.take(LOG_CONTENT_BYTES)
        .read_to_end(&mut content)
        .map_err(|error| CommandError {
            code: "Io".to_string(),
            message: format!("failed to read log {}: {error}", path.display()),
            recoverable: true,
        })?;
    let content = String::from_utf8_lossy(&content).to_string();
    Ok(AppLogContentView {
        path: path.to_path_buf(),
        content,
        truncated,
    })
}

fn is_allowed_app_log_path(temp_dir: &Path, path: &Path) -> bool {
    if classify_app_log(path).is_none() {
        return false;
    }
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    let Ok(canonical_temp_dir) = temp_dir.canonicalize() else {
        return false;
    };
    let Ok(canonical_path) = path.canonicalize() else {
        return false;
    };
    canonical_path.parent() == Some(canonical_temp_dir.as_path())
}

fn classify_app_log(path: &Path) -> Option<&'static str> {
    let filename = path.file_name()?.to_str()?;
    if filename.starts_with("imglab-task-") && filename.ends_with(".log") {
        return Some("task-attempt");
    }
    if filename.starts_with("imglab-codex-cli-") && filename.ends_with(".log") {
        return Some("codex-image-generation");
    }
    if filename.starts_with("imglab-codex-metadata-") && filename.ends_with(".log") {
        return Some("codex-metadata-generation");
    }
    None
}

fn read_preview(path: &Path, limit: usize) -> Result<String, CommandError> {
    let mut file = fs::File::open(path).map_err(|error| CommandError {
        code: "Io".to_string(),
        message: format!("failed to open log {}: {error}", path.display()),
        recoverable: true,
    })?;
    let mut buffer = vec![0_u8; limit];
    let read = file.read(&mut buffer).map_err(|error| CommandError {
        code: "Io".to_string(),
        message: format!("failed to read log {}: {error}", path.display()),
        recoverable: true,
    })?;
    buffer.truncate(read);
    Ok(String::from_utf8_lossy(&buffer).to_string())
}

fn modified_at_string(metadata: &fs::Metadata) -> String {
    metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis().to_string())
        .unwrap_or_else(|| "0".to_string())
}

fn invalid_log_path(path: &Path) -> CommandError {
    CommandError {
        code: "InvalidAppLogPath".to_string(),
        message: format!("path is not an app-owned log: {}", path.display()),
        recoverable: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let counter = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "imglab-log-tests-{}-{counter}-{nanos}",
            std::process::id()
        ))
    }

    #[test]
    fn classifies_supported_log_names() {
        assert_eq!(
            classify_app_log(Path::new("/tmp/imglab-codex-cli-123.log")),
            Some("codex-image-generation")
        );
        assert_eq!(
            classify_app_log(Path::new("/tmp/imglab-codex-metadata-123.log")),
            Some("codex-metadata-generation")
        );
        assert_eq!(
            classify_app_log(Path::new("/tmp/imglab-task-task-1-attempt-1.log")),
            Some("task-attempt")
        );
        assert_eq!(classify_app_log(Path::new("/tmp/other.log")), None);
    }

    #[test]
    fn lists_app_logs_sorted_by_modified_time() {
        let dir = unique_dir();
        fs::create_dir_all(&dir).expect("dir");
        let older = dir.join("imglab-codex-cli-1.log");
        let newer = dir.join("imglab-codex-metadata-2.log");
        fs::write(&older, "old").expect("older");
        std::thread::sleep(std::time::Duration::from_millis(2));
        fs::write(&newer, "new").expect("newer");
        fs::write(dir.join("other.log"), "ignore").expect("other");

        let logs = list_app_logs_in_dir(&dir).expect("logs");

        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].path, newer);
        assert_eq!(logs[0].kind, "codex-metadata-generation");
        assert_eq!(logs[1].path, older);
    }

    #[test]
    fn rejects_non_app_log_paths() {
        let temp_dir = std::env::temp_dir();
        assert!(!is_allowed_app_log_path(
            &temp_dir,
            Path::new("/etc/passwd")
        ));
        assert!(!is_allowed_app_log_path(
            &temp_dir,
            &temp_dir.join("other.log")
        ));
    }

    #[test]
    fn accepts_task_logs_under_allowed_daemon_log_root_only() {
        let dir = unique_dir();
        fs::create_dir_all(&dir).expect("dir");
        let log = dir.join("imglab-task-abc-attempt-1.log");
        fs::write(&log, "task log").expect("log");
        let nested = dir.join("nested").join("imglab-task-def-attempt-1.log");
        fs::create_dir_all(nested.parent().expect("nested parent")).expect("nested dir");
        fs::write(&nested, "nested log").expect("nested log");

        assert!(is_allowed_app_log_path(&dir, &log));
        assert!(!is_allowed_app_log_path(&dir, &nested));
    }

    #[test]
    fn reads_allowed_log_content_with_limit() {
        let dir = std::env::temp_dir();
        let path = dir.join("imglab-codex-metadata-test-read.log");
        fs::write(&path, "hello").expect("write");

        assert!(is_allowed_app_log_path(&dir, &path));
        let content = read_app_log_content(&path).expect("content");
        assert_eq!(content.content, "hello");
        assert!(!content.truncated);
    }
}
