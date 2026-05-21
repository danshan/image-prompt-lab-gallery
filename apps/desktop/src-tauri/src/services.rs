use crate::*;

pub(crate) fn desktop_app(
) -> imglab_core::infrastructure::composition::SqliteImgLabApplication<imglab_core::FakeImageProvider>
{
    desktop_app_with_provider(imglab_core::FakeImageProvider::success("fake"))
}

pub(crate) fn desktop_app_with_provider<P>(
    provider: P,
) -> imglab_core::infrastructure::composition::SqliteImgLabApplication<P>
where
    P: imglab_core::application::ports::ImageGenerationProvider,
{
    imglab_core::infrastructure::composition::sqlite_application(default_registry_path(), provider)
}

pub(crate) fn run_generation<P>(
    provider: P,
    request: GenerateImageRequest,
) -> Result<Vec<VersionView>, CommandError>
where
    P: imglab_core::application::ports::ImageGenerationProvider,
{
    let library_root = request.library_path.clone();
    desktop_app_with_provider(provider)
        .generation()
        .execute(request)
        .map(|versions| {
            versions
                .into_iter()
                .map(|version| version_view_with_library_path(&library_root, version))
                .collect()
        })
        .map_err(Into::into)
}

pub(crate) fn default_registry_path() -> PathBuf {
    std::env::var_os("IMGLAB_REGISTRY")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("imglab-desktop-registry.sqlite"))
}
