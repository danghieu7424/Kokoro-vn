// src/infrastructure/log_compressor.rs
#![allow(dead_code)]
use std::path::PathBuf;
use std::time::{SystemTime, Duration};
use tokio::fs;
use tokio::io::{BufReader, BufWriter};
use async_compression::tokio::write::GzipEncoder;
use tracing::{info, error, instrument};

#[instrument(skip_all)]
pub async fn start_log_compressor_task() {
    let mut interval = tokio::time::interval(Duration::from_secs(12 * 3600));
    
    tokio::spawn(async move {
        loop {
            interval.tick().await;
            
            info!(category = "System", "Bắt đầu quét, nén file log > 7 ngày và xóa file > 30 ngày...");
            if let Err(e) = manage_old_logs().await {
                error!(category = "Error", error_detail = ?e, "Lỗi trong quá trình dọn dẹp log");
            }
        }
    });
}

async fn manage_old_logs() -> std::io::Result<()> {
    let log_dir = "logs";
    
    if !tokio::fs::try_exists(log_dir).await.unwrap_or(false) {
        return Ok(());
    }

    let mut entries = fs::read_dir(log_dir).await?;
    let now = SystemTime::now();
    let seven_days_ago = now - Duration::from_secs(7 * 24 * 3600);
    let thirty_days_ago = now - Duration::from_secs(30 * 24 * 3600);

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        
        if path.is_dir() {
            continue;
        }

        let is_gz = path.extension().and_then(|e| e.to_str()) == Some("gz");
        let metadata = entry.metadata().await?;
        
        if let Ok(modified_time) = metadata.modified() {
            let path_clone = path.clone();
            let file_name = path_clone.file_name().unwrap_or_default().to_string_lossy().to_string();
            
            if is_gz {
                if modified_time < thirty_days_ago {
                    info!(category = "System", file = %file_name, "File nén đã quá 30 ngày. Đang xóa vĩnh viễn...");
                    if let Err(e) = fs::remove_file(&path).await {
                        error!(category = "Error", file = %file_name, "Không thể xóa file rác: {}", e);
                    } else {
                        info!(category = "Complete", file = %file_name, "Đã xóa vĩnh viễn log cũ!");
                    }
                }
            } else {
                if modified_time < seven_days_ago {
                    info!(category = "System", file = %file_name, "Phát hiện log cũ, chuẩn bị nén Async...");
                    
                    if compress_file_async(&path_clone).await.is_ok() {
                        let _ = fs::remove_file(&path).await; 
                        info!(category = "Complete", file = %file_name, "Đã nén thành .gz và xóa bản gốc");
                    }
                }
            }
        }
    }
    Ok(())
}

async fn compress_file_async(path: &PathBuf) -> std::io::Result<()> {
    let input_file = fs::File::open(path).await?;
    let mut reader = BufReader::with_capacity(64 * 1024, input_file);
    
    let gz_path = format!("{}.gz", path.display());
    let output_file = fs::File::create(gz_path).await?;
    let writer = BufWriter::with_capacity(64 * 1024, output_file);
    
    let mut encoder = GzipEncoder::new(writer);
    
    tokio::io::copy(&mut reader, &mut encoder).await?;
    
    use tokio::io::AsyncWriteExt;
    encoder.shutdown().await?;
    
    Ok(())
}

/****
 * Module: TTS Temp Cleaner
 * Chức năng: Dọn dẹp file .wav tạm thời đã quá hạn trong thư mục tts_temp.
 * Chạy ngầm mỗi 60 giây, xóa file cũ hơn max_age_secs.
 ****/
pub async fn start_tts_temp_cleaner(temp_dir: String, max_age_secs: u64) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(e) = clean_temp_files(&temp_dir, max_age_secs).await {
                error!(category = "Error", "Lỗi dọn file TTS tạm: {}", e);
            }
        }
    });
}

async fn clean_temp_files(temp_dir: &str, max_age_secs: u64) -> std::io::Result<()> {
    if !tokio::fs::try_exists(temp_dir).await.unwrap_or(false) {
        return Ok(());
    }
    
    let mut entries = fs::read_dir(temp_dir).await?;
    let now = SystemTime::now();
    let threshold = now - Duration::from_secs(max_age_secs);
    let mut cleaned = 0u32;
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("wav") {
            if let Ok(metadata) = entry.metadata().await {
                if let Ok(modified) = metadata.modified() {
                    if modified < threshold {
                        let _ = fs::remove_file(&path).await;
                        cleaned += 1;
                    }
                }
            }
        }
    }
    
    if cleaned > 0 {
        info!(category = "System", "Đã dọn {} file TTS tạm quá hạn", cleaned);
    }
    
    Ok(())
}
