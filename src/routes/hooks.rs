use crate::{
    router::{Response, Context},
    http::StatusCode,
    State,
    Result,
};

pub async fn post_gitlab(ctx: Context<State>) -> Result<Response> {
    if let Err(e) = valid_token(&ctx).await {
        log::error!("GitLab token validation failed: {}", e);
        let res = Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("".into())?;
        return Ok(res);
    }

    if let Some(event) = ctx.request.headers().get("HTTP_X_GITLAB_EVENT") {
        let event = event.to_str().unwrap().to_string();
        let payload = hyper::body::aggregate(ctx.request).await.unwrap();
        tokio::spawn(handle_event(event, payload));
        return Ok(Response::default());
    }

    let res = Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body("".into())?;
    Ok(res)
}

async fn valid_token(ctx: &Context<State>) -> Result<()> {
    let id = ctx.params.by_name("id").unwrap();
    let hooks = ctx.shared.hooks.read().await;

    let hook_config = hooks
        .get(id)
        .ok_or_else(|| format!("No hook config found for id {}", id))?;

    let remote_token = ctx.request.headers()
        .get("HTTP_X_GITLAB_TOKEN")
        .ok_or_else(|| "Token header missing")?
        .to_str()
        .unwrap();

    if hook_config.gitlab_token != remote_token {
        Err("Invalid token".into())
    } else {
        Ok(())
    }
}

async fn handle_event(event: String, payload: impl bytes::Buf) {
    let _reader = payload.reader();
    log::debug!("event: {}", event);

    // TODO: implement event notifier
}
