# ---------------------------------------------------------------------------
# TeleStore first-time setup script (Windows / PowerShell 5.1+)
#
# Verifies prerequisites, offers to install anything that's missing where
# possible (via winget when available), then runs `npm install` inside `app\`.
# It never modifies system state without telling you first.
#
# Usage:
#   # If script execution is blocked, allow it for this process only:
#   Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
#   .\setup.ps1
# ---------------------------------------------------------------------------

[CmdletBinding()]
param(
    [switch]$NonInteractive
)

$ErrorActionPreference = 'Stop'

# ----- pretty output -------------------------------------------------------
function Write-Info    ($msg) { Write-Host "[info]  $msg" -ForegroundColor Cyan }
function Write-Ok      ($msg) { Write-Host "[ ok ]  $msg" -ForegroundColor Green }
function Write-Warn    ($msg) { Write-Host "[warn]  $msg" -ForegroundColor Yellow }
function Write-Fail    ($msg) { Write-Host "[fail]  $msg" -ForegroundColor Red }
function Write-Section ($msg) {
    Write-Host ""
    Write-Host "== $msg ==" -ForegroundColor Magenta
}

function Confirm-YesNo {
    param([string]$Prompt = 'Continue?')
    if ($NonInteractive) {
        Write-Warn "Non-interactive mode, assuming 'no'."
        return $false
    }
    while ($true) {
        $a = Read-Host "$Prompt [y/N]"
        switch -Regex ($a.Trim().ToLower()) {
            '^(y|yes)$' { return $true }
            '^(n|no|)$' { return $false }
            default { Write-Warn "Please answer y or n." }
        }
    }
}

function Test-Cmd {
    param([string]$Name)
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

# ----- preamble ------------------------------------------------------------
$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$AppDir   = Join-Path $RepoRoot 'app'

if (-not (Test-Path $AppDir)) {
    Write-Fail "Expected to find app\ at $AppDir. Run this script from the repo root."
    exit 1
}

Write-Info "OS: $([System.Environment]::OSVersion.VersionString)"
Write-Info "Repo: $RepoRoot"

$haveWinget = Test-Cmd winget
if ($haveWinget) {
    Write-Info "winget detected; offers to auto-install will use it."
} else {
    Write-Warn "winget not detected; you'll need to install missing prerequisites manually."
}

# ----- 1. Git --------------------------------------------------------------
Write-Section "Git"
if (Test-Cmd git) {
    Write-Ok ("git is installed (" + (git --version) + ")")
} else {
    Write-Fail "git not found."
    if ($haveWinget -and (Confirm-YesNo "Install Git via winget now?")) {
        winget install -e --id Git.Git --accept-package-agreements --accept-source-agreements
        Write-Warn "Open a new PowerShell window so PATH picks up git, then re-run setup.ps1."
        exit 0
    } else {
        Write-Info "Download from https://git-scm.com/download/win and re-run."
        exit 1
    }
}

# ----- 2. Node.js ----------------------------------------------------------
Write-Section "Node.js (>= 18)"
$needNode = $false
if (Test-Cmd node) {
    $nodeVerRaw = (node --version).Trim().TrimStart('v')
    try {
        $nodeMajor = [int]($nodeVerRaw.Split('.')[0])
    } catch {
        $nodeMajor = 0
    }
    if ($nodeMajor -ge 18) {
        Write-Ok "Node.js v$nodeVerRaw"
    } else {
        Write-Warn "Node.js v$nodeVerRaw is too old (need >= 18)"
        $needNode = $true
    }
} else {
    Write-Warn "Node.js not found"
    $needNode = $true
}

if ($needNode) {
    if ($haveWinget -and (Confirm-YesNo "Install Node.js LTS via winget now?")) {
        winget install -e --id OpenJS.NodeJS.LTS --accept-package-agreements --accept-source-agreements
        Write-Warn "Open a new PowerShell window so PATH picks up node/npm, then re-run setup.ps1."
        exit 0
    } else {
        Write-Info "Download the LTS installer from https://nodejs.org/ and re-run."
        exit 1
    }
}

if (-not (Test-Cmd npm)) {
    Write-Fail "npm not found alongside Node. Reinstall Node.js."
    exit 1
}
Write-Ok ("npm " + (npm --version))

# ----- 3. Rust toolchain ---------------------------------------------------
Write-Section "Rust toolchain (rustup / cargo)"
if ((Test-Cmd cargo) -and (Test-Cmd rustc)) {
    Write-Ok ((rustc --version) -as [string])
    Write-Ok ((cargo --version) -as [string])
} else {
    Write-Warn "Rust toolchain not found"
    if ($haveWinget -and (Confirm-YesNo "Install Rustup via winget now?")) {
        winget install -e --id Rustlang.Rustup --accept-package-agreements --accept-source-agreements
        Write-Warn "Close and reopen PowerShell so PATH picks up cargo, then re-run setup.ps1."
        exit 0
    } else {
        Write-Info "Download and run rustup-init.exe from https://rustup.rs/"
        Write-Info "After install, open a fresh PowerShell window and re-run setup.ps1."
        exit 1
    }
}

# ----- 4. MSVC C++ Build Tools --------------------------------------------
Write-Section "MSVC C++ Build Tools (required by Rust on Windows)"
$vswherePath = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$haveBuildTools = $false
if (Test-Path $vswherePath) {
    try {
        $result = & $vswherePath -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($LASTEXITCODE -eq 0 -and $result) { $haveBuildTools = $true }
    } catch {
        $haveBuildTools = $false
    }
}

if ($haveBuildTools) {
    Write-Ok "VC++ Build Tools detected"
} else {
    Write-Warn "MSVC C++ Build Tools (Desktop development with C++) not detected."
    Write-Info "Without these, 'cargo build' fails with: linker 'link.exe' not found."
    if ($haveWinget -and (Confirm-YesNo "Install 'Microsoft.VisualStudio.2022.BuildTools' via winget now?")) {
        winget install -e --id Microsoft.VisualStudio.2022.BuildTools --accept-package-agreements --accept-source-agreements --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows11SDK.22621 --includeRecommended"
        Write-Warn "Re-open PowerShell after install completes, then re-run setup.ps1."
        exit 0
    } else {
        Write-Info "Manual download: https://visualstudio.microsoft.com/visual-cpp-build-tools/"
        Write-Info "During install, tick the 'Desktop development with C++' workload."
        exit 1
    }
}

# ----- 5. WebView2 Runtime -------------------------------------------------
Write-Section "WebView2 Runtime"
$webview2Found = $false
$wvKeys = @(
    'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}',
    'HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}',
    'HKCU:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
)
foreach ($k in $wvKeys) {
    if (Test-Path $k) { $webview2Found = $true; break }
}

if ($webview2Found) {
    Write-Ok "WebView2 Runtime detected"
} else {
    Write-Warn "WebView2 Runtime not detected (most Windows 10 2004+ / Windows 11 already have it)."
    if ($haveWinget -and (Confirm-YesNo "Install Microsoft.EdgeWebView2Runtime via winget now?")) {
        winget install -e --id Microsoft.EdgeWebView2Runtime --accept-package-agreements --accept-source-agreements
    } else {
        Write-Info "Download from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section"
    }
}

# ----- 6. Frontend dependencies -------------------------------------------
Write-Section "Frontend dependencies (npm install)"
Write-Info "Running 'npm install' inside $AppDir ..."
Push-Location $AppDir
try {
    npm install
    if ($LASTEXITCODE -ne 0) { throw "npm install exited with code $LASTEXITCODE" }
    Write-Ok "Frontend dependencies installed"
} finally {
    Pop-Location
}

# ----- Done ---------------------------------------------------------------
Write-Section "All set!"
Write-Host ""
Write-Host "TeleStore is ready to build." -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:"
Write-Host "  1. Obtain Telegram API credentials at https://my.telegram.org"
Write-Host "     (See README section 2.5 for a walk-through.)"
Write-Host "  2. Launch the dev build:"
Write-Host "        cd app"
Write-Host "        npm run tauri dev"
Write-Host "     (The very first run compiles ~300 Rust crates and may take 5-15 minutes.)"
Write-Host "  3. When ready to ship, produce a release binary:"
Write-Host "        cd app"
Write-Host "        npm run tauri build"
Write-Host ""
Write-Host "Build artifacts will land in:"
Write-Host "  app\src-tauri\target\release\"
Write-Host "  app\src-tauri\target\release\bundle\"
Write-Host ""
Write-Host "Happy hacking!"
