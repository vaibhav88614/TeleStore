//! TeleStore CLI (Track D extension exercise)
//! ===========================================
//!
//! A tiny command-line client for the TeleStore REST API. It talks to a running
//! instance (the desktop app with the API enabled, or the `td-headless` server)
//! over HTTP, so it needs no Telegram credentials of its own — just the API key.
//!
//! Build (opt-in; never part of the desktop build):
//!     cargo build --release --features cli --bin td-cli
//!
//! Configure via env:
//!     TD_API_BASE   default: http://127.0.0.1:8550/api/v1
//!     TD_API_KEY    your API key (required for everything except `health`)
//!
//! Usage:
//!     td-cli health
//!     td-cli folders
//!     td-cli ls [folder_id]
//!     td-cli stats
//!     td-cli get <message_id> <out_path>

#[cfg(feature = "cli")]
mod cli_impl {
    use std::io::Write;

    fn base() -> String {
        std::env::var("TD_API_BASE").unwrap_or_else(|_| "http://127.0.0.1:8550/api/v1".to_string())
    }

    fn key() -> Option<String> {
        std::env::var("TD_API_KEY").ok()
    }

    fn client() -> reqwest::Client {
        reqwest::Client::builder()
            .build()
            .expect("failed to build HTTP client")
    }

    fn auth(req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match key() {
            Some(k) => req.header("X-API-Key", k),
            None => req,
        }
    }

    async fn get_json(path: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}{}", base(), path);
        let resp = auth(client().get(&url)).send().await.map_err(|e| e.to_string())?;
        let status = resp.status();
        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        if !status.is_success() {
            return Err(format!("HTTP {status}: {body}"));
        }
        Ok(body)
    }

    fn print_json(v: &serde_json::Value) {
        println!("{}", serde_json::to_string_pretty(v).unwrap_or_default());
    }

    pub async fn run() -> Result<(), String> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let cmd = args.first().map(String::as_str).unwrap_or("help");

        match cmd {
            "health" => print_json(&get_json("/health").await?),
            "folders" => print_json(&get_json("/folders").await?),
            "stats" => print_json(&get_json("/storage/stats").await?),
            "ls" => {
                let path = match args.get(1) {
                    Some(folder_id) => format!("/files?folder_id={folder_id}"),
                    None => "/files".to_string(),
                };
                print_json(&get_json(&path).await?);
            }
            "get" => {
                let id = args.get(1).ok_or("usage: td-cli get <message_id> <out_path>")?;
                let out = args.get(2).ok_or("usage: td-cli get <message_id> <out_path>")?;
                let url = format!("{}/files/{}/download", base(), id);
                let resp = auth(client().get(&url)).send().await.map_err(|e| e.to_string())?;
                if !resp.status().is_success() {
                    return Err(format!("HTTP {}", resp.status()));
                }
                let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
                let mut f = std::fs::File::create(out).map_err(|e| e.to_string())?;
                f.write_all(&bytes).map_err(|e| e.to_string())?;
                println!("Saved {} bytes to {out}", bytes.len());
            }
            _ => {
                eprintln!(
                    "TeleStore CLI\n\
                     Commands: health | folders | ls [folder_id] | stats | get <message_id> <out_path>\n\
                     Env: TD_API_BASE (default http://127.0.0.1:8550/api/v1), TD_API_KEY"
                );
            }
        }
        Ok(())
    }
}

#[cfg(feature = "cli")]
#[tokio::main]
async fn main() {
    if let Err(e) = cli_impl::run().await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!(
        "td-cli was built without the `cli` feature.\n\
         Rebuild with:  cargo build --release --features cli --bin td-cli"
    );
    std::process::exit(1);
}
