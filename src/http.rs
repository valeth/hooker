use std::convert::TryInto;
use hyper_rustls::HttpsConnector;
use hyper::client::HttpConnector;
pub use hyper::http::StatusCode;
use anyhow::anyhow;
use crate::Result;

pub type Request<T = hyper::Body> = hyper::Request<T>;
pub type Response<T = hyper::Body> = hyper::Response<T>;
pub type HttpMethod = hyper::http::Method;
pub type Params = matchit::Params;

pub struct Client(hyper::Client<HttpsConnector<HttpConnector>>);

impl Client {
    pub fn new() -> Self {
        let connector = HttpsConnector::with_native_roots();
        let client = hyper::Client::builder().build(connector);
        Self(client)
    }

    pub async fn post<U, B>(&self, uri: U, body: B) -> Result<Response>
    where U: TryInto<hyper::Uri>,
          B: Into<hyper::Body>
    {
        let uri = uri.try_into().or_else(|_| Err(anyhow!("Failed to parse URI")))?;
        let request = hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .header("Content-Type","application/json")
            .body(body.into())?;

        Ok(self.0.request(request).await?)
    }
}

