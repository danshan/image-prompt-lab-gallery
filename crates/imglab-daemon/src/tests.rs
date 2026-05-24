use super::*;
use imglab_core::interface_contracts::dto::{
    AppendTaskAttemptRequest, CreateLibraryRequest, CreatePromptDocumentRequest, CreateTaskInput,
    SavePromptVersionRequest,
};
use std::net::{Ipv4Addr, SocketAddrV4};

fn test_root(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("imglab-daemon-{name}-{}", generate_session_token()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove test root");
    }
    root
}

fn test_state(name: &str) -> DaemonState {
    let root = test_root(name);
    fs::create_dir_all(root.join("logs")).expect("create log root");
    DaemonState::new(root.join("registry.sqlite"), root.join("logs"))
}

fn json_request(method: &str, path: &str, body: Value) -> String {
    format!(
        "{method} {path} HTTP/1.1\r\nAuthorization: Bearer secret\r\nContent-Type: application/json\r\n\r\n{}",
        serde_json::to_string(&body).expect("serialize body")
    )
}

fn auth_get(path: &str) -> String {
    format!("GET {path} HTTP/1.1\r\nAuthorization: Bearer secret\r\n\r\n")
}

fn json_value(response: &HttpResponse) -> Value {
    serde_json::from_str(&response.body).expect("parse response body")
}

fn create_open_library(state: &mut DaemonState, name: &str) -> String {
    let library_root = test_root(name).join("library");
    let library = state
        .library_lifecycle()
        .create_library(CreateLibraryRequest {
            root_path: library_root.clone(),
            name: name.to_string(),
        })
        .expect("create library");
    let request = json_request(
        "POST",
        LIBRARY_OPEN_PATH,
        serde_json::json!({ "libraryPath": library_root }),
    );
    let response = handle_http_request_with_state(&request, "secret", state);
    assert_eq!(response.status_code, 200);
    library.id.0
}

mod api;
mod recovery;
mod scheduler;
