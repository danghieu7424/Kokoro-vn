# Kokoro-RS Vietnamese: Hướng dẫn Sử dụng & Chuẩn hóa Text

Kokoro-RS là phiên bản port sang Rust của hệ thống Kokoro TTS, được tối ưu hóa đặc biệt cho Tiếng Việt với tốc độ cực nhanh (Zero-Python) và bộ xử lý Text Normalizer (NLP) "chuẩn bản địa".

## 🚀 Cách sử dụng (CLI)

Để biên dịch và chạy hệ thống, sử dụng lệnh `cargo run`:

```bash
cargo run --release -- -t "Nội dung văn bản cần đọc" -o "outputs/audio.wav"
```

**Các tham số chính:**
- `-t`, `--text`: Chuỗi văn bản tiếng Việt cần đọc.
- `-o`, `--output`: Đường dẫn lưu file âm thanh `.wav` đầu ra (Mặc định: `output.wav`).
- `-v`, `--voice`: Tên giọng đọc chính (Mặc định: `diem_trinh`).
- `-s`, `--speed`: Tốc độ đọc (Mặc định: `1.0`).
- `-m`, `--mix-blend`: Trộn Tone (Pha trộn nhiều giọng). VD: `"diem_trinh=70,duc_duy=30"`.
- `-p`, `--pitch`: Tăng giảm Pitch (Cent, giống FL Studio). Tăng 100 = lên 1 nửa cung.

Ví dụ nâng cao:
```bash
cargo run --release -- -t "Hôm nay là 13/09/2026 lúc 11:30. Giá 500đ" -v "tuan_ngoc" -s 1.2 -m "tuan_ngoc=80,diem_trinh=20" -o "outputs/test.wav"
```

### 🗣 Danh sách Giọng đọc (Voices) hỗ trợ

| Mã Giọng (`--voice`) | Giới tính / Mô tả |
| --- | --- |
| `diem_trinh` | Nữ (Giọng chuẩn, truyền cảm) |
| `hung_thinh` | Nam |
| `mai_linh` | Nữ |
| `manh_dung` | Nam |
| `my_yen` | Nữ |
| `ngoc_huyen` | Nữ |
| `phat_tai` | Nam |
| `thanh_dat` | Nam |
| `thuc_trinh` | Nữ |
| `tuan_ngoc` | Nam |
| `duc_an` | Nam |
| `duc_duy` | Nam |

---


## 🧠 Sức mạnh của Bộ Chuẩn hóa Văn bản (Normalizer)

Bộ Normalizer của Kokoro-RS được thiết kế để giải quyết các trường hợp giao tiếp lắt léo nhất trong tiếng Việt mà các hệ thống TTS nước ngoài thường bỏ sót. Dưới đây là các định dạng hệ thống tự động hỗ trợ:

### 1. Số đếm và Tiền tệ
- Đọc mượt mà số siêu lớn lên tới tỷ tỷ: `1.000.000` $\rightarrow$ "một triệu".
- Xử lý số thập phân (cả dấu chấm và phẩy): `1,5` hoặc `1.5` $\rightarrow$ "một phẩy năm".
- Nhận diện tiền tệ tự động: 
  - `500đ` $\rightarrow$ "năm trăm đồng".
  - `200 VNĐ` / `VND` $\rightarrow$ "hai trăm việt nam đồng".
  - `10$` $\rightarrow$ "mười đô".

### 2. Ngày tháng và Giờ giấc (Thông minh)
Hệ thống tự động phân biệt dấu gạch chéo `/` của Toán học và `/` của Ngày tháng theo thói quen người Việt.
- **Ngày tháng đầy đủ:** `13/09/2026` hoặc `13-09-2026` $\rightarrow$ "ngày mười ba tháng chín năm hai không hai sáu" (Lọc bỏ số 0 vô nghĩa ở tháng, đọc năm từng số).
- **Ngày tháng rút gọn:** `30/4` $\rightarrow$ "ba mươi tháng tư", `01/09` $\rightarrow$ "mùng một tháng chín". Các ngày từ 1 đến 9 tự động thêm chữ "mùng".
- **Giờ giấc:** 
  - `11:30:03` $\rightarrow$ "mười một giờ ba mươi phút không ba giây".
  - `12:30` $\rightarrow$ "mười hai giờ ba mươi" (rút gọn chữ phút tự nhiên).
  - `08:00` $\rightarrow$ "tám giờ".

### 3. Ký hiệu Toán học & Phân số
- **Cơ bản:** `+` (cộng), `-` (trừ), `*` (nhân), `=` (bằng), `^` (mũ), `√` (căn).
- **Phân số:** `1/2` $\rightarrow$ "một phần hai", `3/4` $\rightarrow$ "ba phần bốn". 
- **Tỉ lệ:** `1:2` $\rightarrow$ "một chia hai" (Tự động phân biệt với giờ giấc nhờ quy luật số lượng chữ số).

### 4. Đơn vị Vật lý & IT
Nhận diện và bung chữ các đơn vị viết tắt dính liền với số:
- **Độ dài / Khối lượng / Thể tích:** `km`, `m`, `cm`, `mm`, `µm`, `kg`, `g`, `ml`...
- **IT:** `GB`, `MB`, `KB`, `TB`, `PB` (vd: `50GB/s` $\rightarrow$ "năm mươi ghi ga bai trên giây").
- **Điện học:** `V`, `kV`, `A`, `mA`, `W`, `kW`, `Ω`, `pF`, `µF` (muy cờ rô fa ra), `mH`...
- **Đặc biệt:** Ký hiệu tốc độ `/s` và `/h` được bảo vệ (Chỉ dịch là "trên giây" hoặc "trên giờ" khi đứng sau đơn vị đo lường, không làm hỏng URL như `http://.../s`).

### 5. Sửa lỗi chính tả & Phát âm
- **Lỗi vần "ua":** Vá lỗi tách âm của từ điển gốc đối với các từ có vần "ua" (của, chúa, múa, lụa). Hệ thống ngầm định chuyển đổi `cuả` thành `của` và đọc mượt mà không bị vấp thành "cu a".
- **Chuẩn hóa dấu thanh:** Tự động quy chuẩn vị trí dấu thanh (vd: `hòa` $\rightarrow$ `hoà`, `thủy` $\rightarrow$ `thuỷ`) để map chính xác 100% với từ điển âm vị.

---

## 🛠 Kiến trúc Hệ thống (Hybrid FSD + Atomic)
Mã nguồn được thiết kế theo triết lý **Feature-Sliced Design (FSD)** lai với **Atomic**:
- `src/atoms/normalizer.rs`: Chịu trách nhiệm bóc tách và chuẩn hóa chuỗi đầu vào.
- `src/atoms/vn_g2p.rs`: Chuyển đổi Grapheme sang Phoneme siêu tốc qua bộ từ điển 65,000+ từ load sẵn trên RAM.
- `src/atoms/inference.rs`: Quản lý ONNX session của engine Kokoro.

Tất cả chạy hoàn toàn bằng C++ / Rust, triệt tiêu hoàn toàn độ trễ của Python, mang lại trải nghiệm Real-time thực thụ.
