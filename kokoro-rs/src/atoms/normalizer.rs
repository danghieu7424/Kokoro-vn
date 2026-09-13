/****
 * [MODULE]: normalizer
 * Chức năng: Text Normalization cho Tiếng Việt.
 * Xử lý số đếm, ký tự đặc biệt (%, $, &) và từ mượn tiếng Anh thông dụng
 * trước khi đưa vào module G2P.
 ****/
use regex::{Regex, Captures};
use lazy_static::lazy_static;

lazy_static! {
    static ref NUM_RE: Regex = Regex::new(r"\b(\d+)\b").unwrap();
}

fn number_to_words(n: u64) -> String {
    if n == 0 { return "không".to_string(); }
    
    let units = ["", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín"];
    let scales = ["", "nghìn", "triệu", "tỉ", "nghìn tỉ", "triệu tỉ", "tỉ tỉ"];
    
    let mut chunks = Vec::new();
    let mut temp = n;
    while temp > 0 {
        chunks.push((temp % 1000) as u32);
        temp /= 1000;
    }
    
    let mut words = Vec::new();
    let num_chunks = chunks.len();
    let mut has_higher = false;
    
    for (i, &chunk) in chunks.iter().enumerate().rev() {
        if chunk == 0 {
            if has_higher && i > 0 && chunks[0..i].iter().any(|&x| x > 0) {
                words.push("không".to_string());
                words.push(scales[i].to_string());
            }
            continue;
        }
        
        has_higher = true;
        let c = chunk / 100;
        let b = (chunk % 100) / 10;
        let a = chunk % 10;
        
        let mut chunk_words = Vec::new();
        
        // Trăm
        if num_chunks > 1 && i < num_chunks - 1 {
            chunk_words.push(units[c as usize].to_string());
            chunk_words.push("trăm".to_string());
        } else if c > 0 {
            chunk_words.push(units[c as usize].to_string());
            chunk_words.push("trăm".to_string());
        }
        
        // Chục
        if b == 0 {
            if a > 0 && (c > 0 || (num_chunks > 1 && i < num_chunks - 1)) {
                chunk_words.push("lẻ".to_string());
            }
        } else if b == 1 {
            chunk_words.push("mười".to_string());
        } else {
            chunk_words.push(units[b as usize].to_string());
            chunk_words.push("mươi".to_string());
        }
        
        // Đơn vị
        if a > 0 {
            if a == 1 && b > 1 {
                chunk_words.push("mốt".to_string());
            } else if a == 5 && b > 0 {
                chunk_words.push("lăm".to_string());
            } else if a == 4 && b > 1 {
                chunk_words.push("tư".to_string());
            } else {
                chunk_words.push(units[a as usize].to_string());
            }
        }
        
        if !chunk_words.is_empty() {
            words.push(chunk_words.join(" "));
            if !scales[i].is_empty() {
                words.push(scales[i].to_string());
            }
        }
    }
    
    words.join(" ")
}

fn read_number(num_str: &str) -> String {
    // Nếu là số bắt đầu bằng 0 (VD: 098... số điện thoại), đọc từng chữ số
    if num_str.starts_with('0') && num_str.len() > 1 {
        let digit_map = ["không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín"];
        let mut res = Vec::new();
        for c in num_str.chars() {
            if let Some(d) = c.to_digit(10) {
                res.push(digit_map[d as usize].to_string());
            }
        }
        return res.join(" ");
    }
    
    if let Ok(n) = num_str.parse::<u64>() {
        return number_to_words(n);
    }
    
    num_str.to_string()
}

pub fn normalize(text: &str) -> String {
    let mut t = text.to_lowercase();
    
    // 0. Chuẩn hóa vị trí dấu thanh (Kiểu cũ -> Kiểu mới để map đúng với từ điển sinh ra)
    // Vần "oa"
    t = t.replace("óa", "oá").replace("òa", "oà").replace("ỏa", "oả").replace("õa", "oã").replace("ọa", "oạ");
    // Vần "oe"
    t = t.replace("óe", "oé").replace("òe", "oè").replace("ỏe", "oẻ").replace("õe", "oẽ").replace("ọe", "oẹ");
    // Vần "uy"
    t = t.replace("úy", "uý").replace("ùy", "uỳ").replace("ủy", "uỷ").replace("ũy", "uỹ").replace("ụy", "uỵ");

    // 1. Ký tự đặc biệt
    t = t.replace("%", " phần trăm ");
    t = t.replace("&", " và ");
    t = t.replace("+", " cộng ");
    
    // 2. Các từ vựng tiếng Anh công nghệ phổ biến
    t = t.replace("rust", " rớt ");
    t = t.replace("mix", " mích ");
    t = t.replace("fl", " ép eo ");
    
    // 3. Quy đổi số thành chữ
    let t = NUM_RE.replace_all(&t, |caps: &Captures| {
        read_number(&caps[1])
    });
    
    t.into_owned()
}
