mod atoms;

use atoms::g2p::Tokenizer;
use atoms::voicepack::Voicepack;
use atoms::inference::KokoroEngine;
use atoms::audio::save_wav;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    println!("Khởi tạo hệ thống Kokoro-RS (100% Rust)...");

    // 1. Khởi tạo Tokenizer
    let tokenizer = Tokenizer::new("config.json")?;
    
    // 2. Load Voicepack (.npy)
    println!("Đang tải Voicepack Diem Trinh...");
    let voicepack = Voicepack::load("../voicepacks_npy/diem_trinh.npy")?;
    
    // 3. Load ONNX Model
    println!("Đang tải mô hình ONNX...");
    let engine = KokoroEngine::new("kokoro_vi.onnx")?;

    // --- PoC Phase ---
    // Do chưa có NLP G2P hoàn chỉnh trong Rust, ta sẽ dùng chuỗi phonemes cứng
    // Chuỗi phonemes này ứng với câu: "Xin chào" -> "sˈin ʧˈaː↘w"
    let dummy_phonemes = "sˈin ʧˈaː↘w";
    println!("Phonemes đầu vào: {}", dummy_phonemes);
    
    // 4. Tokenize
    let input_ids = tokenizer.encode(dummy_phonemes);
    println!("Token IDs: {:?}", input_ids);
    
    // 5. Trích xuất Style cho độ dài phonemes
    let ref_s = voicepack.get_style_for_length(input_ids.len())?;
    
    // 6. Chạy Inference
    let start_time = Instant::now();
    let audio_output = engine.synthesize(input_ids, ref_s.to_owned(), 1.0)?;
    let duration = start_time.elapsed();
    
    println!("Tạo âm thanh thành công! Thời gian xử lý: {:?}", duration);
    
    // 7. Lưu ra file WAV
    let output_file = "output_rs.wav";
    save_wav(&audio_output, output_file, 24000)?;
    println!("Đã lưu file âm thanh tại: {}", output_file);
    
    Ok(())
}
