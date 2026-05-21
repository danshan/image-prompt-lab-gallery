use super::{TaskErrorClassification, TaskStatus};

pub fn is_terminal_status(status: TaskStatus) -> bool {
    matches!(
        status,
        TaskStatus::FailedFinal
            | TaskStatus::Canceled
            | TaskStatus::Completed
            | TaskStatus::InterruptedFinal
    )
}

pub fn is_retryable_status(status: TaskStatus) -> bool {
    matches!(
        status,
        TaskStatus::RetryWaiting | TaskStatus::FailedRetryable | TaskStatus::InterruptedRetryable
    )
}

pub fn should_auto_retry(classification: TaskErrorClassification) -> bool {
    matches!(classification, TaskErrorClassification::Transient)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_statuses_are_explicit() {
        assert!(is_terminal_status(TaskStatus::Completed));
        assert!(is_terminal_status(TaskStatus::FailedFinal));
        assert!(!is_terminal_status(TaskStatus::RetryWaiting));
        assert!(!is_terminal_status(TaskStatus::Running));
    }

    #[test]
    fn retryable_statuses_are_explicit() {
        assert!(is_retryable_status(TaskStatus::FailedRetryable));
        assert!(is_retryable_status(TaskStatus::InterruptedRetryable));
        assert!(!is_retryable_status(TaskStatus::FailedFinal));
    }

    #[test]
    fn only_transient_errors_auto_retry() {
        assert!(should_auto_retry(TaskErrorClassification::Transient));
        assert!(!should_auto_retry(TaskErrorClassification::Final));
        assert!(!should_auto_retry(TaskErrorClassification::RetryableManual));
    }
}
