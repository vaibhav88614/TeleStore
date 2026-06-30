# Code Signing & Release Trust (Track C)

To ship TeleStore as a real product, builds must be signed so Windows
SmartScreen and macOS Gatekeeper don't block users. This guide pairs with
`.github/workflows/release-signed.yml`.

## 1. Updater signing key (only if you re-enable auto-update)

Auto-update is **disabled** in this build (see `hooks/useUpdateCheck.ts`). If you
run your own release feed, regenerate the trust key — never reuse upstream's:

```bash
npx @tauri-apps/cli signer generate -w ~/.tauri/telestore.key
```

Then put the **public** key back in `app/src-tauri/tauri.conf.json` under
`plugins.updater.pubkey`, restore the endpoints to YOUR repo, re-add
`tauri_plugin_updater` in `src/lib.rs` + the `updater:default` capability, and
restore the plugin-based `useUpdateCheck` hook. Store the private key +
password as `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.

## 2. Windows Authenticode

1. Obtain a code-signing certificate (OV/EV from a CA, or an Azure Trusted
   Signing account). Export as `.pfx`.
2. Base64-encode and store as secrets:
   - `WINDOWS_CERTIFICATE` = `base64 cert.pfx`
   - `WINDOWS_CERTIFICATE_PASSWORD`
3. `tauri-action` consumes `TAURI_WINDOWS_CERTIFICATE` /
   `TAURI_WINDOWS_CERTIFICATE_PASSWORD` (already mapped in the workflow).

EV certs reset SmartScreen reputation immediately; OV certs build reputation
over downloads/time.

## 3. macOS Developer ID + notarization

1. In the Apple Developer portal create a **Developer ID Application** cert,
   export as `.p12`.
2. Secrets:
   - `APPLE_CERTIFICATE` = `base64 cert.p12`
   - `APPLE_CERTIFICATE_PASSWORD`
   - `APPLE_SIGNING_IDENTITY` = `Developer ID Application: Your Name (TEAMID)`
   - `APPLE_ID`, `APPLE_PASSWORD` (app-specific password), `APPLE_TEAM_ID`
3. The workflow imports the cert into a temp keychain; `tauri-action` signs and
   notarizes when the `APPLE_*` env vars are present.

## 4. Signed Android APK/AAB

1. Generate a release keystore (keep it secret, back it up):
   ```bash
   keytool -genkey -v -keystore release.keystore -alias telestore \
     -keyalg RSA -keysize 2048 -validity 10000
   ```
2. Secrets:
   - `ANDROID_KEYSTORE_BASE64` = `base64 release.keystore`
   - `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`
3. Initialize the Tauri Android project once locally
   (`npx tauri android init` — produces the gitignored `src-android`), commit it
   to a private fork or restore it in CI, then `npx tauri android build --apk`
   and sign with `apksigner` using the decoded keystore. Fill in the TODO step
   in `release-signed.yml`.

## 5. Supply-chain hardening (recommended)

- Enable Dependabot (`.github/dependabot.yml`) for `cargo` + `npm`.
- Enable CodeQL scanning.
- Pin third-party GitHub Actions to commit SHAs (the upstream workflows use
  floating tags like `@v4`).
- The `audit.yml` workflow already runs `cargo audit` + `npm audit`.
