use crate::ConfidenceScoreView;
use serde_json::Value;

pub const PENDING_REVIEW_STATUS: &str = "pending_review";
pub const ACCEPTED_STATUS: &str = "accepted";
pub const REJECTED_STATUS: &str = "rejected";

pub fn is_pending_review(status: &str) -> bool {
    status == PENDING_REVIEW_STATUS
}

pub fn normalize_confidence_json(confidence_json: &str) -> ConfidenceScoreView {
    let Ok(value) = serde_json::from_str::<Value>(confidence_json) else {
        return empty_confidence();
    };
    let fields = value.get("fields");
    ConfidenceScoreView {
        overall: normalize_score(value.get("overall")),
        title: normalize_score(fields.and_then(|fields| fields.get("title"))),
        description: normalize_score(fields.and_then(|fields| fields.get("description"))),
        schema_prompt: normalize_score(fields.and_then(|fields| fields.get("schemaPrompt"))),
        tags: normalize_score(fields.and_then(|fields| fields.get("tags"))),
        category: normalize_score(fields.and_then(|fields| fields.get("category"))),
    }
}

fn empty_confidence() -> ConfidenceScoreView {
    ConfidenceScoreView {
        overall: None,
        title: None,
        description: None,
        schema_prompt: None,
        tags: None,
        category: None,
    }
}

fn normalize_score(value: Option<&Value>) -> Option<u8> {
    let value = value?.as_f64()?;
    if !(0.0..=100.0).contains(&value) {
        return None;
    }
    let normalized = if value <= 1.0 { value * 100.0 } else { value };
    Some(normalized.round().clamp(0.0, 100.0) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_review_status_is_explicit() {
        assert!(is_pending_review(PENDING_REVIEW_STATUS));
        assert!(!is_pending_review(ACCEPTED_STATUS));
        assert!(!is_pending_review(REJECTED_STATUS));
    }

    #[test]
    fn confidence_scores_accept_ratio_or_percent_values() {
        let confidence = normalize_confidence_json(
            r#"{"overall":0.5,"fields":{"title":75,"description":1,"schemaPrompt":101}}"#,
        );

        assert_eq!(confidence.overall, Some(50));
        assert_eq!(confidence.title, Some(75));
        assert_eq!(confidence.description, Some(100));
        assert_eq!(confidence.schema_prompt, None);
    }

    #[test]
    fn invalid_confidence_json_returns_empty_scores() {
        let confidence = normalize_confidence_json("not-json");
        assert_eq!(confidence.overall, None);
        assert_eq!(confidence.title, None);
        assert_eq!(confidence.tags, None);
    }
}
