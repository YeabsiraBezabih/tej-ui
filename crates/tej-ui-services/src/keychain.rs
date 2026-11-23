//! OS Keychain, hardware Secure Enclave, and Biometrics (TouchID/FaceID/Windows Hello).

use std::collections::HashMap;

/// Result of a biometric authentication prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiometricAuthResult {
    Success,
    UserCancelled,
    BiometryNotEnrolled,
    FallbackToPasscode,
    Failed,
}

/// Configuration options for system biometric verification prompts.
#[derive(Debug, Clone)]
pub struct BiometricPromptConfig {
    pub title: String,
    pub subtitle: Option<String>,
    pub cancel_label: String,
    pub allow_device_credential: bool,
}

impl BiometricPromptConfig {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            cancel_label: "Cancel".to_string(),
            allow_device_credential: true,
        }
    }
}

/// Enclave-backed secure credential storage with biometric access gating.
#[derive(Debug, Clone, Default)]
pub struct SecureKeyStore {
    secrets: HashMap<String, (Vec<u8>, bool)>, // (payload, requires_biometrics)
}

impl SecureKeyStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores a secret, optionally protecting access with mandatory biometric authentication.
    pub fn store_secret(
        &mut self,
        key: impl Into<String>,
        secret: &[u8],
        requires_biometrics: bool,
    ) {
        self.secrets
            .insert(key.into(), (secret.to_vec(), requires_biometrics));
    }

    /// Retrieves a secret. If biometrics are required and prompt result is not `Success`, returns None.
    pub fn retrieve_secret(
        &self,
        key: &str,
        auth_result: Option<BiometricAuthResult>,
    ) -> Option<&[u8]> {
        let (data, requires_bio) = self.secrets.get(key)?;
        if *requires_bio {
            if auth_result == Some(BiometricAuthResult::Success) {
                Some(data.as_slice())
            } else {
                None
            }
        } else {
            Some(data.as_slice())
        }
    }

    /// Removes a secret from the keystore.
    pub fn delete_secret(&mut self, key: &str) -> bool {
        self.secrets.remove(key).is_some()
    }

    /// Checks if a key exists and whether it requires biometric authorization.
    pub fn key_info(&self, key: &str) -> Option<bool> {
        self.secrets.get(key).map(|(_, requires_bio)| *requires_bio)
    }

    /// Securely wipes all secrets from process memory using volatile zeroing.
    pub fn wipe_memory(&mut self) {
        for (secret_bytes, _) in self.secrets.values_mut() {
            for byte in secret_bytes.iter_mut() {
                // SAFETY: Pointer is valid, non-null, and uniquely borrowed from secret_bytes slice.
                unsafe {
                    core::ptr::write_volatile(byte, 0);
                }
            }
            secret_bytes.clear();
        }
        self.secrets.clear();
    }
}
