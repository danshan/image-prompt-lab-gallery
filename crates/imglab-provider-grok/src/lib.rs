use imglab_core::application::ports::ImageGenerationProvider;
use imglab_core::domain::generation::{
    GenerationOperation, GenerationParameters, GenerationResult,
};
use imglab_core::{DomainError, DomainResult};

pub struct GrokImageProvider;

impl ImageGenerationProvider for GrokImageProvider {
    fn name(&self) -> &'static str {
        "grok"
    }

    fn supports_operation(&self, operation: GenerationOperation) -> bool {
        matches!(
            operation,
            GenerationOperation::TextToImage | GenerationOperation::ImageToImage
        )
    }

    fn validate_parameters(&self, parameters: &GenerationParameters) -> DomainResult<()> {
        if parameters.prompt.trim().is_empty() {
            return Err(DomainError::InvalidGenerationParameters {
                message: "prompt must not be empty".to_string(),
            });
        }

        Ok(())
    }

    fn generate_from_text(
        &self,
        _parameters: &GenerationParameters,
    ) -> DomainResult<GenerationResult> {
        Err(DomainError::ProviderUnavailable {
            provider: self.name().to_string(),
        })
    }

    fn generate_from_image(
        &self,
        _parameters: &GenerationParameters,
        _input: &[u8],
    ) -> DomainResult<GenerationResult> {
        Err(DomainError::ProviderUnavailable {
            provider: self.name().to_string(),
        })
    }
}
