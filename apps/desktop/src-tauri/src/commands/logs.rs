use crate::*;

#[tauri::command]
pub(crate) fn list_app_logs() -> Result<Vec<AppLogView>, CommandError> {
    app_logs::list_app_logs()
}

#[tauri::command]
pub(crate) fn read_app_log(input: ReadAppLogInput) -> Result<AppLogContentView, CommandError> {
    app_logs::read_app_log(&input.path)
}
