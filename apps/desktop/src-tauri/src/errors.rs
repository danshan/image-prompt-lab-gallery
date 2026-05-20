#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandError {
    code: String,
    message: String,
    recoverable: bool,
}

impl From<DomainError> for CommandError {
    fn from(error: DomainError) -> Self {
        Self {
            code: error.code().to_string(),
            message: error.to_string(),
            recoverable: error.recoverable(),
        }
    }
}

fn updater_error(error: impl std::fmt::Display) -> CommandError {
    CommandError {
        code: "UpdaterError".to_string(),
        message: error.to_string(),
        recoverable: true,
    }
}


fn invalid_path_error(message: String) -> CommandError {
    CommandError {
        code: "InvalidPath".to_string(),
        message,
        recoverable: true,
    }
}
