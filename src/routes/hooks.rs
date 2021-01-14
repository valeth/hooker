use anyhow::{anyhow, bail};
use bytes::Buf;
use crate::{
    router::Context,
    http::{StatusCode, Response},
    models::discord::Embed,
    State,
    Result,
};

const GITLAB_EVENT_HEADER: &str = "X-Gitlab-Event";
const GITLAB_TOKEN_HEADER: &str = "X-Gitlab-Token";
const DISCORD_RATELIMIT_RESET_HEADER: &str = "X-RateLimit-Reset";

pub async fn post_gitlab(ctx: Context<State>) -> Result<Response> {
    if let Err(e) = valid_token(&ctx).await {
        log::error!("GitLab token validation failed: {}", e);
        let res = Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("".into())?;
        return Ok(res);
    }

    if let Some(event) = ctx.request.headers().get(GITLAB_EVENT_HEADER) {
        let event = event.to_str().unwrap().to_string();
        tokio::spawn(handle_event(ctx, event));
        return Ok(Response::default());
    }

    let res = Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body("".into())?;
    Ok(res)
}

async fn valid_token(ctx: &Context<State>) -> Result<()> {
    let id = ctx.params.by_name("id").expect("id parameter");
    let hook_config = ctx.shared.hooks.get(id).await?;

    let remote_token = ctx.request.headers()
        .get(GITLAB_TOKEN_HEADER)
        .ok_or_else(|| anyhow!("Token header missing"))?
        .to_str()
        .unwrap();

    if hook_config.gitlab_token != remote_token {
        bail!("Invalid token")
    } else {
        Ok(())
    }
}

async fn handle_event(ctx: Context<State>, event: String) {
    let payload = hyper::body::aggregate(ctx.request).await.unwrap();

    let result = match &*event {
        "Push Hook" => handle_push_hook(payload).await,
        "Issue Hook" => handle_issue_hook(payload).await,
        "Merge Request Hook" => handle_merge_request_hook(payload).await,
        "Pipeline Hook" => handle_pipeline_hook(payload).await,
        _ => {
            log::debug!("Received unhandled event {}", event);
            return;
        },
    };

    match result {
        Ok(Some(embed)) => {
            let id = ctx.params.by_name("id").unwrap();
            let hook_config = ctx.shared.hooks.get(id).await.unwrap();
            let uri = &hook_config.discord_url;

            log::debug!("{:#?}", embed);

            let payload = serde_json::json!({ "embeds": [&embed] });
            let json = serde_json::to_string(&payload).unwrap();

            'retry: loop {
                match ctx.client.post(uri, json.clone()).await {
                    Err(err) => log::error!("{}", err),
                    Ok(res) if res.status() == StatusCode::TOO_MANY_REQUESTS => {
                        let reset_secs = res.headers()
                            .get(DISCORD_RATELIMIT_RESET_HEADER).unwrap()
                            .to_str().unwrap()
                            .parse().unwrap();
                        let reset_time = std::time::Duration::from_secs_f64(reset_secs);
                        log::warn!("Reached Discord rate limit, reset in {} seconds", reset_time.as_secs());
                        tokio::time::sleep(reset_time).await;
                        continue 'retry;
                    },
                    Ok(res) if res.status().is_client_error() => {
                        log::error!("Headers:\n{:#?}", res.headers());
                        let buf = hyper::body::aggregate(res).await.unwrap();
                        let reader = buf.reader();
                        let json: serde_json::Value = serde_json::from_reader(reader).unwrap();
                        log::error!("Response Payload:\n{:?}", json);
                    },
                    Ok(_) => ()
                }

                break
            }
        },
        Err(err) => log::error!("{}", err),
        _ => (),
    }
}

async fn handle_push_hook(payload: impl bytes::Buf) -> Result<Option<Embed>> {
    use crate::models::gitlab::PushEvent;

    let reader = payload.reader();
    let event: PushEvent = serde_json::from_reader(reader)?;

    if event.total_commits_count != 0 {
        return Ok(Some(event.into()))
    }

    Ok(None)
}

async fn handle_issue_hook(payload: impl bytes::Buf) -> Result<Option<Embed>> {
    use crate::models::gitlab::IssueEvent;

    let reader = payload.reader();
    let event: IssueEvent = serde_json::from_reader(reader)?;

    if let "open" | "close" = &*event.attributes.action {
        return Ok(Some(event.into()))
    }

    Ok(None)
}

async fn handle_merge_request_hook(payload: impl bytes::Buf) -> Result<Option<Embed>> {
    use crate::models::gitlab::MergeRequestEvent;

    let reader = payload.reader();
    let event: MergeRequestEvent = serde_json::from_reader(reader)?;

    if let "open" | "close" | "merge" = &*event.attributes.action {
        return Ok(Some(event.into()))
    }

    Ok(None)
}

async fn handle_pipeline_hook(payload: impl bytes::Buf) -> Result<Option<Embed>> {
    use crate::models::gitlab::PipelineEvent;

    let reader = payload.reader();
    let event: PipelineEvent = serde_json::from_reader(reader)?;

    if let "success" | "failed" = &*event.attributes.status {
        return Ok(Some(event.into()))
    }

    Ok(None)
}
