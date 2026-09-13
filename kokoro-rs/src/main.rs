mod atoms;

use atoms::g2p::Tokenizer;
use atoms::voicepack::Voicepack;
use atoms::inference::KokoroEngine;
use atoms::audio::save_wav;
use atoms::vn_g2p;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

lazy_static! {
    // 1. Khởi tạo Tokenizer
    static ref TOKENIZER: Tokenizer = Tokenizer::new("config.json").expect("Lỗi load config.json");
    
    // 2. Load Voicepack (.npy)
    static ref VOICEPACK: Voicepack = Voicepack::load("../voicepacks_npy/diem_trinh.npy").expect("Lỗi load voicepack");
    
    // 3. Load ONNX Model bọc trong Mutex để an toàn đa luồng
    static ref ENGINE: Arc<Mutex<KokoroEngine>> = Arc::new(Mutex::new(
        KokoroEngine::new("kokoro_vi.onnx").expect("Lỗi load ONNX model")
    ));
}

fn process_text(id: usize, text: &str) -> anyhow::Result<()> {
    println!("[Luồng {}] Đang xử lý: '{}'", id, text);
    
    // Bước 1: G2P NLP (Text to Phonemes)
    let phonemes = vn_g2p::phonemize(text);
    println!("[Luồng {}] Phonemes: {}", id, phonemes);
    
    // Bước 2: Tokenize (Phonemes to IDs)
    let input_ids = TOKENIZER.encode(&phonemes);
    
    // Bước 3: Trích xuất Style
    let ref_s = VOICEPACK.get_style_for_length(input_ids.len())?;
    
    // Bước 4: Chạy Inference (Khóa Mutex để tránh đụng độ)
    let start_time = Instant::now();
    let audio_output = {
        let mut engine_lock = ENGINE.lock().unwrap();
        engine_lock.synthesize(input_ids, ref_s.to_owned(), 1.0)?
    };
    let duration = start_time.elapsed();
    
    println!("[Luồng {}] Tạo âm thanh xong! Thời gian: {:?}", id, duration);
    
    // Bước 5: Lưu Audio
    let output_file = format!("output_rs_{}.wav", id);
    save_wav(&audio_output, &output_file, 24000)?;
    println!("[Luồng {}] Đã lưu {}", id, output_file);
    
    Ok(())
}

fn main() -> anyhow::Result<()> {
    println!("Khởi tạo hệ thống Kokoro-RS (100% Rust) với Global Memory & Multithreading...");

    // Gọi lần đầu để kích hoạt lazy_static nạp vào RAM
    let _ = &*TOKENIZER;
    let _ = &*VOICEPACK;
    let _ = &*ENGINE;
    
    println!("Đã nạp toàn bộ Model và Voicepack vào RAM!");

    let texts = vec![
        "Xin chào thế giới",
        "Tôi là người việt nam",
        "Xin chào việt nam"
    ];

    let mut handles = vec![];
    
    let total_start = Instant::now();

    for (i, text) in texts.into_iter().enumerate() {
        let txt = text.to_string();
        let handle = thread::spawn(move || {
            if let Err(e) = process_text(i + 1, &txt) {
                eprintln!("[Luồng {}] Lỗi: {:?}", i + 1, e);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Hoàn tất toàn bộ yêu cầu trong: {:?}", total_start.elapsed());
    
    Ok(())
}
