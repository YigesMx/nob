use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use sea_orm::DatabaseConnection;
use tauri::{AppHandle, Wry};

use crate::core::Feature;
use crate::infrastructure::tray::TrayManager;

/// 应用全局状态
///
/// 管理所有 Features 和基础设施组件
pub struct AppState {
    app_handle: AppHandle<Wry>,
    db: DatabaseConnection,
    features: HashMap<&'static str, Arc<dyn Feature>>,

    // 系统托盘管理器
    tray_manager: TrayManager,

    // 当前主题缓存 (用于同步访问，如托盘菜单)
    current_theme: Mutex<String>,
}

impl AppState {
    pub fn new(
        app_handle: AppHandle<Wry>,
        db: DatabaseConnection,
        features: Vec<Arc<dyn Feature>>,
    ) -> Self {
        let mut feature_map = HashMap::new();
        for feature in features {
            feature_map.insert(feature.name(), feature);
        }

        Self {
            app_handle,
            db,
            features: feature_map,
            tray_manager: TrayManager::new(),
            current_theme: Mutex::new("system".to_string()),
        }
    }

    /// 获取数据库连接
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    /// 获取 AppHandle
    pub fn app_handle(&self) -> AppHandle<Wry> {
        self.app_handle.clone()
    }

    /// 获取 TrayManager
    pub fn tray_manager(&self) -> &TrayManager {
        &self.tray_manager
    }

    /// 获取当前主题
    pub fn get_theme(&self) -> String {
        self.current_theme.lock().unwrap().clone()
    }

    /// 设置当前主题
    pub fn set_theme(&self, theme: String) {
        *self.current_theme.lock().unwrap() = theme;
    }

    /// 根据名称获取 Feature
    pub fn get_feature(&self, name: &str) -> Option<&Arc<dyn Feature>> {
        self.features.get(name)
    }

    /// 获取所有 Features
    pub fn features(&self) -> &HashMap<&'static str, Arc<dyn Feature>> {
        &self.features
    }

    /// 设置托盘注册表
    pub fn set_tray_registry(&mut self, registry: crate::infrastructure::tray::TrayRegistry) {
        self.tray_manager.set_registry(registry);
    }

    /// 后初始化阶段（在 app.manage() 之后调用）
    ///
    /// 此时 AppState 已经被 Tauri 托管，可以通过 app.try_state() 访问。
    /// 执行需要访问已托管状态的初始化逻辑。
    pub async fn post_initialize(&self, app: &AppHandle<Wry>) -> anyhow::Result<()> {
        // 初始化主题缓存
        use crate::features::settings::core::service::SettingService;
        let theme = SettingService::get_or_default(self.db(), "ui.theme", "system").await?;
        self.set_theme(theme);

        // 创建系统托盘
        self.tray_manager
            .create_tray(app)
            .map_err(|e| anyhow::anyhow!("Failed to create tray: {}", e))?;

        Ok(())
    }
}
