use crate::{
    router::{Response, Context},
    http::StatusCode,
    State,
};

pub async fn post_gitlab(ctx: Context<State>) -> Response {
    if let Err(_) = valid_token(&ctx).await {
        log::error!("GitLab token validation failed");
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("".into())
            .unwrap();
    }

    if let Some(event) = ctx.request.headers().get("HTTP_X_GITLAB_EVENT") {
        let event = event.to_str().unwrap().to_string();
        let payload = hyper::body::aggregate(ctx.request).await.unwrap();
        tokio::spawn(handle_event(event, payload));
        return Response::default();
    }

    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body("".into())
        .unwrap()
}

async fn valid_token(ctx: &Context<State>) -> Result<(), Box<dyn std::error::Error>> {
    let id = ctx.params.by_name("id").unwrap();
    let hooks = ctx.shared.hooks.read().await;

    let local_token = hooks.get(id).unwrap();
    let remote_token = ctx.request.headers().get("HTTP_X_GITLAB_TOKEN").unwrap().to_str()?;

    if local_token != remote_token {
        return Err("Invalid token".into());
    }

    Ok(())
}

async fn handle_event(event: String, payload: impl bytes::Buf) {
    let _reader = payload.reader();
    log::debug!("event: {}", event);

    // TODO: implement event notifier
}
