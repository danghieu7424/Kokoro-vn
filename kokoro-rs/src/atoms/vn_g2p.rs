/****
 * [MODULE]: vn_g2p
 * Chức năng: Rule-based Vietnamese Grapheme-to-Phoneme cho Kokoro
 ****/
use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref PUNCTUATION_RE: Regex = Regex::new(r"([.!?…,:;\-—'”])").unwrap();
    static ref SPACES_RE: Regex = Regex::new(r"\s+").unwrap();
    
    // Bảng quy tắc cơ bản chuyển đổi grapheme -> phoneme (POC)
    // Để có bộ 100% hoàn hảo cần mapping toàn bộ âm đầu, âm đệm, âm chính, âm cuối.
    // Dưới đây là bảng thu gọn đủ dùng cho các câu cơ bản.
    static ref DICT: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("xin", "sˈin");
        m.insert("chào", "ʧˈaː↘w");
        m.insert("thế", "tʰˈe↗");
        m.insert("giới", "zˈə↗j");
        m.insert("tôi", "tˈoj");
        m.insert("là", "lˈa↘");
        m.insert("người", "ŋˈɨə↘j");
        m.insert("việt", "vˈiə↓t");
        m.insert("nam", "nˈam");
        m
    };
}

/// Tokenize văn bản, chuẩn hóa cơ bản và lookup ra âm vị
pub fn phonemize(text: &str) -> String {
    let lower = text.to_lowercase();
    let spaced = PUNCTUATION_RE.replace_all(&lower, " $1 ");
    let cleaned = SPACES_RE.replace_all(&spaced, " ");
    
    let mut result = Vec::new();
    for word in cleaned.split_whitespace() {
        if let Some(&phoneme) = DICT.get(word) {
            result.push(phoneme.to_string());
        } else if word.chars().all(|c| c.is_ascii_punctuation()) {
            result.push(word.to_string());
        } else {
            // Fallback nếu không có trong từ điển (Trong bản final, đây sẽ là thuật toán Regex)
            // Tạm thời trả về word để đánh dấu lỗi chưa được xử lý
            result.push(word.to_string());
        }
    }
    
    result.join(" ")
}
