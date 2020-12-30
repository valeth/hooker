#![allow(unused_variables)]

use bytes::Buf;
use crate::{
    router::{Context, Response},
    models::HookConfig,
    State,
};

pub async fn get_hooks(ctx: Context<State>) -> Response {
    let hook_configs = ctx.shared.hooks.read().await;
    let hooks: Vec<_> = hook_configs.values().collect();
    let json = serde_json::to_string(&hooks).unwrap();

    Response::builder()
        .header("Content-Type", "application/json")
        .body(json.into())
        .unwrap()
}

pub async fn put_hook(ctx: Context<State>) -> Response {
    let reader = hyper::body::aggregate(ctx.request).await.unwrap().reader();
    let hook_config: HookConfig = serde_json::from_reader(reader).unwrap();
    let hook_id = hook_config.id.clone();

    let mut hook_configs = ctx.shared.hooks.write().await;
    hook_configs.insert(hook_id, hook_config);

    Response::default()
}

pub async fn delete_hook(ctx: Context<State>) -> Response {
    let id = ctx.params.by_name("id").unwrap();
    let mut hook_configs = ctx.shared.hooks.write().await;
    hook_configs.remove(id).unwrap();

    Response::default()
}
