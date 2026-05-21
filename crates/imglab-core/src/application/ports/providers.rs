pub use crate::{ImageProvider, ProviderCredentialStore, ProviderCredentials};

use crate::{DomainResult, GenerationOperation, GenerationParameters, GenerationResult};

pub trait ImageGenerationProvider {
    fn name(&self) -> &'static str;
    fn supports_operation(&self, operation: GenerationOperation) -> bool;
    fn validate_parameters(&self, parameters: &GenerationParameters) -> DomainResult<()>;
    fn generate_from_text(
        &self,
        parameters: &GenerationParameters,
    ) -> DomainResult<GenerationResult>;
    fn generate_from_image(
        &self,
        parameters: &GenerationParameters,
        input: &[u8],
    ) -> DomainResult<GenerationResult>;
}

impl<T> ImageGenerationProvider for T
where
    T: ImageProvider,
{
    fn name(&self) -> &'static str {
        ImageProvider::name(self)
    }

    fn supports_operation(&self, operation: GenerationOperation) -> bool {
        ImageProvider::supports_operation(self, operation)
    }

    fn validate_parameters(&self, parameters: &GenerationParameters) -> DomainResult<()> {
        ImageProvider::validate_parameters(self, parameters)
    }

    fn generate_from_text(
        &self,
        parameters: &GenerationParameters,
    ) -> DomainResult<GenerationResult> {
        ImageProvider::generate_from_text(self, parameters)
    }

    fn generate_from_image(
        &self,
        parameters: &GenerationParameters,
        input: &[u8],
    ) -> DomainResult<GenerationResult> {
        ImageProvider::generate_from_image(self, parameters, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FakeImageProvider;

    #[test]
    fn legacy_image_provider_implements_application_provider_port() {
        fn assert_application_provider<P: ImageGenerationProvider>(provider: &P) -> &'static str {
            provider.name()
        }

        let provider = FakeImageProvider::success("fake");

        assert_eq!(assert_application_provider(&provider), "fake");
    }
}
