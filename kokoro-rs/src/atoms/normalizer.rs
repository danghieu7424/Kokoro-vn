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
            if c == 0 {
                chunk_words.push("không".to_string());
            } else {
                chunk_words.push(units[c as usize].to_string());
            }
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
    // Xử lý các từ viết tắt nhạy cảm chữ hoa chữ thường TRƯỚC KHI to_lowercase
    let mut t = text.replace("pH", " pê hát ").replace("PH", " pê hát ");
    t = t.replace("UBND", "Ủy ban nhân dân");
    t = t.replace("TP.HCM", "Thành phố Hồ Chí Minh");
    t = t.replace("TP HCM", "Thành phố Hồ Chí Minh");
    t = t.replace("CSGT", "Cảnh sát giao thông");
    // Số La Mã (Chỉ áp dụng khi đi kèm 'thế kỷ' để tránh nhầm với chữ XI trong 'xi măng')
    t = t.replace("thế kỷ XXI", "thế kỷ hai mươi mốt");
    t = t.replace("thế kỷ XX", "thế kỷ hai mươi");
    t = t.replace("thế kỷ XIX", "thế kỷ mười chín");
    t = t.replace("thế kỷ XVIII", "thế kỷ mười tám");
    t = t.replace("thế kỷ XVII", "thế kỷ mười bảy");
    t = t.replace("thế kỷ XVI", "thế kỷ mười sáu");
    t = t.replace("thế kỷ XV", "thế kỷ mười lăm");
    t = t.replace("thế kỷ XIV", "thế kỷ mười bốn");
    t = t.replace("thế kỷ XIII", "thế kỷ mười ba");
    t = t.replace("thế kỷ XII", "thế kỷ mười hai");
    t = t.replace("thế kỷ XI", "thế kỷ mười một");
    t = t.replace("thế kỷ X", "thế kỷ mười");
    t = t.replace("thế kỷ IX", "thế kỷ chín");
    t = t.replace("thế kỷ VIII", "thế kỷ tám");
    t = t.replace("thế kỷ VII", "thế kỷ bảy");
    t = t.replace("thế kỷ VI", "thế kỷ sáu");
    t = t.replace("thế kỷ V", "thế kỷ năm");
    t = t.replace("thế kỷ IV", "thế kỷ bốn");
    t = t.replace("thế kỷ III", "thế kỷ ba");
    t = t.replace("thế kỷ II", "thế kỷ hai");
    t = t.replace("thế kỷ I", "thế kỷ một");
    t = t.replace("Quý I", "Quý một");
    t = t.replace("Quý II", "Quý hai");
    t = t.replace("Quý III", "Quý ba");
    t = t.replace("Quý IV", "Quý tư");
    
    t = t.to_lowercase();
    
    // 0. Chuẩn hóa vị trí dấu thanh (Kiểu cũ -> Kiểu mới để map đúng với từ điển sinh ra)
    // Vần "oa"
    t = t.replace("óa", "oá").replace("òa", "oà").replace("ỏa", "oả").replace("õa", "oã").replace("ọa", "oạ");
    // Vần "oe"
    t = t.replace("óe", "oé").replace("òe", "oè").replace("ỏe", "oẻ").replace("õe", "oẽ").replace("ọe", "oẹ");
    // Vần "uy"
    t = t.replace("úy", "uý").replace("ùy", "uỳ").replace("ủy", "uỷ").replace("ũy", "uỹ").replace("ụy", "uỵ");
    // Vần "ua" (Do từ điển gốc Kokoro sinh ra lỗi bỏ dấu trên chữ a thay vì chữ u)
    t = t.replace("úa", "uá").replace("ùa", "uà").replace("ủa", "uả").replace("ũa", "uã").replace("ụa", "uạ");

    // Sửa lỗi chính tả "hoặc" bị lỗi unicode trong từ điển
    t = t.replace("hoặc", "hoạc");

    // Xử lý số điện thoại: 098.123.4567 hoặc 098 123 4567 -> xóa phân cách để đọc từng số
    t = Regex::new(r"\b(0\d{2,3})[.\s]+(\d{3})[.\s]+(\d{3,4})\b").unwrap().replace_all(&t, "$1$2$3").into_owned();
    t = Regex::new(r"\b(\+84\d{2})[.\s]+(\d{3})[.\s]+(\d{3,4})\b").unwrap().replace_all(&t, "$1$2$3").into_owned();

    // Xóa dấu phân cách hàng nghìn chuẩn (Ví dụ: 1.500.000 hoặc 1,500,000)
    // Để không phá hỏng cấu trúc, ta duyệt và xóa các dấu . , nằm giữa các bộ 3 số
    t = Regex::new(r"\b(\d{1,3}(?:[.,]\d{3})+)\b").unwrap().replace_all(&t, |caps: &Captures| {
        caps[1].replace(".", "").replace(",", "")
    }).into_owned();

    // Số thập phân
    loop {
        let new_t = Regex::new(r"(\d+)[.,](\d+)").unwrap().replace_all(&t, "$1 phẩy $2").into_owned();
        if new_t == t { break; }
        t = new_t;
    }

    // 2. Ngày giờ và Thời gian (Phải xử lý trước toán học để tránh bị nhầm thành phép chia/trừ)
    t = Regex::new(r"\b(\d{1,2})[-/](\d{1,2})[-/](\d{4})\b").unwrap().replace_all(&t, |caps: &Captures| {
        let d_num: u32 = caps[1].parse().unwrap_or(10);
        let m = caps[2].trim_start_matches('0');
        let y = &caps[3];
        
        let d = if d_num >= 1 && d_num <= 9 {
            format!("mùng {}", d_num)
        } else {
            format!("ngày {}", d_num)
        };
        
        // Tách rời từng số của năm để đọc (2026 -> 2 0 2 6 -> hai không hai sáu)
        let y_spaced = y.chars().map(|c| c.to_string()).collect::<Vec<String>>().join(" ");
        format!("{} tháng {} năm {}", d, m, y_spaced)
    }).into_owned();
    t = Regex::new(r"\b(\d{1,2}):(\d{2}):(\d{2})\b").unwrap().replace_all(&t, |caps: &Captures| {
        let h = caps[1].parse::<u32>().unwrap_or(0);
        format!("{} giờ {} phút {} giây", h, &caps[2], &caps[3])
    }).into_owned();
    t = Regex::new(r"\b(\d{1,2}):(\d{2})\b").unwrap().replace_all(&t, |caps: &Captures| {
        let h = caps[1].parse::<u32>().unwrap_or(0);
        let m = &caps[2];
        if m == "00" { format!("{} giờ", h) } else { format!("{} giờ {}", h, m) }
    }).into_owned();

    // Dấu ':' giữa 2 số còn sót lại (sau khi xử lý giờ) thường là tỉ lệ hoặc phép chia
    t = Regex::new(r"(\d+)\s*:\s*(\d+)").unwrap().replace_all(&t, "$1 chia $2").into_owned();

    // Xử lý ngày tháng rút gọn (VD: 30/4, 01-09) - Tránh phá hỏng phân số 1/2, 3/4
    t = Regex::new(r"(?i)(ngày\s+)?\b(\d{1,2})([-/])(\d{1,2})\b").unwrap().replace_all(&t, |caps: &Captures| {
        let has_ngay = caps.get(1).is_some();
        let ngay_str = caps.get(1).map_or("", |m| m.as_str());
        let d_str = caps[2].to_string();
        let sep = caps[3].to_string();
        let m_str_raw = caps[4].to_string();
        
        let d_num: u32 = d_str.parse().unwrap_or(10);
        let m_num: u32 = m_str_raw.parse().unwrap_or(10);
        
        let is_date = has_ngay || d_num > 12 || d_str.starts_with('0') || m_str_raw.starts_with('0');
        
        if is_date && m_num >= 1 && m_num <= 12 && d_num >= 1 && d_num <= 31 {
            let m = m_str_raw.trim_start_matches('0');
            let m_str = if m == "4" { "tư" } else { m };
            
            // "30/4" -> 30 tháng tư. "01/09" -> mùng 1 tháng 9.
            let d_word = if has_ngay {
                format!("{} {}", ngay_str.trim(), d_num)
            } else if d_num >= 1 && d_num <= 9 {
                format!("mùng {}", d_num)
            } else {
                format!("{}", d_num) // Để im số cho regex số đếm tự đọc
            };
            format!("{} tháng {}", d_word, m_str)
        } else {
            let prefix = if has_ngay { ngay_str } else { "" };
            format!("{}{}{}{}", prefix, d_str, sep, m_str_raw)
        }
    }).into_owned();

    // 3. Ký hiệu toán học
    t = t.replace("+", " cộng ");
    t = t.replace("*", " nhân ");
    t = t.replace("=", " bằng ");
    t = t.replace("^", " mũ ");
    t = t.replace("√", " căn ");
    
    // Dấu Trừ và Khoảng cách (Range): 
    // Nếu dính liền 5-10 -> đọc là "đến" (trang 5 đến 10). Nếu có khoảng cách 5 - 10 -> "trừ".
    t = Regex::new(r"(\d+)-(\d+)").unwrap().replace_all(&t, "$1 đến $2").into_owned();
    loop {
        let new_t = Regex::new(r"(\d+)\s+-\s+(\d+)").unwrap().replace_all(&t, "$1 trừ $2").into_owned();
        if new_t == t { break; }
        t = new_t;
    }
    // Phép chia (/), loại bỏ nếu nằm trong từ (như URL) - Người Việt hay đọc là 'phần' (1/2 -> một phần hai)
    loop {
        let new_t = Regex::new(r"(\d+)\s*/\s*(\d+)").unwrap().replace_all(&t, "$1 phần $2").into_owned();
        if new_t == t { break; }
        t = new_t;
    }

    // 3. Đại lượng đo lường
    t = t.replace("°c", " độ xê ").replace("°f", " độ ép ").replace("°", " độ ");
    
    // Tiền tệ ($ phải xử lý riêng vì nó là ký tự đặc biệt trong Regex và không ăn \b)
    // Giọng TTS đôi khi đọc "đô la" thành "đô lả", đổi thành "đô" cho giao tiếp tự nhiên
    t = Regex::new(r"(\d+)\s*\$").unwrap().replace_all(&t, "$1 đô").into_owned();
    
    let units = vec![
        // Tiền tệ
        ("đ", "đồng"),
        ("vnđ", "việt nam đồng"),
        ("vnd", "việt nam đồng"),

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
        
        // Điện dung (Farad)
        ("pf", "pi cô fa ra"),
        ("nf", "na nô fa ra"),
        ("uf", "muy cờ rô fa ra"),
        ("μf", "muy cờ rô fa ra"),
        ("µf", "muy cờ rô fa ra"),
        ("mf", "mi li fa ra"),
        ("f", "fa ra"),
        
        // Điện cảm (Henry)
        // Bỏ qua pH (pi cô hen ri) để tránh trùng chữ "ph" (phút) hoặc "độ pH"
        ("nh", "na nô hen ri"),
        ("uh", "muy cờ rô hen ri"),
        ("μh", "muy cờ rô hen ri"),
        ("µh", "muy cờ rô hen ri"),
        ("mh", "mi li hen ri"),
        ("h", "hen ri"),
        
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

    // Xử lý tốc độ (/s -> trên giây, /h -> trên giờ) dính liền với đơn vị đo lường
    // Chỉ kích hoạt khi đằng trước là các chữ đặc thù của đơn vị để chống nhận nhầm URL
    let rate_s = Regex::new(r"(mét|lít|gam|héc|bai|ra|ri|ôm|pe|vôn|oát|độ)\s*/\s*s\b").unwrap();
    t = rate_s.replace_all(&t, "$1 trên giây").into_owned();
    let rate_h = Regex::new(r"(mét|lít|gam|héc|bai|ra|ri|ôm|pe|vôn|oát|độ)\s*/\s*h\b").unwrap();
    t = rate_h.replace_all(&t, "$1 trên giờ").into_owned();

    // Tách chữ và số dính liền nhau (VD: 32a -> 32 a)
    t = Regex::new(r"(\d)([a-zA-Z])").unwrap().replace_all(&t, "$1 $2").into_owned();
    t = Regex::new(r"([a-zA-Z])(\d)").unwrap().replace_all(&t, "$1 $2").into_owned();

    // 4. Ký tự đặc biệt khác
    t = t.replace("%", " phần trăm ");
    t = t.replace("&", " và ");
    
    // 5. Các từ vựng tiếng Anh công nghệ phổ biến
    t = t.replace("github", " gít hắp ");
    t = t.replace("git", " gít ");
    t = t.replace("terminal", " tơ mi nồ ");
    t = t.replace("vs code", " vi ét cốt ");
    t = t.replace("vs", " vi ét ");
    t = t.replace("code", " cốt ");
    t = t.replace("fork", " phóc ");
    t = t.replace("file", " phai ");
    t = t.replace("folder", " phôn đơ ");
    t = t.replace("web", " goép ");
    t = t.replace("app", " áp ");
    t = t.replace("server", " sơ vơ ");
    t = t.replace("api", " ây pi ai ");
    t = t.replace("bug", " bắc ");
    t = t.replace("error", " e rơ ");
    t = t.replace("rust", " rớt ");
    t = t.replace("mix", " mích ");
    t = t.replace("fl", " ép eo ");
    
    // 6. Các từ vựng thể loại âm nhạc
    t = t.replace("chill house", " chiu hao ");
    t = t.replace("house lak", " hao lắc ");
    t = t.replace("house", " hao ");
    t = t.replace("pop balat", " póp ba lát ");
    t = t.replace("ballad", " ba lát ");
    t = t.replace("pop", " póp ");
    t = t.replace("drill", " đờ riu ");
    t = t.replace("lofi", " lô phai ");
    t = t.replace("remix", " ri mích ");
    t = t.replace("trap", " tráp ");
    t = t.replace("hiphop", " híp hóp ");
    t = t.replace("rap", " ráp ");
    t = t.replace("edm", " i đi em ");
    t = t.replace("dj", " đi dây ");
    
    // 7. Quy đổi số thành chữ
    let t = NUM_RE.replace_all(&t, |caps: &Captures| {
        read_number(&caps[1])
    });
    
    t.into_owned()
}
