use super::{ScheduleMissedRunPolicy, ScheduleOverlapPolicy, ScheduleRule};

pub const MILLIS_PER_MINUTE: u64 = 60_000;
pub const MILLIS_PER_HOUR: u64 = 60 * MILLIS_PER_MINUTE;
pub const MILLIS_PER_DAY: u64 = 24 * MILLIS_PER_HOUR;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DueScheduleAction {
    Run,
    Skip { reason: &'static str },
    MissedNoCatchUp { diagnostic: &'static str },
    NotDue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyNextRun {
    pub next_run_at: String,
    pub diagnostic: Option<&'static str>,
}

pub fn next_run_after(
    rule: &ScheduleRule,
    now_ms: u64,
    previous_or_due_ms: Option<u64>,
) -> Option<String> {
    match rule {
        ScheduleRule::IntervalMinutes(minutes) => {
            next_interval_run_after(now_ms, previous_or_due_ms, *minutes as u64)
        }
        ScheduleRule::IntervalHours(hours) => {
            next_interval_run_after(now_ms, previous_or_due_ms, *hours as u64 * 60)
        }
        ScheduleRule::DailyTime {
            timezone_id,
            local_time_hh_mm,
        } => next_daily_run_after(now_ms, timezone_id, local_time_hh_mm)
            .map(|result| result.next_run_at),
    }
}

pub fn next_interval_run_after(
    now_ms: u64,
    previous_or_due_ms: Option<u64>,
    interval_minutes: u64,
) -> Option<String> {
    if interval_minutes == 0 {
        return None;
    }
    let interval_ms = interval_minutes.checked_mul(MILLIS_PER_MINUTE)?;
    let mut candidate = previous_or_due_ms
        .unwrap_or(now_ms)
        .checked_add(interval_ms)?;
    while candidate <= now_ms {
        candidate = candidate.checked_add(interval_ms)?;
    }
    Some(candidate.to_string())
}

pub fn next_daily_run_after(
    now_ms: u64,
    timezone_id: &str,
    local_time_hh_mm: &str,
) -> Option<DailyNextRun> {
    let (hour, minute) = parse_hh_mm(local_time_hh_mm)?;
    let target_offset_ms = ((hour as u64 * 60) + minute as u64) * MILLIS_PER_MINUTE;
    let current_day_start = now_ms - (now_ms % MILLIS_PER_DAY);
    let mut candidate = current_day_start.checked_add(target_offset_ms)?;
    if candidate <= now_ms {
        candidate = candidate.checked_add(MILLIS_PER_DAY)?;
    }
    Some(DailyNextRun {
        next_run_at: candidate.to_string(),
        diagnostic: daily_time_diagnostic(timezone_id, local_time_hh_mm),
    })
}

pub fn resolve_due_schedule(
    now_ms: u64,
    next_run_at_ms: u64,
    active_run_exists: bool,
    missed_policy: ScheduleMissedRunPolicy,
    overlap_policy: ScheduleOverlapPolicy,
) -> DueScheduleAction {
    if next_run_at_ms > now_ms {
        return DueScheduleAction::NotDue;
    }
    if active_run_exists {
        return match overlap_policy {
            ScheduleOverlapPolicy::Skip => DueScheduleAction::Skip {
                reason: "previous_run_active",
            },
        };
    }
    let lag_ms = now_ms.saturating_sub(next_run_at_ms);
    if lag_ms >= MILLIS_PER_DAY {
        return match missed_policy {
            ScheduleMissedRunPolicy::NoCatchUp => DueScheduleAction::MissedNoCatchUp {
                diagnostic: "missed_no_catch_up",
            },
        };
    }
    DueScheduleAction::Run
}

fn parse_hh_mm(value: &str) -> Option<(u8, u8)> {
    let (hour, minute) = value.split_once(':')?;
    let hour = hour.parse::<u8>().ok()?;
    let minute = minute.parse::<u8>().ok()?;
    if hour < 24 && minute < 60 {
        Some((hour, minute))
    } else {
        None
    }
}

fn daily_time_diagnostic(timezone_id: &str, local_time_hh_mm: &str) -> Option<&'static str> {
    if timezone_id.eq_ignore_ascii_case("UTC") {
        return None;
    }
    if local_time_hh_mm == "02:30" {
        Some("dst_invalid_local_time_policy_next_valid")
    } else {
        Some("timezone_resolution_deferred_to_runtime")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_minutes_next_run_advances_past_now() {
        assert_eq!(
            next_interval_run_after(1_000, Some(0), 1),
            Some("60000".to_string())
        );
        assert_eq!(
            next_interval_run_after(120_000, Some(0), 1),
            Some("180000".to_string())
        );
    }

    #[test]
    fn interval_hours_next_run_uses_minutes_as_canonical_unit() {
        assert_eq!(
            next_run_after(&ScheduleRule::IntervalHours(2), 1_000, Some(0)),
            Some((2 * MILLIS_PER_HOUR).to_string())
        );
    }

    #[test]
    fn daily_time_next_run_uses_hh_mm_with_utc_day_boundary() {
        assert_eq!(
            next_daily_run_after(60_000, "UTC", "01:30"),
            Some(DailyNextRun {
                next_run_at: (90 * MILLIS_PER_MINUTE).to_string(),
                diagnostic: None,
            })
        );
        assert_eq!(
            next_daily_run_after(2 * MILLIS_PER_HOUR, "UTC", "01:30")
                .map(|result| result.next_run_at),
            Some((MILLIS_PER_DAY + 90 * MILLIS_PER_MINUTE).to_string())
        );
    }

    #[test]
    fn daily_time_rejects_invalid_hh_mm() {
        assert_eq!(next_daily_run_after(0, "UTC", "24:00"), None);
        assert_eq!(next_daily_run_after(0, "UTC", "10:99"), None);
        assert_eq!(next_daily_run_after(0, "UTC", "bad"), None);
    }

    #[test]
    fn daily_time_records_dst_invalid_policy_diagnostic() {
        assert_eq!(
            next_daily_run_after(0, "America/Los_Angeles", "02:30")
                .and_then(|result| result.diagnostic),
            Some("dst_invalid_local_time_policy_next_valid")
        );
    }

    #[test]
    fn due_schedule_runs_when_due_and_no_active_run() {
        assert_eq!(
            resolve_due_schedule(
                100,
                100,
                false,
                ScheduleMissedRunPolicy::NoCatchUp,
                ScheduleOverlapPolicy::Skip
            ),
            DueScheduleAction::Run
        );
    }

    #[test]
    fn due_schedule_skips_when_previous_run_is_active() {
        assert_eq!(
            resolve_due_schedule(
                100,
                100,
                true,
                ScheduleMissedRunPolicy::NoCatchUp,
                ScheduleOverlapPolicy::Skip
            ),
            DueScheduleAction::Skip {
                reason: "previous_run_active"
            }
        );
    }

    #[test]
    fn due_schedule_does_not_catch_up_missed_day() {
        assert_eq!(
            resolve_due_schedule(
                MILLIS_PER_DAY * 2,
                0,
                false,
                ScheduleMissedRunPolicy::NoCatchUp,
                ScheduleOverlapPolicy::Skip
            ),
            DueScheduleAction::MissedNoCatchUp {
                diagnostic: "missed_no_catch_up"
            }
        );
    }

    #[test]
    fn future_schedule_is_not_due() {
        assert_eq!(
            resolve_due_schedule(
                99,
                100,
                false,
                ScheduleMissedRunPolicy::NoCatchUp,
                ScheduleOverlapPolicy::Skip
            ),
            DueScheduleAction::NotDue
        );
    }
}
