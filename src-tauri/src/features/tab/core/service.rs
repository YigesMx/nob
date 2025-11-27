use anyhow::Result;
use chrono::Utc;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use uuid::Uuid;

use crate::features::tab::core::models::{CreateTabPayload, ReorderTabsPayload, UpdateTabPayload};
use crate::features::tab::data::entity::{self, Entity as TabEntity, Model};

pub struct TabService;

impl TabService {
    pub async fn list(db: &DatabaseConnection) -> Result<Vec<Model>> {
        let tabs = TabEntity::find()
            .order_by_desc(entity::Column::IsPinned)
            .order_by_asc(entity::Column::SortOrder)
            .order_by_desc(entity::Column::LastOpenedAt)
            .all(db)
            .await?;

        Ok(tabs)
    }

    pub async fn create(db: &DatabaseConnection, payload: CreateTabPayload) -> Result<Model> {
        let txn = db.begin().await?;
        let now = Utc::now();
        let next_order = Self::next_sort_order(&txn).await?;
        let should_activate = payload.activate.unwrap_or(true);

        if should_activate {
            Self::deactivate_all(&txn).await?;
        }

        let active_model = entity::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            title: Set(payload.title.unwrap_or_else(|| payload.url.clone())),
            url: Set(payload.url.clone()),
            initial_url: Set(payload.url),
            favicon_url: Set(payload.favicon_url),
            is_pinned: Set(payload.is_pinned.unwrap_or(false)),
            is_active: Set(should_activate),
            sort_order: Set(next_order),
            last_opened_at: Set(now),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let created = active_model.insert(&txn).await?;
        txn.commit().await?;

        Ok(created)
    }

    pub async fn update(db: &DatabaseConnection, payload: UpdateTabPayload) -> Result<Option<Model>> {
        let existing = TabEntity::find_by_id(payload.id.clone()).one(db).await?;
        if let Some(model) = existing {
            let mut active_model: entity::ActiveModel = model.into();
            if let Some(title) = payload.title {
                active_model.title = Set(title);
            }
            if let Some(url) = payload.url {
                active_model.url = Set(url);
            }
            if let Some(favicon_url) = payload.favicon_url {
                active_model.favicon_url = Set(Some(favicon_url));
            }
            if let Some(is_pinned) = payload.is_pinned {
                active_model.is_pinned = Set(is_pinned);
            }
            active_model.updated_at = Set(Utc::now());

            let updated = active_model.update(db).await?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    pub async fn activate(db: &DatabaseConnection, id: &str) -> Result<Option<Model>> {
        let txn = db.begin().await?;
        let tab = TabEntity::find_by_id(id.to_string()).one(&txn).await?;

        if let Some(model) = tab {
            Self::deactivate_all(&txn).await?;

            let mut active_model: entity::ActiveModel = model.into();
            active_model.is_active = Set(true);
            active_model.last_opened_at = Set(Utc::now());
            active_model.updated_at = Set(Utc::now());

            let updated = active_model.update(&txn).await?;
            txn.commit().await?;
            Ok(Some(updated))
        } else {
            txn.rollback().await.ok();
            Ok(None)
        }
    }

    pub async fn update_active_url(db: &DatabaseConnection, url: String) -> Result<Option<Model>> {
        let txn = db.begin().await?;
        // Find the currently active tab
        let active_tab = TabEntity::find()
            .filter(entity::Column::IsActive.eq(true))
            .one(&txn)
            .await?;

        if let Some(model) = active_tab {
            let mut active_model: entity::ActiveModel = model.into();
            active_model.url = Set(url);
            active_model.updated_at = Set(Utc::now());

            let updated = active_model.update(&txn).await?;
            txn.commit().await?;
            Ok(Some(updated))
        } else {
            txn.rollback().await.ok();
            Ok(None)
        }
    }

    pub async fn update_active_title(db: &DatabaseConnection, title: String) -> Result<Option<Model>> {
        let txn = db.begin().await?;
        // Find the currently active tab
        let active_tab = TabEntity::find()
            .filter(entity::Column::IsActive.eq(true))
            .one(&txn)
            .await?;

        if let Some(model) = active_tab {
            let mut active_model: entity::ActiveModel = model.into();
            active_model.title = Set(title);
            active_model.updated_at = Set(Utc::now());

            let updated = active_model.update(&txn).await?;
            txn.commit().await?;
            Ok(Some(updated))
        } else {
            txn.rollback().await.ok();
            Ok(None)
        }
    }

    /// 关闭指定标签，返回是否激活了新的标签
    pub async fn close(
        db: &DatabaseConnection,
        id: &str,
    ) -> Result<Option<Model>> {
        let txn = db.begin().await?;
        let tab = TabEntity::find_by_id(id.to_string()).one(&txn).await?;

        if let Some(model) = tab {
            let was_active = model.is_active;

            TabEntity::delete_by_id(id.to_string())
                .exec(&txn)
                .await?;

            let activated = if was_active {
                Self::activate_next_available(&txn).await?
            } else {
                None
            };

            txn.commit().await?;
            Ok(activated)
        } else {
            txn.rollback().await.ok();
            Ok(None)
        }
    }

    pub async fn reorder(db: &DatabaseConnection, payload: ReorderTabsPayload) -> Result<()> {
        let txn = db.begin().await?;

        for (idx, tab_id) in payload.ordered_ids.iter().enumerate() {
            TabEntity::update_many()
                .col_expr(entity::Column::SortOrder, Expr::value(idx as i32))
                .filter(entity::Column::Id.eq(tab_id.clone()))
                .exec(&txn)
                .await?;
        }

        txn.commit().await?;
        Ok(())
    }

    pub async fn activate_next(db: &DatabaseConnection) -> Result<Option<Model>> {
        Self::activate_adjacent(db, true).await
    }

    pub async fn activate_previous(db: &DatabaseConnection) -> Result<Option<Model>> {
        Self::activate_adjacent(db, false).await
    }

    pub async fn close_active(db: &DatabaseConnection) -> Result<Option<Model>> {
        if let Some(active) = TabEntity::find()
            .filter(entity::Column::IsActive.eq(true))
            .one(db)
            .await?
        {
            Self::close(db, &active.id).await
        } else {
            Ok(None)
        }
    }

    pub async fn get(db: &DatabaseConnection, id: &str) -> Result<Option<Model>> {
        let tab = TabEntity::find_by_id(id.to_string()).one(db).await?;
        Ok(tab)
    }

    pub async fn get_active(db: &DatabaseConnection) -> Result<Option<Model>> {
        let tab = TabEntity::find()
            .filter(entity::Column::IsActive.eq(true))
            .one(db)
            .await?;
        Ok(tab)
    }

    async fn activate_adjacent(db: &DatabaseConnection, forward: bool) -> Result<Option<Model>> {
        let txn = db.begin().await?;
        let tabs = TabEntity::find()
            .order_by_desc(entity::Column::IsPinned)
            .order_by_asc(entity::Column::SortOrder)
            .order_by_desc(entity::Column::LastOpenedAt)
            .all(&txn)
            .await?;

        if tabs.is_empty() {
            txn.rollback().await.ok();
            return Ok(None);
        }

        let current_idx = tabs.iter().position(|t| t.is_active);
        let next_idx = match current_idx {
            Some(idx) if forward => (idx + 1) % tabs.len(),
            Some(idx) if !forward => {
                if idx == 0 {
                    tabs.len() - 1
                } else {
                    idx - 1
                }
            }
            None => 0,
            _ => 0,
        };

        let target = tabs.get(next_idx).cloned();

        if let Some(target_tab) = target {
            Self::deactivate_all(&txn).await?;

            let mut active_model: entity::ActiveModel = target_tab.into();
            active_model.is_active = Set(true);
            active_model.last_opened_at = Set(Utc::now());
            active_model.updated_at = Set(Utc::now());
            let updated = active_model.update(&txn).await?;

            txn.commit().await?;
            Ok(Some(updated))
        } else {
            txn.rollback().await.ok();
            Ok(None)
        }
    }

    async fn deactivate_all<C>(conn: &C) -> Result<()>
    where
        C: ConnectionTrait,
    {
        TabEntity::update_many()
            .col_expr(entity::Column::IsActive, Expr::value(false))
            .exec(conn)
            .await?;
        Ok(())
    }

    async fn next_sort_order<C>(conn: &C) -> Result<i32>
    where
        C: ConnectionTrait,
    {
        let last = TabEntity::find()
            .order_by_desc(entity::Column::SortOrder)
            .one(conn)
            .await?;

        Ok(last.map(|t| t.sort_order + 1).unwrap_or(0))
    }

    async fn activate_next_available(txn: &DatabaseTransaction) -> Result<Option<Model>> {
        if let Some(next) = TabEntity::find()
            .order_by_desc(entity::Column::IsPinned)
            .order_by_asc(entity::Column::SortOrder)
            .order_by_desc(entity::Column::LastOpenedAt)
            .one(txn)
            .await?
        {
            let mut active_model: entity::ActiveModel = next.into();
            active_model.is_active = Set(true);
            active_model.last_opened_at = Set(Utc::now());
            active_model.updated_at = Set(Utc::now());
            let updated = active_model.update(txn).await?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }
}
