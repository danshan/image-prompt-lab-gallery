pub fn handle_http_request(request: &str, token: &str) -> HttpResponse {
    let registry_path = std::env::temp_dir().join("imglab-daemon-stateless-registry.sqlite");
    let log_root = std::env::temp_dir().join("imglab-daemon-logs");
    let mut state = DaemonState::new(registry_path, log_root);
    handle_http_request_with_state(request, token, &mut state)
}

pub fn handle_http_request_with_state(
    request: &str,
    token: &str,
    state: &mut DaemonState,
) -> HttpResponse {
    let Some(request_line) = request.lines().next() else {
        return error_response(
            400,
            ApiErrorView {
                code: "bad_request".to_string(),
                message: "bad request".to_string(),
                recoverable: false,
            },
        );
    };
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or_default();
    let (path, query) = split_target(target);

    if !request_has_valid_token(request, token) {
        return error_response(
            401,
            ApiErrorView {
                code: "unauthorized".to_string(),
                message: "unauthorized".to_string(),
                recoverable: false,
            },
        );
    }

    match route_request(method, path, query, request_body(request), state) {
        Ok(Some(response)) => response,
        Ok(None) => match (method, path) {
            ("GET", HEALTH_PATH) => json_response(200, &health_view()),
            ("GET", CAPABILITIES_PATH) => json_response(200, &capabilities_view()),
            _ => error_response(
                404,
                ApiErrorView {
                    code: "not_found".to_string(),
                    message: "not found".to_string(),
                    recoverable: false,
                },
            ),
        },
        Err(error) => domain_error_response(error),
    }
}

pub fn handle_http_request_with_shared_state(
    request: &str,
    token: &str,
    state: &SharedDaemonState,
) -> HttpResponse {
    if let Some(response) = handle_lock_free_shared_request(request, token) {
        return response;
    }
    match state.lock() {
        Ok(mut guard) => handle_http_request_with_state(request, token, &mut guard),
        Err(_) => error_response(
            500,
            ApiErrorView {
                code: "StateLockPoisoned".to_string(),
                message: "daemon state lock is poisoned".to_string(),
                recoverable: false,
            },
        ),
    }
}

fn handle_lock_free_shared_request(request: &str, token: &str) -> Option<HttpResponse> {
    let request_line = request.lines().next()?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or_default();
    let (path, _) = split_target(target);
    if !matches!(
        (method, path),
        ("GET", HEALTH_PATH) | ("GET", CAPABILITIES_PATH)
    ) {
        return None;
    }
    if !request_has_valid_token(request, token) {
        return Some(error_response(
            401,
            ApiErrorView {
                code: "unauthorized".to_string(),
                message: "unauthorized".to_string(),
                recoverable: false,
            },
        ));
    }
    Some(match path {
        HEALTH_PATH => json_response(200, &health_view()),
        CAPABILITIES_PATH => json_response(200, &capabilities_view()),
        _ => unreachable!("path checked above"),
    })
}

pub fn serve_one(listener: &TcpListener, token: &str) -> DomainResult<()> {
    let registry_path = std::env::temp_dir().join("imglab-daemon-stateless-registry.sqlite");
    let log_root = std::env::temp_dir().join("imglab-daemon-logs");
    let mut state = DaemonState::new(registry_path, log_root);
    serve_one_with_state(listener, token, &mut state)
}

pub fn serve_forever(
    listener: &TcpListener,
    token: &str,
    state: &mut DaemonState,
) -> DomainResult<()> {
    loop {
        serve_one_with_state(listener, token, state)?;
    }
}

pub fn serve_forever_shared(
    listener: &TcpListener,
    token: &str,
    state: SharedDaemonState,
) -> DomainResult<()> {
    loop {
        serve_one_with_shared_state(listener, token, &state)?;
    }
}

pub fn serve_one_with_state(
    listener: &TcpListener,
    token: &str,
    state: &mut DaemonState,
) -> DomainResult<()> {
    let (mut stream, _) = listener.accept().map_err(|error| DomainError::Io {
        path: listener
            .local_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
        message: error.to_string(),
    })?;
    handle_stream(&mut stream, token, state)
}

pub fn serve_one_with_shared_state(
    listener: &TcpListener,
    token: &str,
    state: &SharedDaemonState,
) -> DomainResult<()> {
    let (mut stream, _) = listener.accept().map_err(|error| DomainError::Io {
        path: listener
            .local_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
        message: error.to_string(),
    })?;
    handle_shared_stream(&mut stream, token, state)
}

