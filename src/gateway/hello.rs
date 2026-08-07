use std::sync::Arc;

use axum::{Json, extract::State};
use axum_anyhow::ApiResult;
use axum_valid::Valid;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

use crate::WebState;

use super::HELLO_TAG;

#[derive(Deserialize, ToSchema, Validate)]
pub struct HelloRequest {
    name: String,
}
#[derive(Serialize, ToSchema, Validate)]
pub struct HelloResponse {
    text: String,
}
#[utoipa::path(
    post,
    path = "/hello",
    request_body(content = HelloRequest, content_type = "application/json"),
    responses(
        (status = OK, body =  HelloResponse, content_type = "application/json"),
        (status = 400, description = "The request body failed validation."),
    ),
    tag = HELLO_TAG
)]
pub async fn hello(
    State(_state): State<Arc<WebState>>,
    Valid(Json(request)): Valid<Json<HelloRequest>>,
) -> ApiResult<Json<HelloResponse>> {
    Ok(Json(HelloResponse {
        text: format!("hello {}", request.name),
    }))
}

pub fn router(state: Arc<WebState>) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(hello))
        .with_state(state.clone())
}
