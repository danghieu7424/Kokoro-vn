/****
 * [MODULE]: vn_g2p
 * Chức năng: Vietnamese Grapheme-to-Phoneme hoàn chỉnh (Zero-Python)
 * Sử dụng bộ từ điển 65,000 âm tiết đã được tinh chỉnh IPA để chống ngọng.
 ****/
use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::Regex;
use super::normalizer;

lazy_static! {
    static ref PUNCTUATION_RE: Regex = Regex::new(r#"([.!?…,:;\-—'"”“()\[\]{}%])"#).unwrap();
    static ref SPACES_RE: Regex = Regex::new(r"\s+").unwrap();
    
    // Nạp toàn bộ 65,000+ từ điển âm vị tĩnh vào RAM lúc khởi động (Zero-latency)
    static ref DICT: HashMap<String, String> = {
        let json_str = include_str!("../../vi_syllables_refined.json");
        serde_json::from_str(json_str).expect("Lỗi parse vi_syllables_refined.json")
    };
}

/// Tokenize văn bản, chuẩn hóa cơ bản và tra cứu từ điển âm vị Kokoro
pub fn phonemize(text: &str) -> String {
    let norm_text = normalizer::normalize(text);
    let lower = norm_text.to_lowercase();
    let spaced = PUNCTUATION_RE.replace_all(&lower, " $1 ");
    let cleaned = SPACES_RE.replace_all(&spaced, " ");
    
    let mut result = Vec::new();
    for word in cleaned.split_whitespace() {
        if let Some(phoneme) = DICT.get(word) {
            result.push(phoneme.to_string());
        } else if word.chars().all(|c| c.is_ascii_punctuation()) {
            result.push(word.to_string());
        } else {
            // Out of vocabulary (e.g. số, từ tiếng Anh)
            result.push(word.to_string());
        }
    }
    
    result.join(" ")
}

