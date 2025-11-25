use tauri::State;

use crate::core::AppState;
use crate::features::tab::core::models::{
    CreateTabPayload, ReorderTabsPayload, Tab, UpdateTabPayload,
};
use crate::features::tab::core::service::TabService;
use crate::features::window::manager as window_manager;
use serde_json::json;
use tauri::Emitter;

#[tauri::command]
pub async fn tabs_list(app_state: State<'_, AppState>) -> Result<Vec<Tab>, String> {
    TabService::list(app_state.db())
        .await
        .map(|tabs| tabs.into_iter().map(Tab::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tabs_create(
    app_state: State<'_, AppState>,
    payload: CreateTabPayload,
) -> Result<Tab, String> {
    let tab = TabService::create(app_state.db(), payload)
        .await
        .map(Tab::from)
        .map_err(|e| e.to_string())?;

    // 打开或导航内容窗口
    let _ = window_manager::open_or_navigate_content_window(&app_state.app_handle(), &tab.url);
    let _ = emit_tab_event(&app_state, "created", json!({ "tab": tab.clone() }));

    Ok(tab)
}

#[tauri::command]
pub async fn tabs_update(
    app_state: State<'_, AppState>,
    payload: UpdateTabPayload,
) -> Result<Option<Tab>, String> {
    let updated = TabService::update(app_state.db(), payload)
        .await
        .map(|res| res.map(Tab::from))
        .map_err(|e| e.to_string())?;

    if let Some(ref tab) = updated {
        let _ = emit_tab_event(&app_state, "updated", json!({ "tab": tab }));
    }

    Ok(updated)
}

#[tauri::command]
pub async fn tabs_activate(app_state: State<'_, AppState>, id: String) -> Result<Option<Tab>, String> {
    let tab = TabService::activate(app_state.db(), &id)
        .await
        .map(|res| res.map(Tab::from))
        .map_err(|e| e.to_string())?;

    if let Some(ref tab) = tab {
        let _ = window_manager::open_or_navigate_content_window(&app_state.app_handle(), &tab.url);
        let _ = emit_tab_event(&app_state, "activated", json!({ "tab": tab }));
    }

    Ok(tab)
}

#[tauri::command]
pub async fn tabs_close(app_state: State<'_, AppState>, id: String) -> Result<Option<Tab>, String> {
    let activated = TabService::close(app_state.db(), &id)
        .await
        .map(|res| res.map(Tab::from))
        .map_err(|e| e.to_string())?;

    let _ = emit_tab_event(&app_state, "closed", json!({ "id": id }));

    if let Some(ref tab) = activated {
        let _ = window_manager::open_or_navigate_content_window(&app_state.app_handle(), &tab.url);
        let _ = emit_tab_event(&app_state, "activated", json!({ "tab": tab }));
    }

    Ok(activated)
}

#[tauri::command]
pub async fn tabs_reorder(
    app_state: State<'_, AppState>,
    payload: ReorderTabsPayload,
) -> Result<(), String> {
    TabService::reorder(app_state.db(), payload)
        .await
        .map_err(|e| e.to_string())
        .map(|_| {
            let _ = emit_tab_event(
                &app_state,
                "reordered",
                json!({ "at": chrono::Utc::now().to_rfc3339() }),
            );
        })
}

#[tauri::command]
pub async fn tabs_activate_next(app_state: State<'_, AppState>) -> Result<Option<Tab>, String> {
    let tab = TabService::activate_next(app_state.db())
        .await
        .map(|res| res.map(Tab::from))
        .map_err(|e| e.to_string())?;

    if let Some(ref tab) = tab {
        let _ = window_manager::open_or_navigate_content_window(&app_state.app_handle(), &tab.url);
        let _ = emit_tab_event(&app_state, "activated", json!({ "tab": tab }));
    }

    Ok(tab)
}

#[tauri::command]
pub async fn tabs_activate_previous(app_state: State<'_, AppState>) -> Result<Option<Tab>, String> {
    let tab = TabService::activate_previous(app_state.db())
        .await
        .map(|res| res.map(Tab::from))
        .map_err(|e| e.to_string())?;

    if let Some(ref tab) = tab {
        let _ = window_manager::open_or_navigate_content_window(&app_state.app_handle(), &tab.url);
        let _ = emit_tab_event(&app_state, "activated", json!({ "tab": tab }));
    }

    Ok(tab)
}

#[tauri::command]
pub async fn tabs_close_active(app_state: State<'_, AppState>) -> Result<Option<Tab>, String> {
    let activated = TabService::close_active(app_state.db())
        .await
        .map(|res| res.map(Tab::from))
        .map_err(|e| e.to_string())?;

    let _ = emit_tab_event(&app_state, "closed", json!({ "active": true }));

    if let Some(ref tab) = activated {
        let _ = window_manager::open_or_navigate_content_window(&app_state.app_handle(), &tab.url);
        let _ = emit_tab_event(&app_state, "activated", json!({ "tab": tab }));
    }

    Ok(activated)
}

fn emit_tab_event(app_state: &AppState, action: &str, payload: serde_json::Value) -> tauri::Result<()> {
    app_state
        .app_handle()
        .emit("tabs-changed", json!({ "action": action, "data": payload }))
}
