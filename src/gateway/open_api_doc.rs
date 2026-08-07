use std::sync::Arc;

use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::WebState;

use super::hello;

pub const HELLO_TAG: &str = "Hello";

#[derive(OpenApi)]
#[openapi(
    info(
        title = "rust web template",
        description = "rust web template",
        version = "0.0.1",
        contact(
            name = "API Support",
        )
    ),
    tags(
         (name = HELLO_TAG, description = "example"), 
    ),
)]
pub struct OpenApiDoc;

pub fn router(state: Arc<WebState>) -> OpenApiRouter {
    OpenApiRouter::new().merge(hello::router(state.clone()))
}
