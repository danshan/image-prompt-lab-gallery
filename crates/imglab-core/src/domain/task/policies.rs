use super::{TaskErrorClassification, TaskStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuccessfulAttemptResolution {
    pub task_status: TaskStatus,
    pub extra_event_type: Option<&'static str>,
    pub extra_event_message: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FailedAttemptResolution {
    pub task_status: TaskStatus,
    pub event_type: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancelRequestResolution {
    pub task_status: TaskStatus,
    pub write_cancel_marker: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryInterruptionResolution {
    pub task_status: TaskStatus,
    pub error_classification: TaskErrorClassification,
    pub retryable: bool,
}

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

pub fn resolve_successful_attempt(current_status: TaskStatus) -> SuccessfulAttemptResolution {
    if current_status == TaskStatus::CancelRequested {
        SuccessfulAttemptResolution {
            task_status: TaskStatus::Completed,
            extra_event_type: Some("completed_after_cancel_requested"),
            extra_event_message: Some("Task completed after cancel was requested"),
        }
    } else {
        SuccessfulAttemptResolution {
            task_status: TaskStatus::Completed,
            extra_event_type: None,
            extra_event_message: None,
        }
    }
}

pub fn resolve_canceled_attempt_status() -> TaskStatus {
    TaskStatus::Canceled
}

pub fn resolve_cancel_request(current_status: TaskStatus) -> CancelRequestResolution {
    match current_status {
        TaskStatus::Running | TaskStatus::CancelRequested => CancelRequestResolution {
            task_status: TaskStatus::CancelRequested,
            write_cancel_marker: true,
        },
        TaskStatus::Queued | TaskStatus::RetryWaiting => CancelRequestResolution {
            task_status: TaskStatus::Canceled,
            write_cancel_marker: false,
        },
        status => CancelRequestResolution {
            task_status: status,
            write_cancel_marker: false,
        },
    }
}

pub fn resolve_recovered_running_with_outputs() -> TaskStatus {
    TaskStatus::Completed
}

pub fn resolve_interrupted_recovery(
    attempt_count: u32,
    max_attempts: u32,
) -> RecoveryInterruptionResolution {
    let retryable = attempt_count < max_attempts;
    RecoveryInterruptionResolution {
        task_status: if retryable {
            TaskStatus::InterruptedRetryable
        } else {
            TaskStatus::InterruptedFinal
        },
        error_classification: if retryable {
            TaskErrorClassification::Transient
        } else {
            TaskErrorClassification::Final
        },
        retryable,
    }
}

pub fn resolve_failed_attempt(
    classification: TaskErrorClassification,
    can_auto_retry: bool,
) -> FailedAttemptResolution {
    if can_auto_retry {
        FailedAttemptResolution {
            task_status: TaskStatus::RetryWaiting,
            event_type: "retry_scheduled",
        }
    } else if classification == TaskErrorClassification::RetryableManual {
        FailedAttemptResolution {
            task_status: TaskStatus::FailedRetryable,
            event_type: "attempt_failed",
        }
    } else {
        FailedAttemptResolution {
            task_status: TaskStatus::FailedFinal,
            event_type: "attempt_failed",
        }
    }
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

    #[test]
    fn successful_attempt_completion_policy_records_late_cancel_signal() {
        let resolution = resolve_successful_attempt(TaskStatus::CancelRequested);

        assert_eq!(resolution.task_status, TaskStatus::Completed);
        assert_eq!(
            resolution.extra_event_type,
            Some("completed_after_cancel_requested")
        );
    }

    #[test]
    fn failed_attempt_policy_selects_retry_or_terminal_status() {
        assert_eq!(
            resolve_failed_attempt(TaskErrorClassification::Transient, true),
            FailedAttemptResolution {
                task_status: TaskStatus::RetryWaiting,
                event_type: "retry_scheduled",
            }
        );
        assert_eq!(
            resolve_failed_attempt(TaskErrorClassification::RetryableManual, false).task_status,
            TaskStatus::FailedRetryable
        );
        assert_eq!(
            resolve_failed_attempt(TaskErrorClassification::Final, false).task_status,
            TaskStatus::FailedFinal
        );
    }

    #[test]
    fn cancel_request_policy_selects_status_and_marker_need() {
        assert_eq!(
            resolve_cancel_request(TaskStatus::Queued),
            CancelRequestResolution {
                task_status: TaskStatus::Canceled,
                write_cancel_marker: false,
            }
        );
        assert_eq!(
            resolve_cancel_request(TaskStatus::Running),
            CancelRequestResolution {
                task_status: TaskStatus::CancelRequested,
                write_cancel_marker: true,
            }
        );
        assert_eq!(
            resolve_cancel_request(TaskStatus::Completed),
            CancelRequestResolution {
                task_status: TaskStatus::Completed,
                write_cancel_marker: false,
            }
        );
    }

    #[test]
    fn interrupted_recovery_policy_selects_retryable_or_terminal_state() {
        assert_eq!(
            resolve_interrupted_recovery(1, 3),
            RecoveryInterruptionResolution {
                task_status: TaskStatus::InterruptedRetryable,
                error_classification: TaskErrorClassification::Transient,
                retryable: true,
            }
        );
        assert_eq!(
            resolve_interrupted_recovery(3, 3),
            RecoveryInterruptionResolution {
                task_status: TaskStatus::InterruptedFinal,
                error_classification: TaskErrorClassification::Final,
                retryable: false,
            }
        );
    }
}
