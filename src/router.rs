use std::collections::HashMap;
use async_trait::async_trait;

pub type Request<T = hyper::Body> = hyper::Request<T>;
pub type Response<T = hyper::Body> = hyper::Response<T>;
pub type HttpMethod = hyper::http::Method;
pub type Params = matchit::Params;

#[derive(Debug)]
pub struct Context<T>
where T: Send + Sync
{
    pub request: Request,
    pub params: Params,
    pub shared: T
}

#[async_trait]
pub trait RouteHandler<C>
where C: Send + Sync + 'static
{
    async fn call(&self, ctx: Context<C>) -> Response;
}

#[async_trait]
impl<F, T, C> RouteHandler<C> for T
where T: Send + Sync + 'static,
      C: Send + Sync + 'static,
      T: Fn(Context<C>) -> F,
      F: std::future::Future + Send + 'static,
      F::Output: Into<Response>
{
    async fn call(&self, ctx: Context<C>) -> Response {
        self(ctx).await.into()
    }
}

pub struct Router<C>
where C: Send + Sync + 'static
{
    routes: HashMap<HttpMethod, matchit::Node<Box<dyn RouteHandler<C> + Send + Sync>>>,
}

impl<C> Router<C>
where C: Send + Sync + 'static
{
    const METHODS: [HttpMethod; 4] = [HttpMethod::GET, HttpMethod::POST, HttpMethod::DELETE, HttpMethod::PUT];

    pub fn new() -> Self {
        let mut routes = HashMap::new();
        for method in Self::METHODS.iter() {
            routes.insert(method.clone(), matchit::Node::default());
        }

        Self { routes }
    }

    pub async fn handle_route(&self, request: Request, shared: C) -> Response {
        use hyper::{StatusCode, Body};

        let method = request.method();

        if let Some(method_handler) = self.routes.get(method) {
            let path = request.uri().path();

            if let Ok(path_handler) = method_handler.match_path(path) {
                log::debug!("{} {}", method, path);
                let ctx = Context { request, params: path_handler.params, shared };
                return path_handler.value.call(ctx).await
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
