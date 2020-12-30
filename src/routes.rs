#![allow(unused_variables)]

pub mod api;

use crate::router::{Params, Request, Response};

pub async fn post_gitlab(request: Request, params: Params) -> Response {
    log::debug!("{:?}", params.by_name("id"));
    Response::default()
}


