pub fn health_view() -> HealthView {
    HealthView {
        status: "ok".to_string(),
        api_version: API_VERSION.to_string(),
        schema_version: CURRENT_SCHEMA_VERSION,
        provider_capabilities: provider_capabilities_view(),
    }
}

fn provider_capabilities_view() -> Vec<ProviderCapabilityView> {
    let codex = CodexCliImageProvider::default();
    let fake = imglab_core::FakeImageProvider::success("fake");
    [
        (&codex as &dyn ImageProvider),
        (&fake as &dyn ImageProvider),
    ]
    .into_iter()
    .map(|provider| ProviderCapabilityView {
        provider: provider.name().to_string(),
        supported_operations: [
            GenerationOperation::TextToImage,
            GenerationOperation::ImageToImage,
        ]
        .into_iter()
        .filter(|operation| provider.supports_operation(*operation))
        .map(generation_operation_as_str)
        .map(str::to_string)
        .collect(),
    })
    .collect()
}

pub fn capabilities_view() -> CapabilitiesView {
    CapabilitiesView {
        api_version: API_VERSION.to_string(),
        task_types: vec![
            TaskType::ImageGeneration.as_str().to_string(),
            TaskType::MetadataFieldGeneration.as_str().to_string(),
            TaskType::MetadataSuggestionGeneration.as_str().to_string(),
        ],
    }
}

