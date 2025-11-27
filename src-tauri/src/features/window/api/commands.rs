use crate::features::window::manager;

#[tauri::command]
pub async fn set_content_window_pinned(pinned: bool) -> Result<(), String> {
    manager::set_content_window_pinned(pinned);
    Ok(())
}

#[tauri::command]
pub async fn resize_main_window(app: tauri::AppHandle, width: f64, height: f64) -> Result<(), String> {
    manager::resize_main_window(&app, width, height)
}
