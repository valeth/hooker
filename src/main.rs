#![warn(rust_2018_idioms)]
#![deny(unused_must_use)]

mod http;
mod routes;
mod models;
mod store;

use std::{net::SocketAddr, collections::HashMap};
use tokio::sync::RwLock;
use routerify::{Router, RouterService};
pub use anyhow::Result;

pub type UserMap = HashMap<String, String>;
pub type Users = RwLock<UserMap>;
pub type HookRegistry = RwLock<store::HookRegistry>;

#[derive(Debug, argh::FromArgs)]
/// GitLab to Discord webhook server
struct AppArgs {
    #[argh(option, from_str_fn(parse_user))]
    /// an colon separated pair of user and hashed password
    user: Vec<(String, String)>,

    #[argh(switch)]
    /// enables debug logging
    debug: bool,
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
async fn main() -> Result<()> {
    use hyper::Server;

    let args: AppArgs = argh::from_env();

    initialize_logger(args.debug)?;

    let users = args.user.into_iter().collect::<UserMap>();
    let client = http::Client::new();
    let hooks = store::HookRegistry::load()?;

    let router = Router::builder()
        .data(RwLock::new(users))
        .data(RwLock::new(hooks))
        .data(client)
        .get("/api/hooks", routes::api::get_hooks)
        .post("/api/hook", routes::api::post_hook)
        .delete("/api/hook/:id", routes::api::delete_hook)
        .post("/hooks/gitlab/:id", routes::hooks::post_gitlab)
        .build()?;

    let addr: SocketAddr = ([0, 0, 0, 0], 9292).into();
    let service = RouterService::new(router)?;
    let server = Server::try_bind(&addr)?.serve(service);

    log::info!("Starting server on http://{}", addr);
    server.await?;

    Ok(())
}

fn initialize_logger(debug: bool) -> Result<()> {
    use simplelog::{TermLogger, Config, TerminalMode};

    let log_level = if debug {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    TermLogger::init(log_level, Config::default(), TerminalMode::Mixed)?;

    log::info!("Logger initialized");
    Ok(())
}
