//! OS Keychain, hardware Secure Enclave, and Biometrics (TouchID/FaceID/Windows Hello).
//!
//! # Security Scope
//!
//! [`SecureKeyStore`] in this module provides an **in-memory fallback** credential store
//! backed by [`zeroize::Zeroizing`] for automatic memory zeroization on overwrite, deletion,
//! and `Drop` with zero `unsafe` code, plus pluggable biometric gating via
//! [`BiometricVerifier`]. It does **not** persist secrets to a hardware Secure Enclave,
//! macOS Keychain, Windows DPAPI/Credential Manager, or Linux Secret Service on its own;
//! production builds requiring OS-managed hardware-backed persistence should integrate
//! a platform keychain backend.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use zeroize::Zeroizing;

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

/// Platform biometric authenticator invoked by [`SecureKeyStore`] before releasing
/// biometrics-protected secrets.
pub trait BiometricVerifier: Send + Sync {
    /// Performs a biometric authentication check using the provided prompt configuration.
    fn authenticate(&self, prompt: &BiometricPromptConfig) -> BiometricAuthResult;
}

/// In-memory secure credential store with [`Zeroizing`] memory wiping and biometric access gating.
///
/// Intentionally does **not** implement `Clone` so plaintext secret buffers cannot be
/// silently duplicated in process memory. All stored secret bytes are wrapped in
/// `Zeroizing<Vec<u8>>` and automatically zeroized on overwrite, deletion,
/// explicit [`Self::wipe_memory`], and [`Drop`].
///
/// Note: This is an in-memory fallback store rather than hardware Secure Enclave persistence.
#[derive(Default)]
pub struct SecureKeyStore {
    secrets: HashMap<String, (Zeroizing<Vec<u8>>, bool)>, // (payload, requires_biometrics)
    verifier: Option<Arc<dyn BiometricVerifier>>,
}

/// Access protection policy for a secret stored in [`SecureKeyStore`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretProtection {
    /// Standard OS keychain protection without a biometric gate on read.
    Standard,
    /// Requires biometric authentication before reading the secret.
    BiometricRequired,
}

impl fmt::Debug for SecureKeyStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecureKeyStore")
            .field("stored_keys", &self.secrets.len())
            .field("has_verifier", &self.verifier.is_some())
            .finish()
    }
}

impl SecureKeyStore {
    /// Creates a new in-memory keystore with no biometric verifier configured.
    ///
    /// Biometrics-protected secrets fail closed (`None`) until a [`BiometricVerifier`]
    /// is configured via [`Self::with_verifier`] or [`Self::set_verifier`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new in-memory keystore configured with a [`BiometricVerifier`].
    #[must_use]
    pub fn with_verifier(verifier: Arc<dyn BiometricVerifier>) -> Self {
        Self {
            secrets: HashMap::new(),
            verifier: Some(verifier),
        }
    }

    /// Configures or replaces the [`BiometricVerifier`] used to gate protected secrets.
    pub fn set_verifier(&mut self, verifier: Arc<dyn BiometricVerifier>) {
        self.verifier = Some(verifier);
    }

    /// Stores a secret, optionally protecting access with mandatory biometric authentication.
    ///
    /// If a secret already exists under `key`, its previous [`Zeroizing`] buffer is dropped
    /// and automatically zeroized.
    pub fn store_secret(
        &mut self,
        key: impl Into<String>,
        secret: &[u8],
        protection: SecretProtection,
    ) {
        let requires_biometrics = matches!(protection, SecretProtection::BiometricRequired);
        self.secrets.insert(
            key.into(),
            (Zeroizing::new(secret.to_vec()), requires_biometrics),
        );
    }

    /// Retrieves a secret.
    ///
    /// If the key requires biometrics, authentication is delegated to the configured
    /// [`BiometricVerifier`]. Fails closed (`None`) if no verifier is configured or if
    /// authentication does not return [`BiometricAuthResult::Success`].
    #[must_use]
    pub fn retrieve_secret(
        &self,
        key: &str,
        prompt: Option<&BiometricPromptConfig>,
    ) -> Option<&[u8]> {
        let (data, requires_bio) = self.secrets.get(key)?;
        if *requires_bio {
            let verifier = self.verifier.as_ref()?;
            let default_prompt = BiometricPromptConfig::new("Authenticate to access secret");
            let effective_prompt = prompt.unwrap_or(&default_prompt);
            if verifier.authenticate(effective_prompt) == BiometricAuthResult::Success {
                Some(data.as_slice())
            } else {
                None
            }
        } else {
            Some(data.as_slice())
        }
    }

    /// Removes a secret from the keystore, dropping and zeroizing its [`Zeroizing`] buffer.
    pub fn delete_secret(&mut self, key: &str) -> bool {
        self.secrets.remove(key).is_some()
    }

    /// Checks if a key exists and whether it requires biometric authorization.
    #[must_use]
    pub fn key_info(&self, key: &str) -> Option<bool> {
        self.secrets.get(key).map(|(_, requires_bio)| *requires_bio)
    }

    /// Securely wipes all secrets from process memory by dropping every [`Zeroizing`] buffer.
    pub fn wipe_memory(&mut self) {
        self.secrets.clear();
    }
}

impl Drop for SecureKeyStore {
    fn drop(&mut self) {
        self.wipe_memory();
    }
}
