use axum::{
    extract::Request,
    response::{Response, IntoResponse},
    routing::Router,
    http::{Method, HeaderValue, header::{self, HeaderName}},
};
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use dotenvy::dotenv;
use std::fs;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn};
use colored::Colorize;
use tracing_subscriber::{
    prelude::*,
    EnvFilter,
};

mod core;
mod infrastructure;
mod helpers;
mod routes;
mod utils;

// Thêm include file HTML vào binary (tương tự FAVICON)
static FAVICON: &[u8] = include_bytes!("../static/favicon.ico");
static INDEX_HTML: &str = include_str!("../index.html");

async fn favicon_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/x-icon")],
        FAVICON,
    )
}

// Handler trả về giao diện HTML
async fn index_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        INDEX_HTML,
    )
}

#[tokio::main]
async fn main() {
    // 1. Tự động chuyển Working Directory về thư mục chứa file .exe
    // Giúp server luôn tìm thấy .env, kokoro-rs.exe và model dù khởi chạy từ Task Scheduler (SYSTEM)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let _ = std::env::set_current_dir(exe_dir);
        }
    }

    dotenv().ok();

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let cmd = args[1].as_str();
        if matches!(cmd, "cli" | "setup" | "--setup" | "-c" | "start" | "--start" | "-s" | "stop" | "--stop" | "-k" | "status" | "--status" | "-t" | "logs" | "--logs" | "-l") {
            utils::cli::handle_cli(args).await;
            return;
        }
    }

    colored::control::set_override(true);
    
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    let is_prod = std::env::var("APP_ENV").unwrap_or_default() == "production";

    let file_appender = tracing_appender::rolling::daily("logs", "app.log");
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,hyper=warn,reqwest=warn,tower=warn,h2=warn"));

    if is_prod {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_writer(non_blocking_writer)
            )
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .compact()
                    .with_ansi(false)
                    .with_writer(non_blocking_writer)
            )
            .with(crate::core::logger::ColorTerminalLayer)
            .init();
    }

    let port_str = std::env::var("PORT").unwrap_or_else(|_| "7424".to_string());

    // Đường dẫn đến kokoro-rs.exe (tương đối hoặc tuyệt đối)
    let tts_engine_path = std::env::var("TTS_ENGINE_PATH")
        .unwrap_or_else(|_| "./kokoro-rs.exe".to_string());

    // Thư mục chứa file .wav tạm thời
    let tts_temp_dir = "tts_temp".to_string();
    fs::create_dir_all(&tts_temp_dir).expect("Không tạo được thư mục tts_temp");

    // Thời gian sống tối đa của file tạm (mặc định 5 phút)
    let temp_max_age: u64 = std::env::var("TTS_TEMP_MAX_AGE_SECS")
        .unwrap_or_else(|_| "300".to_string())
        .parse()
        .unwrap_or(300);

    let app_state = Arc::new(crate::core::state::AppState {
        tts_engine_path: tts_engine_path.clone(),
        tts_temp_dir: tts_temp_dir.clone(),
        // Giới hạn tối đa 3 request TTS đồng thời (ONNX inference rất nặng CPU)
        tts_semaphore: Arc::new(tokio::sync::Semaphore::new(3)),
    });

    let allowed_origins_str = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
        
    let allowed_origins: Vec<HeaderValue> = allowed_origins_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .filter_map(|s| {
            s.parse::<HeaderValue>().map_err(|e| {
                tracing::warn!(category = "Warning", origin = s, error = %e, "Bỏ qua CORS origin không hợp lệ");
            }).ok()
        })
        .collect();

    let cors_layer = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            HeaderName::from_static("x-requested-with"),
        ])
        .allow_credentials(true);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            let req_id = crate::helpers::suid::generate_random_hex()[..8].to_string();
            let method = request.method().to_string();
            let uri = request.uri().to_string();
            tracing::info_span!("HTTP", id = %req_id, method = %method, uri = %uri)
        })
        .on_response(|response: &Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
            let status = response.status();
            let status_code = status.as_u16();
            let reason_str = status.canonical_reason().unwrap_or("Unknown");
            let category = if status.is_server_error() || status.is_client_error() { "Error" } else { "Complete" };
            info!(
                category = category,
                status = status_code, 
                reason = reason_str,
                latency_ms = latency.as_millis(), 
                "Phản hồi HTTP"
            );
        });

    let app = Router::new()
        .route("/", axum::routing::get(index_handler))
        .route("/favicon.ico", axum::routing::get(favicon_handler))
        .nest("/api", crate::routes::tts::router())
        .layer(cors_layer)
        .layer(trace_layer)
        .with_state(app_state.clone());

    let _addr = format!("0.0.0.0:{}", port_str);
    let listener = match tokio::net::TcpListener::bind(&_addr).await {
        Ok(l) => l,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                let msg = format!("Cổng {} đã bị chiếm bởi tiến trình khác!", port_str);
                eprintln!("{}", msg.as_str().red().bold());
                eprintln!("Gợi ý: Dùng lệnh '{} -k' để dừng server cũ, hoặc đổi PORT trong file .env", 
                    std::env::current_exe().unwrap_or_default().display());
            } else {
                eprintln!("Không thể mở cổng {}: {}", port_str, e);
            }
            std::process::exit(1);
        }
    };
    
    // Khởi động các task ngầm
    crate::infrastructure::log_compressor::start_log_compressor_task().await;
    crate::infrastructure::log_compressor::start_tts_temp_cleaner(tts_temp_dir, temp_max_age).await;

    info!(category = "System", "Kokoro TTS API Server đang chạy tại http://{}", _addr);
    info!(category = "System", "TTS Engine: {}", tts_engine_path);
    info!(category = "System", "Endpoints: POST /api/tts | GET /api/voices | GET /api/health");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Không thể cài đặt Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Không thể cài đặt SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    warn!(category = "Warning", "Đã nhận tín hiệu tắt máy! Tiến hành Graceful Shutdown...");
    info!(category = "System", "Hoàn tất dọn dẹp. Tạm biệt!");
}
