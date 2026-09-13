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

/****
 * handle_cli: Hàm chính điều hướng các lệnh CLI
 * Phân tích tham số để vào menu tương tác hoặc thực thi trực tiếp các lệnh.
 ****/
pub async fn handle_cli(args: Vec<String>) {
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("");
    match command {
        "setup" | "--setup" | "-c" => setup_wizard(),
        "start" | "--start" | "-s" => start_daemon(),
        "stop" | "--stop" | "-k" => stop_daemon(),
        "status" | "--status" | "-t" => check_status(),
        "logs" | "--logs" | "-l" => view_logs(),
        "cli" | _ => interactive_menu(),
    }
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
        println!("0. Thoát");
        
        print!("Chọn một chức năng (0-5): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        match input.trim() {
            "1" => setup_wizard(),
            "2" => start_daemon(),
            "3" => check_status(),
            "4" => stop_daemon(),
            "5" => view_logs(),
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

    let mut table = comfy_table::Table::new();
    table.load_style(comfy_table::presets::UTF8_FULL.with_rounded_corners());

    table.set_header(vec![
        comfy_table::Cell::new("service").fg(comfy_table::Color::Cyan),
        comfy_table::Cell::new("status").fg(comfy_table::Color::Cyan),
        comfy_table::Cell::new("cpu").fg(comfy_table::Color::Cyan),
        comfy_table::Cell::new("memory").fg(comfy_table::Color::Cyan),
    ]);

    let (status_text, status_color) = if is_online {
        ("online", comfy_table::Color::Green)
    } else {
        ("offline", comfy_table::Color::Red)
    };

    table.add_row(vec![
        comfy_table::Cell::new("kokoro-api"),
        comfy_table::Cell::new(status_text).fg(status_color).add_attribute(comfy_table::Attribute::Bold),
        comfy_table::Cell::new(format!("{:.0}%", cpu_usage)),
        comfy_table::Cell::new(format!("{:.1}mb", memory_mb)),
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
