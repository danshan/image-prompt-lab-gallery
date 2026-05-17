use crate::CommandError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const TITLE_MAX_CHARS: usize = 120;
const DESCRIPTION_MAX_CHARS: usize = 800;
const CODEX_METADATA_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ReviewField {
    Title,
    Description,
    SchemaPrompt,
}

impl ReviewField {
    fn as_str(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Description => "description",
            Self::SchemaPrompt => "schemaPrompt",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFieldContext {
    pub current_title: Option<String>,
    pub current_description: Option<String>,
    pub current_schema_prompt: Option<String>,
    pub asset_title: Option<String>,
    pub asset_prompt: Option<String>,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub provider: Option<String>,
    pub model_label: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateReviewFieldInput {
    pub library_path: PathBuf,
    pub asset_id: String,
    pub suggestion_id: String,
    pub field: ReviewField,
    pub context: ReviewFieldContext,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedReviewFieldView {
    pub field: ReviewField,
    pub value: String,
    pub log_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CodexCliMetadataGenerator {
    codex_bin: PathBuf,
    work_dir: PathBuf,
    log_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct CodexMetadataCommand {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub prompt: String,
}

impl CodexCliMetadataGenerator {
    pub fn new(codex_bin: impl Into<PathBuf>, work_dir: impl Into<PathBuf>) -> Self {
        Self {
            codex_bin: codex_bin.into(),
            work_dir: work_dir.into(),
            log_path: None,
        }
    }

    pub fn build_command(&self, input: &GenerateReviewFieldInput) -> CodexMetadataCommand {
        let prompt = metadata_prompt(input);
        CodexMetadataCommand {
            program: self.codex_bin.clone(),
            args: vec![
                "exec".to_string(),
                "--cd".to_string(),
                self.work_dir.display().to_string(),
                "--sandbox".to_string(),
                "read-only".to_string(),
                "--skip-git-repo-check".to_string(),
                "--json".to_string(),
                prompt.clone(),
            ],
            prompt,
        }
    }

    pub fn generate(
        &self,
        input: &GenerateReviewFieldInput,
    ) -> Result<GeneratedReviewFieldView, CommandError> {
        let command = self.build_command(input);
        let log_path = self.log_path();
        let mut log_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&log_path)
            .map_err(|error| io_command_error(&log_path, error))?;
        writeln!(
            log_file,
            "program: {}\nargs: {}\nfield: {}\nprompt:\n{}\n",
            command.program.display(),
            command.args.join(" "),
            input.field.as_str(),
            command.prompt
        )
        .map_err(|error| io_command_error(&log_path, error))?;

        let mut child = Command::new(&command.program)
            .args(&command.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| generation_command_error(input.field, &log_path, error.to_string()))?;

        let stdout_handle = child
            .stdout
            .take()
            .map(|stdout| stream_to_log(stdout, log_path.clone(), "[stdout] ".to_string()));
        let stderr_handle = child
            .stderr
            .take()
            .map(|stderr| stream_to_log(stderr, log_path.clone(), "[stderr] ".to_string()));
        let status = wait_with_timeout(&mut child, input.field, &log_path)?;
        let stdout = join_stream(stdout_handle, &log_path, input.field)?;
        let stderr = join_stream(stderr_handle, &log_path, input.field)?;
        let combined_output = format!("{stdout}\n{stderr}");

        if !status.success() {
            return Err(generation_command_error(
                input.field,
                &log_path,
                format!("codex CLI exited with {status}"),
            ));
        }

        let final_output = extract_final_text(&combined_output);
        let value = parse_field_value(input.field, &final_output)
            .map_err(|message| generation_command_error(input.field, &log_path, message))?;

        Ok(GeneratedReviewFieldView {
            field: input.field,
            value,
            log_path,
        })
    }

    fn log_path(&self) -> PathBuf {
        if let Some(path) = &self.log_path {
            return path.clone();
        }
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        std::env::temp_dir().join(format!(
            "imglab-codex-metadata-{}-{nanos}.log",
            std::process::id()
        ))
    }
}

fn wait_with_timeout(
    child: &mut std::process::Child,
    field: ReviewField,
    log_path: &Path,
) -> Result<std::process::ExitStatus, CommandError> {
    let started = Instant::now();
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| generation_command_error(field, log_path, error.to_string()))?
        {
            return Ok(status);
        }
        if started.elapsed() >= CODEX_METADATA_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return Err(generation_command_error(
                field,
                log_path,
                format!(
                    "codex CLI timed out after {} seconds",
                    CODEX_METADATA_TIMEOUT.as_secs()
                ),
            ));
        }
        thread::sleep(Duration::from_millis(100));
    }
}

pub fn parse_field_value(field: ReviewField, output: &str) -> Result<String, String> {
    match field {
        ReviewField::Title => parse_text_field(field, output, TITLE_MAX_CHARS),
        ReviewField::Description => parse_text_field(field, output, DESCRIPTION_MAX_CHARS),
        ReviewField::SchemaPrompt => parse_schema_prompt(output),
    }
}

fn parse_text_field(field: ReviewField, output: &str, max_chars: usize) -> Result<String, String> {
    let value = extract_value_from_json(output, field).unwrap_or_else(|| {
        strip_markdown_fence(output)
            .trim()
            .trim_matches('"')
            .to_string()
    });
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(format!(
            "{} generation returned empty output",
            field.as_str()
        ));
    }
    if normalized.chars().count() > max_chars {
        return Err(format!(
            "{} generation returned too much text",
            field.as_str()
        ));
    }
    Ok(normalized.to_string())
}

fn parse_schema_prompt(output: &str) -> Result<String, String> {
    let source = extract_value_from_json(output, ReviewField::SchemaPrompt)
        .unwrap_or_else(|| output.to_string());
    let json = extract_first_json_object(&source)
        .ok_or_else(|| "schemaPrompt generation did not include a JSON object".to_string())?;
    let value: Value = serde_json::from_str(&json)
        .map_err(|error| format!("schemaPrompt generation returned invalid JSON: {error}"))?;
    if !value.is_object() {
        return Err("schemaPrompt generation must return a JSON object".to_string());
    }
    serde_json::to_string_pretty(&value)
        .map_err(|error| format!("failed to format schemaPrompt JSON: {error}"))
}

fn extract_value_from_json(output: &str, field: ReviewField) -> Option<String> {
    let value: Value = serde_json::from_str(output.trim()).ok()?;
    for key in ["value", field.as_str(), "text", "content"] {
        if let Some(text) = value.get(key).and_then(Value::as_str) {
            return Some(text.to_string());
        }
    }
    None
}

fn extract_final_text(output: &str) -> String {
    let mut last_candidate = None;
    for line in output.lines() {
        let trimmed = line.trim().strip_prefix("[stdout] ").unwrap_or(line.trim());
        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };
        if let Some(text) = extract_codex_agent_message(&value) {
            last_candidate = Some(text);
        }
    }
    last_candidate.unwrap_or_else(|| {
        output
            .lines()
            .map(|line| line.trim().strip_prefix("[stdout] ").unwrap_or(line.trim()))
            .filter(|line| !line.is_empty())
            .last()
            .unwrap_or(output)
            .to_string()
    })
}

fn extract_codex_agent_message(value: &Value) -> Option<String> {
    let event = value.as_object()?;
    if event.get("type").and_then(Value::as_str) != Some("item.completed") {
        return None;
    }
    let item = event.get("item")?.as_object()?;
    if item.get("type").and_then(Value::as_str) != Some("agent_message") {
        return None;
    }
    item.get("text")
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn strip_markdown_fence(output: &str) -> String {
    let trimmed = output.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }
    let mut lines = trimmed.lines();
    lines.next();
    let body: Vec<_> = lines
        .take_while(|line| !line.trim_start().starts_with("```"))
        .collect();
    body.join("\n")
}

fn extract_first_json_object(output: &str) -> Option<String> {
    let bytes = output.as_bytes();
    let mut start = None;
    let mut depth = 0_i32;
    let mut in_string = false;
    let mut escaped = false;

    for (index, byte) in bytes.iter().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
            continue;
        }

        match *byte {
            b'"' => in_string = true,
            b'{' => {
                if depth == 0 {
                    start = Some(index);
                }
                depth += 1;
            }
            b'}' => {
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 {
                        let start = start?;
                        return output.get(start..=index).map(ToString::to_string);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn metadata_prompt(input: &GenerateReviewFieldInput) -> String {
    let field_instruction = match input.field {
        ReviewField::Title => {
            "Return exactly one concise Simplified Chinese title. Do not include explanations or Markdown."
        }
        ReviewField::Description => {
            "Return one or two Simplified Chinese sentences for album browsing and search. Do not include Markdown."
        }
        ReviewField::SchemaPrompt => {
            "Return exactly one valid JSON object, not Markdown. Include GLOBAL_SETTINGS, ENVIRONMENT, CORE_ASSETS, and OUTPUT keys."
        }
    };
    let context_json = serde_json::json!({
        "assetId": input.asset_id,
        "suggestionId": input.suggestion_id,
        "field": input.field.as_str(),
        "currentTitle": input.context.current_title,
        "currentDescription": input.context.current_description,
        "currentSchemaPrompt": input.context.current_schema_prompt,
        "assetTitle": input.context.asset_title,
        "assetPrompt": input.context.asset_prompt,
        "tags": input.context.tags,
        "category": input.context.category,
        "provider": input.context.provider,
        "modelLabel": input.context.model_label,
        "width": input.context.width,
        "height": input.context.height
    });
    format!(
        r#"Generate one Image Prompt Lab review metadata field.

Field: {field}
Instruction: {field_instruction}

Context JSON:
{context_json}

Output protocol:
- For title and description, return only the requested field text.
- For schemaPrompt, return only a valid JSON object.
- Do not modify files.
- Do not generate an image."#,
        field = input.field.as_str()
    )
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
    field: ReviewField,
) -> Result<String, CommandError> {
    let Some(handle) = handle else {
        return Ok(String::new());
    };
    let bytes = handle.join().map_err(|_| {
        generation_command_error(
            field,
            log_path,
            "failed to collect codex CLI output".to_string(),
        )
    })?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

fn io_command_error(path: &Path, error: std::io::Error) -> CommandError {
    CommandError {
        code: "Io".to_string(),
        message: format!("{}: {error}", path.display()),
        recoverable: true,
    }
}

fn generation_command_error(field: ReviewField, log_path: &Path, message: String) -> CommandError {
    CommandError {
        code: "MetadataGenerationFailed".to_string(),
        message: format!(
            "failed to generate {} with Codex CLI: {}; log: {}",
            field.as_str(),
            message,
            log_path.display()
        ),
        recoverable: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(field: ReviewField) -> GenerateReviewFieldInput {
        GenerateReviewFieldInput {
            library_path: PathBuf::from("/tmp/library"),
            asset_id: "asset-1".to_string(),
            suggestion_id: "suggestion-1".to_string(),
            field,
            context: ReviewFieldContext {
                current_title: None,
                current_description: None,
                current_schema_prompt: None,
                asset_title: Some("Neon Botanical Study".to_string()),
                asset_prompt: Some("botanical neon study".to_string()),
                tags: vec!["botanical".to_string()],
                category: Some("study".to_string()),
                provider: Some("codex-cli".to_string()),
                model_label: None,
                width: Some(1024),
                height: Some(1024),
            },
        }
    }

    #[test]
    fn builds_codex_metadata_command_without_imagegen_prompt() {
        let generator = CodexCliMetadataGenerator::new("codex", "/tmp/library");
        let command = generator.build_command(&input(ReviewField::Title));

        assert_eq!(command.program, PathBuf::from("codex"));
        assert!(command.args.contains(&"exec".to_string()));
        assert!(command.args.contains(&"--json".to_string()));
        assert!(command.args.contains(&"read-only".to_string()));
        assert!(command.args.contains(&"--skip-git-repo-check".to_string()));
        assert!(command.prompt.contains("Field: title"));
        assert!(command.prompt.contains("Simplified Chinese"));
        assert!(!command.prompt.contains("Use the imagegen skill"));
    }

    #[test]
    fn parses_text_field_from_plain_output() {
        let value = parse_field_value(ReviewField::Title, "霓虹植物研究").expect("title");
        assert_eq!(value, "霓虹植物研究");
    }

    #[test]
    fn rejects_empty_text_field() {
        let error = parse_field_value(ReviewField::Description, "   ").expect_err("empty");
        assert!(error.contains("empty"));
    }

    #[test]
    fn extracts_and_formats_schema_json() {
        let output = r#"Here is the JSON:
{"GLOBAL_SETTINGS":{"aspect_ratio":"1:1"},"ENVIRONMENT":{},"CORE_ASSETS":{},"OUTPUT":{}}
"#;
        let value = parse_field_value(ReviewField::SchemaPrompt, output).expect("schema");
        assert!(value.contains("\"GLOBAL_SETTINGS\""));
        assert!(serde_json::from_str::<Value>(&value)
            .expect("valid json")
            .is_object());
    }

    #[test]
    fn extracts_agent_message_from_codex_json_events() {
        let output = r#"[stdout] {"type":"thread.started","thread_id":"thread-1"}
[stdout] {"type":"turn.started"}
[stdout] {"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"霓虹电影海报"}}
[stdout] {"type":"turn.completed","usage":{"input_tokens":1,"output_tokens":1}}"#;

        let value = extract_final_text(output);

        assert_eq!(value, "霓虹电影海报");
    }

    #[test]
    fn ignores_turn_completed_event_type_when_extracting_output() {
        let output =
            r#"[stdout] {"type":"turn.completed","usage":{"input_tokens":1,"output_tokens":1}}"#;

        let value = extract_final_text(output);

        assert_ne!(value, "turn.completed");
    }

    #[test]
    fn rejects_invalid_schema_json() {
        let error = parse_field_value(ReviewField::SchemaPrompt, "not json").expect_err("invalid");
        assert!(error.contains("JSON object"));
    }
}
