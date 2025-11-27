use tauri::{AppHandle, Emitter, Manager};

use crate::core::AppState;
use crate::features::settings::core::service::SettingService;
use crate::infrastructure::tray::TrayMenuItem;

pub fn theme_light_item() -> TrayMenuItem {
    TrayMenuItem::always_visible("theme_light", "外观：亮色", |app: &AppHandle| {
        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
            if let Some(state) = app_handle.try_state::<AppState>() {
                let _ = SettingService::set(state.db(), "ui.theme", "light").await;
                state.set_theme("light".to_string());
                let _ = state.tray_manager().update_tray_menu(&app_handle);
                let _ = app_handle.emit("theme-changed", "light");
            }
        });
    })
    .with_enabled(|app: &AppHandle| {
        if let Some(state) = app.try_state::<AppState>() {
            state.get_theme() != "light"
        } else {
            true
        }
    })
}

pub fn theme_dark_item() -> TrayMenuItem {
    TrayMenuItem::always_visible("theme_dark", "外观：暗色", |app: &AppHandle| {
        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
            if let Some(state) = app_handle.try_state::<AppState>() {
                let _ = SettingService::set(state.db(), "ui.theme", "dark").await;
                state.set_theme("dark".to_string());
                let _ = state.tray_manager().update_tray_menu(&app_handle);
                let _ = app_handle.emit("theme-changed", "dark");
            }
        });
    })
    .with_enabled(|app: &AppHandle| {
        if let Some(state) = app.try_state::<AppState>() {
            state.get_theme() != "dark"
        } else {
            true
        }
    })
}

pub fn theme_system_item() -> TrayMenuItem {
    TrayMenuItem::always_visible("theme_system", "外观：跟随系统", |app: &AppHandle| {
        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
            if let Some(state) = app_handle.try_state::<AppState>() {
                let _ = SettingService::set(state.db(), "ui.theme", "system").await;
                state.set_theme("system".to_string());
                let _ = state.tray_manager().update_tray_menu(&app_handle);
                let _ = app_handle.emit("theme-changed", "system");
            }
        });
    })
    .with_enabled(|app: &AppHandle| {
        if let Some(state) = app.try_state::<AppState>() {
            state.get_theme() != "system"
        } else {
            true
        }
    })
}
