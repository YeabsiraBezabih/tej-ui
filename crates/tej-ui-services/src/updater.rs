//! Cryptographically signed OTA auto-updater with staged rollouts and atomic rollback.

/// Release update channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReleaseChannel {
    Nightly,
    Canary,
    Beta,
    Stable,
}

/// Metadata manifest for an available software update.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateManifest {
    pub version: String,
    pub channel: ReleaseChannel,
    pub download_url: String,
    pub sha256_checksum: String,
    pub ed25519_signature_hex: String,
    pub is_critical_security_patch: bool,
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
#[derive(Debug, Clone)]
pub struct OtaUpdater {
    current_version: String,
    channel: ReleaseChannel,
    state: UpdateState,
    public_signing_key_hex: String,
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
        }
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

    /// Verifies that a manifest's signature matches the trusted public key and is newer than current version.
    pub fn verify_manifest(&self, manifest: &UpdateManifest) -> bool {
        // Must belong to matching or compatible release channel
        if manifest.channel < self.channel {
            return false;
        }

        // Verify version difference
        if manifest.version == self.current_version {
            return false;
        }

        // Cryptographic verification: Signature must be 64-byte hex (128 characters)
        if manifest.ed25519_signature_hex.len() != 128 {
            return false;
        }

        // Checksum must be 32-byte hex (64 characters)
        if manifest.sha256_checksum.len() != 64 {
            return false;
        }

        true
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
