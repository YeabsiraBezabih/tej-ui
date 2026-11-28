//! Comprehensive security unit tests for `OtaUpdater` and `SecureKeyStore`.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tej_ui_services::{
    BiometricAuthResult, BiometricPromptConfig, BiometricVerifier, OtaUpdater, ReleaseChannel,
    SecureKeyStore, SignatureVerifier, UpdateManifest, parse_semver,
};

struct TestSignatureVerifier {
    expected_sig: String,
    expected_sha: String,
    ed25519_calls: Arc<AtomicUsize>,
    sha256_calls: Arc<AtomicUsize>,
}

impl SignatureVerifier for TestSignatureVerifier {
    fn verify_ed25519(&self, pubkey_hex: &str, payload: &[u8], sig_hex: &str) -> bool {
        self.ed25519_calls.fetch_add(1, Ordering::SeqCst);
        pubkey_hex.len() == 64 && !payload.is_empty() && sig_hex == self.expected_sig
    }

    fn verify_sha256(&self, data: &[u8], expected_hex: &str) -> bool {
        self.sha256_calls.fetch_add(1, Ordering::SeqCst);
        !data.is_empty() && expected_hex == self.expected_sha
    }
}

struct CountingBiometricVerifier {
    result: BiometricAuthResult,
    calls: Arc<AtomicUsize>,
}

impl BiometricVerifier for CountingBiometricVerifier {
    fn authenticate(&self, _prompt: &BiometricPromptConfig) -> BiometricAuthResult {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.result
    }
}

fn valid_pubkey_hex() -> String {
    "0123456789abcdef".repeat(4)
}

fn sample_manifest(version: &str, sig: String, sha: String) -> UpdateManifest {
    UpdateManifest {
        version: version.to_string(),
        channel: ReleaseChannel::Stable,
        download_url: format!("https://updates.tej.dev/v{version}.tar.gz"),
        sha256_checksum: sha,
        ed25519_signature_hex: sig,
        is_critical_security_patch: false,
    }
}

#[test]
fn test_ota_updater_fails_closed_without_verifier() {
    let updater = OtaUpdater::new("1.0.0", ReleaseChannel::Stable, valid_pubkey_hex());
    let manifest = sample_manifest("1.1.0", "a".repeat(128), "b".repeat(64));

    // Must fail closed when no SignatureVerifier is configured
    assert!(!updater.verify_manifest(&manifest));
    assert!(!updater.verify_artifact(&manifest, b"payload-bytes"));
}

#[test]
fn test_ota_updater_rejects_non_hex_signature_and_checksum() {
    let ed_calls = Arc::new(AtomicUsize::new(0));
    let sha_calls = Arc::new(AtomicUsize::new(0));
    let valid_sig = "a".repeat(128);
    let valid_sha = "b".repeat(64);

    let verifier = Arc::new(TestSignatureVerifier {
        expected_sig: valid_sig.clone(),
        expected_sha: valid_sha.clone(),
        ed25519_calls: Arc::clone(&ed_calls),
        sha256_calls: Arc::clone(&sha_calls),
    });

    let updater = OtaUpdater::new("1.0.0", ReleaseChannel::Stable, valid_pubkey_hex())
        .with_verifier(verifier);

    // 128 chars but contains non-hex 'z'
    let bad_sig = format!("{}z", "a".repeat(127));
    let manifest_bad_sig = sample_manifest("1.1.0", bad_sig, valid_sha.clone());
    assert!(!updater.verify_manifest(&manifest_bad_sig));
    assert_eq!(ed_calls.load(Ordering::SeqCst), 0);

    // 64 chars but contains non-hex 'g'
    let bad_sha = format!("{}g", "b".repeat(63));
    let manifest_bad_sha = sample_manifest("1.1.0", valid_sig.clone(), bad_sha);
    assert!(!updater.verify_manifest(&manifest_bad_sha));
    assert!(!updater.verify_artifact(&manifest_bad_sha, b"artifact"));
    assert_eq!(sha_calls.load(Ordering::SeqCst), 0);

    // Valid hex + matching verifier passes
    let good_manifest = sample_manifest("1.1.0", valid_sig, valid_sha);
    assert!(updater.verify_manifest(&good_manifest));
    assert!(updater.verify_artifact(&good_manifest, b"artifact"));
    assert_eq!(ed_calls.load(Ordering::SeqCst), 1);
    assert_eq!(sha_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn test_ota_updater_prevents_semver_downgrades_and_replays() {
    let valid_sig = "c".repeat(128);
    let valid_sha = "d".repeat(64);
    let verifier = Arc::new(TestSignatureVerifier {
        expected_sig: valid_sig.clone(),
        expected_sha: valid_sha.clone(),
        ed25519_calls: Arc::new(AtomicUsize::new(0)),
        sha256_calls: Arc::new(AtomicUsize::new(0)),
    });

    let updater = OtaUpdater::new("2.1.0", ReleaseChannel::Stable, valid_pubkey_hex())
        .with_verifier(verifier);

    // Downgrades must be rejected
    assert!(!updater.verify_manifest(&sample_manifest(
        "1.9.9",
        valid_sig.clone(),
        valid_sha.clone()
    )));
    assert!(!updater.verify_manifest(&sample_manifest(
        "2.0.99",
        valid_sig.clone(),
        valid_sha.clone()
    )));
    // Identical version replay must be rejected
    assert!(!updater.verify_manifest(&sample_manifest(
        "2.1.0",
        valid_sig.clone(),
        valid_sha.clone()
    )));
    // Malformed SemVer must be rejected
    assert!(!updater.verify_manifest(&sample_manifest(
        "not-a-version",
        valid_sig.clone(),
        valid_sha.clone()
    )));
    // Strictly newer patch/minor/major accepted
    assert!(updater.verify_manifest(&sample_manifest(
        "2.1.1",
        valid_sig.clone(),
        valid_sha.clone()
    )));
    assert!(updater.verify_manifest(&sample_manifest(
        "2.2.0",
        valid_sig.clone(),
        valid_sha.clone()
    )));
    assert!(updater.verify_manifest(&sample_manifest("v3.0.0", valid_sig, valid_sha)));
    assert_eq!(parse_semver("v1.2.3-rc.1"), Some((1, 2, 3)));
    assert_eq!(parse_semver("1.2"), None);
}

#[test]
fn test_secure_keystore_biometric_verifier_and_delete_wipe() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut store = SecureKeyStore::new();
    store.store_secret(
        "api_key",
        b"sk-live-secret-999",
        tej_ui_services::SecretProtection::BiometricRequired,
    );
    store.store_secret(
        "public_id",
        b"pub-123",
        tej_ui_services::SecretProtection::Standard,
    );

    // Debug output must not leak raw secret bytes
    let debug_str = format!("{store:?}");
    assert!(!debug_str.contains("sk-live-secret-999"));

    // Fails closed without verifier
    assert_eq!(store.retrieve_secret("api_key", None), None);
    // Non-biometric key is accessible without verifier
    assert_eq!(
        store.retrieve_secret("public_id", None),
        Some(b"pub-123".as_slice())
    );

    // Denied by biometric verifier
    store.set_verifier(Arc::new(CountingBiometricVerifier {
        result: BiometricAuthResult::Failed,
        calls: Arc::clone(&calls),
    }));
    let prompt = BiometricPromptConfig::new("Unlock API Key");
    assert_eq!(store.retrieve_secret("api_key", Some(&prompt)), None);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    // Approved by biometric verifier
    store.set_verifier(Arc::new(CountingBiometricVerifier {
        result: BiometricAuthResult::Success,
        calls: Arc::clone(&calls),
    }));
    assert_eq!(
        store.retrieve_secret("api_key", Some(&prompt)),
        Some(b"sk-live-secret-999".as_slice())
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    // Delete removes and wipes secret
    assert!(store.delete_secret("api_key"));
    assert!(!store.delete_secret("api_key"));
    assert_eq!(store.retrieve_secret("api_key", Some(&prompt)), None);
    assert_eq!(store.key_info("api_key"), None);

    // Drop executes cleanly (invoking wipe_memory)
    drop(store);
}
