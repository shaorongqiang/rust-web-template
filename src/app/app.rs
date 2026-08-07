use std::{sync::Arc, time::Duration};

use anyhow::Result;
use axum::{Router, middleware::from_fn};
use tokio::{net::TcpListener, signal, time};
use tokio_util::sync::CancellationToken;
use utoipa::{
    OpenApi,
    openapi::{
        Components,
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
        server::Server,
    },
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use super::AUTH_TOKEN_TYPE;
use crate::{OpenApiDoc, ServerConfig, WebState, create_cors_layer, gateway, log_middleware};

pub struct Application {
    shutdown: CancellationToken,
    state: Arc<WebState>,
}
impl Application {
    pub fn new(state: Arc<WebState>) -> Self {
        let app = Self {
            state,
            shutdown: CancellationToken::new(),
        };
        app.init();
        app
    }

    pub async fn init_database(&self) -> Result<()> {
        Ok(())
    }

    pub fn crate_app(&self, access_url: &str, allowed_origins: &[&str]) -> Router {
        let mut components = Components::new();
        components.security_schemes.insert(
            AUTH_TOKEN_TYPE.to_string(),
            SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
        );

        let mut openapi = OpenApiDoc::openapi();
        openapi.components = Some(components);
        openapi.servers = Some(vec![Server::new(access_url.trim_end_matches('/'))]);

        let (router, api) = OpenApiRouter::with_openapi(openapi)
            .nest("/api/v1", gateway::router(self.state.clone()))
            .layer(from_fn(log_middleware))
            .layer(create_cors_layer(allowed_origins))
            .split_for_parts();

        let swagger_ui = SwaggerUi::new("/").url("/apidoc/openapi.json", api);

        router.merge(swagger_ui)
    }

    fn init(&self) {
        let signal_shutdown = self.shutdown.clone();

        tokio::spawn(async move {
            if let Err(e) = signal::ctrl_c().await {
                tracing::error!(
                    error = ?e,
                    "ctrl_c await error"
                );
            }
            tracing::info!("shutdown application");
            signal_shutdown.cancel();
        });
    }

    pub async fn run(self, cfg: &ServerConfig) -> Result<()> {
        let app = self.crate_app(
            cfg.access_url.as_str(),
            &cfg.allowed_origins
                .iter()
                .map(|v| v.as_str())
                .collect::<Vec<_>>(),
        );
        let listener = TcpListener::bind(cfg.listen).await?;
        tracing::info!("Listening on http://{}", cfg.listen);

        axum::serve(listener, app)
            .with_graceful_shutdown({
                let shutdown = self.shutdown.clone();
                async move {
                    shutdown.cancelled().await;
                }
            })
            .await?;
        Ok(())
    }

    pub async fn run_background(&self) -> Result<()> {
        let blacklist = self.state.token_blacklist.clone();
        let shutdown = self.shutdown.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        blacklist.cleanup();
                    }
                    _ = shutdown.cancelled() => {
                        tracing::info!("background task stopped");
                        break;
                    }
                }
            }
        });
        Ok(())
    }
}
