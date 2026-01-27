//! Secure token storage using the OS keyring

use anyhow::Result;
use keyring::Entry;

const SERVICE_NAME: &str = "cloudflare-admin";
const TOKEN_KEY: &str = "api_token";
const APPEARANCE_KEY: &str = "appearance_mode";

/// Store the API token securely in the OS keyring
pub fn store_token(token: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, TOKEN_KEY)?;
    entry.set_password(token)?;
    Ok(())
}

/// Retrieve the API token from the OS keyring
pub fn get_token() -> Result<Option<String>> {
    let entry = Entry::new(SERVICE_NAME, TOKEN_KEY)?;
    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Delete the API token from the OS keyring
pub fn delete_token() -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, TOKEN_KEY)?;
    match entry.delete_credential() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
        Err(e) => Err(e.into()),
    }
}

/// Check if a token exists in the keyring
pub fn has_token() -> bool {
    get_token().map(|t| t.is_some()).unwrap_or(false)
}

/// Store the appearance mode preference
pub fn store_appearance_mode(mode: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, APPEARANCE_KEY)?;
    entry.set_password(mode)?;
    Ok(())
}

/// Retrieve the appearance mode preference
pub fn get_appearance_mode() -> Result<Option<String>> {
    let entry = Entry::new(SERVICE_NAME, APPEARANCE_KEY)?;
    match entry.get_password() {
        Ok(mode) => Ok(Some(mode)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
