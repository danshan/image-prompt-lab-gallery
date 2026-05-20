use crate::commands::daemon::execute_generation;
use crate::*;

#[tauri::command]
pub(crate) async fn generate_image(
    input: GenerateImageInput,
) -> Result<Vec<VersionView>, CommandError> {
    tauri::async_runtime::spawn_blocking(move || execute_generation(input, None))
        .await
        .map_err(|error| CommandError {
            code: "GenerationFailed".to_string(),
            message: format!("generation worker failed: {error}"),
            recoverable: true,
        })?
}
