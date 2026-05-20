#[tauri::command]
fn list_app_logs() -> Result<Vec<AppLogView>, CommandError> {
    app_logs::list_app_logs()
}

#[tauri::command]
fn read_app_log(input: ReadAppLogInput) -> Result<AppLogContentView, CommandError> {
    app_logs::read_app_log(&input.path)
}
