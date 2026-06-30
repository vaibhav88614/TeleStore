# Telegram Drive

**Telegram Drive** is an open-source, cross-platform desktop application that turns
your Telegram account into an unlimited, secure cloud storage drive. Built with
**Tauri**, **Rust**, and **React**.

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20MacOS%20%7C%20Linux-blue)]()
![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/vaibhav88614/TeleStore/total?style=flat)

</div>

##  What is Telegram Drive?

Telegram Drive leverages the Telegram API to allow you to upload, organize, and manage files directly on Telegram's servers. It treats your "Saved Messages" and created Channels as folders, giving you a familiar file explorer interface for your Telegram cloud.

###  Key Features

*   **Unlimited Cloud Storage**: Utilizing Telegram's generous cloud infrastructure.
*   **High Performance Grid**: Virtual scrolling handles folders with thousands of files instantly.
*   **Auto-Updates**: Seamless updates for Windows, macOS, and Linux.
*   **Media Streaming**: Stream video and audio files directly without downloading.
*   **PDF Viewer:** Built-in PDF support with infinite scrolling for seamless document reading.
*   **Drag & Drop**: Intuitive drag-and-drop upload and file management.
*   **Thumbnail Previews**: Inline thumbnails for images and media files.
*   **Folder Management**: Create "Folders" (private Telegram Channels) to organize content.
*   **Shareable Links**: Generate direct download links with optional password protection and expiration, and revoke access anytime from the dashboard. Also supports copying native Telegram message links for files in public channels.
*   **REST API for AI Integration**: Secure local API (off by default) with configurable port and API key auth. OpenAPI spec for seamless LLM and tool integration.
*   **Proxy Support**: Native integration for SOCKS5 and MTProto proxies to bypass regional restrictions and secure your traffic.
*   **VPN Optimizer**: Aggressive network tuning including bandwidth throttling, adjustable transfer chunk sizing, and adaptive keep-alives to ensure maximum stability on high-latency connections.
*   **Privacy Focused**: API keys and data stay local. No third-party servers.
*   **Cross-Platform**: Native apps for macOS (Intel/ARM), Windows, Linux and Android.

## Android (Pre‑built, Unsigned APK)

A pre-built **unsigned APK** is available for Android sideloading via the [v2.1.5-android release](https://github.com/vaibhav88614/TeleStore/releases/tag/Androidv2.1.5beta).

> [!WARNING]
> This APK is **not signed** and is **not available on the Google Play Store**. You must enable "Install from Unknown Sources" on your device to install it. This build contains **Google AdMob banner ads** to support development.

### How to Sideload

1. Download `Telegram-Drive-v2.1.5-beta.apk` from the [v2.1.5-android release](https://github.com/vaibhav88614/TeleStore/releases/tag/Androidv2.1.5beta).
2. On your Android device, go to **Settings → Apps → Special App Access → Install unknown apps** and allow your browser or file manager.
3. Open the downloaded APK and tap **Install**.
4. Enter your Telegram API credentials on first launch (same as the desktop app).

> [!NOTE]
> - **Compatibility**: Requires **Android 7.0 (API level 24)** or higher.
> - **Android 15+ Installation**: If you encounter blocks or security restrictions when installing on Android 15+ emulator/device, bypass it using ADB:
>   ```bash
>   adb install --bypass-low-target-sdk-block Telegram-Drive-v2.1.5-beta.apk
>   ```
> - The Android build is a **community/beta release** compiled locally. The desktop app (Windows/macOS/Linux) remains the primary supported platform, built and signed automatically by GitHub CI.

---

##  Tech Stack

*   **Frontend**: React, TypeScript, TailwindCSS, Framer Motion
*   **Backend**: Rust (Tauri), Grammers (Telegram Client)
*   **Build Tool**: Vite


##  Getting Started

This guide walks you through every step of getting a local development build of
TeleStore running on Windows, macOS, or Linux. If you have never built a Tauri
or Rust project before, follow each section in order.

### 1. Quick Start (Automated Setup)

For convenience, this repo ships with bootstrap scripts that check every
prerequisite, install what's missing where possible, and prepare the project so
you can launch it with a single command.

*   **macOS / Linux**
    ```bash
    git clone https://github.com/vaibhav88614/TeleStore.git
    cd TeleStore
    chmod +x setup.sh
    ./setup.sh
    ```
*   **Windows (PowerShell)**
    ```powershell
    git clone https://github.com/vaibhav88614/TeleStore.git
    cd TeleStore
    # If script execution is blocked, allow it for this process only:
    Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
    .\setup.ps1
    ```

The scripts are **non-destructive** — they only ever read your system, install
missing packages from official sources, and run `npm install` inside `app/`.
They never modify global settings without telling you.

If the script reports a missing prerequisite it cannot install for you, follow
the **Manual Setup** below for that component.

### 2. Manual Setup

#### 2.1 Prerequisites Checklist

| Requirement              | Minimum Version | How to verify                       |
| ------------------------ | --------------- | ----------------------------------- |
| Git                      | 2.30+           | `git --version`                     |
| Node.js (with npm)       | 18.0+           | `node --version` / `npm --version`  |
| Rust toolchain (rustup)  | 1.77+ stable    | `rustc --version` / `cargo --version` |
| OS build tools           | see 2.4         | varies                              |
| Telegram API credentials | n/a             | obtained from my.telegram.org       |

#### 2.2 Install Node.js

TeleStore is built with Vite and Node.js 18 or newer is required.

*   **macOS / Linux** — The recommended way is via [`nvm`](https://github.com/nvm-sh/nvm):
    ```bash
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
    # restart your shell, then:
    nvm install --lts
    nvm use --lts
    ```
*   **Windows** — Download the LTS installer from [nodejs.org](https://nodejs.org/),
    or install via `winget`:
    ```powershell
    winget install -e --id OpenJS.NodeJS.LTS
    ```

Verify:
```bash
node --version   # should print v18.x.x or higher
npm  --version
```

#### 2.3 Install Rust

Rust compiles the Tauri backend. Install the toolchain via [rustup](https://rustup.rs/):

*   **macOS / Linux**
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    # follow the prompts (default install is fine), then:
    source "$HOME/.cargo/env"
    ```
*   **Windows** — Download and run [`rustup-init.exe`](https://rustup.rs/) and
    accept the defaults. The installer will set up `cargo`, `rustc`, and add
    them to your `PATH`. Close and reopen your terminal afterwards.

Verify:
```bash
rustc --version
cargo --version
```

#### 2.4 Install OS-Specific Tauri Build Tools

Tauri compiles into a native binary and therefore needs the same toolchain a
C/C++ developer would use on your platform.

*   **Windows**
    1.  Install the [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
        During installation, tick the **"Desktop development with C++"** workload.
        Without this you will see `linker 'link.exe' not found` when running `cargo build`.
    2.  Ensure the [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section)
        is installed. Windows 10 (2004+) and Windows 11 ship with it; older
        installs need it manually.
*   **macOS**
    1.  Install the Xcode Command Line Tools:
        ```bash
        xcode-select --install
        ```
    2.  Optional: a full Xcode install is **not** required.
*   **Linux (Ubuntu/Debian and derivatives)**
    ```bash
    sudo apt update
    sudo apt install -y \
        libwebkit2gtk-4.1-dev \
        build-essential curl wget file \
        libxdo-dev libssl-dev \
        libayatana-appindicator3-dev \
        librsvg2-dev pkg-config
    ```
*   **Linux (Fedora/RHEL)**
    ```bash
    sudo dnf install -y \
        webkit2gtk4.1-devel openssl-devel curl wget file \
        libappindicator-gtk3-devel librsvg2-devel \
        gcc gcc-c++ make pkgconf-pkg-config libxdo-devel
    ```
*   **Linux (Arch / Manjaro)**
    ```bash
    sudo pacman -S --needed \
        webkit2gtk-4.1 base-devel curl wget file \
        openssl appmenu-gtk-module libappindicator-gtk3 \
        librsvg xdotool
    ```

For anything else, see the official
[Tauri v2 Prerequisites Guide](https://v2.tauri.app/start/prerequisites/).

#### 2.5 Obtain Telegram API Credentials

TeleStore communicates directly with Telegram's MTProto API. Telegram requires
each client to register its own `api_id` and `api_hash`. They are free, take
~2 minutes, and stay private to your account.

1.  Open [https://my.telegram.org](https://my.telegram.org) in a browser.
2.  Log in with your phone number and the confirmation code Telegram sends.
3.  Click **API development tools**.
4.  Fill in any "App title" and "Short name" (e.g. `TeleStore Personal`). The
    URL and platform fields can be left blank.
5.  Click **Create application** to reveal your **`api_id`** (a number) and
    **`api_hash`** (a 32-character hex string).
6.  Keep both values handy — you will paste them on the first launch screen.

> [!IMPORTANT]
> Treat your `api_hash` like a password. Do **not** commit it to git, post it
> in screenshots, or share it publicly. Anyone with both values can impersonate
> your client app to Telegram.

#### 2.6 Clone the Repository

```bash
git clone https://github.com/vaibhav88614/TeleStore.git
cd TeleStore
```

#### 2.7 Install Node Dependencies

All JavaScript/TypeScript source code lives in the `app/` subfolder.

```bash
cd app
npm install
```

This pulls in React, Tauri's JS bindings, Vite, and other frontend libraries
(~250 packages). It can take 1–3 minutes on a fresh checkout.

#### 2.8 Run the App in Development Mode

Still inside `app/`:

```bash
npm run tauri dev
```

What happens on first run:

1.  Vite spins up the React dev server on `http://localhost:1420`.
2.  Cargo downloads and compiles **\~300 Rust crates** for the Tauri backend.
    This step is one-time and typically takes **5–15 minutes**, depending on
    your CPU and disk speed.
3.  A native window opens with the **Welcome / Connect** screen.
4.  Paste your `api_id` and `api_hash`, click **Connect**, then complete the
    Telegram login flow (QR code on desktop, phone number + SMS otherwise).

Subsequent runs reuse cached crates and start in seconds.

#### 2.9 Build a Production Binary

When you're ready to ship a release build:

```bash
npm run tauri build
```

The finished binary, installer, and (on macOS) `.app` bundle land in
`app/src-tauri/target/release/` and `app/src-tauri/target/release/bundle/`.

### 3. Troubleshooting Common First-Run Issues

| Symptom                                                          | Fix                                                                                                         |
| ---------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| `linker 'link.exe' not found` (Windows)                          | Install the **C++ Build Tools** workload from Visual Studio Installer (see 2.4).                            |
| `failed to run custom build command for openssl-sys` (Linux)     | Install `libssl-dev` (Debian/Ubuntu) or `openssl-devel` (Fedora).                                           |
| `webkit2gtk-4.1` not found (Linux)                               | Install the GTK/WebKit packages listed in 2.4 for your distro.                                              |
| `npm ERR! code EACCES` (Linux/macOS)                             | Don't `sudo npm`. Use `nvm` so your user owns the Node prefix (see 2.2).                                    |
| `xcrun: error: invalid active developer path` (macOS)            | Re-run `xcode-select --install` and accept the licence with `sudo xcodebuild -license`.                     |
| Window is blank after `tauri dev` finishes building (Windows)    | Install the **WebView2 Runtime** (see 2.4).                                                                 |
| `FLOOD_WAIT_X` from Telegram on first login                      | Wait the indicated number of seconds. Telegram rate-limits new clients aggressively for the first few uses. |
| Vite reports port `1420` is in use                               | Another instance is already running, or change the port in `app/vite.config.ts` and `tauri.conf.json`.      |

> [!NOTE]
> **NPM Vulnerabilities:** You may see vulnerability warnings during `npm install`.
> These usually relate to build tools and dev dependencies. You can optionally
> run `npm audit fix`, but it is not required to run the app.

##  Open Source & License

This project is **Free and Open Source Software**. You are free to use, modify, and distribute it.

Licensed under the **MIT License**.

---
*Disclaimer: This application is not affiliated with Telegram FZ-LLC. Use responsibly and in accordance with Telegram's Terms of Service.*

