#![allow(unused_variables)]

use bytes::Buf;
use crate::{
    router::{Context, Response},
    models::HookConfig,
    http::StatusCode,
    State,
};

macro_rules! require_auth {
    [$ctx:expr] => {
        if let Ok(false) | Err(_) = is_authorized(&$ctx).await {
            log::error!("Failed to authorize user");
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("WWW-Authenticate", "Basic")
                .body("".into())
                .unwrap()
        }
    }
}

pub async fn get_hooks(ctx: Context<State>) -> Response {
    require_auth!(ctx);

    let hook_configs = ctx.shared.hooks.read().await;
    let hooks: Vec<_> = hook_configs.values().collect();
    let json = serde_json::to_string(&hooks).unwrap();

    Response::builder()
        .header("Content-Type", "application/json")
        .body(json.into())
        .unwrap()
}

pub async fn put_hook(ctx: Context<State>) -> Response {
    require_auth!(ctx);

    let reader = hyper::body::aggregate(ctx.request).await.unwrap().reader();
    let hook_config: HookConfig = serde_json::from_reader(reader).unwrap();
    let hook_id = hook_config.id.clone();

    let mut hook_configs = ctx.shared.hooks.write().await;
    hook_configs.insert(hook_id, hook_config);

    Response::default()
}

pub async fn delete_hook(ctx: Context<State>) -> Response {
    require_auth!(ctx);

    let id = ctx.params.by_name("id").unwrap();
    let mut hook_configs = ctx.shared.hooks.write().await;
    hook_configs.remove(id).unwrap();

    Response::default()
}

async fn is_authorized(ctx: &Context<State>) -> Result<bool, Box<dyn std::error::Error>> {
    if let Some(auth_header) = ctx.request.headers().get("Authorization") {
        let auth_header = auth_header.to_str().unwrap();
        let parts = auth_header.split(' ').collect::<Vec<_>>();

        if let &["Basic", credentials] = &parts[..] {
            let decoded = decode_auth_header(credentials)?;

            if let [username, password] = &decoded[..] {
                let users = ctx.shared.users.read().await;

                if let Some(hashed_pw) = users.get(username) {
                    return Ok(hashed_pw == &hexdigest(password));
                }
            }
        }
    }

    Ok(false)
}

fn decode_auth_header<B: AsRef<[u8]>>(data: B) -> Result<Vec<String>, Box<dyn std::error::Error>>{
    Ok(base64::decode(data)?
        .split(|&x| x == 0x3A)
        .map(|v| {
            String::from_utf8(v.to_vec()).unwrap()
        })
        .collect::<Vec<_>>())
}

fn hexdigest<B: AsRef<[u8]>>(data: B) -> String {
    use sha2::{Sha256, Digest};
    let digest = Sha256::digest(data.as_ref());
    hex::encode(digest)
}
