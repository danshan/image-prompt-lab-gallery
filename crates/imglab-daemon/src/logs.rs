fn handle_stream(stream: &mut TcpStream, token: &str, state: &mut DaemonState) -> DomainResult<()> {
    let mut buffer = [0_u8; 8192];
    let read = stream.read(&mut buffer).map_err(|error| DomainError::Io {
        path: "daemon-http-stream".to_string(),
        message: error.to_string(),
    })?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let response = handle_http_request_with_state(&request, token, state);
    stream
        .write_all(http_response_bytes(&response).as_bytes())
        .map_err(|error| DomainError::Io {
            path: "daemon-http-stream".to_string(),
            message: error.to_string(),
        })
}

fn handle_shared_stream(
    stream: &mut TcpStream,
    token: &str,
    state: &SharedDaemonState,
) -> DomainResult<()> {
    let mut buffer = [0_u8; 8192];
    let read = stream.read(&mut buffer).map_err(|error| DomainError::Io {
        path: "daemon-http-stream".to_string(),
        message: error.to_string(),
    })?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let response = handle_http_request_with_shared_state(&request, token, state);
    stream
        .write_all(http_response_bytes(&response).as_bytes())
        .map_err(|error| DomainError::Io {
            path: "daemon-http-stream".to_string(),
            message: error.to_string(),
        })
}

