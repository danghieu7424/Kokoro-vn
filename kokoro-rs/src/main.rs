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
use atoms::dsp;

/// Kokoro-RS TTS (100% Rust)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Văn bản cần đọc
    #[arg(short = 't', long)]
    text: String,

    /// Tên file đầu ra (VD: output.wav)
    #[arg(short = 'o', long, default_value = "output.wav")]
    output: String,

    /// Tên giọng đọc chính (VD: diem_trinh)
    #[arg(short = 'v', long, default_value = "diem_trinh")]
    voice: String,

    /// Tốc độ đọc (Mặc định 1.0)
    #[arg(short = 's', long, default_value_t = 1.0)]
    speed: f32,

    /// Trộn Tone (Pha trộn Voicepack). VD: "diem_trinh=70,duc_duy=30"
    #[arg(short = 'm', long)]
    mix_blend: Option<String>,

    /// Pitch shift (Cent, giống FL Studio). +100 = lên 1 nửa cung (semitone).
    #[arg(short = 'p', long, default_value_t = 0.0)]
    pitch: f32,
}

fn process_voice(args: &Args) -> anyhow::Result<()> {
    println!("\n--- Đang xử lý G2P NLP ---");
    let phonemes = vn_g2p::phonemize(&args.text);
    println!("Phonemes: {}", phonemes);
    
    let input_ids = TOKENIZER.encode(&phonemes);
    
    // Logic Lấy Style & Trộn Tone
    let ref_s = if let Some(blend_str) = &args.mix_blend {
        println!("--- Đang pha trộn Tone: {} ---", blend_str);
        let mut mixed_style = ndarray::Array2::<f32>::zeros((1, 256));
        let mut total_weight = 0.0;
        
        let parts: Vec<&str> = blend_str.split(',').collect();
        for part in parts {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() == 2 {
                let v_name = kv[0].trim();
                let weight: f32 = kv[1].trim().parse().unwrap_or(0.0);
                total_weight += weight;
                
                let pack = VOICEPACKS.get(v_name).ok_or_else(|| anyhow::anyhow!("Không tìm thấy giọng: {}", v_name))?;
                let style = pack.get_style_for_length(input_ids.len())?;
                mixed_style = mixed_style + (&style * weight);
            }
        }
        
        if total_weight > 0.0 {
            mixed_style = mixed_style / total_weight;
        } else {
            return Err(anyhow::anyhow!("Trọng số pha trộn Tone không hợp lệ"));
        }
        mixed_style
    } else {
        println!("--- Đang dùng giọng đơn: {} ---", args.voice);
        let pack = VOICEPACKS.get(&args.voice).ok_or_else(|| anyhow::anyhow!("Không tìm thấy giọng: {}", args.voice))?;
        pack.get_style_for_length(input_ids.len())?
    };
    
    // Tính toán tỷ lệ Pitch theo Cents (Mỗi 100 cents = 1 nửa cung)
    // Tốc độ Kokoro sẽ được sinh ra chậm lại (hoặc nhanh lên) tương ứng, sau đó Resample để ép Pitch
    let pitch_ratio = 2.0_f32.powf(args.pitch / 1200.0);
    let kokoro_speed = args.speed / pitch_ratio;

    if args.pitch != 0.0 {
        println!("--- Pitch Shift: {} cents (Ratio: {:.3}) ---", args.pitch, pitch_ratio);
    }
    
    // Inference
    let start_time = Instant::now();
    let mut audio_output = {
        let mut engine_lock = ENGINE.lock().unwrap();
        engine_lock.synthesize(input_ids, ref_s, kokoro_speed)?
    };

    // Resampling để khôi phục tốc độ và bóp méo Pitch
    if args.pitch != 0.0 {
        audio_output = dsp::resample_audio(&audio_output, pitch_ratio);
    }
    
    // Lưu file
    if let Some(parent) = std::path::Path::new(&args.output).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    
    save_wav(&audio_output, &args.output, 24000)?;
    println!("✅ Đã lưu thành công: {} (Thời gian xử lý: {:?})", args.output, start_time.elapsed());
    
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    let _ = &*TOKENIZER;
    let _ = &*VOICEPACKS;
    let _ = &*ENGINE;
    
    if let Err(e) = process_voice(&args) {
        eprintln!("❌ Lỗi: {:?}", e);
    }
    
    Ok(())
}
