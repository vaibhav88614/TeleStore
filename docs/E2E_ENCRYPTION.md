# Client-Side End-to-End Encryption (Track C)

Files stored in Telegram "Saved Messages" / private channels are **not**
end-to-end encrypted — Telegram (the company) can read them. TeleStore adds an
optional layer so file bytes are encrypted **locally** before upload and
decrypted **after** download, leaving only opaque ciphertext on Telegram.

## What ships now

`app/src-tauri/src/crypto.rs` is a complete, unit-tested primitive:

- **Key derivation:** Argon2id(passphrase, salt) → 32-byte key
- **Cipher:** XChaCha20-Poly1305 (AEAD; 24-byte nonce, 16-byte tag)
- **Self-describing blob:** `magic("TSE1") | salt[16] | nonce[24] | ciphertext+tag`

```rust
use app_lib::crypto;

let blob = crypto::encrypt(passphrase, plaintext)?;   // before upload
let plain = crypto::decrypt(passphrase, &blob)?;      // after download
let looks_encrypted = crypto::is_encrypted_blob(&bytes);
```

Run the tests:

```bash
cd app/src-tauri
cargo test --lib crypto
```

## Integration points (to wire the pipeline)

The streaming upload/download paths are chunked, so a production integration
should stream-encrypt rather than buffering whole files. Two strategies:

1. **Whole-file (simplest, good for < ~100 MB):** buffer the file, call
   `crypto::encrypt`, upload the blob; on download, buffer + `crypto::decrypt`.
2. **Chunked/streaming (preferred for large files):** switch to a framed AEAD
   such as `chacha20poly1305`'s `STREAM` construction (one nonce prefix +
   per-chunk counter), encrypting each upload chunk and decrypting each ranged
   download chunk. Note: this breaks HTTP range requests / seekable streaming,
   so encrypted media can't be streamed mid-file — it must be fully fetched
   then decrypted. Gate this behind a per-file "encrypted" flag.

Concrete hook locations:

| Direction | File | Where |
| --- | --- | --- |
| Upload | `app/src-tauri/src/upload_service.rs` and `commands/fs.rs` (`cmd_upload_file`, `initiate_upload`) | Encrypt bytes/chunks before handing them to grammers. Store an `encrypted=true` marker (e.g. in the message caption or a filename suffix like `.tse`). |
| Download | `commands/fs.rs` (`cmd_download_file`) | After fetching, detect `crypto::is_encrypted_blob` (or the marker) and `decrypt` before writing to disk. |
| Streaming | `server.rs` (`build_media_response`) | Encrypted files cannot be byte-range streamed; fall back to full download + decrypt, or disable in-app streaming for `.tse` files. |

### Passphrase handling

- Prompt for the encryption passphrase once per session; keep the derived key in
  memory only (never persist the passphrase).
- Recommended: reuse the **App Lock** PIN flow as the unlock gate, and store the
  long encryption passphrase in the OS keychain (see below) unlocked by the PIN.

## OS-keychain upgrade (session + passphrase at rest)

The current **App Lock** (`hooks/useAppLock.ts`) is a UI gate only; the grammers
`telegram.session` file is still plaintext on disk. To close that gap:

1. Add the `keyring` crate (feature-gated to avoid affecting the default build):
   ```toml
   [features]
   keychain = ["dep:keyring"]
   [dependencies]
   keyring = { version = "3", optional = true }
   ```
2. On first setup, generate a random 32-byte master key, store it in the OS
   keychain (`keyring::Entry::new("com.telestore.app", "session-key")`), and use
   `crypto`-style AEAD to encrypt `telegram.session` at rest.
3. On launch, read the master key from the keychain (unlocked by the OS login /
   the App Lock PIN), decrypt the session into a temp/in-memory location, and
   re-encrypt on exit.

Windows → Credential Manager, macOS → Keychain, Linux → Secret Service. This is
documented as an upgrade rather than enabled by default because it changes the
session storage format and needs a migration path.
