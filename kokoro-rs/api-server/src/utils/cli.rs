use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use colored::*;
use sysinfo::System;

const PID_FILE: &str = ".daemon.pid";
const ENV_FILE: &str = ".env";
const SERVICE_NAME: &str = "KokoroTTS";

/****
 * handle_cli: Hàm chính điều hướng các lệnh CLI
 * Phân tích tham số để vào menu tương tác hoặc thực thi trực tiếp các lệnh.
 * Hỗ trợ: setup, start, stop, status, logs, install, uninstall
 ****/
pub async fn handle_cli(args: Vec<String>) {
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("");
    match command {
        "setup" | "--setup" | "-c" => setup_wizard(),
        "start" | "--start" | "-s" => start_daemon(),
        "stop" | "--stop" | "-k" => stop_daemon(),
        "status" | "--status" | "-t" => check_status(),
        "logs" | "--logs" | "-l" => view_logs(),
        "install" | "--install" | "-i" => install_service(),
        "uninstall" | "--uninstall" | "-u" => uninstall_service(),
        "help" | "--help" | "-h" => print_help(),
        "cli" | _ => interactive_menu(),
    }
}

fn print_help() {
    println!("\n{}", "KOKORO TTS API SERVER".bold().cyan());
    println!("{}", "=====================".cyan());
    println!("  {}   | {}  Cài đặt cấu hình", "-c".green(), "setup".green());
    println!("  {}   | {}  Khởi chạy server ẩn (daemon)", "-s".green(), "start".green());
    println!("  {}   | {}   Dừng server ẩn", "-k".green(), "stop".green());
    println!("  {}   | {} Kiểm tra trạng thái", "-t".green(), "status".green());
    println!("  {}   | {}   Xem log theo thời gian thực", "-l".green(), "logs".green());
    println!("  {}  | {} Cài đặt tự khởi chạy cùng hệ thống (trước login)", "-i".green(), "install".green());
    println!("  {}  | {} Gỡ bỏ tự khởi chạy", "-u".green(), "uninstall".green());
    println!("  {}  | {}   Hiển thị trợ giúp này", "-h".green(), "help".green());
    println!("  (không tham số)     Menu tương tác");
}

/****
 * interactive_menu: Hiển thị giao diện người dùng trên Terminal
 * Cho phép người dùng chọn chức năng bằng cách nhập số.
 ****/
fn interactive_menu() {
    loop {
        println!("\n{}", "=== KOKORO TTS API SERVER CLI ===".bold().cyan());
        println!("1. Cài đặt cấu hình (Setup)");
        println!("2. Khởi chạy Server ẩn (Start Daemon)");
        println!("3. Kiểm tra trạng thái (Status)");
        println!("4. Dừng Server ẩn (Stop Daemon)");
        println!("5. Xem Logs (View Logs)");
        println!("6. Cài đặt tự khởi chạy cùng hệ thống (Install Service)");
        println!("7. Gỡ bỏ tự khởi chạy (Uninstall Service)");
        println!("0. Thoát");
        
        print!("Chọn một chức năng (0-7): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        match input.trim() {
            "1" => setup_wizard(),
            "2" => start_daemon(),
            "3" => check_status(),
            "4" => stop_daemon(),
            "5" => view_logs(),
            "6" => install_service(),
            "7" => uninstall_service(),
            "0" => {
                println!("Đã thoát.");
                break;
            }
            _ => println!("{}", "Lựa chọn không hợp lệ, vui lòng thử lại!".red()),
        }
    }
}

/****
 * setup_wizard: Trình cài đặt cấu hình tương tác
 * Đọc file .env, hiển thị giá trị hiện tại, cho phép thay đổi và ghi đè.
 ****/
fn setup_wizard() {
    println!("\n{}", "--- Cài đặt cấu hình Kokoro TTS API (Để trống = giữ nguyên) ---".yellow());
    
    let current_port = std::env::var("PORT").unwrap_or_else(|_| "7424".to_string());
    let current_engine = std::env::var("TTS_ENGINE_PATH").unwrap_or_else(|_| "./kokoro-rs.exe".to_string());
    
    let port = prompt(&format!("Cổng (PORT) [{}]: ", current_port), &current_port);
    let engine = prompt(&format!("Đường dẫn TTS Engine [{}]: ", current_engine), &current_engine);
    
    update_env_file("PORT", &port);
    update_env_file("TTS_ENGINE_PATH", &format!("\"{}\"", engine.trim_matches('"')));
    
    println!("{}", "Đã lưu cấu hình thành công!".green());
}

/****
 * start_daemon: Chạy server ở chế độ ẩn (background)
 * Sinh một tiến trình con không có cửa sổ và lưu PID.
 ****/
fn start_daemon() {
    if is_daemon_running() {
        println!("{}", "Server đang chạy ẩn rồi! (Hãy dùng Status để kiểm tra)".yellow());
        return;
    }
    
    let exe_path = env::current_exe().expect("Không thể lấy đường dẫn thực thi");
    
    fs::create_dir_all("logs").unwrap_or_default();
    let out_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("logs/daemon.log")
        .expect("Không thể tạo file logs/daemon.log");
    
    let err_file = out_file.try_clone().expect("Lỗi clone file handle");

    #[cfg(windows)]
    let child_res = {
        // Cờ CREATE_NO_WINDOW = 0x08000000
        Command::new(&exe_path)
            .stdout(std::process::Stdio::from(out_file))
            .stderr(std::process::Stdio::from(err_file))
            .creation_flags(0x08000000)
            .spawn()
    };
    
    #[cfg(not(windows))]
    let child_res = Command::new(&exe_path)
        .stdout(std::process::Stdio::from(out_file))
        .stderr(std::process::Stdio::from(err_file))
        .spawn();
    
    match child_res {
        Ok(process) => {
            let pid = process.id();
            if let Err(e) = fs::write(PID_FILE, pid.to_string()) {
                println!("{}: {}", "Lỗi khi lưu PID".red(), e);
            } else {
                println!("{} (PID: {})", "Kokoro TTS API Server đã khởi chạy trong nền!".green(), pid);
            }
        }
        Err(e) => {
            println!("{}: {}", "Không thể khởi chạy tiến trình".red(), e);
        }
    }
}

/****
 * stop_daemon: Tắt server đang chạy ẩn
 ****/
fn stop_daemon() {
    if let Ok(pid_str) = fs::read_to_string(PID_FILE) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            let mut sys = System::new_all();
            sys.refresh_all();
            
            if sys.process(sysinfo::Pid::from_u32(pid)).is_some() {
                #[cfg(windows)]
                let _ = Command::new("taskkill")
                    .args(&["/F", "/PID", &pid.to_string()])
                    .output();
                
                #[cfg(not(windows))]
                let _ = Command::new("kill")
                    .args(&["-9", &pid.to_string()])
                    .output();
                    
                println!("{}", "Đã dừng Kokoro TTS API Server!".green());
            } else {
                println!("{}", "Server không hoạt động, tiến trình có thể đã tắt từ trước.".yellow());
            }
        }
    } else {
        println!("{}", "Không tìm thấy file .pid. Server chưa được khởi chạy ẩn.".yellow());
        return;
    }

    let _ = fs::remove_file(PID_FILE);
}

/****
 * check_status: Kiểm tra trạng thái server ẩn
 ****/
fn check_status() {
    let pid_str = fs::read_to_string(PID_FILE).unwrap_or_default();
    let pid = pid_str.trim().parse::<u32>().unwrap_or(0);
    
    let mut is_online = false;
    let mut memory_mb = 0.0;
    let mut cpu_usage = 0.0;

    if pid > 0 {
        let mut sys = System::new_all();
        sys.refresh_all();
        std::thread::sleep(std::time::Duration::from_millis(200));
        sys.refresh_all();
        
        if let Some(process) = sys.process(sysinfo::Pid::from_u32(pid)) {
            is_online = true;
            memory_mb = process.memory() as f64 / 1024.0 / 1024.0;
            cpu_usage = process.cpu_usage();
        }
    }

    // Kiểm tra trạng thái auto-start
    let autostart_status = check_autostart_installed();

    let mut table = comfy_table::Table::new();
    table.load_style(comfy_table::presets::UTF8_FULL.with_rounded_corners());

    table.set_header(vec![
        comfy_table::Cell::new("service").fg(comfy_table::Color::Cyan),
        comfy_table::Cell::new("status").fg(comfy_table::Color::Cyan),
        comfy_table::Cell::new("cpu").fg(comfy_table::Color::Cyan),
        comfy_table::Cell::new("memory").fg(comfy_table::Color::Cyan),
        comfy_table::Cell::new("auto-start").fg(comfy_table::Color::Cyan),
    ]);

    let (status_text, status_color) = if is_online {
        ("online", comfy_table::Color::Green)
    } else {
        ("offline", comfy_table::Color::Red)
    };

    let (autostart_text, autostart_color) = if autostart_status {
        ("enabled", comfy_table::Color::Green)
    } else {
        ("disabled", comfy_table::Color::DarkGrey)
    };

    table.add_row(vec![
        comfy_table::Cell::new("kokoro-api"),
        comfy_table::Cell::new(status_text).fg(status_color).add_attribute(comfy_table::Attribute::Bold),
        comfy_table::Cell::new(format!("{:.0}%", cpu_usage)),
        comfy_table::Cell::new(format!("{:.1}mb", memory_mb)),
        comfy_table::Cell::new(autostart_text).fg(autostart_color),
    ]);

    println!("{table}");

    if is_online {
        let current_port = std::env::var("PORT").unwrap_or_else(|_| "7424".to_string());
        println!("API Server: {}", format!("http://localhost:{}", current_port).cyan());
        println!("Health:     {}", format!("http://localhost:{}/api/health", current_port).cyan());
    } else {
        let _ = fs::remove_file(PID_FILE);
    }
}

/****
 * install_service: Đăng ký server tự khởi chạy cùng hệ thống (TRƯỚC KHI đăng nhập)
 * - Windows: Dùng schtasks chạy dưới tài khoản SYSTEM (chạy trước login, giống Google Remote Desktop)
 * - Linux: Sinh file systemd service với WantedBy=multi-user.target (chạy trước login)
 ****/
fn install_service() {
    let exe_path = env::current_exe().expect("Không thể lấy đường dẫn thực thi");
    let exe_str = exe_path.to_string_lossy().to_string();
    let work_dir = exe_path.parent().unwrap().to_string_lossy().to_string();

    #[cfg(windows)]
    {
        println!("{}", "Đang đăng ký Kokoro TTS API chạy cùng hệ thống (Windows Task Scheduler)...".cyan());
        
        // Dùng schtasks với /RU SYSTEM để chạy trước khi bất kỳ user nào đăng nhập
        // /SC ONSTART = Kích hoạt khi hệ thống khởi động
        // /RU SYSTEM = Chạy dưới tài khoản SYSTEM (không cần login)
        // /RL HIGHEST = Quyền cao nhất
        // /F = Force overwrite nếu đã tồn tại
        let output = Command::new("schtasks")
            .args(&[
                "/Create",
                "/SC", "ONSTART",
                "/TN", SERVICE_NAME,
                "/TR", &format!("\"{}\"", exe_str),
                "/RU", "SYSTEM",
                "/RL", "HIGHEST",
                "/F",
            ])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                println!("{}", "Đã cài đặt thành công!".green().bold());
                println!("  Task name: {}", SERVICE_NAME.cyan());
                println!("  Exe path:  {}", exe_str.cyan());
                println!("  Run as:    {}", "SYSTEM (chạy trước login)".yellow());
                println!("\nServer sẽ tự khởi chạy mỗi khi bật máy, không cần đăng nhập.");
                println!("Dùng '{} -u' để gỡ bỏ.", exe_str);
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                let stdout = String::from_utf8_lossy(&o.stdout);
                eprintln!("{}", "Không thể cài đặt. Có thể cần chạy với quyền Administrator.".red());
                eprintln!("Chi tiết: {} {}", stdout, stderr);
            }
            Err(e) => {
                eprintln!("{}: {}", "Lỗi thực thi schtasks".red(), e);
            }
        }
    }

    #[cfg(not(windows))]
    {
        println!("{}", "Đang tạo systemd service cho Kokoro TTS API...".cyan());
        
        let service_content = format!(
            r#"[Unit]
Description=Kokoro TTS API Server
After=network.target
StartLimitIntervalSec=60
StartLimitBurst=5

[Service]
Type=simple
ExecStart={exe}
WorkingDirectory={work_dir}
Restart=always
RestartSec=5
StandardOutput=append:{work_dir}/logs/daemon.log
StandardError=append:{work_dir}/logs/daemon.log

# Chạy với quyền root để bind cổng < 1024 nếu cần
# Nếu muốn chạy với user khác, thay đổi dòng dưới:
# User=kokoro
# Group=kokoro

[Install]
# multi-user.target = chạy trước khi user đăng nhập (giống Google Remote Desktop)
WantedBy=multi-user.target
"#,
            exe = exe_str,
            work_dir = work_dir
        );

        let service_path = format!("/etc/systemd/system/{}.service", SERVICE_NAME.to_lowercase());
        
        // Tạo thư mục logs trước
        let _ = Command::new("mkdir").args(&["-p", &format!("{}/logs", work_dir)]).output();

        match fs::write(&service_path, &service_content) {
            Ok(_) => {
                // Reload systemd và enable service
                let _ = Command::new("systemctl").args(&["daemon-reload"]).output();
                let enable_output = Command::new("systemctl")
                    .args(&["enable", &SERVICE_NAME.to_lowercase()])
                    .output();
                let start_output = Command::new("systemctl")
                    .args(&["start", &SERVICE_NAME.to_lowercase()])
                    .output();
                
                match (enable_output, start_output) {
                    (Ok(e), Ok(s)) if e.status.success() && s.status.success() => {
                        println!("{}", "Đã cài đặt và khởi chạy systemd service thành công!".green().bold());
                        println!("  Service:   {}", SERVICE_NAME.to_lowercase().cyan());
                        println!("  File:      {}", service_path.cyan());
                        println!("  Target:    {}", "multi-user.target (chạy trước login)".yellow());
                        println!("\nLệnh hữu ích:");
                        println!("  sudo systemctl status {}", SERVICE_NAME.to_lowercase());
                        println!("  sudo journalctl -u {} -f", SERVICE_NAME.to_lowercase());
                    }
                    _ => {
                        println!("{}", "Đã tạo file service nhưng không thể enable/start.".yellow());
                        println!("Thử chạy thủ công:");
                        println!("  sudo systemctl daemon-reload");
                        println!("  sudo systemctl enable {}", SERVICE_NAME.to_lowercase());
                        println!("  sudo systemctl start {}", SERVICE_NAME.to_lowercase());
                    }
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    eprintln!("{}", "Cần quyền root! Hãy chạy lại với sudo:".red());
                    eprintln!("  sudo {} install", exe_str);
                } else {
                    eprintln!("{}: {}", "Lỗi tạo file service".red(), e);
                }
            }
        }
    }
}

/****
 * uninstall_service: Gỡ bỏ server khỏi danh sách tự khởi chạy
 * - Windows: Xóa task trong Task Scheduler
 * - Linux: Disable và xóa file systemd service
 ****/
fn uninstall_service() {
    #[cfg(windows)]
    {
        println!("{}", "Đang gỡ bỏ Kokoro TTS API khỏi Task Scheduler...".cyan());
        
        let output = Command::new("schtasks")
            .args(&["/Delete", "/TN", SERVICE_NAME, "/F"])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                println!("{}", "Đã gỡ bỏ thành công! Server sẽ không tự khởi chạy khi bật máy nữa.".green().bold());
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                if stderr.contains("does not exist") || stderr.contains("không tồn tại") {
                    println!("{}", "Service chưa được cài đặt.".yellow());
                } else {
                    eprintln!("{}", "Không thể gỡ bỏ. Có thể cần chạy với quyền Administrator.".red());
                    eprintln!("Chi tiết: {}", stderr);
                }
            }
            Err(e) => {
                eprintln!("{}: {}", "Lỗi thực thi schtasks".red(), e);
            }
        }
    }

    #[cfg(not(windows))]
    {
        println!("{}", "Đang gỡ bỏ Kokoro TTS API khỏi systemd...".cyan());
        
        let service_name = SERVICE_NAME.to_lowercase();
        let service_path = format!("/etc/systemd/system/{}.service", service_name);
        
        // Stop -> Disable -> Remove file -> Reload
        let _ = Command::new("systemctl").args(&["stop", &service_name]).output();
        let _ = Command::new("systemctl").args(&["disable", &service_name]).output();
        
        match fs::remove_file(&service_path) {
            Ok(_) => {
                let _ = Command::new("systemctl").args(&["daemon-reload"]).output();
                println!("{}", "Đã gỡ bỏ thành công! Server sẽ không tự khởi chạy khi bật máy nữa.".green().bold());
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!("{}", "Service chưa được cài đặt.".yellow());
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                eprintln!("{}", "Cần quyền root! Hãy chạy lại với sudo.".red());
            }
            Err(e) => {
                eprintln!("{}: {}", "Lỗi xóa file service".red(), e);
            }
        }
    }
}

/****
 * check_autostart_installed: Kiểm tra xem service đã được cài đặt auto-start chưa
 ****/
fn check_autostart_installed() -> bool {
    #[cfg(windows)]
    {
        let output = Command::new("schtasks")
            .args(&["/Query", "/TN", SERVICE_NAME])
            .output();
        matches!(output, Ok(o) if o.status.success())
    }
    
    #[cfg(not(windows))]
    {
        let service_path = format!("/etc/systemd/system/{}.service", SERVICE_NAME.to_lowercase());
        Path::new(&service_path).exists()
    }
}

fn is_daemon_running() -> bool {
    if let Ok(pid_str) = fs::read_to_string(PID_FILE) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            let mut sys = System::new_all();
            sys.refresh_all();
            return sys.process(sysinfo::Pid::from_u32(pid)).is_some();
        }
    }
    false
}

fn prompt(message: &str, default: &str) -> String {
    print!("{}", message.cyan());
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();
    if input.is_empty() {
        default.to_string()
    } else {
        input.to_string()
    }
}

fn update_env_file(key: &str, value: &str) {
    let path = Path::new(ENV_FILE);
    let mut content = String::new();
    if path.exists() {
        content = fs::read_to_string(path).unwrap_or_default();
    }
    
    let mut new_lines = Vec::new();
    let mut found = false;
    
    for line in content.lines() {
        if line.starts_with(&format!("{}=", key)) {
            new_lines.push(format!("{}={}", key, value));
            found = true;
        } else {
            new_lines.push(line.to_string());
        }
    }
    
    if !found {
        new_lines.push(format!("{}={}", key, value));
    }
    
    let new_content = new_lines.join("\n") + "\n";
    if let Err(e) = fs::write(path, new_content) {
        println!("{}: {}", "Lỗi ghi file .env".red(), e);
    }
}

fn view_logs() {
    let log_file = Path::new("logs/daemon.log");
    if !log_file.exists() {
        println!("{}", "Không tìm thấy file logs/daemon.log! Server ẩn có thể chưa từng chạy.".red());
        return;
    }

    println!("Đang theo dõi log: {}", log_file.display().to_string().yellow());
    println!("{}", "Nhấn Ctrl+C để thoát chế độ xem log.\n".cyan());

    if let Ok(mut file) = fs::File::open(log_file) {
        use std::io::{Read, Seek, SeekFrom};
        let mut buffer = String::new();
        
        let metadata = file.metadata().unwrap();
        let file_size = metadata.len();
        let start_pos = if file_size > 10240 { file_size - 10240 } else { 0 };
        file.seek(SeekFrom::Start(start_pos)).unwrap_or_default();
        
        if let Ok(_) = file.read_to_string(&mut buffer) {
            print!("{}", buffer);
        }

        loop {
            let mut chunk = String::new();
            match file.read_to_string(&mut chunk) {
                Ok(bytes_read) if bytes_read > 0 => {
                    print!("{}", chunk);
                    use std::io::Write;
                    std::io::stdout().flush().unwrap_or_default();
                }
                Ok(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }
        }
    }
}
