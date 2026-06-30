//! System / diagnostics commands (Track D refactor).
//!
//! Extracted from `lib.rs` to demonstrate the module-per-concern pattern the
//! rest of `commands/*` already follows. Follow-up: the Android/JNI share-cache
//! handlers (`cmd_open_file_externally`, `cmd_get_pending_share_count`,
//! `cmd_list_cached_files`, `cmd_remove_cached_path`) still live in `lib.rs`
//! because of their platform-gated JNI bodies; move them here next, preserving
//! every `#[cfg(...)]` attribute.

use tauri::Manager;

/// Gather system diagnostics and environment info for debugging.
/// Returns a formatted string suitable for copying to clipboard.
#[tauri::command]
pub fn cmd_get_system_diagnostics(app: tauri::AppHandle) -> Result<String, String> {
    let mut lines: Vec<String> = Vec::new();

    lines.push("=== TeleStore Diagnostics ===".into());
    lines.push(format!("Package: {}", env!("CARGO_PKG_NAME")));
    lines.push(format!("Version: {}", env!("CARGO_PKG_VERSION")));

    // OS info
    lines.push(format!("OS: {} {}", std::env::consts::OS, std::env::consts::ARCH));

    #[cfg(target_os = "linux")]
    {
        lines.push(format!("XDG_SESSION_TYPE: {}",
            std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".into())));
        lines.push(format!("XDG_CURRENT_DESKTOP: {}",
            std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "unknown".into())));
        lines.push(format!("WEBKIT_DISABLE_DMABUF_RENDERER: {}",
            std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").unwrap_or_else(|_| "unset".into())));
    }

    #[cfg(target_os = "macos")]
    {
        lines.push("Package Type: macOS bundle".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        lines.push("Package Type: Windows installer".to_string());
    }

    // App data dir
    if let Ok(dir) = app.path().app_data_dir() {
        lines.push(format!("App Data: {}", dir.display()));
    }

    // Check for FFmpeg
    #[cfg(unix)]
    {
        let which = std::process::Command::new("which")
            .arg("ffmpeg")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
        lines.push(format!("FFmpeg: {}", which.unwrap_or_else(|| "not found".into())));
    }

    lines.push("==================================".into());

    Ok(lines.join("\n"))
}
