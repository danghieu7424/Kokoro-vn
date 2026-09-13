mod atoms;

use atoms::g2p::Tokenizer;
use atoms::voicepack::Voicepack;
use atoms::inference::KokoroEngine;
use atoms::audio::save_wav;
use atoms::vn_g2p;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;
use std::time::Instant;
use std::fs;

lazy_static! {
    // 1. Khởi tạo Tokenizer
    static ref TOKENIZER: Tokenizer = Tokenizer::new("config.json").expect("Lỗi load config.json");
    
    // 2. Load tất cả Voicepacks vào RAM (12 giọng x ~500KB = ~6MB siêu nhẹ)
    static ref VOICEPACKS: HashMap<String, Voicepack> = {
        let mut m = HashMap::new();
        let voices = vec![
            "diem_trinh", "hung_thinh", "mai_linh", "manh_dung", 
            "my_yen", "ngoc_huyen", "phat_tai", "thanh_dat", "thuc_trinh", 
            "tuan_ngoc", "duc_an", "duc_duy"
        ];
        for v in voices {
            let path = format!("../voicepacks_npy/{}.npy", v);
            let pack = Voicepack::load(&path).unwrap_or_else(|_| panic!("Lỗi load {}", v));
            m.insert(v.to_string(), pack);
        }
        m
    };
    
    // 3. Load ONNX Model bọc trong Mutex để an toàn đa luồng
    static ref ENGINE: Arc<Mutex<KokoroEngine>> = Arc::new(Mutex::new(
        KokoroEngine::new("kokoro_vi.onnx").expect("Lỗi load ONNX model")
    ));
}

use clap::Parser;

/// Kokoro-RS TTS (100% Rust)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Văn bản cần đọc
    #[arg(short = 't', long)]
    text: String,

    /// Tên file đầu ra (VD: audio.wav)
    #[arg(short = 'o', long, default_value = "output.wav")]
    output: String,

    /// Tên giọng đọc (VD: diem_trinh, hung_thinh...)
    #[arg(short = 'v', long, default_value = "diem_trinh")]
    voice: String,

    /// Tốc độ đọc (Mặc định 1.0)
    #[arg(short = 's', long, default_value_t = 1.0)]
    speed: f32,
}

fn process_voice(voice: &str, text: &str, output: &str, speed: f32) -> anyhow::Result<()> {
    println!("\n--- Đang xử lý giọng: {} ---", voice);
    
    // NLP G2P
    let phonemes = vn_g2p::phonemize(text);
    println!("Phonemes: {}", phonemes);
    
    // Tokenize
    let input_ids = TOKENIZER.encode(&phonemes);
    
    // Extract Style
    let pack = VOICEPACKS.get(voice).ok_or_else(|| anyhow::anyhow!("Không tìm thấy giọng: {}", voice))?;
    let ref_s = pack.get_style_for_length(input_ids.len())?;
    
    // Inference
    let start_time = Instant::now();
    let audio_output = {
        let mut engine_lock = ENGINE.lock().unwrap();
        engine_lock.synthesize(input_ids, ref_s.to_owned(), speed)?
    };
    
    // Tạo thư mục nếu đường dẫn có chứa thư mục
    if let Some(parent) = std::path::Path::new(output).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    
    save_wav(&audio_output, output, 24000)?;
    println!("✅ Đã lưu thành công: {} (Thời gian xử lý: {:?})", output, start_time.elapsed());
    
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // Kích hoạt lazy_static nạp vào RAM
    let _ = &*TOKENIZER;
    let _ = &*VOICEPACKS;
    let _ = &*ENGINE;
    
    if let Err(e) = process_voice(&args.voice, &args.text, &args.output, args.speed) {
        eprintln!("❌ Lỗi: {:?}", e);
    }
    
    Ok(())
}
