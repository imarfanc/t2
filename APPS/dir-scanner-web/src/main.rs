mod payload;
mod report;
mod scan;
mod terminal;

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use axum::extract::State;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use crate::payload::{system_info, KeyValue, ScanResponse};
use crate::scan::{parse_ignore_list, real_root, scan_tree, ScanOptions, DEFAULT_SCAN_ROOT};
use serde::{Deserialize, Serialize};
use tower_http::services::ServeDir;

#[derive(Debug, Deserialize)]
struct ScanQuery {
    root: Option<String>,
    ignore_dirs: Option<String>,
    ignore_files: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

#[derive(Debug, Serialize)]
struct SaveBody {
    path: String,
}

#[derive(Debug, Serialize)]
struct BrowseBody {
    path: Option<String>,
}

fn main() {
    let mut cfg = terminal::load_config();
    if let Ok(port) = env::var("DIR_SCANNER_WEB_PORT").map(|value| value.parse::<u16>()) {
        if let Ok(port) = port {
            cfg.port = port;
        }
    }
    let static_dir = terminal::static_dir();
    let url = terminal::url(&cfg);
    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port)
        .parse()
        .expect("socket address");
    let log_requests = cfg.log_requests;
    let serve_dir = static_dir.clone();
    let app_config = cfg.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(async move {
            let app = build_app(serve_dir, log_requests, app_config);
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .expect("bind port");
            axum::serve(listener, app).await.expect("server failed");
        });
    });

    terminal::banner(&static_dir, &url);

    if cfg.open_on_start {
        terminal::open_browser(&url);
    }

    terminal::run_hotkeys(&url);
}

#[derive(Debug, Serialize)]
struct UiConfig {
    ignore_dirs: String,
    ignore_files: String,
}

fn build_app(static_dir: PathBuf, log_requests: bool, cfg: terminal::Config) -> Router {
    let mut router = Router::new()
        .route("/api/config", get(api_config))
        .route("/api/system", get(api_system))
        .route("/api/browse", post(api_browse))
        .route("/api/scan", get(api_scan))
        .route("/api/save", post(api_save))
        .fallback_service(ServeDir::new(static_dir))
        .with_state(cfg);

    if log_requests {
        router = router.layer(middleware::from_fn(log_requests_middleware));
    }

    router
}

async fn log_requests_middleware(
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let start = Instant::now();
    let response = next.run(request).await;
    terminal::log_request(
        &method,
        &path,
        response.status().as_u16(),
        start.elapsed().as_micros(),
    );
    response
}

async fn api_config(State(cfg): State<terminal::Config>) -> Json<UiConfig> {
    Json(UiConfig {
        ignore_dirs: cfg.ignore_dirs,
        ignore_files: cfg.ignore_files,
    })
}

async fn api_system() -> Json<Vec<KeyValue>> {
    Json(system_info())
}

fn scan_options_from_query(query: &ScanQuery) -> ScanOptions {
    let ignore_dirs = query
        .ignore_dirs
        .as_deref()
        .map(parse_ignore_list)
        .unwrap_or_default();
    let ignore_files = query
        .ignore_files
        .as_deref()
        .map(parse_ignore_list)
        .unwrap_or_default();
    ScanOptions::with_extra_ignores(&ignore_dirs, &ignore_files)
}

fn pick_folder() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(r#"POSIX path of (choose folder with prompt "Choose folder to scan")"#)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let path = String::from_utf8(output.stdout).ok()?;
        let path = path.trim();
        if path.is_empty() {
            None
        } else {
            Some(PathBuf::from(path))
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = ();
        None
    }
}

async fn api_browse() -> Json<BrowseBody> {
    let path = tokio::task::spawn_blocking(pick_folder)
        .await
        .ok()
        .flatten();

    Json(BrowseBody {
        path: path.map(|selected| selected.display().to_string()),
    })
}

async fn api_scan(Query(query): Query<ScanQuery>) -> Result<Json<ScanResponse>, AppError> {
    let options = scan_options_from_query(&query);
    let root = query
        .root
        .unwrap_or_else(|| DEFAULT_SCAN_ROOT.into());
    let data = real_root(PathBuf::from(root).as_path())
        .map(|path| scan_tree(&path, &options))
        .map_err(AppError::from)?;
    Ok(Json(ScanResponse::from_scan(&data)))
}

async fn api_save(Query(query): Query<ScanQuery>) -> Result<Json<SaveBody>, AppError> {
    let options = scan_options_from_query(&query);
    let root = query
        .root
        .unwrap_or_else(|| DEFAULT_SCAN_ROOT.into());
    let data = real_root(PathBuf::from(root).as_path())
        .map(|path| scan_tree(&path, &options))
        .map_err(AppError::from)?;
    let path = report::write_report(&data).map_err(AppError::from)?;
    Ok(Json(SaveBody {
        path: path.display().to_string(),
    }))
}

struct AppError(String);

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self(value.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorBody { error: self.0 }),
        )
            .into_response()
    }
}
