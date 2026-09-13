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

    // 1. Số thập phân và hàng nghìn (loop để xử lý chuỗi kiểu 1.000.000)
    loop {
        let new_t = Regex::new(r"(\d+)[.,](\d+)").unwrap().replace_all(&t, |caps: &Captures| {
            let p1 = &caps[1];
            let p2 = &caps[2];
            // Nếu có đúng 3 chữ số phía sau và số đầu không phải 0 -> coi là phân cách hàng nghìn
            if p2.len() == 3 && p1 != "0" {
                format!("{}{}", p1, p2)
            } else {
                format!("{} phẩy {}", p1, p2)
            }
        }).into_owned();
        if new_t == t { break; }
        t = new_t;
    }

    // 2. Ký hiệu toán học
    t = t.replace("+", " cộng ");
    t = t.replace("*", " nhân ");
    t = t.replace("=", " bằng ");
    t = t.replace("^", " mũ ");
    t = t.replace("√", " căn ");
    
    // Dùng Regex vòng lặp cho Trừ và Chia để tránh ăn mất dấu ngắt câu hoặc text thông thường (chỉ đổi khi nằm giữa 2 số)
    loop {
        let new_t = Regex::new(r"(\d+)\s*-\s*(\d+)").unwrap().replace_all(&t, "$1 trừ $2").into_owned();
        if new_t == t { break; }
        t = new_t;
    }
    loop {
        let new_t = Regex::new(r"(\d+)\s*/\s*(\d+)").unwrap().replace_all(&t, "$1 chia $2").into_owned();
        if new_t == t { break; }
        t = new_t;
    }

    // 3. Đại lượng đo lường
    t = t.replace("°c", " độ xê ").replace("°f", " độ ép ").replace("°", " độ ");
    
    let units = vec![
        // Độ dài
        ("nm", "na nô mét"),
        ("pm", "pi cô mét"),
        ("um", "muy cờ rô mét"),
        ("μm", "muy cờ rô mét"),
        ("µm", "muy cờ rô mét"),
        ("mm", "mi li mét"),
        ("cm", "xen ti mét"),
        ("dm", "đề xi mét"),
        ("m", "mét"),
        ("km", "ki lô mét"),
        
        // Thể tích & Khối lượng
        ("ml", "mi li lít"),
        ("l", "lít"),
        ("mg", "mi li gam"),
        ("g", "gam"),
        ("kg", "ki lô gam"),
        
        // Tần số
        ("hz", "héc"),
        ("mhz", "mê ga héc"),
        ("ghz", "ghi ga héc"),
        
        // Dữ liệu IT
        ("kb", "ki lô bai"),
        ("mb", "mê ga bai"),
        ("gb", "ghi ga bai"),
        ("tb", "tê ra bai"),
        ("pb", "pê ta bai"),
        
        // Điện học (Vật lý)
        ("Ω", "ôm"),
        ("ω", "ôm"),
        ("ohm", "ôm"),
        ("ampe", "am pe"),
        ("ma", "mi li am pe"),
        ("a", "am pe"),
        ("kv", "ki lô vôn"),
        ("mv", "mi li vôn"),
        ("v", "vôn"),
        ("kw", "ki lô oát"),
        ("mw", "mê ga oát"),
        ("w", "oát"),
    ];
    for (sym, text) in units {
        let re = Regex::new(&format!(r"(\d+)\s*{}\b", sym)).unwrap();
        t = re.replace_all(&t, format!("$1 {}", text)).into_owned();
    }
    t = t.replace("log", " lốc ").replace("ôm", " ôm "); // Tách chữ ôm nếu đứng liền

    // Tách chữ và số dính liền nhau (VD: 32a -> 32 a)
    t = Regex::new(r"(\d)([a-zA-Z])").unwrap().replace_all(&t, "$1 $2").into_owned();
    t = Regex::new(r"([a-zA-Z])(\d)").unwrap().replace_all(&t, "$1 $2").into_owned();

    // 4. Ký tự đặc biệt khác
    t = t.replace("%", " phần trăm ");
    t = t.replace("&", " và ");
    
    // 5. Các từ vựng tiếng Anh công nghệ phổ biến
    t = t.replace("rust", " rớt ");
    t = t.replace("mix", " mích ");
    t = t.replace("fl", " ép eo ");
    
    // 6. Quy đổi số thành chữ
    let t = NUM_RE.replace_all(&t, |caps: &Captures| {
        read_number(&caps[1])
    });
    
    t.into_owned()
}
