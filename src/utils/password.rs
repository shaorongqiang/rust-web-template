use anyhow::{Result, anyhow};
use argon2::{
    Argon2, PasswordHash,
    password_hash::{PasswordHasher, PasswordVerifier, SaltString},
};
use rand_core::OsRng;

pub fn generate_salt() -> String {
    SaltString::generate(&mut OsRng).to_string()
}

pub fn hash_password(password: &str, salt: &str) -> Result<String> {
    let salt = SaltString::from_b64(salt).map_err(|e| anyhow!("{}", e))?;

    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("{}", e))?;

    Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let password_hash = PasswordHash::new(hash).map_err(|e| anyhow!("{}", e))?;

    match Argon2::default().verify_password(password.as_bytes(), &password_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_password() {
        let password = "0x48656c6c6f20576f726c64";
        let salt = generate_salt();

        let hash = hash_password(password, &salt).unwrap();

        // Verify correct password
        let is_valid = verify_password(password, &hash).unwrap();
        assert!(is_valid);

        // Verify wrong password
        let wrong_password = "0x576f6e672050617373776f7264";
        let is_invalid = verify_password(wrong_password, &hash).unwrap();
        assert!(!is_invalid);
    }
}
