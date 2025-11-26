use anyhow::Context;
use serde_json::json;

use crate::infrastructure::webserver::{HandlerRegistry, WsMessage};

use crate::features::tab::core::models::{
    CreateTabPayload, ReorderTabsPayload, Tab, UpdateTabPayload,
};
use crate::features::tab::core::service::TabService;
use crate::infrastructure::webserver::core::ws::ApiContext;

/// 注册 Tab Feature 的 WebSocket handlers
pub fn register_handlers(registry: &mut HandlerRegistry) {
    // 声明可订阅事件
    registry.register_event("tabs.created", "标签页创建");
    registry.register_event("tabs.updated", "标签页更新");
    registry.register_event("tabs.activated", "标签页切换");
    registry.register_event("tabs.closed", "标签页关闭");
    registry.register_event("tabs.reordered", "标签页重排");

    // 列表
    registry.register_call("tabs.list", |_method, _params, ctx| {
        Box::pin(async move {
            let tabs = TabService::list(ctx.db()).await?;
            Ok(json!({ "tabs": tabs.into_iter().map(Tab::from).collect::<Vec<_>>() }))
        })
    });

    // 创建
    registry.register_call("tabs.create", |_method, params, ctx| {
        Box::pin(async move {
            let payload: CreateTabPayload = serde_json::from_value(params)
                .context("invalid payload for tabs.create")?;
            let tab = TabService::create(ctx.db(), payload).await?;
            let tab_dto = Tab::from(tab);
            broadcast_tab(&ctx, "tabs.created", &tab_dto).await;
            Ok(json!({ "tab": tab_dto }))
        })
    });

    // 更新
    registry.register_call("tabs.update", |_method, params, ctx| {
        Box::pin(async move {
            let payload: UpdateTabPayload = serde_json::from_value(params)
                .context("invalid payload for tabs.update")?;
            let updated = TabService::update(ctx.db(), payload).await?;
            if let Some(tab) = updated {
                let tab_dto = Tab::from(tab);
                broadcast_tab(&ctx, "tabs.updated", &tab_dto).await;
                Ok(json!({ "tab": tab_dto }))
            } else {
                Ok(json!({ "tab": Option::<Tab>::None }))
            }
        })
    });

    // 激活
    registry.register_call("tabs.activate", |_method, params, ctx| {
        Box::pin(async move {
            let id: String = serde_json::from_value(params)
                .context("invalid params for tabs.activate (expected id string)")?;
            let activated = TabService::activate(ctx.db(), &id).await?;
            if let Some(tab) = activated {
                let tab_dto = Tab::from(tab);
                broadcast_tab(&ctx, "tabs.activated", &tab_dto).await;
                Ok(json!({ "tab": tab_dto }))
            } else {
                Ok(json!({ "tab": Option::<Tab>::None }))
            }
        })
    });

    // 关闭
    registry.register_call("tabs.close", |_method, params, ctx| {
        Box::pin(async move {
            let id: String = serde_json::from_value(params)
                .context("invalid params for tabs.close (expected id string)")?;
            let activated = TabService::close(ctx.db(), &id).await?;
            broadcast_simple(&ctx, "tabs.closed", json!({ "id": id })).await;
            if let Some(tab) = activated {
                let tab_dto = Tab::from(tab);
                broadcast_tab(&ctx, "tabs.activated", &tab_dto).await;
                Ok(json!({ "activated": tab_dto }))
            } else {
                Ok(json!({ "activated": Option::<Tab>::None }))
            }
        })
    });

    // 重排
    registry.register_call("tabs.reorder", |_method, params, ctx| {
        Box::pin(async move {
            let payload: ReorderTabsPayload = serde_json::from_value(params)
                .context("invalid payload for tabs.reorder")?;
            TabService::reorder(ctx.db(), payload.clone()).await?;
            broadcast_simple(
                &ctx,
                "tabs.reordered",
                json!({ "ordered_ids": payload.ordered_ids }),
            )
            .await;
            Ok(json!({ "ok": true }))
        })
    });

    // 下一/上一
    registry.register_call("tabs.activate_next", |_method, _params, ctx| {
        Box::pin(async move {
            let result = TabService::activate_next(ctx.db()).await?;
            if let Some(tab) = result {
                let tab_dto = Tab::from(tab);
                broadcast_tab(&ctx, "tabs.activated", &tab_dto).await;
                Ok(json!({ "tab": tab_dto }))
            } else {
                Ok(json!({ "tab": Option::<Tab>::None }))
            }
        })
    });

    registry.register_call("tabs.activate_previous", |_method, _params, ctx| {
        Box::pin(async move {
            let result = TabService::activate_previous(ctx.db()).await?;
            if let Some(tab) = result {
                let tab_dto = Tab::from(tab);
                broadcast_tab(&ctx, "tabs.activated", &tab_dto).await;
                Ok(json!({ "tab": tab_dto }))
            } else {
                Ok(json!({ "tab": Option::<Tab>::None }))
            }
        })
    });

    // 关闭当前
    registry.register_call("tabs.close_active", |_method, _params, ctx| {
        Box::pin(async move {
            let activated = TabService::close_active(ctx.db()).await?;
            broadcast_simple(&ctx, "tabs.closed", json!({ "active": true })).await;
            if let Some(tab) = activated {
                let tab_dto = Tab::from(tab);
                broadcast_tab(&ctx, "tabs.activated", &tab_dto).await;
                Ok(json!({ "activated": tab_dto }))
            } else {
                Ok(json!({ "activated": Option::<Tab>::None }))
            }
        })
    });
}

async fn broadcast_tab(ctx: &ApiContext, channel: &str, tab: &Tab) {
    if let Ok(payload) = serde_json::to_value(tab) {
        broadcast_simple(ctx, channel, payload).await;
    }
}

async fn broadcast_simple(ctx: &ApiContext, channel: &str, payload: serde_json::Value) {
    let message = WsMessage::event(channel.to_string(), payload);
    ctx.connection_manager()
        .broadcast_to_channel(&channel.to_string(), message)
        .await;
}
