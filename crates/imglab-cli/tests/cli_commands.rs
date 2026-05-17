use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TestWorkspace {
    root: PathBuf,
    registry: PathBuf,
}

impl TestWorkspace {
    fn new(name: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("imglab-cli-{name}-{nanos}"));
        fs::create_dir_all(&root).expect("create workspace");
        let registry = root.join("registry.sqlite");
        Self { root, registry }
    }

    fn library_path(&self) -> PathBuf {
        self.root.join("library")
    }

    fn source_image(&self) -> PathBuf {
        let path = self.root.join("source.png");
        fs::write(&path, b"png bytes").expect("write source image");
        path
    }

    fn run(&self, args: &[&str]) -> CommandOutput {
        let output = Command::new(env!("CARGO_BIN_EXE_imglab"))
            .args(args)
            .env("IMGLAB_REGISTRY", &self.registry)
            .output()
            .expect("run imglab");
        CommandOutput {
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8(output.stdout).expect("stdout is utf8"),
            stderr: String::from_utf8(output.stderr).expect("stderr is utf8"),
        }
    }
}

struct CommandOutput {
    status: i32,
    stdout: String,
    stderr: String,
}

impl CommandOutput {
    fn assert_success(&self) -> Value {
        assert_eq!(self.status, 0, "stderr: {}", self.stderr);
        serde_json::from_str(&self.stdout).expect("stdout json")
    }

    fn assert_error(&self, expected_status: i32) -> Value {
        assert_eq!(self.status, expected_status, "stdout: {}", self.stdout);
        serde_json::from_str(&self.stderr).expect("stderr json")
    }
}

#[test]
fn init_import_search_and_rate_emit_json() {
    let workspace = TestWorkspace::new("main-flow");
    let library = workspace.library_path();
    let source = workspace.source_image();

    let init = workspace
        .run(&["init", path(&library), "--name", "Test Library", "--json"])
        .assert_success();
    assert_eq!(init["name"], "Test Library");

    let imported = workspace
        .run(&[
            "import",
            "--library",
            path(&library),
            path(&source),
            "--json",
        ])
        .assert_success();
    let asset_id = imported["asset_id"].as_str().expect("asset id");
    assert_eq!(imported["checksum_algorithm"], "SHA-256");
    assert!(imported["checksum"].as_str().is_some());

    let search = workspace
        .run(&["search", "--library", path(&library), "--json"])
        .assert_success();
    assert_eq!(search["assets"].as_array().expect("assets").len(), 1);

    let rated = workspace
        .run(&["rate", "--library", path(&library), asset_id, "5", "--json"])
        .assert_success();
    assert_eq!(rated["rating"], 5);

    let repair = workspace
        .run(&["library", "repair", "--library", path(&library), "--json"])
        .assert_success();
    assert_eq!(repair["dry_run"], true);
    assert_eq!(repair["scanned_versions"], 1);
    assert!(repair["issues"].as_array().expect("issues").is_empty());
}

#[test]
fn import_dry_run_does_not_copy_file() {
    let workspace = TestWorkspace::new("dry-run");
    let library = workspace.library_path();
    let source = workspace.source_image();

    workspace
        .run(&["init", path(&library), "--name", "Dry Run"])
        .assert_success();
    let preview = workspace
        .run(&[
            "import",
            "--library",
            path(&library),
            path(&source),
            "--dry-run",
            "--json",
        ])
        .assert_success();

    assert_eq!(preview["dry_run"], true);
    assert!(!library.join("originals").join("imported").exists());
}

#[test]
fn fake_generate_tag_album_and_suggestion_flow_work() {
    let workspace = TestWorkspace::new("workflow");
    let library = workspace.library_path();

    workspace
        .run(&["init", path(&library), "--name", "Workflow"])
        .assert_success();
    let generated = workspace
        .run(&[
            "generate",
            "--library",
            path(&library),
            "--provider",
            "fake",
            "--prompt",
            "make a test image",
            "--json",
        ])
        .assert_success();
    let version = &generated["versions"].as_array().expect("versions")[0];
    let asset_id = version["asset_id"].as_str().expect("asset id");

    workspace
        .run(&[
            "tag",
            "add",
            "--library",
            path(&library),
            asset_id,
            "favorite",
        ])
        .assert_success();
    let album = workspace
        .run(&["album", "create", "--library", path(&library), "Favorites"])
        .assert_success();
    let album_id = album["id"].as_str().expect("album id");
    workspace
        .run(&["album", "add", album_id, asset_id])
        .assert_success();

    let suggestion = workspace
        .run(&[
            "suggestion",
            "create",
            "--library",
            path(&library),
            asset_id,
            "--title",
            "Suggested",
            "--tag",
            "reviewed",
        ])
        .assert_success();
    let suggestion_id = suggestion["id"].as_str().expect("suggestion id");

    let pending = workspace
        .run(&["suggestion", "list", "--library", path(&library)])
        .assert_success();
    assert_eq!(
        pending["suggestions"]
            .as_array()
            .expect("suggestions")
            .len(),
        1
    );

    let accepted = workspace
        .run(&[
            "suggestion",
            "accept",
            "--library",
            path(&library),
            suggestion_id,
            "--title",
            "Suggested",
            "--tag",
            "reviewed",
        ])
        .assert_success();
    assert_eq!(accepted["title"], "Suggested");
}

#[test]
fn missing_library_returns_stable_json_error() {
    let workspace = TestWorkspace::new("error");
    let missing = workspace.root.join("missing");

    let error = workspace
        .run(&["search", "--library", path(&missing), "--json"])
        .assert_error(3);

    assert_eq!(error["code"], "LibraryNotFound");
    assert_eq!(error["recoverable"], true);
}

fn path(path: &Path) -> &str {
    path.to_str().expect("utf8 path")
}
