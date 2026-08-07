use axum::http::{HeaderName, HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};

pub fn create_cors_layer(allowed_origins: &[&str]) -> CorsLayer {
    let cors_builder = CorsLayer::new()
        .allow_methods([Method::OPTIONS])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
            HeaderName::from_static("x-requested-with"),
            HeaderName::from_static("accept"),
            HeaderName::from_static("origin"),
            HeaderName::from_static("access-control-request-method"),
            HeaderName::from_static("access-control-request-headers"),
        ])
        .expose_headers([
            HeaderName::from_static("content-length"),
            HeaderName::from_static("content-type"),
        ])
        .max_age(std::time::Duration::from_secs(86400));

    if allowed_origins.is_empty() {
        cors_builder.allow_origin(Any)
    } else {
        let header_values: Vec<HeaderValue> = allowed_origins
            .into_iter()
            .filter_map(|origin| origin.parse().ok())
            .collect();
        cors_builder.allow_origin(header_values)
    }
}
