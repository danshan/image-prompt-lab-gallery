#[tauri::command]
async fn check_for_update(app: tauri::AppHandle) -> Result<UpdateCheckView, CommandError> {
    let current_version = app.package_info().version.to_string();
    let update = app
        .updater()
        .map_err(updater_error)?
        .check()
        .await
        .map_err(updater_error)?;

    Ok(UpdateCheckView {
        current_version,
        available: update.is_some(),
        update: update.map(|update| UpdateInfoView {
            version: update.version.to_string(),
            date: update.date.map(|date| date.to_string()),
            body: update.body,
        }),
    })
}

#[tauri::command]
async fn install_update(app: tauri::AppHandle) -> Result<UpdateInstallView, CommandError> {
    let update = app
        .updater()
        .map_err(updater_error)?
        .check()
        .await
        .map_err(updater_error)?;

    let Some(update) = update else {
        return Ok(UpdateInstallView {
            installed: false,
            version: None,
        });
    };

    let version = update.version.to_string();
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(updater_error)?;

    Ok(UpdateInstallView {
        installed: true,
        version: Some(version),
    })
}

#[tauri::command]
fn restart_app(app: tauri::AppHandle) -> Result<(), CommandError> {
    app.restart();
}
