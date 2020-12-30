#![warn(rust_2018_idioms)]

mod routes;
mod router;

use std::{net::SocketAddr, sync::Arc, collections::HashMap};
use tokio::sync::RwLock;
use router::Router;
pub use hyper::http;

#[derive(Default, Clone)]
pub struct State {
    pub hooks: Arc<RwLock<HashMap<String, String>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use hyper::service::{make_service_fn, service_fn};

    initialize_logger()?;

    let state = initialize_state().await;

    let mut routes = Router::new();
    routes.get("/api/hooks", routes::api::get_hooks)
        .put("/api/hook", routes::api::put_hook)
        .delete("/api/hook", routes::api::delete_hook)
        .post("/hooks/gitlab/:id", routes::post_gitlab);

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

async fn initialize_state() -> State {
    let state = State::default();

    let mut hooks = state.hooks.write().await;
    hooks.insert("111".into(), "12345".into());
    drop(hooks);

    state
}

fn initialize_logger() -> Result<(), Box<dyn std::error::Error>> {
    use simplelog::{TermLogger, Config, TerminalMode};

    TermLogger::init(log::LevelFilter::Debug, Config::default(), TerminalMode::Mixed)?;

    log::info!("Logger initialized");
    Ok(())
}
