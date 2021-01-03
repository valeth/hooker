use bytes::Buf;
use crate::{
    models::CreateHookConfig,
    router::Context,
    http::{StatusCode, Response},
    store::HookConfig,
    Result,
    State,
};

macro_rules! require_auth {
    [$ctx:expr] => {
        if !is_authorized(&$ctx).await {
            log::error!("Failed to authorize user");
            return Ok(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("WWW-Authenticate", "Basic")
                .body("".into())?);
        }
    }
}

pub async fn get_hooks(ctx: Context<State>) -> Result<Response> {
    require_auth!(ctx);

    let hooks = ctx.shared.hooks.all().await;
    let json = serde_json::to_string(&hooks)?;

    let res = Response::builder()
        .header("Content-Type", "application/json")
        .body(json.into())?;

    Ok(res)
}

pub async fn put_hook(mut ctx: Context<State>) -> Result<Response> {
    require_auth!(ctx);

    let reader = hyper::body::aggregate(ctx.request).await?.reader();
    let config: CreateHookConfig =  serde_json::from_reader(reader)?;
    let config: HookConfig = config.into();

    let json = serde_json::to_string(&config)?;
    ctx.shared.hooks.insert(config).await?;

    let res = Response::builder()
        .header("Content-Type", "application/json")
        .body(json.into())?;

    Ok(res)
}

pub async fn delete_hook(mut ctx: Context<State>) -> Result<Response> {
    require_auth!(ctx);

    let id = ctx.params.by_name("id").expect("id parameter");
    ctx.shared.hooks.delete(id).await?;

    Ok(Response::default())
}

async fn is_authorized(ctx: &Context<State>) -> bool {
    if let Some(auth_header) = ctx.request.headers().get("Authorization") {
        let auth_header = auth_header.to_str().unwrap();
        let parts = auth_header.split(' ').collect::<Vec<_>>();

        if let &["Basic", credentials] = &parts[..] {
            let decoded = match decode_auth_header(credentials) {
                Ok(d) => d,
                Err(_) => {
                    log::error!("Failed to decode Authorization header");
                    return false;
                },
            };

            if let [username, password] = &decoded[..] {
                let users = ctx.shared.users.read().await;

                if let Some(hashed_pw) = users.get(username) {
                    return hashed_pw == &hexdigest(password);
                }
            }
        }
    }

    false
}

fn decode_auth_header<B: AsRef<[u8]>>(data: B) -> Result<Vec<String>>{
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
