# Kokoro TTS API Server

API Server HTTP bọc quanh `kokoro-rs.exe` để biến engine TTS tiếng Việt thành dịch vụ có thể gọi từ bất kỳ client nào (Web, Mobile, IoT).

## Cấu trúc thư mục phân phối (dist)

```
dist/
├── kokoro-api.exe       # API Server (Axum)
├── kokoro-rs.exe        # TTS Engine (ONNX Runtime)
├── kokoro_vi.onnx       # Model giọng nói (~300MB)
├── config.json          # Cấu hình phoneme
├── voicepacks_npy/      # 12 giọng đọc tiếng Việt
├── .env                 # File cấu hình server
└── tts_temp/            # File audio tạm (tự dọn dẹp)
```

## Cài đặt nhanh

### 1. Tải về và giải nén

Copy toàn bộ thư mục `dist/` vào bất kỳ vị trí nào trên máy. Ví dụ:
- Windows: `C:\TTS_server\`
- Linux: `/opt/kokoro-tts/`

### 2. Cấu hình (Tùy chọn)

Mở file `.env` để chỉnh sửa:

```env
PORT=7424                          # Cổng HTTP (mặc định 7424)
APP_ENV="development"              # development hoặc production
ALLOWED_ORIGINS=http://localhost:3000  # Danh sách CORS
TTS_ENGINE_PATH="./kokoro-rs.exe"  # Đường dẫn đến TTS engine
TTS_TEMP_MAX_AGE_SECS=300          # Xóa file tạm sau 5 phút
```

Hoặc dùng trình cài đặt tương tác:
```bash
./kokoro-api.exe setup
# hoặc
./kokoro-api.exe -c
```

### 3. Khởi chạy

```bash
# Chạy trực tiếp (hiển thị log trên terminal)
./kokoro-api.exe

# Chạy ẩn trong nền (daemon)
./kokoro-api.exe start
# hoặc
./kokoro-api.exe -s
```

Sau khi chạy, mở trình duyệt tại `http://localhost:7424` để sử dụng giao diện Web.

## Danh sách lệnh CLI

| Cờ   | Lệnh        | Chức năng                                          |
|------|-------------|-----------------------------------------------------|
| `-c` | `setup`     | Cài đặt cấu hình tương tác                         |
| `-s` | `start`     | Khởi chạy server ẩn (daemon)                        |
| `-k` | `stop`      | Dừng server ẩn                                      |
| `-t` | `status`    | Kiểm tra trạng thái (CPU, RAM, auto-start)          |
| `-l` | `logs`      | Xem log theo thời gian thực                         |
| `-i` | `install`   | Cài đặt tự khởi chạy cùng hệ thống (trước login)   |
| `-u` | `uninstall` | Gỡ bỏ tự khởi chạy khỏi hệ thống                   |
| `-h` | `help`      | Hiển thị trợ giúp                                   |
| (không có) | (menu) | Menu tương tác (chọn bằng số)                  |

## Tự khởi chạy cùng hệ thống (Auto-start)

### Cài đặt (install)

Server sẽ tự chạy **trước khi đăng nhập** (giống Google Remote Desktop), không cần mở khóa máy.

**Windows** (cần chạy với quyền Administrator):
```powershell
# Mở PowerShell hoặc CMD với quyền Admin
cd C:\TTS_server
.\kokoro-api.exe install
```

**Linux** (cần quyền root):
```bash
sudo /opt/kokoro-tts/kokoro-api install
```

### Gỡ bỏ (uninstall)

Khi **không muốn dùng nữa**, chạy lệnh `uninstall` để xóa sạch khỏi hệ thống:

**Windows**:
```powershell
.\kokoro-api.exe uninstall
# hoặc
.\kokoro-api.exe -u
```

**Linux**:
```bash
sudo /opt/kokoro-tts/kokoro-api uninstall
```

Sau khi `uninstall`, server sẽ **không tự khởi chạy** khi bật máy nữa. Bạn vẫn có thể chạy thủ công bằng `./kokoro-api.exe` bất cứ lúc nào.

> **Lưu ý**: Lệnh `uninstall` chỉ gỡ bỏ việc tự khởi chạy. Nó KHÔNG xóa file. Để xóa hoàn toàn, hãy xóa thư mục `dist/` (hoặc `C:\TTS_server\`) sau khi `uninstall`.

## API Endpoints

### `POST /api/tts` — Tạo giọng nói từ văn bản

**Request:**
```json
{
    "text": "Xin chào Việt Nam",
    "voice": "diem_trinh",
    "speed": 1.0,
    "pitch": 0.0
}
```

| Trường  | Kiểu   | Mặc định     | Mô tả                        |
|---------|--------|-------------|-------------------------------|
| `text`  | string | (bắt buộc)  | Văn bản cần đọc (tối đa 10.000 ký tự) |
| `voice` | string | `diem_trinh` | Tên giọng đọc                |
| `speed` | float  | `1.0`       | Tốc độ đọc (0.5 - 2.0)      |
| `pitch` | float  | `0.0`       | Cao độ giọng                 |

**Response:** File audio `Content-Type: audio/wav`

**Ví dụ với curl:**
```bash
curl -X POST http://localhost:7424/api/tts \
  -H "Content-Type: application/json" \
  -d '{"text":"Xin chào Việt Nam","voice":"diem_trinh"}' \
  --output output.wav
```

### `GET /api/voices` — Danh sách giọng đọc

**Response:**
```json
[
    {"id": "diem_trinh", "label": "Diễm Trinh", "gender": "Nữ"},
    {"id": "hung_thinh", "label": "Hưng Thịnh", "gender": "Nam"},
    ...
]
```

### `GET /api/health` — Kiểm tra trạng thái server

**Response:**
```json
{
    "status": "ok",
    "engine": "./kokoro-rs.exe",
    "version": "0.1.0"
}
```

## Danh sách giọng đọc hỗ trợ

| ID           | Tên         | Giới tính |
|-------------|-------------|-----------|
| diem_trinh  | Diễm Trinh  | Nữ        |
| hung_thinh  | Hưng Thịnh  | Nam       |
| mai_linh    | Mai Linh    | Nữ        |
| manh_dung   | Mạnh Dũng   | Nam       |
| my_yen      | Mỹ Yến      | Nữ        |
| ngoc_huyen  | Ngọc Huyền  | Nữ        |
| phat_tai    | Phát Tài    | Nam       |
| thanh_dat   | Thành Đạt   | Nam       |
| thuc_trinh  | Thục Trinh  | Nữ        |
| tuan_ngoc   | Tuấn Ngọc   | Nam       |
| duc_an      | Đức An      | Nam       |
| duc_duy     | Đức Duy     | Nam       |

## Xử lý sự cố

### Lỗi "Cổng đã bị chiếm" (AddrInUse)
```
Cổng 7424 đã bị chiếm bởi tiến trình khác!
```
**Cách khắc phục:**
1. Dừng server cũ: `./kokoro-api.exe -k`
2. Hoặc đổi cổng trong file `.env`: `PORT=7425`

### Lỗi "Không tìm thấy TTS Engine"
```
Không thể khởi chạy TTS engine: The system cannot find the file specified.
```
**Cách khắc phục:**
- Đảm bảo file `kokoro-rs.exe` nằm cùng thư mục với `kokoro-api.exe`
- Hoặc chỉnh đường dẫn trong `.env`: `TTS_ENGINE_PATH="C:\path\to\kokoro-rs.exe"`

### File tạm (.wav) trong thư mục tts_temp
- Hệ thống tự động dọn dẹp file cũ hơn **5 phút** (cấu hình bằng `TTS_TEMP_MAX_AGE_SECS`)
- Chu kỳ quét: mỗi **60 giây**
- Không cần lo về dung lượng ổ cứng

## Build từ mã nguồn

```bash
cd kokoro-rs/api-server
cargo build --release

# Copy ra dist
cp target/release/kokoro-api.exe ../dist/
cp .env ../dist/
```
