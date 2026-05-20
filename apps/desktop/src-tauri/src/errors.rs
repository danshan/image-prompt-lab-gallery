use crate::*;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommandError {
    pub(crate) code: String,
    pub(crate) message: String,
    pub(crate) recoverable: bool,
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

pub(crate) fn updater_error(error: impl std::fmt::Display) -> CommandError {
    CommandError {
        code: "UpdaterError".to_string(),
        message: error.to_string(),
        recoverable: true,
    }
}

pub(crate) fn invalid_path_error(message: String) -> CommandError {
    CommandError {
        code: "InvalidPath".to_string(),
        message,
        recoverable: true,
    }
}
