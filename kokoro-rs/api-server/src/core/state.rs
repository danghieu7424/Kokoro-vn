/****
 * Module: Core State
 * Chức năng: Lưu trữ trạng thái toàn cục của API Server (Shared State).
 * Biến đầu vào: Đường dẫn TTS Engine, thư mục tạm, Semaphore giới hạn đồng thời.
 ****/
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct AppState {
    /// Đường dẫn tuyệt đối đến file kokoro-rs.exe
    pub tts_engine_path: String,
    /// Thư mục chứa file .wav tạm thời sinh ra bởi TTS engine
    pub tts_temp_dir: String,
    /// Giới hạn số lượng request TTS đồng thời (tránh nghẽn CPU/RAM khi ONNX inference)
    pub tts_semaphore: Arc<Semaphore>,
}
