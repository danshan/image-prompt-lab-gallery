use imglab_core::{
    DomainError, DomainResult, GenerationParameters, GenerationResult, ImageProvider,
};

pub struct GrokImageProvider;

impl ImageProvider for GrokImageProvider {
    fn name(&self) -> &'static str {
        "grok"
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
