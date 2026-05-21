use imglab_core::task_scheduler::{RetryPolicy, TaskSchedulerConfig};
use imglab_daemon::{
    bind_loopback_listener, generate_session_token, recover_open_libraries, serve_forever_shared,
    spawn_scheduler_loop, write_runtime_file, write_token_file, DaemonConfig, DaemonState,
    RuntimeFile, SharedDaemonState, API_VERSION, DEFAULT_SCHEDULER_INTERVAL,
};
use std::env;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let runtime_dir = env::var_os("IMGLAB_DAEMON_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("imglab-daemon"));
    let token = generate_session_token();
    let config = DaemonConfig {
        bind_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)),
        runtime_path: runtime_dir.join("runtime.json"),
        token_path: runtime_dir.join("token"),
        token,
    };
    let listener = bind_loopback_listener(config.bind_addr)?;
    let port = listener.local_addr()?.port();
    write_token_file(&config.token_path, &config.token)?;
    write_runtime_file(
        &config.runtime_path,
        &RuntimeFile {
            api_version: API_VERSION.to_string(),
            pid: std::process::id(),
            port,
            token_path: PathBuf::from(&config.token_path),
        },
    )?;
    println!(
        "imglab-daemon listening on 127.0.0.1:{port}; runtime={}",
        config.runtime_path.display()
    );
    let state: SharedDaemonState = Arc::new(Mutex::new(DaemonState::new(
        runtime_dir.join("registry.sqlite"),
        runtime_dir.join("task-logs"),
    )));
    {
        let mut guard = state.lock().map_err(|_| "daemon state lock poisoned")?;
        recover_open_libraries(&mut guard, &RetryPolicy::default())?;
    }
    let _scheduler = spawn_scheduler_loop(
        Arc::clone(&state),
        TaskSchedulerConfig::default(),
        RetryPolicy::default(),
        DEFAULT_SCHEDULER_INTERVAL,
    );
    serve_forever_shared(&listener, &config.token, state)?;
    Ok(())
}
