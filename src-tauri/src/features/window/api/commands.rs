use tauri::{AppHandle, Runtime, State};
use crate::core::AppState;
use crate::features::window::manager;

#[tauri::command]
pub async fn window_drag_start(app_state: State<'_, AppState>) -> Result<(), String> {
    manager::hide_content_window(&app_state.app_handle());
    Ok(())
}
