use crate::*;
use std::fs;
use std::process::Stdio;
use std::thread;
use std::time::{Duration, Instant};

const LAUNCH_AGENT_LABEL: &str = "com.imagepromptlab.daemon";
const PLIST_NAME: &str = "com.imagepromptlab.daemon.plist";
const LAUNCHCTL_TIMEOUT: Duration = Duration::from_secs(4);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AutomationDaemonStatusView {
    pub(crate) enabled: bool,
    pub(crate) healthy: bool,
    pub(crate) launch_agent_path: PathBuf,
    pub(crate) runtime_path: PathBuf,
    pub(crate) recoverable_error: Option<String>,
}

pub(crate) fn automation_daemon_status() -> AutomationDaemonStatusView {
    let launch_agent_path = launch_agent_path();
    let runtime_path = background_daemon_runtime_dir().join("runtime.json");
    let enabled = launch_agent_path.exists();
    match daemon_client::discover_daemon(&runtime_path) {
        Ok(Some(_)) => AutomationDaemonStatusView {
            enabled,
            healthy: true,
            launch_agent_path,
            runtime_path,
            recoverable_error: None,
        },
        Ok(None) => AutomationDaemonStatusView {
            enabled,
            healthy: false,
            launch_agent_path,
            runtime_path,
            recoverable_error: enabled
                .then(|| "background daemon runtime is not available".to_string()),
        },
        Err(error) => AutomationDaemonStatusView {
            enabled,
            healthy: false,
            launch_agent_path,
            runtime_path,
            recoverable_error: Some(error.message),
        },
    }
}

pub(crate) fn install_automation_daemon() -> Result<AutomationDaemonStatusView, CommandError> {
    let daemon_bin = daemon_binary_path()?;
    let plist_path = launch_agent_path();
    if let Some(parent) = plist_path.parent() {
        fs::create_dir_all(parent).map_err(daemon_service_io_error)?;
    }
    fs::write(
        &plist_path,
        launch_agent_plist(
            &daemon_bin,
            &background_daemon_runtime_dir(),
            &default_registry_path(),
        ),
    )
    .map_err(daemon_service_io_error)?;
    let domain = launchctl_gui_domain();
    run_launchctl(&["bootstrap", &domain, &plist_path.display().to_string()]).ok();
    run_launchctl(&["enable", &format!("{domain}/{LAUNCH_AGENT_LABEL}")]).ok();
    Ok(automation_daemon_status())
}

pub(crate) fn uninstall_automation_daemon() -> Result<AutomationDaemonStatusView, CommandError> {
    let plist_path = launch_agent_path();
    let domain = launchctl_gui_domain();
    run_launchctl(&["bootout", &domain, &plist_path.display().to_string()]).ok();
    match fs::remove_file(&plist_path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(daemon_service_io_error(error)),
    }
    Ok(automation_daemon_status())
}

pub(crate) fn restart_automation_daemon() -> Result<AutomationDaemonStatusView, CommandError> {
    let plist_path = launch_agent_path();
    if !plist_path.exists() {
        return install_automation_daemon();
    }
    let domain = launchctl_gui_domain();
    run_launchctl(&["kickstart", "-k", &format!("{domain}/{LAUNCH_AGENT_LABEL}")]).ok();
    Ok(automation_daemon_status())
}

pub(crate) fn repair_automation_daemon() -> Result<AutomationDaemonStatusView, CommandError> {
    install_automation_daemon()
}

pub(crate) fn background_daemon_runtime_dir() -> PathBuf {
    std::env::var_os("IMGLAB_BACKGROUND_DAEMON_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            home_dir()
                .unwrap_or_else(|_| std::env::temp_dir())
                .join("Library")
                .join("Application Support")
                .join("Image Prompt Lab")
                .join("daemon")
        })
}

pub(crate) fn launch_agent_path() -> PathBuf {
    std::env::var_os("IMGLAB_LAUNCH_AGENT_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            home_dir()
                .unwrap_or_else(|_| std::env::temp_dir())
                .join("Library")
                .join("LaunchAgents")
                .join(PLIST_NAME)
        })
}

pub(crate) fn launch_agent_plist(
    daemon_bin: &Path,
    runtime_dir: &Path,
    registry_path: &Path,
) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{label}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{daemon_bin}</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>IMGLAB_DAEMON_RUNTIME_DIR</key>
    <string>{runtime_dir}</string>
    <key>IMGLAB_REGISTRY</key>
    <string>{registry_path}</string>
  </dict>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>{runtime_dir}/logs/stdout.log</string>
  <key>StandardErrorPath</key>
  <string>{runtime_dir}/logs/stderr.log</string>
</dict>
</plist>
"#,
        label = LAUNCH_AGENT_LABEL,
        daemon_bin = escape_plist_text(&daemon_bin.display().to_string()),
        runtime_dir = escape_plist_text(&runtime_dir.display().to_string()),
        registry_path = escape_plist_text(&registry_path.display().to_string()),
    )
}

fn run_launchctl(args: &[&str]) -> Result<(), CommandError> {
    if !cfg!(target_os = "macos") {
        return Ok(());
    }
    let mut child = Command::new("launchctl")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(daemon_service_io_error)?;
    let started_at = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(daemon_service_io_error)? {
            break status;
        }
        if started_at.elapsed() >= LAUNCHCTL_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return Err(CommandError {
                code: "AutomationDaemonServiceTimeout".to_string(),
                message: format!("launchctl timed out after {:?}", LAUNCHCTL_TIMEOUT),
                recoverable: true,
            });
        }
        thread::sleep(Duration::from_millis(50));
    };
    if status.success() {
        Ok(())
    } else {
        Err(CommandError {
            code: "AutomationDaemonServiceError".to_string(),
            message: format!("launchctl exited with {status}"),
            recoverable: true,
        })
    }
}

fn launchctl_gui_domain() -> String {
    let uid = Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|output| {
            output
                .status
                .success()
                .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "501".to_string());
    format!("gui/{uid}")
}

fn escape_plist_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn daemon_service_io_error(error: std::io::Error) -> CommandError {
    CommandError {
        code: "AutomationDaemonIo".to_string(),
        message: error.to_string(),
        recoverable: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_agent_plist_uses_runtime_and_registry_paths() {
        let plist = launch_agent_plist(
            Path::new("/tmp/imglab-daemon"),
            Path::new("/tmp/imglab runtime"),
            Path::new("/tmp/imglab registry.sqlite"),
        );

        assert!(plist.contains("com.imagepromptlab.daemon"));
        assert!(plist.contains("/tmp/imglab-daemon"));
        assert!(plist.contains("/tmp/imglab runtime"));
        assert!(plist.contains("/tmp/imglab registry.sqlite"));
        assert!(plist.contains("IMGLAB_REGISTRY"));
        assert!(plist.contains("KeepAlive"));
    }
}
