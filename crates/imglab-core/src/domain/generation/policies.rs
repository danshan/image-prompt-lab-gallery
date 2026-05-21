use crate::{AssetVersionId, DomainError, DomainResult, GenerationOperation};
use std::path::PathBuf;

pub fn normalize_provider_name(provider: &str) -> DomainResult<String> {
    match provider {
        "codex" | "codex-cli" => Ok("codex-cli".to_string()),
        "fake" => Ok("fake".to_string()),
        other => Err(DomainError::InvalidGenerationParameters {
            message: format!("unsupported provider: {other}"),
        }),
    }
}

pub fn default_generation_model_label(provider: &str) -> &'static str {
    match provider {
        "fake" => "fake-image",
        _ => "imagegen-skill",
    }
}

pub fn infer_generation_operation(
    input_file: Option<&PathBuf>,
    input_version_id: Option<&AssetVersionId>,
) -> GenerationOperation {
    if input_file.is_some() || input_version_id.is_some() {
        GenerationOperation::ImageToImage
    } else {
        GenerationOperation::TextToImage
    }
}

pub fn operation_to_str(operation: GenerationOperation) -> &'static str {
    match operation {
        GenerationOperation::TextToImage => "text_to_image",
        GenerationOperation::ImageToImage => "image_to_image",
    }
}

pub fn operation_from_str(value: &str) -> DomainResult<GenerationOperation> {
    match value {
        "text_to_image" => Ok(GenerationOperation::TextToImage),
        "image_to_image" => Ok(GenerationOperation::ImageToImage),
        _ => Err(DomainError::Database {
            message: format!("unknown generation operation: {value}"),
        }),
    }
}

pub fn provider_supports_operation_by_default(operation: GenerationOperation) -> bool {
    matches!(operation, GenerationOperation::TextToImage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_names_are_normalized() {
        assert_eq!(
            normalize_provider_name("codex").expect("codex"),
            "codex-cli"
        );
        assert_eq!(normalize_provider_name("fake").expect("fake"), "fake");
        assert!(matches!(
            normalize_provider_name("unknown"),
            Err(DomainError::InvalidGenerationParameters { .. })
        ));
    }

    #[test]
    fn operation_is_inferred_from_image_inputs() {
        assert_eq!(
            infer_generation_operation(None, None),
            GenerationOperation::TextToImage
        );
        assert_eq!(
            infer_generation_operation(Some(&PathBuf::from("input.png")), None),
            GenerationOperation::ImageToImage
        );
        assert_eq!(
            infer_generation_operation(None, Some(&AssetVersionId("v1".to_string()))),
            GenerationOperation::ImageToImage
        );
    }

    #[test]
    fn operation_round_trips_to_storage_value() {
        assert_eq!(
            operation_from_str(operation_to_str(GenerationOperation::ImageToImage))
                .expect("operation"),
            GenerationOperation::ImageToImage
        );
    }
}
