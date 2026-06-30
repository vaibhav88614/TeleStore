#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# TeleStore first-time setup script (macOS / Linux)
#
# Verifies prerequisites, offers to install anything that's missing where it
# can be done safely, then runs `npm install` inside `app/`. The script is
# non-destructive: it never modifies global state without printing what it is
# about to do, and it always asks before invoking sudo.
#
# Usage:
#   chmod +x setup.sh
#   ./setup.sh
# ---------------------------------------------------------------------------

set -euo pipefail

# ----- pretty output helpers -----------------------------------------------
if [ -t 1 ]; then
    BOLD="\033[1m"; DIM="\033[2m"; RED="\033[31m"; GRN="\033[32m"
    YLW="\033[33m"; BLU="\033[34m"; CYA="\033[36m"; RST="\033[0m"
else
    BOLD=""; DIM=""; RED=""; GRN=""; YLW=""; BLU=""; CYA=""; RST=""
fi

info()    { printf "%b[info]%b  %s\n"  "${BLU}" "${RST}" "$*"; }
ok()      { printf "%b[ ok ]%b  %s\n"  "${GRN}" "${RST}" "$*"; }
warn()    { printf "%b[warn]%b  %s\n"  "${YLW}" "${RST}" "$*"; }
error()   { printf "%b[fail]%b  %s\n"  "${RED}" "${RST}" "$*" >&2; }
section() { printf "\n%b== %s ==%b\n"  "${BOLD}${CYA}" "$*" "${RST}"; }

ask_yes_no() {
    local prompt="${1:-Continue?} [y/N] "
    local answer=""
    if [ ! -t 0 ]; then
        warn "non-interactive shell, assuming 'no'"
        return 1
    fi
    read -r -p "$prompt" answer
    case "${answer,,}" in
        y|yes) return 0 ;;
        *)     return 1 ;;
    esac
}

have_cmd() { command -v "$1" >/dev/null 2>&1; }

# ----- detect OS / distro --------------------------------------------------
OS="$(uname -s)"
DISTRO=""
PKG_MGR=""

detect_linux_distro() {
    if [ -r /etc/os-release ]; then
        # shellcheck disable=SC1091
        . /etc/os-release
        DISTRO="${ID:-unknown}"
    fi
    case "$DISTRO" in
        ubuntu|debian|linuxmint|pop|elementary|raspbian) PKG_MGR="apt"  ;;
        fedora|rhel|centos|rocky|almalinux)              PKG_MGR="dnf"  ;;
        arch|manjaro|endeavouros|garuda)                 PKG_MGR="pacman" ;;
        opensuse*|sles)                                  PKG_MGR="zypper" ;;
        *)                                               PKG_MGR=""     ;;
    esac
}

case "$OS" in
    Darwin) info "Detected macOS" ;;
    Linux)
        detect_linux_distro
        info "Detected Linux distro: ${DISTRO:-unknown} (package manager: ${PKG_MGR:-none})"
        ;;
    *)
        error "Unsupported OS: $OS. Use setup.ps1 on Windows."
        exit 1
        ;;
esac

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
APP_DIR="${REPO_ROOT}/app"

if [ ! -d "$APP_DIR" ]; then
    error "Expected to find app/ at ${APP_DIR}. Run this script from the repo root."
    exit 1
fi

# ----- 1. Git --------------------------------------------------------------
section "Git"
if have_cmd git; then
    ok "git is installed ($(git --version))"
else
    error "git not found. Install it from https://git-scm.com/ and retry."
    exit 1
fi

# ----- 2. Node.js ---------------------------------------------------------
section "Node.js (>= 18)"
need_node_install=0
if have_cmd node; then
    NODE_VERSION="$(node --version | sed 's/^v//')"
    NODE_MAJOR="${NODE_VERSION%%.*}"
    if [ "${NODE_MAJOR:-0}" -ge 18 ]; then
        ok "Node.js v${NODE_VERSION}"
    else
        warn "Node.js v${NODE_VERSION} is too old (need >= 18)"
        need_node_install=1
    fi
else
    warn "Node.js not found"
    need_node_install=1
fi

if [ "$need_node_install" -eq 1 ]; then
    info "Recommended install method: nvm (https://github.com/nvm-sh/nvm)"
    cat <<EOF
   curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
   # then restart your shell and run:
   nvm install --lts
   nvm use --lts
EOF
    error "Install Node.js >= 18 and re-run this script."
    exit 1
fi

if ! have_cmd npm; then
    error "npm not found alongside Node. Reinstall Node.js."
    exit 1
fi
ok "npm $(npm --version)"

# ----- 3. Rust toolchain --------------------------------------------------
section "Rust toolchain (rustup / cargo)"
if have_cmd cargo && have_cmd rustc; then
    ok "$(rustc --version)"
    ok "$(cargo --version)"
else
    warn "Rust toolchain not found"
    info "rustup install command:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    if ask_yes_no "Run the rustup installer now (recommended)?"; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
        # shellcheck disable=SC1091
        . "$HOME/.cargo/env"
        ok "$(rustc --version)"
    else
        error "Install Rust manually and re-run this script."
        exit 1
    fi
fi

# ----- 4. Platform-specific Tauri prerequisites ---------------------------
section "Tauri platform prerequisites"

install_apt() {
    local pkgs=(
        libwebkit2gtk-4.1-dev
        build-essential curl wget file
        libxdo-dev libssl-dev
        libayatana-appindicator3-dev
        librsvg2-dev pkg-config
    )
    info "Will install (apt): ${pkgs[*]}"
    if ask_yes_no "Run 'sudo apt install' for these now?"; then
        sudo apt update
        sudo apt install -y "${pkgs[@]}"
        ok "apt packages installed"
    else
        warn "Skipped apt install; install the packages manually before building."
    fi
}

install_dnf() {
    local pkgs=(
        webkit2gtk4.1-devel openssl-devel curl wget file
        libappindicator-gtk3-devel librsvg2-devel
        gcc gcc-c++ make pkgconf-pkg-config libxdo-devel
    )
    info "Will install (dnf): ${pkgs[*]}"
    if ask_yes_no "Run 'sudo dnf install' for these now?"; then
        sudo dnf install -y "${pkgs[@]}"
        ok "dnf packages installed"
    else
        warn "Skipped dnf install; install the packages manually before building."
    fi
}

install_pacman() {
    local pkgs=(
        webkit2gtk-4.1 base-devel curl wget file
        openssl appmenu-gtk-module libappindicator-gtk3
        librsvg xdotool
    )
    info "Will install (pacman): ${pkgs[*]}"
    if ask_yes_no "Run 'sudo pacman -S --needed' for these now?"; then
        sudo pacman -S --needed --noconfirm "${pkgs[@]}"
        ok "pacman packages installed"
    else
        warn "Skipped pacman install; install the packages manually before building."
    fi
}

install_zypper() {
    local pkgs=(
        webkit2gtk3-soup2-devel libopenssl-devel curl wget file
        libappindicator3-1 librsvg-devel
        gcc gcc-c++ make pkg-config xdotool-devel
    )
    info "Will install (zypper): ${pkgs[*]}"
    if ask_yes_no "Run 'sudo zypper install' for these now?"; then
        sudo zypper install -y "${pkgs[@]}"
        ok "zypper packages installed"
    else
        warn "Skipped zypper install; install the packages manually before building."
    fi
}

case "$OS" in
    Darwin)
        if xcode-select -p >/dev/null 2>&1; then
            ok "Xcode Command Line Tools installed ($(xcode-select -p))"
        else
            warn "Xcode Command Line Tools missing"
            info "Triggering installer (a GUI prompt will appear)..."
            xcode-select --install || true
            warn "Re-run this script after the installer finishes."
            exit 1
        fi
        ;;
    Linux)
        case "$PKG_MGR" in
            apt)    install_apt    ;;
            dnf)    install_dnf    ;;
            pacman) install_pacman ;;
            zypper) install_zypper ;;
            *)
                warn "Unknown package manager. Install Tauri prerequisites manually:"
                warn "https://v2.tauri.app/start/prerequisites/"
                ;;
        esac
        ;;
esac

# ----- 5. Node dependencies ----------------------------------------------
section "Frontend dependencies (npm install)"
info "Running 'npm install' inside ${APP_DIR} ..."
(cd "$APP_DIR" && npm install)
ok "Frontend dependencies installed"

# ----- Done --------------------------------------------------------------
section "All set!"
cat <<EOF
${GRN}TeleStore is ready to build.${RST}

Next steps:
  ${BOLD}1.${RST}  Obtain Telegram API credentials at https://my.telegram.org
      (See README \xc2\xa7 2.5 for a walk-through.)
  ${BOLD}2.${RST}  Launch the dev build:
        cd app
        npm run tauri dev
      ${DIM}(The very first run compiles ~300 Rust crates and may take 5\xe2\x80\x9315 minutes.)${RST}
  ${BOLD}3.${RST}  When ready to ship, produce a release binary:
        cd app
        npm run tauri build

Build artifacts will land in:
  app/src-tauri/target/release/
  app/src-tauri/target/release/bundle/

Happy hacking!
EOF
