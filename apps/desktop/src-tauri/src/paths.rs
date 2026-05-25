use crate::*;

pub(crate) fn ensure_daemon_client(
    state: &tauri::State<'_, DesktopState>,
) -> Result<daemon_client::DaemonClient, CommandError> {
    let runtime_dir = daemon_runtime_dir();
    let runtime_path = runtime_dir.join("runtime.json");
    let background_runtime_path =
        crate::automation_daemon::background_daemon_runtime_dir().join("runtime.json");
    {
        let mut guard = state.daemon_sidecar.lock().map_err(|_| CommandError {
            code: "ConcurrentWriteConflict".to_string(),
            message: "daemon sidecar state lock poisoned".to_string(),
            recoverable: false,
        })?;
        if let Some(sidecar) = guard.as_ref() {
            if sidecar.client.health().is_ok() {
                return Ok(sidecar.client.clone());
            }
        }
        if let Some(mut sidecar) = guard.take() {
            let _ = sidecar.child.kill();
            let _ = sidecar.child.wait();
        }
    }

    if let Some(client) = discover_reusable_daemon(&background_runtime_path)? {
        return Ok(client);
    }

    if !should_start_managed_daemon() {
        if let Some(client) = discover_reusable_daemon(&runtime_path)? {
            return Ok(client);
        }
    }

    let daemon_bin = daemon_binary_path()?;
    let sidecar = daemon_client::start_daemon_sidecar(&daemon_bin, &runtime_dir)?;
    let client = sidecar.client.clone();
    let mut guard = state.daemon_sidecar.lock().map_err(|_| CommandError {
        code: "ConcurrentWriteConflict".to_string(),
        message: "daemon sidecar state lock poisoned".to_string(),
        recoverable: false,
    })?;
    *guard = Some(sidecar);
    Ok(client)
}

fn discover_reusable_daemon(
    runtime_path: &Path,
) -> Result<Option<daemon_client::DaemonClient>, CommandError> {
    match daemon_client::discover_daemon(runtime_path) {
        Ok(client) => Ok(client),
        Err(error) if error.recoverable => Ok(None),
        Err(error) => Err(error),
    }
}

pub(crate) fn daemon_runtime_dir() -> PathBuf {
    std::env::var_os("IMGLAB_DAEMON_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("imglab-desktop-daemon"))
}

pub(crate) fn daemon_binary_path() -> Result<PathBuf, CommandError> {
    if let Some(path) = std::env::var_os("IMGLAB_DAEMON_BIN").map(PathBuf::from) {
        return existing_daemon_binary_path(path);
    }
    if let Some(path) = workspace_debug_daemon_binary() {
        return Ok(path);
    }
    let exe = std::env::current_exe().map_err(|error| CommandError {
        code: "DaemonStartFailed".to_string(),
        message: format!("failed to locate current executable: {error}"),
        recoverable: true,
    })?;
    let Some(dir) = exe.parent() else {
        return Err(CommandError {
            code: "DaemonStartFailed".to_string(),
            message: "failed to resolve daemon binary directory".to_string(),
            recoverable: true,
        });
    };
    let candidate_names = daemon_binary_candidate_names();
    let mut candidates = Vec::new();
    for name in &candidate_names {
        candidates.push(dir.join(name));
        if let Some(contents_dir) = dir.parent() {
            candidates.push(contents_dir.join("Resources").join(name));
        }
    }
    candidates
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| CommandError {
            code: "DaemonBinaryMissing".to_string(),
            message: format!(
                "daemon binary was not found. Build it with `cargo build -p imglab-daemon` or set IMGLAB_DAEMON_BIN to an existing {} binary",
                daemon_binary_name()
            ),
            recoverable: true,
        })
}

pub(crate) fn should_start_managed_daemon() -> bool {
    std::env::var_os("IMGLAB_DAEMON_REUSE_RUNTIME").is_none()
        && (std::env::var_os("IMGLAB_DAEMON_BIN").is_some()
            || workspace_debug_daemon_binary().is_some())
}

pub(crate) fn workspace_debug_daemon_binary() -> Option<PathBuf> {
    if !cfg!(debug_assertions) {
        return None;
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent()?.parent()?.parent()?;
    let path = workspace_root
        .join("target")
        .join("debug")
        .join(daemon_binary_name());
    path.exists().then_some(path)
}

fn existing_daemon_binary_path(path: PathBuf) -> Result<PathBuf, CommandError> {
    if path.is_file() {
        return Ok(path);
    }
    Err(CommandError {
        code: "DaemonBinaryMissing".to_string(),
        message: format!("daemon binary does not exist: {}", path.display()),
        recoverable: true,
    })
}

fn daemon_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "imglab-daemon.exe"
    } else {
        "imglab-daemon"
    }
}

fn daemon_binary_candidate_names() -> Vec<String> {
    let binary_name = daemon_binary_name();
    let mut names = vec![binary_name.to_string()];
    if let Some(target_triple) = option_env!("TAURI_ENV_TARGET_TRIPLE") {
        let platform_name = if cfg!(target_os = "windows") {
            format!("imglab-daemon-{target_triple}.exe")
        } else {
            format!("imglab-daemon-{target_triple}")
        };
        names.push(platform_name);
    }
    names
}

pub(crate) fn normalize_library_root_path(path: PathBuf) -> Result<PathBuf, CommandError> {
    let path = expand_home_path(path)?;
    if path.is_absolute() {
        Ok(path)
    } else {
        Err(invalid_path_error(
            "library path must be absolute or start with ~/".to_string(),
        ))
    }
}

pub(crate) fn expand_home_path(path: PathBuf) -> Result<PathBuf, CommandError> {
    let raw = path.to_string_lossy();
    if raw == "~" {
        return home_dir();
    }

    if let Some(rest) = raw.strip_prefix("~/") {
        return home_dir().map(|home| home.join(rest));
    }

    if raw.starts_with('~') {
        return Err(invalid_path_error(
            "library path only supports ~ for the current user".to_string(),
        ));
    }

    Ok(path)
}

pub(crate) fn home_dir() -> Result<PathBuf, CommandError> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .ok_or_else(|| invalid_path_error("HOME is not set to an absolute path".to_string()))
}

pub(crate) fn reveal_path(path: &Path) -> Result<(), CommandError> {
    let status = if cfg!(target_os = "macos") {
        Command::new("open").arg(path).status()
    } else if cfg!(target_os = "windows") {
        Command::new("explorer").arg(path).status()
    } else {
        Command::new("xdg-open").arg(path).status()
    }
    .map_err(|error| CommandError {
        code: "RevealFailed".to_string(),
        message: format!("failed to open folder: {error}"),
        recoverable: true,
    })?;

    if status.success() {
        Ok(())
    } else {
        Err(CommandError {
            code: "RevealFailed".to_string(),
            message: format!("open folder command exited with status: {status}"),
            recoverable: true,
        })
    }
}
