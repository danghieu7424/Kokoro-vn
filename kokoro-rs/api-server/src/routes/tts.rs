/****
 * Module: TTS Routes
 * Chức năng: Xử lý các API endpoint liên quan đến Text-to-Speech.
 * - POST /api/tts: Nhận text, gọi kokoro-rs.exe, trả về file audio .wav
 * - GET /api/voices: Trả về danh sách giọng đọc hỗ trợ
 * - GET /api/health: Health check endpoint
 ****/
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::core::{error::AppError, state::AppState};
use crate::helpers::suid::generate_random_hex;

#[derive(Deserialize)]
pub struct TtsRequest {
    pub text: String,
    #[serde(default = "default_voice")]
    pub voice: String,
    #[serde(default = "default_speed")]
    pub speed: f32,
    #[serde(default)]
    pub pitch: f32,
}

fn default_voice() -> String { "diem_trinh".to_string() }
fn default_speed() -> f32 { 1.0 }

#[derive(Serialize)]
struct VoiceInfo {
    id: &'static str,
    label: &'static str,
    gender: &'static str,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    engine: String,
    version: &'static str,
}

const VOICES: &[(&str, &str, &str)] = &[
    ("diem_trinh", "Diễm Trinh", "Nữ"),
    ("hung_thinh", "Hưng Thịnh", "Nam"),
    ("mai_linh", "Mai Linh", "Nữ"),
    ("manh_dung", "Mạnh Dũng", "Nam"),
    ("my_yen", "Mỹ Yến", "Nữ"),
    ("ngoc_huyen", "Ngọc Huyền", "Nữ"),
    ("phat_tai", "Phát Tài", "Nam"),
    ("thanh_dat", "Thành Đạt", "Nam"),
    ("thuc_trinh", "Thục Trinh", "Nữ"),
    ("tuan_ngoc", "Tuấn Ngọc", "Nam"),
    ("duc_an", "Đức An", "Nam"),
    ("duc_duy", "Đức Duy", "Nam"),
];

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/tts", post(handle_tts))
        .route("/voices", get(handle_voices))
        .route("/health", get(handle_health))
}

/****
 * handle_tts: API chính tạo giọng nói từ văn bản
 * Quy trình: Validate -> Acquire Semaphore -> Spawn kokoro-rs.exe -> Stream WAV -> Cleanup
 * Semaphore giới hạn tối đa 3 request TTS đồng thời để tránh nghẽn ONNX inference
 ****/
async fn handle_tts(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TtsRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate đầu vào
    if payload.text.trim().is_empty() {
        return Err(AppError::Custom(StatusCode::BAD_REQUEST, "Trường 'text' không được để trống".to_string()));
    }
    if payload.text.len() > 10000 {
        return Err(AppError::Custom(StatusCode::BAD_REQUEST, "Văn bản quá dài (tối đa 10.000 ký tự)".to_string()));
    }

    // Kiểm tra voice có hợp lệ không
    let valid_voice = VOICES.iter().any(|(id, _, _)| *id == payload.voice);
    if !valid_voice {
        return Err(AppError::Custom(
            StatusCode::BAD_REQUEST,
            format!("Giọng đọc '{}' không tồn tại. Dùng GET /api/voices để xem danh sách.", payload.voice),
        ));
    }

    info!(
        category = "TTS",
        voice = %payload.voice,
        speed = %payload.speed,
        text_len = payload.text.len(),
        "Nhận yêu cầu TTS"
    );

    // Acquire semaphore (Giới hạn đồng thời)
    let _permit = state.tts_semaphore.acquire().await
        .map_err(|_| AppError::TtsEngineFailed("Semaphore bị đóng".to_string()))?;

    // Tạo file tạm với tên ngẫu nhiên
    let file_id = generate_random_hex();
    let output_path = format!("{}/{}.wav", state.tts_temp_dir, file_id);

    // Xây dựng lệnh gọi kokoro-rs.exe
    let start = std::time::Instant::now();
    let mut cmd = tokio::process::Command::new(&state.tts_engine_path);
    cmd.arg("-t").arg(&payload.text)
       .arg("-v").arg(&payload.voice)
       .arg("-s").arg(payload.speed.to_string())
       .arg("-o").arg(&output_path);

    // Chỉ truyền pitch nếu khác 0 để tránh resampling thừa
    if payload.pitch != 0.0 {
        cmd.arg("-p").arg(payload.pitch.to_string());
    }

    let output = cmd.output().await
        .map_err(|e| AppError::TtsEngineFailed(format!("Không thể khởi chạy TTS engine: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::TtsEngineFailed(format!("TTS engine thất bại: {}", stderr)));
    }

    let elapsed = start.elapsed();
    info!(
        category = "TTS",
        latency_ms = elapsed.as_millis(),
        file = %file_id,
        "Đã tạo audio thành công"
    );

    // Đọc file wav và trả về response
    let wav_bytes = tokio::fs::read(&output_path).await
        .map_err(|e| AppError::TtsEngineFailed(format!("Không đọc được file audio: {}", e)))?;

    Ok((
        [
            (header::CONTENT_TYPE, "audio/wav"),
            (header::CONTENT_DISPOSITION, "inline; filename=\"tts_output.wav\""),
        ],
        wav_bytes,
    ))
}

async fn handle_voices() -> impl IntoResponse {
    let voices: Vec<VoiceInfo> = VOICES.iter()
        .map(|(id, label, gender)| VoiceInfo { id, label, gender })
        .collect();
    Json(voices)
}

async fn handle_health(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        engine: state.tts_engine_path.clone(),
        version: "0.1.0",
    })
}
