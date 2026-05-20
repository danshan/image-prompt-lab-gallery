fn parse_json_body<T: for<'de> Deserialize<'de>>(body: &str) -> DomainResult<T> {
    serde_json::from_str(body).map_err(serialization_error)
}

fn split_target(target: &str) -> (&str, Option<&str>) {
    match target.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (target, None),
    }
}

fn request_body(request: &str) -> &str {
    request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("")
}

fn query_value(query: &str, name: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == name).then(|| decode_query_component(value))
    })
}

fn decode_query_component(value: &str) -> String {
    value.replace('+', " ")
}

fn request_has_valid_token(request: &str, token: &str) -> bool {
    request.lines().any(|line| {
        let lower = line.to_ascii_lowercase();
        lower == format!("authorization: bearer {}", token.to_ascii_lowercase())
            || lower == format!("x-imglab-token: {}", token.to_ascii_lowercase())
    })
}

fn json_response<T: Serialize>(status_code: u16, value: &T) -> HttpResponse {
    match serde_json::to_string(value) {
        Ok(body) => response(status_code, &body),
        Err(error) => response(
            500,
            &format!(
                "{{\"error\":\"{}\"}}",
                escape_json_string(&error.to_string())
            ),
        ),
    }
}

fn error_response(status_code: u16, error: ApiErrorView) -> HttpResponse {
    json_response(status_code, &error)
}

fn domain_error_response(error: DomainError) -> HttpResponse {
    let status_code = match error {
        DomainError::InvalidTaskReference { .. } => 404,
        DomainError::InvalidGenerationParameters { .. } | DomainError::Serialization { .. } => 400,
        _ => 500,
    };
    error_response(
        status_code,
        ApiErrorView {
            code: error.code().to_string(),
            message: error.to_string(),
            recoverable: error.recoverable(),
        },
    )
}

fn response(status_code: u16, body: &str) -> HttpResponse {
    HttpResponse {
        status_code,
        body: body.to_string(),
    }
}

fn http_response_bytes(response: &HttpResponse) -> String {
    let status_text = match response.status_code {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        _ => "Internal Server Error",
    };
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.status_code,
        status_text,
        response.body.len(),
        response.body
    )
}

fn escape_json_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn io_error(path: &Path, error: std::io::Error) -> DomainError {
    DomainError::Io {
        path: path.display().to_string(),
        message: error.to_string(),
    }
}

fn serialization_error(error: serde_json::Error) -> DomainError {
    DomainError::Serialization {
        message: error.to_string(),
    }
}

