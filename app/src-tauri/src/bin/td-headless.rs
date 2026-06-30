//! TeleStore headless server
//! ==========================
//!
//! A Tauri-free entry point that runs ONLY the REST API (Actix) server, reusing
//! the exact route handlers from the desktop app (`app_lib::api_routes`). It is
//! meant for self-hosting the API on a server / container behind a reverse proxy
//! or a private network (Tailscale), so tools / LLMs can talk to your Telegram
//! drive without keeping the desktop GUI open.
//!
//! This binary is compiled only when the `headless` Cargo feature is enabled:
//!     cargo build --release --features headless --bin td-headless
//! so it never affects the normal `tauri build`/`tauri dev` flow.
//!
//! ── How sessions work ────────────────────────────────────────────────────────
//! This binary does NOT implement the interactive login flow (phone code / QR).
//! You must FIRST authenticate once with the desktop app, then point this binary
//! at the same data directory. It reuses:
//!   - `telegram.session`   (grammers auth session)
//!   - `api_settings.json`  (the SHA-256 API key hash configured in the GUI)
//!   - `shares.db`          (share links + folder metadata)
//!
//! ── Configuration (environment variables) ────────────────────────────────────
//!   TD_DATA_DIR   (required)  Directory holding telegram.session / shares.db /
//!                             api_settings.json (copy it from the desktop app's
//!                             app-data dir, or mount it as a Docker volume).
//!   TD_API_ID     (required)  Your Telegram api_id (integer, from my.telegram.org).
//!   TD_CACHE_DIR  (optional)  Thumbnail/preview cache dir (default: TD_DATA_DIR/cache).
//!   TD_BIND       (optional)  Bind address (default: 127.0.0.1). Use 0.0.0.0 only
//!                             behind a trusted reverse proxy / private network.
//!   TD_PORT       (optional)  Port (default: api_settings.json port, else 8550).

#[cfg(feature = "headless")]
mod headless_impl {
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;
    use std::sync::Arc;

    use actix_cors::Cors;
    use actix_web::{web, App, HttpServer};
    use tokio::sync::Mutex;

    use grammers_client::Client;
    use grammers_mtsender::{ConnectionParams, SenderPool};
    use grammers_session::storages::SqliteSession;

    use app_lib::api_routes::{self, ApiState, CacheDirs};
    use app_lib::bandwidth::BandwidthManager;
    use app_lib::commands::api_settings::ApiSettingsFile;
    use app_lib::commands::TelegramState;
    use app_lib::vpn_optimizer::NetworkConfig;

    fn env_required(key: &str) -> String {
        std::env::var(key).unwrap_or_else(|_| {
            eprintln!("FATAL: required environment variable {key} is not set");
            std::process::exit(2);
        })
    }

    pub async fn main() -> std::io::Result<()> {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

        let data_dir = PathBuf::from(env_required("TD_DATA_DIR"));
        let api_id: i32 = env_required("TD_API_ID")
            .parse()
            .expect("TD_API_ID must be a valid integer");
        let cache_dir = std::env::var("TD_CACHE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| data_dir.join("cache"));
        let bind_addr = std::env::var("TD_BIND").unwrap_or_else(|_| "127.0.0.1".to_string());

        // ── Load the API settings the desktop app persisted (key hash + port) ──
        let api_settings: ApiSettingsFile = std::fs::read_to_string(data_dir.join("api_settings.json"))
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default();

        if api_settings.key_hash.is_none() {
            log::warn!(
                "No API key configured in api_settings.json — every authenticated \
                 endpoint will return 401. Set an API key in the desktop app first."
            );
        }

        let port: u16 = std::env::var("TD_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(api_settings.port);

        // ── Open the grammers session and bring up the Telegram client ──
        let session_path = data_dir.join("telegram.session");
        let session = SqliteSession::open(session_path.to_string_lossy().as_ref())
            .unwrap_or_else(|e| {
                eprintln!("FATAL: could not open {session_path:?}: {e}. \
                           Authenticate once with the desktop app first.");
                std::process::exit(3);
            });
        let session = Arc::new(session);

        let pool = SenderPool::with_configuration(session, api_id, ConnectionParams::default());
        let client = Client::new(&pool);
        let SenderPool { runner, .. } = pool;
        tokio::spawn(async move {
            // Mirror the desktop app: run the network loop and ignore the
            // output (it ends only on shutdown / fatal connection loss).
            let _ = runner.run().await;
            log::warn!("grammers network runner exited");
        });

        // Verify the session is actually authenticated before serving.
        match client.get_me().await {
            Ok(me) => log::info!("Authenticated as: {}", me.id()),
            Err(e) => {
                eprintln!("FATAL: session is not authenticated ({e}). \
                           Log in with the desktop app and copy telegram.session.");
                std::process::exit(4);
            }
        }

        // ── Assemble the same shared state the desktop API server uses ──
        let tg_state = Arc::new(TelegramState {
            client: Arc::new(Mutex::new(Some(client))),
            login_token: Arc::new(Mutex::new(None)),
            password_token: Arc::new(Mutex::new(None)),
            api_id: Arc::new(Mutex::new(Some(api_id))),
            runner_shutdown: Arc::new(std::sync::Mutex::new(None)),
            runner_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            peer_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            cancelled_transfers: Arc::new(tokio::sync::RwLock::new(HashSet::new())),
        });

        let db_pool = app_lib::db::init_db_at(&data_dir)
            .expect("failed to open shares.db");
        let bw_manager = Arc::new(BandwidthManager::new_with_dir(data_dir.clone()));
        let net_config = Arc::new(NetworkConfig::new());

        std::fs::create_dir_all(&cache_dir).ok();
        let cache_dirs = CacheDirs {
            thumbnail_dir: cache_dir.join("thumbnails"),
            preview_dir: cache_dir.join("previews"),
        };

        // web::Data wrappers (cloned into each worker)
        let tg_data = web::Data::new(tg_state);
        let api_data = web::Data::new(ApiState { key_hash: api_settings.key_hash.clone() });
        let bw_data = web::Data::new(bw_manager);
        let net_data = web::Data::new(net_config);
        let db_data = web::Data::new(db_pool);
        let cache_data = web::Data::new(cache_dirs);

        log::info!("TeleStore headless API listening on http://{bind_addr}:{port}");

        HttpServer::new(move || {
            // The headless server is expected to sit behind a reverse proxy or a
            // private network, so CORS is permissive; auth is enforced per-request
            // by the X-API-Key check inside the route handlers.
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header();

            App::new()
                .wrap(cors)
                .app_data(tg_data.clone())
                .app_data(api_data.clone())
                .app_data(bw_data.clone())
                .app_data(net_data.clone())
                .app_data(db_data.clone())
                .app_data(cache_data.clone())
                .configure(api_routes::configure_api)
        })
        .bind((bind_addr.as_str(), port))?
        .run()
        .await
    }
}

#[cfg(feature = "headless")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    headless_impl::main().await
}

#[cfg(not(feature = "headless"))]
fn main() {
    eprintln!(
        "td-headless was built without the `headless` feature.\n\
         Rebuild with:  cargo build --release --features headless --bin td-headless"
    );
    std::process::exit(1);
}
