use imglab_core::{
    DomainError, DomainResult, GeneratedImage, GenerationParameters, GenerationResult,
    ImageProvider,
};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct CodexCliImageProvider {
    codex_bin: PathBuf,
    work_dir: PathBuf,
    log_path: Option<PathBuf>,
    cancel_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct CodexCliCommand {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub prompt: String,
}

impl CodexCliImageProvider {
    pub fn new(codex_bin: impl Into<PathBuf>, work_dir: impl Into<PathBuf>) -> Self {
        Self {
            codex_bin: codex_bin.into(),
            work_dir: work_dir.into(),
            log_path: None,
            cancel_path: None,
        }
    }

    pub fn with_log_path(mut self, log_path: impl Into<PathBuf>) -> Self {
        self.log_path = Some(log_path.into());
        self
    }

    pub fn with_cancel_path(mut self, cancel_path: impl Into<PathBuf>) -> Self {
        self.cancel_path = Some(cancel_path.into());
        self
    }

    pub fn build_command(
        &self,
        parameters: &GenerationParameters,
    ) -> DomainResult<CodexCliCommand> {
        self.validate_parameters(parameters)?;
        let prompt = codex_prompt(parameters);
        let args = vec![
            "exec".to_string(),
            "--cd".to_string(),
            self.work_dir.display().to_string(),
            "--sandbox".to_string(),
            "workspace-write".to_string(),
            "--skip-git-repo-check".to_string(),
            "--json".to_string(),
            prompt.clone(),
        ];

        Ok(CodexCliCommand {
            program: self.codex_bin.clone(),
            args,
            prompt,
        })
    }

    pub fn parse_output(
        &self,
        output: &str,
        raw_request_json: String,
        raw_response_json: String,
    ) -> DomainResult<GenerationResult> {
        let files = parse_final_generated_image_paths(output);
        if files.is_empty() {
            return Err(DomainError::GenerationFailed {
                provider: self.name().to_string(),
                message: "codex output did not include a generated image path".to_string(),
            });
        }

        let mut images = Vec::with_capacity(files.len());
        for file in files {
            if !file.is_file() {
                return Err(DomainError::GenerationFailed {
                    provider: self.name().to_string(),
                    message: format!("codex output file does not exist: {}", file.display()),
                });
            }

            let bytes = std::fs::read(&file).map_err(|error| DomainError::Io {
                path: file.display().to_string(),
                message: error.to_string(),
            })?;
            images.push(GeneratedImage {
                bytes,
                mime_type: mime_type_for_path(&file).to_string(),
                provider_metadata_json: serde_json::json!({
                    "file": file,
                    "source": "codex-cli-imagegen"
                })
                .to_string(),
            });
        }

        Ok(GenerationResult {
            images,
            raw_request_json,
            raw_response_json,
        })
    }

    pub fn generate_from_text_until_cancelled<F>(
        &self,
        parameters: &GenerationParameters,
        should_cancel: F,
    ) -> DomainResult<GenerationResult>
    where
        F: Fn() -> bool,
    {
        self.validate_parameters(parameters)?;
        let command = self.build_command(parameters)?;
        let log_path = self.log_path();
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| DomainError::Io {
                path: parent.display().to_string(),
                message: error.to_string(),
            })?;
        }
        let mut log_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&log_path)
            .map_err(|error| DomainError::Io {
                path: log_path.display().to_string(),
                message: error.to_string(),
            })?;
        writeln!(
            log_file,
            "program: {}\nargs: {}\n",
            command.program.display(),
            command.args.join(" ")
        )
        .map_err(|error| DomainError::Io {
            path: log_path.display().to_string(),
            message: error.to_string(),
        })?;

        let program = resolve_codex_program(&command.program).ok_or_else(|| {
            DomainError::GenerationFailed {
                provider: self.name().to_string(),
                message: format!(
                    "codex CLI executable not found. Set CODEX_CLI_BIN to the absolute codex path, or ensure codex is available in PATH. Requested: {}; log: {}",
                    command.program.display(),
                    log_path.display()
                ),
            }
        })?;
        writeln!(log_file, "resolved_program: {}\n", program.display()).map_err(|error| {
            DomainError::Io {
                path: log_path.display().to_string(),
                message: error.to_string(),
            }
        })?;

        let mut child = Command::new(&program)
            .args(&command.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| DomainError::GenerationFailed {
                provider: self.name().to_string(),
                message: format!(
                    "failed to run codex CLI: {error}; log: {}",
                    log_path.display()
                ),
            })?;

        let stdout_handle = child
            .stdout
            .take()
            .map(|stdout| stream_to_log(stdout, log_path.clone(), "[stdout] ".to_string()));
        let stderr_handle = child
            .stderr
            .take()
            .map(|stderr| stream_to_log(stderr, log_path.clone(), "[stderr] ".to_string()));
        let status = loop {
            if should_cancel() {
                let _ = child.kill();
                let _ = child.wait();
                let _ = join_stream(stdout_handle, &log_path, self.name());
                let _ = join_stream(stderr_handle, &log_path, self.name());
                return Err(DomainError::GenerationFailed {
                    provider: self.name().to_string(),
                    message: format!("codex CLI canceled by user; log: {}", log_path.display()),
                });
            }
            match child
                .try_wait()
                .map_err(|error| DomainError::GenerationFailed {
                    provider: self.name().to_string(),
                    message: format!(
                        "failed to poll codex CLI: {error}; log: {}",
                        log_path.display()
                    ),
                })? {
                Some(status) => break status,
                None => thread::sleep(Duration::from_millis(100)),
            }
        };
        let stdout = join_stream(stdout_handle, &log_path, self.name())?;
        let stderr = join_stream(stderr_handle, &log_path, self.name())?;
        let combined_output = format!("{stdout}\n{stderr}");

        if !status.success() {
            return Err(DomainError::GenerationFailed {
                provider: self.name().to_string(),
                message: format!(
                    "codex CLI exited with {status}; log: {}; output: {combined_output}",
                    log_path.display()
                ),
            });
        }

        let raw_request_json = serde_json::json!({
            "program": program,
            "args": command.args,
            "prompt": command.prompt,
            "log_path": log_path
        })
        .to_string();
        self.parse_output(&combined_output, raw_request_json, combined_output.clone())
    }

    fn log_path(&self) -> PathBuf {
        if let Some(path) = &self.log_path {
            return path.clone();
        }

        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or_default();
        std::env::temp_dir()
            .join("imglab-codex-logs")
            .join(format!("imglab-codex-cli-{millis}.log"))
    }
}

impl Default for CodexCliImageProvider {
    fn default() -> Self {
        Self::new("codex", ".")
    }
}

impl ImageProvider for CodexCliImageProvider {
    fn name(&self) -> &'static str {
        "codex-cli"
    }

    fn validate_parameters(&self, parameters: &GenerationParameters) -> DomainResult<()> {
        if parameters.prompt.trim().is_empty() {
            return Err(DomainError::InvalidGenerationParameters {
                message: "prompt must not be empty".to_string(),
            });
        }

        Ok(())
    }

    fn generate_from_text(
        &self,
        parameters: &GenerationParameters,
    ) -> DomainResult<GenerationResult> {
        let cancel_path = self.cancel_path.clone();
        self.generate_from_text_until_cancelled(parameters, move || {
            cancel_path.as_ref().is_some_and(|path| path.exists())
        })
    }

    fn generate_from_image(
        &self,
        parameters: &GenerationParameters,
        input: &[u8],
    ) -> DomainResult<GenerationResult> {
        if input.is_empty() {
            return Err(DomainError::InvalidGenerationParameters {
                message: "input image must not be empty".to_string(),
            });
        }

        self.generate_from_text(parameters)
    }
}

fn stream_to_log<R>(mut reader: R, log_path: PathBuf, prefix: String) -> thread::JoinHandle<Vec<u8>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut output = Vec::new();
        let mut buffer = [0_u8; 8192];
        let mut log_file = File::options().append(true).open(&log_path).ok();
        loop {
            let read = match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => read,
                Err(_) => break,
            };
            output.extend_from_slice(&buffer[..read]);
            if let Some(file) = log_file.as_mut() {
                let _ = file.write_all(prefix.as_bytes());
                let _ = file.write_all(&buffer[..read]);
                let _ = file.flush();
            }
        }
        output
    })
}

fn join_stream(
    handle: Option<thread::JoinHandle<Vec<u8>>>,
    log_path: &Path,
    provider: &str,
) -> DomainResult<String> {
    let Some(handle) = handle else {
        return Ok(String::new());
    };
    let bytes = handle.join().map_err(|_| DomainError::GenerationFailed {
        provider: provider.to_string(),
        message: format!(
            "failed to collect codex CLI output; log: {}",
            log_path.display()
        ),
    })?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

fn resolve_codex_program(requested: &Path) -> Option<PathBuf> {
    if let Some(configured) = std::env::var_os("CODEX_CLI_BIN").map(PathBuf::from) {
        if configured.is_file() {
            return Some(configured);
        }
    }
    if has_path_component(requested) {
        return requested.is_file().then(|| requested.to_path_buf());
    }
    search_executable(requested.to_str().unwrap_or("codex"))
}

fn has_path_component(path: &Path) -> bool {
    path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
        || path.parent().is_some_and(|parent| parent != Path::new(""))
}

fn search_executable(binary_name: &str) -> Option<PathBuf> {
    executable_search_dirs()
        .into_iter()
        .map(|directory| directory.join(binary_name))
        .find(|candidate| candidate.is_file())
}

fn executable_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(path) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&path));
    }
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        dirs.push(home.join(".asdf/shims"));
        dirs.push(home.join(".cargo/bin"));
        dirs.push(home.join(".local/bin"));
    }
    dirs.extend([
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
    ]);
    dedupe_paths(dirs)
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut unique = Vec::new();
    for path in paths {
        if !unique.iter().any(|item| item == &path) {
            unique.push(path);
        }
    }
    unique
}

fn codex_prompt(parameters: &GenerationParameters) -> String {
    format!(
        r#"Use the imagegen skill to generate an image.

Use case: ai-agent-image-prompt-lab
Asset type: managed library image
Primary request: {prompt}
Input images: {input_images}
Scene/backdrop: infer from primary request
Subject: infer from primary request

After generation, locate the newest generated image under $CODEX_HOME/generated_images or $HOME/.codex/generated_images, copy it to a stable temporary path if needed, and include the final absolute image path in your final response."#,
        prompt = parameters.prompt,
        input_images = parameters
            .input_version_id
            .as_ref()
            .map(|id| format!("Image 1: source asset version {}", id.0))
            .unwrap_or_else(|| "none".to_string())
    )
}

fn parse_generated_image_paths(output: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for token in output.split(|character: char| {
        character.is_whitespace() || matches!(character, '`' | '"' | '\'' | ',' | '[' | ']')
    }) {
        let cleaned = token.trim_end_matches(['.', ':', ';', ')']);
        if is_supported_image_path(cleaned) {
            let path = PathBuf::from(cleaned);
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
    }
    paths
}

fn parse_final_generated_image_paths(output: &str) -> Vec<PathBuf> {
    for marker in ["已生成并复制到", "Generated and copied to", "Saved to"] {
        if let Some((_, tail)) = output.rsplit_once(marker) {
            let paths = parse_generated_image_paths(tail);
            if let Some(path) = paths.first() {
                return vec![path.clone()];
            }
        }
    }

    parse_generated_image_paths(output)
        .into_iter()
        .last()
        .into_iter()
        .collect()
}

fn is_supported_image_path(value: &str) -> bool {
    value.starts_with('/')
        && matches!(
            Path::new(value)
                .extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| extension.to_ascii_lowercase())
                .as_deref(),
            Some("png" | "jpg" | "jpeg" | "webp" | "gif" | "avif")
        )
}

fn mime_type_for_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("avif") => "image/avif",
        Some("png") => "image/png",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use imglab_core::GenerationOperation;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn parameters(prompt: &str) -> GenerationParameters {
        GenerationParameters {
            library_path: None,
            provider: "codex-cli".to_string(),
            model: "imagegen".to_string(),
            prompt: prompt.to_string(),
            negative_prompt: None,
            operation: GenerationOperation::TextToImage,
            input_version_id: None,
            parameters_json: "{}".to_string(),
        }
    }

    #[test]
    fn builds_codex_exec_command_without_model_or_output_path() {
        let provider = CodexCliImageProvider::new("codex", "/tmp/work");
        let command = provider
            .build_command(&parameters("make a small icon"))
            .expect("command");

        assert_eq!(command.program, PathBuf::from("codex"));
        assert!(command.args.contains(&"exec".to_string()));
        assert!(command.args.contains(&"--skip-git-repo-check".to_string()));
        assert!(!command.args.contains(&"--model".to_string()));
        assert!(!command.args.contains(&"--output-last-message".to_string()));
        assert!(command.prompt.contains("Use the imagegen skill"));
    }

    #[test]
    fn resolves_codex_from_configured_or_common_paths() {
        let resolved = resolve_codex_program(Path::new("codex"));
        if let Some(path) = resolved {
            assert!(path.ends_with("codex"));
        }
    }

    #[test]
    fn parses_codex_output_and_reads_image_path() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("codex-provider-{nanos}"));
        fs::create_dir_all(&root).expect("create root");
        let image_path = root.join("image.png");
        fs::write(&image_path, b"png bytes").expect("write image");
        let output = format!(
            "已生成并复制到:\n\n`{}`\n\n原始生成文件保留在 $HOME/.codex/generated_images/...",
            image_path.display()
        );

        let provider = CodexCliImageProvider::default();
        let result = provider
            .parse_output(&output, "{}".to_string(), output.clone())
            .expect("parse");

        assert_eq!(result.images.len(), 1);
        assert_eq!(result.images[0].bytes, b"png bytes");
        assert_eq!(result.images[0].mime_type, "image/png");
    }

    #[test]
    fn prefers_final_copied_path_over_generated_images_scan() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("codex-provider-final-{nanos}"));
        fs::create_dir_all(&root).expect("create root");
        let generated_path = root.join("generated_images").join("ig_raw.png");
        fs::create_dir_all(generated_path.parent().expect("parent")).expect("create generated");
        fs::write(&generated_path, b"raw bytes").expect("write raw image");
        let final_path = root.join("final.png");
        fs::write(&final_path, b"final bytes").expect("write final image");
        let output = format!(
            r#"exec
/bin/zsh -lc 'find "${{CODEX_HOME:-$HOME/.codex}}" -path '"'*generated_images*' -type f -mmin -10 | sort | tail -20" succeeded:
{generated}

exec
/bin/zsh -lc 'cp {generated} {final} && ls -lh {final}' succeeded:
-rw-r--r--  1 user  staff  2.4M May 17 00:29 {final}

codex
已生成并复制到:

`{final}`

原始生成文件保留在 `$HOME/.codex/generated_images/...` 下。
"#,
            generated = generated_path.display(),
            final = final_path.display()
        );

        let provider = CodexCliImageProvider::default();
        let result = provider
            .parse_output(&output, "{}".to_string(), output.clone())
            .expect("parse");

        assert_eq!(result.images.len(), 1);
        assert_eq!(result.images[0].bytes, b"final bytes");
    }

    #[test]
    fn rejects_output_without_image_path() {
        let provider = CodexCliImageProvider::default();
        let error = provider
            .parse_output(
                "no image here",
                "{}".to_string(),
                "no image here".to_string(),
            )
            .expect_err("missing path should fail");

        assert!(matches!(error, DomainError::GenerationFailed { .. }));
    }
}
