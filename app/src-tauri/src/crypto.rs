//! Client-side end-to-end encryption primitives (Track C).
//!
//! Files in Telegram "Saved Messages" / private channels are NOT end-to-end
//! encrypted — Telegram (the company) can read them. This module provides the
//! building blocks to encrypt file bytes locally BEFORE upload and decrypt them
//! AFTER download, so Telegram only ever stores opaque ciphertext.
//!
//! Scheme:
//!   - Key derivation:  Argon2id(passphrase, salt) -> 32-byte key
//!   - Encryption:      XChaCha20-Poly1305 (AEAD, 24-byte nonce, 16-byte tag)
//!
//! Container layout for an encrypted blob:
//!   magic[4] = b"TSE1"  | salt[16] | nonce[24] | ciphertext+tag[..]
//!
//! The passphrase never touches disk or Telegram; only the per-file salt and
//! nonce are stored alongside the ciphertext (both are safe to be public).
//!
//! Integration points (see docs/E2E_ENCRYPTION.md):
//!   - Upload:   encrypt the file/stream in `upload_service.rs` before sending.
//!   - Download: decrypt in the download path in `commands/fs.rs`.

use argon2::Argon2;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng, rand_core::RngCore},
    XChaCha20Poly1305, XNonce,
};

/// Magic prefix identifying a TeleStore-encrypted blob (version 1).
pub const MAGIC: &[u8; 4] = b"TSE1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;
const KEY_LEN: usize = 32;
/// Bytes of header before the ciphertext: magic + salt + nonce.
pub const HEADER_LEN: usize = 4 + SALT_LEN + NONCE_LEN;

#[derive(Debug)]
pub enum CryptoError {
    KeyDerivation,
    Encrypt,
    Decrypt,
    BadHeader,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::KeyDerivation => write!(f, "key derivation failed"),
            CryptoError::Encrypt => write!(f, "encryption failed"),
            CryptoError::Decrypt => write!(f, "decryption failed (wrong passphrase or corrupt data)"),
            CryptoError::BadHeader => write!(f, "not a TeleStore-encrypted blob (bad header)"),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Derive a 32-byte key from a passphrase + salt using Argon2id.
pub fn derive_key(passphrase: &[u8], salt: &[u8]) -> Result<[u8; KEY_LEN], CryptoError> {
    let mut key = [0u8; KEY_LEN];
    Argon2::default()
        .hash_password_into(passphrase, salt, &mut key)
        .map_err(|_| CryptoError::KeyDerivation)?;
    Ok(key)
}

/// Encrypt `plaintext` with a passphrase. Output is a self-describing blob:
/// `magic | salt | nonce | ciphertext+tag`.
pub fn encrypt(passphrase: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);

    let key = derive_key(passphrase, &salt)?;
    let cipher = XChaCha20Poly1305::new((&key).into());
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng); // 24 bytes

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| CryptoError::Encrypt)?;

    let mut out = Vec::with_capacity(HEADER_LEN + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&salt);
    out.extend_from_slice(nonce.as_slice());
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt a blob produced by [`encrypt`].
pub fn decrypt(passphrase: &[u8], blob: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if blob.len() < HEADER_LEN || &blob[0..4] != MAGIC {
        return Err(CryptoError::BadHeader);
    }
    let salt = &blob[4..4 + SALT_LEN];
    let nonce_bytes = &blob[4 + SALT_LEN..HEADER_LEN];
    let ciphertext = &blob[HEADER_LEN..];

    let key = derive_key(passphrase, salt)?;
    let cipher = XChaCha20Poly1305::new((&key).into());
    let nonce = XNonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::Decrypt)
}

/// Quick check whether a blob looks like a TeleStore-encrypted file.
pub fn is_encrypted_blob(blob: &[u8]) -> bool {
    blob.len() >= 4 && &blob[0..4] == MAGIC
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_recovers_plaintext() {
        let pass = b"correct horse battery staple";
        let msg = b"the quick brown fox jumps over the lazy dog";
        let blob = encrypt(pass, msg).expect("encrypt");
        assert!(is_encrypted_blob(&blob));
        assert_eq!(&blob[0..4], MAGIC);
        let out = decrypt(pass, &blob).expect("decrypt");
        assert_eq!(out, msg);
    }

    #[test]
    fn wrong_passphrase_fails() {
        let blob = encrypt(b"right-pass", b"secret").expect("encrypt");
        assert!(decrypt(b"wrong-pass", &blob).is_err());
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let mut blob = encrypt(b"pw", b"hello world").expect("encrypt");
        let last = blob.len() - 1;
        blob[last] ^= 0xFF; // flip a bit in the tag/ciphertext
        assert!(decrypt(b"pw", &blob).is_err());
    }

    #[test]
    fn rejects_non_blob() {
        assert!(matches!(decrypt(b"pw", b"not-a-blob"), Err(CryptoError::BadHeader)));
        assert!(!is_encrypted_blob(b"xx"));
    }

    #[test]
    fn unique_salt_and_nonce_per_encryption() {
        let a = encrypt(b"pw", b"data").unwrap();
        let b = encrypt(b"pw", b"data").unwrap();
        // Same plaintext + passphrase must still produce different blobs
        // (random salt + nonce), so ciphertext is non-deterministic.
        assert_ne!(a, b);
    }
}
