use bytes::Buf;
use routerify::ext::RequestExt;
use crate::{
    models::CreateHookConfig,
    http::{StatusCode, Request, Response},
    store::HookConfig,
    Result,
    Users,
    HookRegistry,
};

macro_rules! require_auth {
    [$req:expr] => {
        if !is_authorized(&$req).await {
            log::error!("Failed to authorize user");
            return Ok(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("WWW-Authenticate", "Basic")
                .body("".into())?);
        }
    }
}

pub async fn get_hooks(req: Request) -> Result<Response> {
    require_auth!(req);

    let hooks = req.data::<HookRegistry>().unwrap();
    let hooks = hooks.read().await;
    let json = serde_json::to_string(&hooks.all().await)?;

    let res = Response::builder()
        .header("Content-Type", "application/json")
        .body(json.into())?;

    Ok(res)
}

pub async fn post_hook(mut req: Request) -> Result<Response> {
    require_auth!(req);

    let reader = hyper::body::aggregate(&mut req).await?.reader();
    let config: CreateHookConfig =  serde_json::from_reader(reader)?;
    let config: HookConfig = config.into();

    let json = serde_json::to_string(&config)?;
    let hooks = req.data::<HookRegistry>().unwrap();
    let mut hooks = hooks.write().await;
    hooks.insert(config).await?;

    let res = Response::builder()
        .header("Content-Type", "application/json")
        .body(json.into())?;

    Ok(res)
}

pub async fn delete_hook(req: Request) -> Result<Response> {
    require_auth!(req);

    let id = req.param("id").expect("id parameter");
    let hooks = req.data::<HookRegistry>().unwrap();
    let mut hooks = hooks.write().await;
    hooks.delete(&**id).await?;

    Ok(Response::default())
}

async fn is_authorized(req: &Request) -> bool {
    if let Some(auth_header) = req.headers().get("Authorization") {
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
                let users = req.data::<Users>().unwrap();
                let users = users.read().await;

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
