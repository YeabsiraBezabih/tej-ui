//! Cryptographically signed OTA auto-updater with staged rollouts and atomic rollback.

use std::fmt;
use std::sync::Arc;

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

/// Release update channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReleaseChannel {
    Nightly,
    Canary,
    Beta,
    Stable,
}

/// Cryptographic verifier for OTA update manifests and downloaded artifacts.
pub trait SignatureVerifier: Send + Sync {
    /// Verifies an Ed25519 signature (`sig_hex`, 128 hex chars) over `payload`
    /// using the trusted public key (`pubkey_hex`, 64 hex chars).
    fn verify_ed25519(&self, pubkey_hex: &str, payload: &[u8], sig_hex: &str) -> bool;

    /// Verifies that the SHA-256 digest of `data` matches `expected_hex` (64 hex chars).
    fn verify_sha256(&self, data: &[u8], expected_hex: &str) -> bool;
}

/// Production cryptographic verifier backed by `ed25519-dalek` and `sha2`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Ed25519Sha256Verifier;

impl Ed25519Sha256Verifier {
    /// Creates a new production Ed25519 + SHA-256 verifier.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl SignatureVerifier for Ed25519Sha256Verifier {
    fn verify_ed25519(&self, pubkey_hex: &str, payload: &[u8], sig_hex: &str) -> bool {
        let Ok(pubkey_vec) = hex::decode(pubkey_hex) else {
            return false;
        };
        let Ok(pubkey_bytes): Result<[u8; 32], _> = pubkey_vec.try_into() else {
            return false;
        };
        let Ok(verifying_key) = VerifyingKey::from_bytes(&pubkey_bytes) else {
            return false;
        };

        let Ok(sig_vec) = hex::decode(sig_hex) else {
            return false;
        };
        let Ok(sig_bytes): Result<[u8; 64], _> = sig_vec.try_into() else {
            return false;
        };
        let signature = Signature::from_bytes(&sig_bytes);

        verifying_key.verify(payload, &signature).is_ok()
    }

    fn verify_sha256(&self, data: &[u8], expected_hex: &str) -> bool {
        let Ok(expected_bytes) = hex::decode(expected_hex) else {
            return false;
        };
        if expected_bytes.len() != 32 {
            return false;
        }

        let actual_digest = Sha256::digest(data);
        let diff = actual_digest
            .as_slice()
            .iter()
            .zip(expected_bytes.iter())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b));
        diff == 0
    }
}

/// Parses a SemVer string (`MAJOR.MINOR.PATCH`, optional leading `v`, optional `-pre`/`+build`)
/// into a comparable `(major, minor, patch)` tuple.
#[must_use]
pub fn parse_semver(version: &str) -> Option<(u64, u64, u64)> {
    let trimmed = version.trim();
    let without_v = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed);
    let core = without_v.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

#[inline]
fn is_valid_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len && value.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Metadata manifest for an available software update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateManifest {
    pub version: String,
    pub channel: ReleaseChannel,
    pub download_url: String,
    pub sha256_checksum: String,
    pub ed25519_signature_hex: String,
    pub is_critical_security_patch: bool,
}

impl UpdateManifest {
    /// Returns the canonical byte payload covered by `ed25519_signature_hex`.
    #[must_use]
    pub fn signing_payload(&self) -> Vec<u8> {
        format!(
            "{}|{:?}|{}|{}|{}",
            self.version,
            self.channel,
            self.download_url,
            self.sha256_checksum,
            self.is_critical_security_patch
        )
        .into_bytes()
    }
}

/// OTA update process state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateState {
    Idle,
    Checking,
    UpdateAvailable,
    Downloading { percent: u8 },
    ReadyToInstall,
    RollbackActive,
}

/// Signed OTA update coordinator and verification engine.
///
/// Fails closed by default: [`OtaUpdater::verify_manifest`] and [`OtaUpdater::verify_artifact`]
/// return `false` until a cryptographic [`SignatureVerifier`] is configured.
/// Use [`OtaUpdater::with_default_crypto`] to construct a production updater pre-wired
/// with [`Ed25519Sha256Verifier`].
#[derive(Clone)]
pub struct OtaUpdater {
    current_version: String,
    channel: ReleaseChannel,
    state: UpdateState,
    public_signing_key_hex: String,
    verifier: Option<Arc<dyn SignatureVerifier>>,
}

impl fmt::Debug for OtaUpdater {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OtaUpdater")
            .field("current_version", &self.current_version)
            .field("channel", &self.channel)
            .field("state", &self.state)
            .field("public_signing_key_hex", &self.public_signing_key_hex)
            .field("has_verifier", &self.verifier.is_some())
            .finish()
    }
}

impl OtaUpdater {
    pub fn new(
        current_version: impl Into<String>,
        channel: ReleaseChannel,
        public_key_hex: impl Into<String>,
    ) -> Self {
        Self {
            current_version: current_version.into(),
            channel,
            state: UpdateState::Idle,
            public_signing_key_hex: public_key_hex.into(),
            verifier: None,
        }
    }

    /// Creates a production [`OtaUpdater`] targeting [`ReleaseChannel::Stable`] pre-configured
    /// with [`Ed25519Sha256Verifier`] for real Ed25519 and SHA-256 verification.
    #[must_use]
    pub fn with_default_crypto(
        current_version: impl Into<String>,
        public_signing_key_hex: impl Into<String>,
    ) -> Self {
        Self::new(
            current_version,
            ReleaseChannel::Stable,
            public_signing_key_hex,
        )
        .with_verifier(Arc::new(Ed25519Sha256Verifier))
    }

    /// Attaches a cryptographic [`SignatureVerifier`] to this updater.
    #[must_use]
    pub fn with_verifier(mut self, verifier: Arc<dyn SignatureVerifier>) -> Self {
        self.verifier = Some(verifier);
        self
    }

    /// Configures or replaces the cryptographic [`SignatureVerifier`].
    pub fn set_verifier(&mut self, verifier: Arc<dyn SignatureVerifier>) {
        self.verifier = Some(verifier);
    }

    pub fn current_version(&self) -> &str {
        &self.current_version
    }

    pub fn channel(&self) -> ReleaseChannel {
        self.channel
    }

    pub fn state(&self) -> UpdateState {
        self.state
    }

    pub fn public_signing_key_hex(&self) -> &str {
        &self.public_signing_key_hex
    }

    /// Verifies that a manifest belongs to a compatible channel, is strictly newer
    /// according to SemVer `(major, minor, patch)`, has valid hex-encoded cryptographic
    /// fields, and passes Ed25519 signature verification via the configured verifier.
    ///
    /// Fails closed (`false`) if no [`SignatureVerifier`] is configured.
    #[must_use]
    pub fn verify_manifest(&self, manifest: &UpdateManifest) -> bool {
        // Must belong to matching or compatible release channel
        if manifest.channel < self.channel {
            return false;
        }

        // Strict SemVer comparison to prevent downgrades and replay of identical versions
        let (Some(current_ver), Some(manifest_ver)) = (
            parse_semver(&self.current_version),
            parse_semver(&manifest.version),
        ) else {
            return false;
        };
        if manifest_ver <= current_ver {
            return false;
        }

        // Strict hex validation for public key (32 bytes = 64 hex),
        // Ed25519 signature (64 bytes = 128 hex), and SHA-256 checksum (32 bytes = 64 hex)
        if !is_valid_hex(&self.public_signing_key_hex, 64) {
            return false;
        }
        if !is_valid_hex(&manifest.ed25519_signature_hex, 128) {
            return false;
        }
        if !is_valid_hex(&manifest.sha256_checksum, 64) {
            return false;
        }

        // Fail closed when no cryptographic verifier is configured
        let Some(verifier) = &self.verifier else {
            return false;
        };

        let payload = manifest.signing_payload();
        verifier.verify_ed25519(
            &self.public_signing_key_hex,
            &payload,
            &manifest.ed25519_signature_hex,
        )
    }

    /// Verifies that a downloaded artifact matches the manifest's SHA-256 checksum.
    ///
    /// Fails closed (`false`) if no [`SignatureVerifier`] is configured or the checksum
    /// is not valid 64-character hex.
    #[must_use]
    pub fn verify_artifact(&self, manifest: &UpdateManifest, artifact_bytes: &[u8]) -> bool {
        if !is_valid_hex(&manifest.sha256_checksum, 64) {
            return false;
        }
        let Some(verifier) = &self.verifier else {
            return false;
        };
        verifier.verify_sha256(artifact_bytes, &manifest.sha256_checksum)
    }

    /// Advances the update state.
    pub fn transition_to(&mut self, next: UpdateState) {
        self.state = next;
    }

    /// Triggers atomic rollback on boot failure.
    pub fn trigger_rollback(&mut self) {
        self.state = UpdateState::RollbackActive;
    }
}
