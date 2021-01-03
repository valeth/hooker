#![warn(rust_2018_idioms)]

mod http;
mod routes;
mod router;
mod models;

use std::{net::SocketAddr, sync::Arc, collections::HashMap};
use tokio::sync::RwLock;
use router::Router;
use models::HookConfig;
pub use anyhow::Result;

#[derive(Default, Clone)]
pub struct State {
    pub hooks: Arc<RwLock<HashMap<String, HookConfig>>>,
    pub users: Arc<RwLock<HashMap<String, String>>>,
}

#[derive(Debug, argh::FromArgs)]
/// GitLab to Discord webhook server
struct AppArgs {
    #[argh(option, from_str_fn(parse_user))]
    /// an colon separated pair of user and hashed password
    user: Vec<(String, String)>
}

fn parse_user(value: &str) -> Result<(String, String), String> {
    let parts = value.split(':').collect::<Vec<_>>();
    if let [username, password] = parts[..] {
        if password.len() != 64 {
            Err("Password hash has invalid length".into())
        } else {
            Ok((username.to_string(), password.to_string()))
        }
    } else {
        Err("Invalid format, needs to be user:pass".into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use hyper::service::{make_service_fn, service_fn};

    let args: AppArgs = argh::from_env();

    initialize_logger()?;

    let state = State::default();
    let mut users = state.users.write().await;
    users.extend(args.user);
    drop(users);

    let client = http::Client::new();
    let mut routes = Router::new(client);
    routes.get("/api/hooks", routes::api::get_hooks)
        .put("/api/hook", routes::api::put_hook)
        .delete("/api/hook/:id", routes::api::delete_hook)
        .post("/hooks/gitlab/:id", routes::hooks::post_gitlab);

    let routes = Arc::new(routes);

    let make_service = make_service_fn(move |_conn| {
        let routes = routes.clone();
        let state = state.clone();
        async {
            let service = service_fn(move |req| {
                let routes = routes.clone();
                let state = state.clone();
                async move {
                    let response = routes.handle_route(req, state).await;
                    Ok::<_, hyper::Error>(response)
                }
            });
            Ok::<_, hyper::Error>(service)
        }
    });

    let addr: SocketAddr = ([0, 0, 0, 0], 9292).into();
    let server = hyper::Server::try_bind(&addr)?.serve(make_service);
    log::info!("Starting server on http://{}", addr);
    server.await?;

    Ok(())
}

fn initialize_logger() -> Result<(), Box<dyn std::error::Error>> {
    use simplelog::{TermLogger, Config, TerminalMode};

    TermLogger::init(log::LevelFilter::Debug, Config::default(), TerminalMode::Mixed)?;

    log::info!("Logger initialized");
    Ok(())
}
