//! Biner pendamping ringan untuk hook Claude Code ke Noktah.
//! Dirancang untuk startup super cepat (< 10 ms), fail-open jika Noktah tidak aktif,
//! dan mendukung komunikasi interaktif dua arah untuk event PermissionRequest.

#![cfg_attr(not(debug_assertions), windows_subsystem = "console")]

use std::ffi::c_void;
use std::io::{stdin, stdout, IsTerminal, Read, Write};
use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, LocalFree, ERROR_IO_PENDING, ERROR_PIPE_BUSY,
    GENERIC_READ, GENERIC_WRITE, HANDLE, HLOCAL,
};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::{GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile as ReadFileOverlapped, WriteFile as WriteFileOverlapped,
    FILE_ATTRIBUTE_NORMAL, FILE_FLAG_OVERLAPPED, OPEN_EXISTING,
};
use windows::Win32::System::IO::{
    CancelIoEx, GetOverlappedResult, OVERLAPPED,
};
use windows::Win32::System::Pipes::WaitNamedPipeW;
use windows::Win32::System::Threading::{
    CreateEventW, GetCurrentProcess, OpenProcessToken, WaitForSingleObject,
};

/// Menghasilkan nama pipe unik per-user session.
fn get_user_pipe_name() -> String {
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_ok() {
            let mut len = 0u32;
            let _ = GetTokenInformation(token, TokenUser, None, 0, &mut len);
            if len > 0 {
                let mut buf = vec![0u8; len as usize];
                if GetTokenInformation(
                    token,
                    TokenUser,
                    Some(buf.as_mut_ptr() as *mut c_void),
                    len,
                    &mut len,
                )
                .is_ok()
                {
                    let token_user = &*(buf.as_ptr() as *const TOKEN_USER);
                    let mut sid_string = PWSTR::null();
                    if ConvertSidToStringSidW(token_user.User.Sid, &mut sid_string).is_ok()
                        && !sid_string.is_null()
                    {
                        let sid = sid_string.to_string().unwrap_or_default();
                        let _ = LocalFree(HLOCAL(sid_string.0 as _));
                        let _ = CloseHandle(token);
                        if !sid.is_empty() {
                            return format!(r"\\.\pipe\noktah-{}", sid);
                        }
                    }
                }
            }
            let _ = CloseHandle(token);
        }
    }

    let username = std::env::var("USERNAME").unwrap_or_else(|_| "default".into());
    format!(r"\\.\pipe\noktah-{}", username)
}

/// Menulis buffer ke named pipe secara overlapped dengan timeout opsional.
unsafe fn write_exact_overlapped(pipe: HANDLE, data: &[u8], timeout_ms: u32) -> bool {
    let event = match CreateEventW(None, true, false, PCWSTR::null()) {
        Ok(h) => h,
        Err(_) => return false,
    };

    let mut ol = OVERLAPPED::default();
    ol.hEvent = event;

    let mut written = 0u32;
    let ok = WriteFileOverlapped(
        pipe,
        Some(data),
        Some(&mut written),
        Some(&mut ol),
    );

    let res = if ok.is_ok() {
        true
    } else if GetLastError() == ERROR_IO_PENDING {
        let wait_res = WaitForSingleObject(event, timeout_ms);
        if wait_res.0 == 0 {
            GetOverlappedResult(pipe, &ol, &mut written, false).is_ok()
                && written as usize == data.len()
        } else {
            let _ = CancelIoEx(pipe, Some(&ol));
            false
        }
    } else {
        false
    };

    let _ = CloseHandle(event);
    res
}

/// Membaca buffer dari named pipe secara overlapped dengan timeout tertentu.
unsafe fn read_exact_overlapped(pipe: HANDLE, buf: &mut [u8], timeout_ms: u32) -> bool {
    let event = match CreateEventW(None, true, false, PCWSTR::null()) {
        Ok(h) => h,
        Err(_) => return false,
    };

    let mut ol = OVERLAPPED::default();
    ol.hEvent = event;

    let mut read_bytes = 0u32;
    let ok = ReadFileOverlapped(
        pipe,
        Some(buf),
        Some(&mut read_bytes),
        Some(&mut ol),
    );

    let res = if ok.is_ok() {
        read_bytes as usize == buf.len()
    } else if GetLastError() == ERROR_IO_PENDING {
        let wait_res = WaitForSingleObject(event, timeout_ms);
        if wait_res.0 == 0 {
            GetOverlappedResult(pipe, &ol, &mut read_bytes, false).is_ok()
                && read_bytes as usize == buf.len()
        } else {
            let _ = CancelIoEx(pipe, Some(&ol));
            false
        }
    } else {
        false
    };

    let _ = CloseHandle(event);
    res
}

fn print_help() {
    println!(
        "{}",
        concat!(
            "noktah-hook v",
            env!("CARGO_PKG_VERSION"),
r#" - Universal Mascot & Agent Hook for Noktah

PENGGUNAAN:
    noktah-hook [OPTIONS]
    noktah-hook <STATUS> [TOOL_ATAU_PESAN]
    <command> | noktah-hook [EVENT_NAME]   (Piped stdin JSON dari Claude Code / CI / script)

STATUS / EVENT YANG DIDUKUNG:
    working, tool, busy, build     Maskot hijau zamrud bergetar tekun mengeksekusi tool
    thinking, prompt, query, plan  Maskot violet berpikir & melirik ke atas
    done, completed, success       Maskot toska ceria tersenyum (^ ^) dengan semburan bintang
    error, fail, failed            Maskot merah (#EF4444) menggelengkan kepala
    idle, ready, reset             Maskot tenang bernapas mengikuti kursor
    approval, permission, ask      Memicu modal persetujuan izin di notch Noktah

OPSI:
    -s, --status, -e, --event <EV> Nama status atau event
    -t, --tool <NAMA>              Nama tool atau proses yang berjalan (misal: "cargo build")
    -m, --message, --summary <MSG> Pesan, ringkasan, atau deskripsi
    -p, --prompt <PROMPT>          Prompt tugas atau query
    -i, --input <JSON_ATAU_TEKS>   Payload input tool (JSON atau teks biasa)
    -d, --duration <MS>            Durasi pengerjaan dalam milidetik
        --title <TITLE>            Judul sesi atau nama agent pemanggil
    -h, --help                     Tampilkan panduan bantuan ini

CONTOH PENGGUNAAN:
    noktah-hook working "cargo test"
    noktah-hook done "Seluruh 169 unit test lulus"
    noktah-hook error "Kompilasi gagal di main.rs"
    noktah-hook idle
    noktah-hook --status working --tool "npm run build" --title "Frontend Build"
    noktah-hook --status approval --tool "rm -rf" --input "target/"
"#
        )
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut options_event: Option<String> = None;
    let mut options_tool: Option<String> = None;
    let mut options_message: Option<String> = None;
    let mut options_prompt: Option<String> = None;
    let mut options_input: Option<serde_json::Value> = None;
    let mut options_duration: Option<u64> = None;
    let mut options_title: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "--help" | "-h" | "/?" => {
                print_help();
                return;
            }
            "--status" | "-s" | "--event" | "-e" => {
                if i + 1 < args.len() {
                    options_event = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--tool" | "-t" => {
                if i + 1 < args.len() {
                    options_tool = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--message" | "-m" | "--summary" => {
                if i + 1 < args.len() {
                    options_message = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--prompt" | "-p" => {
                if i + 1 < args.len() {
                    options_prompt = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--input" | "-i" => {
                if i + 1 < args.len() {
                    let val_str = &args[i + 1];
                    let val = serde_json::from_str(val_str)
                        .unwrap_or_else(|_| serde_json::Value::String(val_str.clone()));
                    options_input = Some(val);
                    i += 1;
                }
            }
            "--duration" | "-d" => {
                if i + 1 < args.len() {
                    options_duration = args[i + 1].parse().ok();
                    i += 1;
                }
            }
            "--title" => {
                if i + 1 < args.len() {
                    options_title = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            pos if !pos.starts_with('-') => {
                if options_event.is_none() {
                    options_event = Some(pos.to_string());
                } else if options_tool.is_none() && options_message.is_none() && options_prompt.is_none() {
                    let ev_lower = options_event.as_deref().unwrap_or("").to_lowercase();
                    if matches!(ev_lower.as_str(), "working" | "tool" | "busy" | "run" | "running" | "build" | "pretooluse") {
                        options_tool = Some(pos.to_string());
                    } else if matches!(ev_lower.as_str(), "thinking" | "prompt" | "query" | "plan" | "userpromptsubmit") {
                        options_prompt = Some(pos.to_string());
                    } else {
                        options_message = Some(pos.to_string());
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    let is_stdin_piped = !stdin().is_terminal();
    let mut payload: serde_json::Value = if is_stdin_piped {
        let mut stdin_bytes = Vec::new();
        let _ = stdin().read_to_end(&mut stdin_bytes);
        if !stdin_bytes.is_empty() {
            serde_json::from_slice(&stdin_bytes).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        }
    } else {
        serde_json::json!({})
    };

    // Jika berjalan interaktif di terminal tanpa stdin dan tanpa argumen, tampilkan bantuan
    if !is_stdin_piped && options_event.is_none() {
        print_help();
        return;
    }

    // Gabungkan opsi CLI ke dalam payload JSON
    if let Some(obj) = payload.as_object_mut() {
        // Normalisasi format payload Antigravity (`toolCall: { name, args }`)
        // ke field standar Noktah (`tool_name`, `tool_input`) jika belum ada.
        if !obj.contains_key("tool_name") {
            if let Some(tc) = obj.get("toolCall").and_then(|v| v.as_object()).cloned() {
                if let Some(name) = tc.get("name").cloned() {
                    obj.insert("tool_name".to_string(), name);
                }
                if !obj.contains_key("tool_input") {
                    if let Some(args) = tc.get("args").cloned() {
                        obj.insert("tool_input".to_string(), args);
                    }
                }
            }
        }

        if let Some(ev) = options_event {
            obj.insert("hook_event_name".to_string(), serde_json::Value::String(ev));
        }
        if let Some(tool) = options_tool {
            obj.insert("tool_name".to_string(), serde_json::Value::String(tool));
        }
        if let Some(msg) = options_message {
            obj.insert("last_assistant_message".to_string(), serde_json::Value::String(msg));
        }
        if let Some(prompt) = options_prompt {
            obj.insert("prompt".to_string(), serde_json::Value::String(prompt));
        }
        if let Some(inp) = options_input {
            obj.insert("tool_input".to_string(), inp);
        }
        if let Some(dur) = options_duration {
            obj.insert("duration_ms".to_string(), serde_json::Value::from(dur));
        }
        if let Some(title) = options_title {
            obj.insert("session_title".to_string(), serde_json::Value::String(title));
        } else if !obj
            .get("session_title")
            .and_then(|v| v.as_str())
            .map_or(false, |s| !s.trim().is_empty())
        {
            let derived = obj
                .get("cwd")
                .and_then(|v| v.as_str())
                .and_then(|p| std::path::Path::new(p).file_name())
                .and_then(|n| n.to_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .or_else(|| {
                    std::env::current_dir().ok().and_then(|d| {
                        d.file_name()
                            .and_then(|n| n.to_str())
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(str::to_string)
                    })
                });
            if let Some(title) = derived {
                obj.insert("session_title".to_string(), serde_json::Value::String(title));
            }
        }
    }

    let ev = payload
        .get("hook_event_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_lowercase();
    let is_permission_request = matches!(ev.as_str(), "permissionrequest" | "permission" | "approval" | "ask");

    let serialized_payload = match serde_json::to_vec(&payload) {
        Ok(v) => v,
        Err(_) => std::process::exit(0),
    };

    let pipe_name = get_user_pipe_name();
    let pipe_w = HSTRING::from(&pipe_name);

    // 2. Hubungkan ke Named Pipe Noktah dengan fail-fast
    let mut handle = unsafe {
        CreateFileW(
            PCWSTR(pipe_w.as_ptr()),
            GENERIC_READ.0 | GENERIC_WRITE.0,
            Default::default(),
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
            HANDLE::default(),
        )
    };

    if handle.is_err() || handle.as_ref().unwrap().is_invalid() {
        let err = unsafe { GetLastError() };
        if err == ERROR_PIPE_BUSY {
            // Tunggu maksimal 25ms jika pipe sedang sibuk sesaat
            let _ = unsafe { WaitNamedPipeW(PCWSTR(pipe_w.as_ptr()), 25) };
            handle = unsafe {
                CreateFileW(
                    PCWSTR(pipe_w.as_ptr()),
                    GENERIC_READ.0 | GENERIC_WRITE.0,
                    Default::default(),
                    None,
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
                    HANDLE::default(),
                )
            };
        }
    }

    // FAIL-SAFE: Jika Noktah tidak aktif / pipe tidak ditemukan, silent exit 0 (fail-open)
    let pipe = match handle {
        Ok(h) if !h.is_invalid() => h,
        _ => std::process::exit(0),
    };

    // 3. Kirim Frame Permintaan: 4 byte panjang + payload UTF-8 JSON
    let len_prefix = (serialized_payload.len() as u32).to_le_bytes();
    let write_ok = unsafe {
        write_exact_overlapped(pipe, &len_prefix, 2000)
            && write_exact_overlapped(pipe, &serialized_payload, 5000)
    };

    if !write_ok {
        unsafe { let _ = CloseHandle(pipe); }
        std::process::exit(0);
    }

    // 4. Baca Frame Jawaban:
    // - Untuk event biasa: timeout 150ms (hanya menunggu ACK 0-byte)
    // - Untuk PermissionRequest: timeout 110 detik (menunggu keputusan user di GUI Notch)
    let timeout_ms = if is_permission_request { 110_000 } else { 150 };

    let mut resp_len_buf = [0u8; 4];
    let read_len_ok = unsafe { read_exact_overlapped(pipe, &mut resp_len_buf, timeout_ms) };

    if read_len_ok {
        let resp_len = u32::from_le_bytes(resp_len_buf) as usize;
        if resp_len > 0 && resp_len <= 10 * 1024 * 1024 {
            let mut resp_buf = vec![0u8; resp_len];
            let read_data_ok = unsafe { read_exact_overlapped(pipe, &mut resp_buf, 5000) };
            if read_data_ok {
                let _ = stdout().write_all(&resp_buf);
                let _ = stdout().flush();
            }
        }
    }

    unsafe { let _ = CloseHandle(pipe); }
    std::process::exit(0);
}
