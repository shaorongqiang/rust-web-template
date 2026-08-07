use std::fs;

use anyhow::Result;
use chrono::{DateTime, Utc};
use jwt_simple::{
    claims::{Claims, JWTClaims},
    prelude::{Duration, Ed25519KeyPair, EdDSAKeyPairLike, EdDSAPublicKeyLike},
};
use serde::{Deserialize, Serialize};

use crate::TokenConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub user_id: String,
    pub token_type: TokenType,
    pub first_issued_at: DateTime<Utc>,
}

pub struct TokenManager {
    access_key_pair: Ed25519KeyPair,
    refresh_key_pair: Ed25519KeyPair,
    cfg: TokenConfig,
}

impl TokenManager {
    pub fn new(config: &TokenConfig) -> Result<Self> {
        Ok(Self {
            cfg: config.clone(),
            access_key_pair: Ed25519KeyPair::from_pem(&fs::read_to_string(
                &config.access_key_path,
            )?)?,
            refresh_key_pair: Ed25519KeyPair::from_pem(&fs::read_to_string(
                &config.refresh_key_path,
            )?)?,
        })
    }

    pub fn issue_access_token(&self, user_id: &str) -> Result<String> {
        let claims = TokenClaims {
            user_id: user_id.to_string(),
            token_type: TokenType::Access,
            first_issued_at: Utc::now(),
        };

        let jwt_claims = Claims::with_custom_claims(
            claims,
            Duration::from_mins(self.cfg.access_token_expired_minutes),
        );
        self.access_key_pair.sign(jwt_claims)
    }

    pub fn issue_refresh_token(&self, user_id: &str) -> Result<String> {
        self.re_issue_refresh_token(
            user_id,
            Duration::from_mins(self.cfg.refresh_token_expired_minutes),
            Utc::now(),
        )
    }

    pub fn access_token_expired_seconds(&self) -> i64 {
        (self.cfg.access_token_expired_minutes * 60) as i64
    }

    fn re_issue_refresh_token(
        &self,
        user_id: &str,
        expires_at: Duration,
        first_issued_at: DateTime<Utc>,
    ) -> Result<String> {
        let claims = TokenClaims {
            user_id: user_id.to_string(),
            token_type: TokenType::Refresh,
            first_issued_at,
        };

        let jwt_claims = Claims::with_custom_claims(claims, expires_at);
        self.refresh_key_pair.sign(jwt_claims)
    }

    pub fn verify_access_token(&self, token: &str) -> Result<TokenClaims> {
        let claims: JWTClaims<TokenClaims> = self
            .access_key_pair
            .public_key()
            .verify_token(token, None)?;

        if !matches!(claims.custom.token_type, TokenType::Access) {
            return Err(anyhow::anyhow!("Invalid token type: expected access token"));
        }

        Ok(claims.custom)
    }

    pub fn verify_refresh_token(&self, token: &str) -> Result<TokenClaims> {
        let claims: JWTClaims<TokenClaims> = self
            .refresh_key_pair
            .public_key()
            .verify_token(token, None)?;

        if !matches!(claims.custom.token_type, TokenType::Refresh) {
            return Err(anyhow::anyhow!(
                "Invalid token type: expected refresh token"
            ));
        }

        Ok(claims.custom)
    }

    pub fn refresh_tokens(&self, refresh_token: &str) -> Result<Option<(String, String)>> {
        let refresh_claims = self.verify_refresh_token(refresh_token)?;
        let now = Utc::now();
        let token_age = now.signed_duration_since(refresh_claims.first_issued_at);
        let max_refresh_duration =
            chrono::Duration::minutes(self.cfg.refresh_token_max_expired_minutes as i64);
        let remaining_duration = max_refresh_duration - token_age;

        let new_refresh_duration = std::cmp::min(
            remaining_duration,
            chrono::Duration::minutes(self.cfg.refresh_token_expired_minutes as i64),
        )
        .num_seconds();

        if new_refresh_duration <= 0 {
            return Ok(None);
        }

        let new_access_token = self.issue_access_token(&refresh_claims.user_id)?;
        let new_refresh_token = self.re_issue_refresh_token(
            &refresh_claims.user_id,
            Duration::from_secs(new_refresh_duration as u64),
            refresh_claims.first_issued_at,
        )?;

        Ok(Some((new_access_token, new_refresh_token)))
    }
}
