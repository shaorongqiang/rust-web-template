use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::current_timestamp;

/// In-memory set of revoked tokens, keyed by token string with expiry timestamp.
#[derive(Clone)]
pub struct TokenBlacklist {
    tokens: Arc<RwLock<HashMap<String, i64>>>,
}

impl Default for TokenBlacklist {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenBlacklist {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add(&self, token: String, expires_at: i64) {
        if let Ok(mut map) = self.tokens.write() {
            map.insert(token, expires_at);
        }
    }

    pub fn contains(&self, token: &str) -> bool {
        self.tokens
            .read()
            .map(|map| map.contains_key(token))
            .unwrap_or(false)
    }

    pub fn cleanup(&self) {
        let now = current_timestamp();
        if let Ok(mut map) = self.tokens.write() {
            map.retain(|_, &mut exp| exp > now);
        }
    }
}
