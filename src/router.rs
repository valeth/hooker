use std::{
    collections::HashMap,
    sync::Arc
};
use async_trait::async_trait;
use crate::{
    http::{self, Request, Response, HttpMethod, Params},
    Result
};


pub struct Context<T>
where T: Send + Sync
{
    pub request: Request,
    pub params: Params,
    pub client: Arc<http::Client>,
    pub shared: T
}

#[async_trait]
pub trait RouteHandler<C>
where C: Send + Sync + 'static
{
    async fn call(&self, ctx: Context<C>) -> Result<Response>;
}

#[async_trait]
impl<F, T, C, O> RouteHandler<C> for T
where T: Send + Sync + 'static,
      C: Send + Sync + 'static,
      T: Fn(Context<C>) -> F,
      F: std::future::Future<Output = Result<O>> + Send + 'static,
      O: Into<Response> + 'static
{
    async fn call(&self, ctx: Context<C>) -> Result<Response> {
        Ok(self(ctx).await?.into())
    }
}

pub struct Router<C>
where C: Send + Sync + 'static
{
    client: Arc<http::Client>,
    routes: HashMap<HttpMethod, matchit::Node<Box<dyn RouteHandler<C> + Send + Sync>>>,
}

impl<C> Router<C>
where C: Send + Sync + 'static
{
    const METHODS: [HttpMethod; 4] = [HttpMethod::GET, HttpMethod::POST, HttpMethod::DELETE, HttpMethod::PUT];

    pub fn new(client: http::Client) -> Self {
        let mut routes = HashMap::new();
        for method in Self::METHODS.iter() {
            routes.insert(method.clone(), matchit::Node::default());
        }

        Self {
            client: Arc::new(client),
            routes
        }
    }

    pub async fn handle_route(&self, request: Request, shared: C) -> Response {
        use hyper::{StatusCode, Body};

        let method = request.method();

        if let Some(method_handler) = self.routes.get(method) {
            let path = request.uri().path();

            if let Ok(path_handler) = method_handler.match_path(path) {
                log::debug!("{} {}", method, path);
                let ctx = Context {
                    request,
                    params: path_handler.params,
                    shared,
                    client: self.client.clone()
                };
                return match path_handler.value.call(ctx).await {
                    Ok(response) => response,
                    Err(err) => {
                        log::error!("{}", err);
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::empty())
                            .unwrap()
                    }
                }

            } else {
                log::debug!("No handler for {} {}", method, path);
            }
        }

        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }

    pub fn get(&mut self, path: &str, handler: impl RouteHandler<C> + Send + Sync + 'static) -> &mut Self {
        self.add_handler(HttpMethod::GET, path, handler)
    }

    pub fn post(&mut self, path: &str, handler: impl RouteHandler<C> + Send + Sync + 'static) -> &mut Self {
        self.add_handler(HttpMethod::POST, path, handler)
    }

    pub fn put(&mut self, path: &str, handler: impl RouteHandler<C> + Send + Sync + 'static) -> &mut Self {
        self.add_handler(HttpMethod::PUT, path, handler)
    }

    pub fn delete(&mut self, path: &str, handler: impl RouteHandler<C> + Send + Sync + 'static) -> &mut Self {
        self.add_handler(HttpMethod::DELETE, path, handler)
    }

    fn add_handler(&mut self, method: HttpMethod, path: &str, handler: impl RouteHandler<C> + Send + Sync + 'static) -> &mut Self {
        if let Some(get_routes) = self.routes.get_mut(&method) {
            get_routes.insert(path, Box::new(handler));
        }
        self
    }
}
