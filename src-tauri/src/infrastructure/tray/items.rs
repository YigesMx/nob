use tauri::AppHandle;

use crate::infrastructure::tray::TrayMenuItem;

/// 退出应用菜单项
pub fn quit_app_item() -> TrayMenuItem {
    TrayMenuItem::always_visible("quit", "退出", |app: &AppHandle| {
        app.exit(0);
    })
}
