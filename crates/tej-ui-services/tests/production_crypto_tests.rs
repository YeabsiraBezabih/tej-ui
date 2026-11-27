//! Production cryptographic verification tests for `Ed25519Sha256Verifier`,
//! `OtaUpdater::with_default_crypto`, and `Zeroizing`-backed `SecureKeyStore`.

use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use tej_ui_services::{
    Ed25519Sha256Verifier, OtaUpdater, ReleaseChannel, SecureKeyStore, SignatureVerifier,
    UpdateManifest,
};

fn build_signed_manifest(
    signing_key: &SigningKey,
    version: &str,
    channel: ReleaseChannel,
    artifact_bytes: &[u8],
    is_critical_security_patch: bool,
) -> UpdateManifest {
    let sha256_checksum = hex::encode(Sha256::digest(artifact_bytes));
    let mut manifest = UpdateManifest {
        version: version.to_string(),
        channel,
        download_url: format!("https://releases.tej-ui.dev/app-v{version}.tar.zst"),
        sha256_checksum,
        ed25519_signature_hex: String::new(),
        is_critical_security_patch,
    };
    let signature = signing_key.sign(&manifest.signing_payload());
    manifest.ed25519_signature_hex = hex::encode(signature.to_bytes());
    manifest
}

#[test]
fn test_with_default_crypto_verifies_valid_signed_manifest_and_artifact() {
    let signing_key = SigningKey::from_bytes(&[0x42u8; 32]);
    let pubkey_hex = hex::encode(signing_key.verifying_key().to_bytes());
    let artifact = b"tej-ui release binary payload v1.2.0";

    let manifest = build_signed_manifest(
        &signing_key,
        "1.2.0",
        ReleaseChannel::Stable,
        artifact,
        true,
    );
    let updater = OtaUpdater::with_default_crypto("1.1.0", &pubkey_hex);

    assert_eq!(updater.channel(), ReleaseChannel::Stable);
    assert!(updater.verify_manifest(&manifest));
    assert!(updater.verify_artifact(&manifest, artifact));
}

#[test]
fn test_with_default_crypto_rejects_tampered_manifest_fields() {
    let signing_key = SigningKey::from_bytes(&[0x11u8; 32]);
    let pubkey_hex = hex::encode(signing_key.verifying_key().to_bytes());
    let artifact = b"authentic binary contents";

    let valid_manifest = build_signed_manifest(
        &signing_key,
        "2.0.0",
        ReleaseChannel::Stable,
        artifact,
        false,
    );
    let updater = OtaUpdater::with_default_crypto("1.9.0", &pubkey_hex);
    assert!(updater.verify_manifest(&valid_manifest));

    // Tampered version
    let mut tampered_version = valid_manifest.clone();
    tampered_version.version = "2.0.1".to_string();
    assert!(!updater.verify_manifest(&tampered_version));

    // Tampered download URL
    let mut tampered_url = valid_manifest.clone();
    tampered_url.download_url = "https://evil.example/malware.tar.zst".to_string();
    assert!(!updater.verify_manifest(&tampered_url));

    // Tampered SHA-256 checksum
    let mut tampered_checksum = valid_manifest.clone();
    tampered_checksum.sha256_checksum = hex::encode(Sha256::digest(b"different payload"));
    assert!(!updater.verify_manifest(&tampered_checksum));

    // Tampered critical security patch flag
    let mut tampered_flag = valid_manifest.clone();
    tampered_flag.is_critical_security_patch = true;
    assert!(!updater.verify_manifest(&tampered_flag));
}

#[test]
fn test_with_default_crypto_rejects_wrong_key_and_wrong_signature() {
    let signing_key = SigningKey::from_bytes(&[0x22u8; 32]);
    let other_key = SigningKey::from_bytes(&[0x99u8; 32]);
    let artifact = b"signed artifact bytes";

    let manifest = build_signed_manifest(
        &signing_key,
        "1.5.0",
        ReleaseChannel::Stable,
        artifact,
        false,
    );

    // Updater configured with a different public key must reject the manifest
    let wrong_key_hex = hex::encode(other_key.verifying_key().to_bytes());
    let updater_wrong_key = OtaUpdater::with_default_crypto("1.0.0", &wrong_key_hex);
    assert!(!updater_wrong_key.verify_manifest(&manifest));

    // Corrupted signature (single bit flip in first byte) must be rejected
    let right_key_hex = hex::encode(signing_key.verifying_key().to_bytes());
    let updater_right_key = OtaUpdater::with_default_crypto("1.0.0", &right_key_hex);

    let mut sig_bytes = hex::decode(&manifest.ed25519_signature_hex).expect("valid hex sig");
    sig_bytes[0] ^= 0x01;
    let mut bad_sig_manifest = manifest.clone();
    bad_sig_manifest.ed25519_signature_hex = hex::encode(sig_bytes);
    assert!(!updater_right_key.verify_manifest(&bad_sig_manifest));

    // Signature created by wrong key over same payload must be rejected
    let wrong_sig = other_key.sign(&manifest.signing_payload());
    let mut wrong_signer_manifest = manifest;
    wrong_signer_manifest.ed25519_signature_hex = hex::encode(wrong_sig.to_bytes());
    assert!(!updater_right_key.verify_manifest(&wrong_signer_manifest));
}

#[test]
fn test_with_default_crypto_rejects_corrupted_artifact() {
    let signing_key = SigningKey::from_bytes(&[0x33u8; 32]);
    let pubkey_hex = hex::encode(signing_key.verifying_key().to_bytes());
    let artifact = b"genuine release tarball bytes";

    let manifest = build_signed_manifest(
        &signing_key,
        "3.0.0",
        ReleaseChannel::Stable,
        artifact,
        false,
    );
    let updater = OtaUpdater::with_default_crypto("2.9.9", &pubkey_hex);

    assert!(updater.verify_artifact(&manifest, artifact));
    assert!(!updater.verify_artifact(&manifest, b"genuine release tarball byteZ"));
    assert!(!updater.verify_artifact(&manifest, b""));
}

#[test]
fn test_ed25519_sha256_verifier_edge_cases() {
    let verifier = Ed25519Sha256Verifier::new();
    let signing_key = SigningKey::from_bytes(&[0x77u8; 32]);
    let pubkey_hex = hex::encode(signing_key.verifying_key().to_bytes());
    let payload = b"direct verifier payload";
    let sig_hex = hex::encode(signing_key.sign(payload).to_bytes());

    // Valid lowercase and uppercase hex
    assert!(verifier.verify_ed25519(&pubkey_hex, payload, &sig_hex));
    assert!(verifier.verify_ed25519(&pubkey_hex.to_uppercase(), payload, &sig_hex.to_uppercase()));

    // Invalid hex or wrong lengths
    assert!(!verifier.verify_ed25519("not-hex", payload, &sig_hex));
    assert!(!verifier.verify_ed25519(&pubkey_hex[..32], payload, &sig_hex));
    assert!(!verifier.verify_ed25519(&pubkey_hex, payload, "deadbeef"));

    // Direct SHA-256 verification
    let digest_hex = hex::encode(Sha256::digest(payload));
    assert!(verifier.verify_sha256(payload, &digest_hex));
    assert!(verifier.verify_sha256(payload, &digest_hex.to_uppercase()));
    assert!(!verifier.verify_sha256(b"other", &digest_hex));
    assert!(!verifier.verify_sha256(payload, "not-valid-hex"));
    assert!(!verifier.verify_sha256(payload, "abcd"));
}

#[test]
fn test_ota_updater_new_still_fails_closed() {
    let signing_key = SigningKey::from_bytes(&[0x55u8; 32]);
    let pubkey_hex = hex::encode(signing_key.verifying_key().to_bytes());
    let artifact = b"valid artifact";
    let manifest = build_signed_manifest(
        &signing_key,
        "1.1.0",
        ReleaseChannel::Stable,
        artifact,
        false,
    );

    let fail_closed_updater = OtaUpdater::new("1.0.0", ReleaseChannel::Stable, &pubkey_hex);
    assert!(!fail_closed_updater.verify_manifest(&manifest));
    assert!(!fail_closed_updater.verify_artifact(&manifest, artifact));
}

#[test]
fn test_zeroizing_secure_keystore_lifecycle() {
    let mut store = SecureKeyStore::new();
    store.store_secret("token", b"initial-secret-value", false);
    assert_eq!(
        store.retrieve_secret("token", None),
        Some(b"initial-secret-value".as_slice())
    );

    // Overwriting drops and zeroizes previous Zeroizing<Vec<u8>> buffer
    store.store_secret("token", b"rotated-secret-value", false);
    assert_eq!(
        store.retrieve_secret("token", None),
        Some(b"rotated-secret-value".as_slice())
    );

    // Delete and wipe_memory clear cleanly
    assert!(store.delete_secret("token"));
    assert_eq!(store.retrieve_secret("token", None), None);

    store.store_secret("session", b"ephemeral-key", false);
    store.wipe_memory();
    assert_eq!(store.key_info("session"), None);
}
