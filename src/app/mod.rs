mod app;
pub use app::Application;

mod tracing;
pub use tracing::{init_tracing, log_middleware};

pub const AUTH_TOKEN_TYPE: &str = "Bearer";
