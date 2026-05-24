use crate::application::facade::{ImgLabApplication, ImgLabApplicationParts};
use crate::application::ports::ImageGenerationProvider;
use crate::application::use_cases::albums::{AlbumUseCase, QueryGalleryUseCase, SearchUseCase};
use crate::application::use_cases::assets::AssetUseCase;
use crate::application::use_cases::generation::GenerateImageUseCase;
use crate::application::use_cases::library::LibraryUseCase;
use crate::application::use_cases::metadata_review::ReviewMetadataSuggestionUseCase;
use crate::application::use_cases::prompts::PromptWorkspaceUseCase;
use crate::application::use_cases::tasks::TaskUseCase;
use crate::library::LocalLibraryService;
use std::path::PathBuf;

pub type SqliteAssetUseCase = AssetUseCase<LocalLibraryService, LocalLibraryService>;

pub type SqliteImgLabApplication<P> = ImgLabApplication<
    LocalLibraryService,
    LibraryUseCase<LocalLibraryService>,
    SqliteAssetUseCase,
    GenerateImageUseCase<
        P,
        LocalLibraryService,
        LocalLibraryService,
        LocalLibraryService,
        LocalLibraryService,
    >,
    ReviewMetadataSuggestionUseCase<LocalLibraryService>,
    AlbumUseCase<LocalLibraryService>,
    QueryGalleryUseCase<LocalLibraryService>,
    SearchUseCase<LocalLibraryService>,
    TaskUseCase<LocalLibraryService>,
    PromptWorkspaceUseCase<LocalLibraryService>,
>;

pub fn sqlite_application<P>(
    registry_path: impl Into<PathBuf>,
    provider: P,
) -> SqliteImgLabApplication<P>
where
    P: ImageGenerationProvider,
{
    let service = LocalLibraryService::new(registry_path);
    ImgLabApplication::from_parts(ImgLabApplicationParts {
        library: service.clone(),
        library_lifecycle: LibraryUseCase::new(service.clone()),
        assets: AssetUseCase::new(service.clone(), service.clone()),
        generation: GenerateImageUseCase::new(
            provider,
            service.clone(),
            service.clone(),
            service.clone(),
            service.clone(),
        ),
        metadata_review: ReviewMetadataSuggestionUseCase::new(service.clone()),
        albums: AlbumUseCase::new(service.clone()),
        gallery: QueryGalleryUseCase::new(service.clone()),
        search: SearchUseCase::new(service.clone()),
        tasks: TaskUseCase::new(service.clone()),
        prompts: PromptWorkspaceUseCase::new(service),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FakeImageProvider;

    #[test]
    fn sqlite_composition_root_wires_all_facade_owners() {
        let app = sqlite_application(
            std::env::temp_dir().join("imglab-composition-test-registry.sqlite"),
            FakeImageProvider::success("fake"),
        );

        let _ = app.library();
        let _ = app.library_lifecycle();
        let _ = app.assets();
        let _ = app.generation();
        let _ = app.metadata_review();
        let _ = app.albums();
        let _ = app.gallery();
        let _ = app.search();
        let _ = app.tasks();
        let _ = app.prompts();
    }
}
