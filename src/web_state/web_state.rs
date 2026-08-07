use std::sync::Arc;

use crate::{TokenBlacklist, TokenConfig, TokenManager};
use anyhow::Result;

pub struct WebState {
    //pub db: Db,
    pub token_manager: TokenManager,
    pub token_blacklist: TokenBlacklist,
}

pub fn new_webstate(cfg: &TokenConfig) -> Result<Arc<WebState>> {
    Ok(Arc::new(WebState {
        token_manager: TokenManager::new(cfg)?,
        token_blacklist: TokenBlacklist::new(),
    }))
}
