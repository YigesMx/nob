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
    let _ = window_manager::present_content_window(&app_state.app_handle(), Some(&tab.url));
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
    // 获取当前缓存的 URL，用于判断是否需要导航
    let current_cached_url = window_manager::get_current_url();

    let tab = TabService::activate(app_state.db(), &id)
        .await
        .map(|res| res.map(Tab::from))
        .map_err(|e| e.to_string())?;

    if let Some(ref tab) = tab {
        // 如果 URL 相同，传入 None 以避免刷新；否则传入 Some(url) 进行导航
        let should_navigate = match current_cached_url {
            Some(ref cached) => cached != &tab.url,
            None => true,
        };
        let url_arg = if should_navigate { Some(tab.url.as_str()) } else { None };

        let _ = window_manager::present_content_window(&app_state.app_handle(), url_arg);
        let _ = emit_tab_event(&app_state, "activated", json!({ "tab": tab }));
    } else {
        // 如果 activate 返回 None，可能是因为已经是 active 状态
        // 这种情况下，我们仍然需要确保窗口显示（解决再次点击无法打开的问题）
        if let Ok(Some(current_tab)) = TabService::get(app_state.db(), &id).await {
             let tab = Tab::from(current_tab);
             
             let should_navigate = match current_cached_url {
                Some(ref cached) => cached != &tab.url,
                None => true,
            };
            let url_arg = if should_navigate { Some(tab.url.as_str()) } else { None };

             let _ = window_manager::present_content_window(&app_state.app_handle(), url_arg);
        }
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
        let _ = window_manager::present_content_window(&app_state.app_handle(), Some(&tab.url));
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
