use crate::*;

pub(crate) fn run_generation<P>(
    provider: P,
    request: GenerateImageRequest,
) -> Result<Vec<VersionView>, CommandError>
where
    P: ImageProvider,
{
    let library_root = request.library_path.clone();
    LocalGenerationService::new(provider)
        .generate(request)
        .map(|versions| {
            versions
                .into_iter()
                .map(|version| version_view_with_library_path(&library_root, version))
                .collect()
        })
        .map_err(Into::into)
}

pub(crate) fn service() -> LocalLibraryService {
    LocalLibraryService::new(default_registry_path())
}

pub(crate) fn default_registry_path() -> PathBuf {
    std::env::var_os("IMGLAB_REGISTRY")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("imglab-desktop-registry.sqlite"))
}
