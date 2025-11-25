use serde::{Deserialize, Serialize};

use crate::features::tab::data::entity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: String,
    pub title: String,
    pub url: String,
    pub favicon_url: Option<String>,
    pub is_pinned: bool,
    pub is_active: bool,
    pub sort_order: i32,
    pub last_opened_at: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<entity::Model> for Tab {
    fn from(model: entity::Model) -> Self {
        Self {
            id: model.id,
            title: model.title,
            url: model.url,
            favicon_url: model.favicon_url,
            is_pinned: model.is_pinned,
            is_active: model.is_active,
            sort_order: model.sort_order,
            last_opened_at: model.last_opened_at.to_rfc3339(),
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTabPayload {
    pub url: String,
    pub title: Option<String>,
    pub favicon_url: Option<String>,
    pub is_pinned: Option<bool>,
    /// 是否在创建时激活标签页，默认 true。
    pub activate: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTabPayload {
    pub id: String,
    pub title: Option<String>,
    pub url: Option<String>,
    pub favicon_url: Option<String>,
    pub is_pinned: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReorderTabsPayload {
    pub ordered_ids: Vec<String>,
}
