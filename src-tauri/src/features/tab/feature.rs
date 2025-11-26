use anyhow::Result;
use async_trait::async_trait;
use sea_orm_migration::MigrationTrait;

use crate::core::{AppState, Feature};
use crate::infrastructure::database::DatabaseRegistry;
use crate::infrastructure::webserver::HandlerRegistry;
use crate::features::tab::core::service::TabService;
use crate::features::window::manager as window_manager;

use super::data::migration::TabMigration;

/// Tab Feature - 负责管理浏览器标签页的核心能力。
/// 后续会扩展数据库迁移、Tauri commands、WebSocket handlers 等。
pub struct TabFeature;

impl TabFeature {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Feature for TabFeature {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn name(&self) -> &'static str {
        "tab"
    }

    async fn initialize(&self, app_state: &AppState) -> Result<()> {
        // 初始化时，获取当前激活的 Tab 并设置到 Window Manager
        if let Ok(Some(tab)) = TabService::get_active(app_state.db()).await {
            window_manager::set_current_url(tab.url);
        }
        Ok(())
    }

    fn register_database(&self, registry: &mut DatabaseRegistry) {
        registry.register_migration("tabs_migration", |manager| {
            let migration = TabMigration;
            Box::pin(async move { migration.up(manager).await })
        });
    }

    fn command_names(&self) -> Vec<&'static str> {
        vec![
            "tabs_list",
            "tabs_create",
            "tabs_update",
            "tabs_activate",
            "tabs_close",
            "tabs_reorder",
            "tabs_activate_next",
            "tabs_activate_previous",
            "tabs_close_active",
        ]
    }

    fn register_ws_handlers(&self, registry: &mut HandlerRegistry) {
        super::api::handlers::register_handlers(registry);
    }
}
