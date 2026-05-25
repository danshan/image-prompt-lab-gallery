use crate::{
    DomainError, DomainResult, GeneratedImage, GenerationParameters, GenerationResult,
    ImageProvider, PromptExpansionRequest, PromptExpansionResult,
};

#[derive(Debug, Clone)]
pub enum FakeProviderMode {
    Success,
    Failure { message: String },
    Timeout,
    InvalidParameters { message: String },
}

#[derive(Debug, Clone)]
pub struct FakeImageProvider {
    name: &'static str,
    mode: FakeProviderMode,
}

impl FakeImageProvider {
    pub fn success(name: &'static str) -> Self {
        Self {
            name,
            mode: FakeProviderMode::Success,
        }
    }

    pub fn failure(name: &'static str, message: impl Into<String>) -> Self {
        Self {
            name,
            mode: FakeProviderMode::Failure {
                message: message.into(),
            },
        }
    }

    pub fn timeout(name: &'static str) -> Self {
        Self {
            name,
            mode: FakeProviderMode::Timeout,
        }
    }

    pub fn invalid_parameters(name: &'static str, message: impl Into<String>) -> Self {
        Self {
            name,
            mode: FakeProviderMode::InvalidParameters {
                message: message.into(),
            },
        }
    }

    fn generate_success(
        &self,
        parameters: &GenerationParameters,
    ) -> DomainResult<GenerationResult> {
        self.validate_parameters(parameters)?;
        Ok(GenerationResult {
            images: vec![GeneratedImage {
                bytes: b"fake image bytes".to_vec(),
                mime_type: "image/png".to_string(),
                provider_metadata_json: "{\"fake\":true}".to_string(),
            }],
            raw_request_json: format!("{{\"prompt\":{:?}}}", parameters.prompt),
            raw_response_json: "{\"images\":1}".to_string(),
        })
    }
}

impl ImageProvider for FakeImageProvider {
    fn name(&self) -> &'static str {
        self.name
    }

    fn supports_operation(&self, _operation: crate::GenerationOperation) -> bool {
        true
    }

    fn validate_parameters(&self, parameters: &GenerationParameters) -> DomainResult<()> {
        if parameters.prompt.trim().is_empty() {
            return Err(DomainError::InvalidGenerationParameters {
                message: "prompt must not be empty".to_string(),
            });
        }

        match &self.mode {
            FakeProviderMode::InvalidParameters { message } => {
                Err(DomainError::InvalidGenerationParameters {
                    message: message.clone(),
                })
            }
            _ => Ok(()),
        }
    }

    fn generate_from_text(
        &self,
        parameters: &GenerationParameters,
    ) -> DomainResult<GenerationResult> {
        match &self.mode {
            FakeProviderMode::Success => self.generate_success(parameters),
            FakeProviderMode::Failure { message } => Err(DomainError::GenerationFailed {
                provider: self.name.to_string(),
                message: message.clone(),
            }),
            FakeProviderMode::Timeout => Err(DomainError::ProviderUnavailable {
                provider: self.name.to_string(),
            }),
            FakeProviderMode::InvalidParameters { message } => {
                Err(DomainError::InvalidGenerationParameters {
                    message: message.clone(),
                })
            }
        }
    }

    fn generate_from_image(
        &self,
        parameters: &GenerationParameters,
        input: &[u8],
    ) -> DomainResult<GenerationResult> {
        if input.is_empty() {
            return Err(DomainError::InvalidGenerationParameters {
                message: "input image must not be empty".to_string(),
            });
        }

        self.generate_from_text(parameters)
    }
}

impl crate::application::ports::PromptExpansionProvider for FakeImageProvider {
    fn name(&self) -> &'static str {
        self.name
    }

    fn expand_prompt(
        &self,
        request: &PromptExpansionRequest,
    ) -> DomainResult<PromptExpansionResult> {
        let base = request.base_prompt.trim();
        let dynamic = request.dynamic_prompt.trim();
        if base.is_empty() {
            return Err(DomainError::InvalidGenerationParameters {
                message: "base prompt must not be empty".to_string(),
            });
        }
        if dynamic.is_empty() {
            return Err(DomainError::InvalidGenerationParameters {
                message: "dynamic prompt must not be empty".to_string(),
            });
        }
        match &self.mode {
            FakeProviderMode::Failure { message } => Err(DomainError::GenerationFailed {
                provider: self.name.to_string(),
                message: message.clone(),
            }),
            FakeProviderMode::Timeout => Err(DomainError::ProviderUnavailable {
                provider: self.name.to_string(),
            }),
            FakeProviderMode::InvalidParameters { message } => {
                Err(DomainError::InvalidGenerationParameters {
                    message: message.clone(),
                })
            }
            FakeProviderMode::Success => Ok(PromptExpansionResult {
                expanded_prompt: format!("{base}\n\n{dynamic}"),
                provider_metadata_json: serde_json::json!({
                    "fake": true,
                    "provider": self.name,
                    "model": request.model,
                })
                .to_string(),
                raw_request_json: Some(
                    serde_json::json!({
                        "basePrompt": request.base_prompt,
                        "dynamicPrompt": request.dynamic_prompt,
                    })
                    .to_string(),
                ),
                raw_response_json: Some(serde_json::json!({"expanded": true}).to_string()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::PromptExpansionProvider;
    use crate::GenerationOperation;

    fn parameters(prompt: &str) -> GenerationParameters {
        GenerationParameters {
            library_path: None,
            provider: "fake".to_string(),
            model: "fake-image".to_string(),
            prompt: prompt.to_string(),
            negative_prompt: None,
            operation: GenerationOperation::TextToImage,
            input_version_id: None,
            prompt_version_id: None,
            parameters_json: "{}".to_string(),
        }
    }

    #[test]
    fn fake_provider_returns_successful_image() {
        let provider = FakeImageProvider::success("fake");
        let result = provider
            .generate_from_text(&parameters("make a test image"))
            .expect("generate");

        assert_eq!(result.images.len(), 1);
        assert_eq!(result.images[0].mime_type, "image/png");
    }

    #[test]
    fn fake_provider_can_fail() {
        let provider = FakeImageProvider::failure("fake", "boom");
        let error = provider
            .generate_from_text(&parameters("make a test image"))
            .expect_err("should fail");

        assert!(matches!(error, DomainError::GenerationFailed { .. }));
    }

    #[test]
    fn fake_provider_rejects_empty_prompt() {
        let provider = FakeImageProvider::success("fake");
        let error = provider
            .generate_from_text(&parameters(" "))
            .expect_err("should reject prompt");

        assert!(matches!(
            error,
            DomainError::InvalidGenerationParameters { .. }
        ));
    }

    #[test]
    fn fake_provider_expands_dynamic_prompt_deterministically() {
        let provider = FakeImageProvider::success("fake");
        let result = provider
            .expand_prompt(&PromptExpansionRequest {
                provider: "fake".to_string(),
                model: Some("fake-model".to_string()),
                base_prompt: "A quiet botanical study".to_string(),
                dynamic_prompt: "Use moonlight and glass textures".to_string(),
                context_json: None,
            })
            .expect("expand prompt");

        assert_eq!(
            result.expanded_prompt,
            "A quiet botanical study\n\nUse moonlight and glass textures"
        );
        assert!(result.provider_metadata_json.contains("\"fake\":true"));
    }

    #[test]
    fn fake_provider_rejects_empty_dynamic_prompt() {
        let provider = FakeImageProvider::success("fake");
        let error = provider
            .expand_prompt(&PromptExpansionRequest {
                provider: "fake".to_string(),
                model: None,
                base_prompt: "A quiet botanical study".to_string(),
                dynamic_prompt: "  ".to_string(),
                context_json: None,
            })
            .expect_err("empty dynamic prompt should fail");

        assert!(error.to_string().contains("dynamic prompt"));
    }
}
