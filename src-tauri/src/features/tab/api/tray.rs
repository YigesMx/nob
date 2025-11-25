use tauri::{AppHandle, Manager};

use crate::features::tab::core::service::TabService;
use crate::infrastructure::tray::TrayMenuItem;
use crate::AppState;

/// 快捷切换到下一个标签
pub fn activate_next_item() -> TrayMenuItem {
    TrayMenuItem::always_visible("tabs_activate_next", "切换到下一个标签", |app: &AppHandle| {
        if let Some(state) = app.try_state::<AppState>() {
            let db = state.db().clone();
            tauri::async_runtime::spawn(async move {
                let _ = TabService::activate_next(&db).await;
            });
        }
    })
}

/// 快捷切换到上一个标签
pub fn activate_previous_item() -> TrayMenuItem {
    TrayMenuItem::always_visible("tabs_activate_previous", "切换到上一个标签", |app: &AppHandle| {
        if let Some(state) = app.try_state::<AppState>() {
            let db = state.db().clone();
            tauri::async_runtime::spawn(async move {
                let _ = TabService::activate_previous(&db).await;
            });
        }
    })
}

/// 关闭当前激活标签
pub fn close_active_item() -> TrayMenuItem {
    TrayMenuItem::always_visible("tabs_close_active", "关闭当前标签", |app: &AppHandle| {
        if let Some(state) = app.try_state::<AppState>() {
            let db = state.db().clone();
            tauri::async_runtime::spawn(async move {
                let _ = TabService::close_active(&db).await;
            });
        }
    })
}
