use crate::features::window::manager;

#[tauri::command]
pub async fn set_content_window_pinned(pinned: bool) -> Result<(), String> {
    manager::set_content_window_pinned(pinned);
    Ok(())
}
