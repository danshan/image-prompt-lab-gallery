use crate::*;

pub(crate) fn ensure_daemon_client(
    state: &tauri::State<'_, DesktopState>,
) -> Result<daemon_client::DaemonClient, CommandError> {
    let runtime_dir = daemon_runtime_dir();
    let runtime_path = runtime_dir.join("runtime.json");
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

    if !should_start_managed_daemon() {
        match daemon_client::discover_daemon(&runtime_path) {
            Ok(Some(client)) => return Ok(client),
            Ok(None) => {}
            Err(error) if error.recoverable => {}
            Err(error) => return Err(error),
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

pub(crate) fn daemon_runtime_dir() -> PathBuf {
    std::env::var_os("IMGLAB_DAEMON_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("imglab-desktop-daemon"))
}

pub(crate) fn daemon_binary_path() -> Result<PathBuf, CommandError> {
    if let Some(path) = std::env::var_os("IMGLAB_DAEMON_BIN").map(PathBuf::from) {
        return Ok(path);
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
    Ok(dir.join("imglab-daemon"))
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
    let binary_name = if cfg!(target_os = "windows") {
        "imglab-daemon.exe"
    } else {
        "imglab-daemon"
    };
    let path = workspace_root
        .join("target")
        .join("debug")
        .join(binary_name);
    path.exists().then_some(path)
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
