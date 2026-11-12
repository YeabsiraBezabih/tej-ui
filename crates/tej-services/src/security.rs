use crate::error::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Stored secure credential pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Credential {
    pub service: String,
    pub account: String,
    pub secret: String,
}

/// Abstract OS secure credential store (Keychain / Credential Manager).
#[derive(Debug, Default, Clone)]
pub struct CredentialStore {
    vault: Arc<RwLock<HashMap<(String, String), String>>>,
}

impl CredentialStore {
    /// Create a new credential store handle.
    #[must_use]
    pub fn new() -> Self {
        Self {
            vault: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Save a secure secret for a service and account identifier.
    pub fn save_secret(&self, service: &str, account: &str, secret: &str) -> Result<()> {
        let key = (service.to_string(), account.to_string());
        self.vault.write().insert(key, secret.to_string());
        Ok(())
    }

    /// Retrieve secret for service and account.
    pub fn get_secret(&self, service: &str, account: &str) -> Result<Option<String>> {
        let key = (service.to_string(), account.to_string());
        Ok(self.vault.read().get(&key).cloned())
    }

    /// Delete credential from store.
    pub fn delete_secret(&self, service: &str, account: &str) -> Result<bool> {
        let key = (service.to_string(), account.to_string());
        Ok(self.vault.write().remove(&key).is_some())
    }
}
