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
    tile, tile-set                 Kirim/perbarui status tile kustom ke halaman Sistem / Mini
    tile-remove, tile-del          Hapus tile kustom berdasarkan ID
    tile-clear                     Bersihkan seluruh tile kustom

OPSI:
    -s, --status, -e, --event <EV> Nama status atau event
    -t, --tool <NAMA>              Nama tool atau proses yang berjalan (misal: "cargo build")
    -m, --message, --summary <MSG> Pesan, ringkasan, atau deskripsi
    -p, --prompt <PROMPT>          Prompt tugas atau query
    -i, --input <JSON_ATAU_TEKS>   Payload input tool (JSON atau teks biasa)
    -d, --duration <MS>            Durasi pengerjaan dalam milidetik
        --title <TITLE>            Judul sesi atau nama agent pemanggil
        --id <ID>                  ID tile status kustom
    -v, --value <VALUE>            Nilai status tile (misal: "95%", "OK")
        --subtitle <SUB>           Subjudul atau deskripsi singkat tile
        --tone <TONE>              Warna/nuansa tile: neutral, accent, success, warning, error
        --progress <PCT>           Progres numerik 0.0-1.0 atau 0-100 (misal: 75)
        --ttl <DETIK>              Masa berlaku tile sebelum kedaluwarsa otomatis (detik)
        --pin-mini                 Sematkan tile ini ke pill mini Noktah saat idle
    -h, --help                     Tampilkan panduan bantuan ini

CONTOH PENGGUNAAN:
    noktah-hook working "cargo test"
    noktah-hook done "Seluruh unit test lulus"
    noktah-hook error "Kompilasi gagal di main.rs"
    noktah-hook idle
    noktah-hook tile cpu "85%" "High load" --tone warning
    noktah-hook tile build "Passed" --tone success --progress 100 --pin-mini
    noktah-hook tile-remove cpu
    noktah-hook tile-clear
    noktah-hook --status working --tool "npm run build" --title "Frontend Build"
    noktah-hook --status approval --tool "rm -rf" --input "target/"
"#
        )
    );
}

/// Membangun payload JSON murni dari argumen CLI dan stdin JSON.
/// Mengembalikan None jika argumen meminta bantuan (--help, -h, /?).
pub fn build_payload_from_args(
    args: &[String],
    stdin_payload: serde_json::Value,
) -> Option<serde_json::Value> {
    let actual_args = if !args.is_empty()
        && (args[0].ends_with("noktah-hook")
            || args[0].ends_with("noktah-hook.exe")
            || (args.len() > 1
                && !args[0].starts_with('-')
                && !args[0].starts_with('/')
                && (args[0].contains('/') || args[0].contains('\\'))))
    {
        &args[1..]
    } else {
        args
    };

    for arg in actual_args {
        if arg == "--help" || arg == "-h" || arg == "/?" {
            return None;
        }
    }

    let mut options_event: Option<String> = None;
    let mut options_tool: Option<String> = None;
    let mut options_message: Option<String> = None;
    let mut options_prompt: Option<String> = None;
    let mut options_input: Option<serde_json::Value> = None;
    let mut options_duration: Option<u64> = None;
    let mut options_title: Option<String> = None;

    let mut options_tile_id: Option<String> = None;
    let mut options_tile_title: Option<String> = None;
    let mut options_tile_title_explicit: Option<String> = None;
    let mut options_tile_value: Option<String> = None;
    let mut options_tile_subtitle: Option<String> = None;
    let mut options_tile_tone: Option<String> = None;
    let mut options_tile_progress: Option<f32> = None;
    let mut options_tile_ttl: Option<u64> = None;
    let mut options_tile_pin_mini: Option<bool> = None;

    let mut i = 0;
    while i < actual_args.len() {
        let arg = &actual_args[i];
        match arg.as_str() {
            "--status" | "-s" | "--event" | "-e" => {
                if i + 1 < actual_args.len() {
                    options_event = Some(actual_args[i + 1].clone());
                    i += 1;
                }
            }
            "--tool" | "-t" => {
                if i + 1 < actual_args.len() {
                    options_tool = Some(actual_args[i + 1].clone());
                    i += 1;
                }
            }
            "--message" | "-m" | "--summary" => {
                if i + 1 < actual_args.len() {
                    options_message = Some(actual_args[i + 1].clone());
                    i += 1;
                }
            }
            "--prompt" | "-p" => {
                if i + 1 < actual_args.len() {
                    options_prompt = Some(actual_args[i + 1].clone());
                    i += 1;
                }
            }
            "--input" | "-i" => {
                if i + 1 < actual_args.len() {
                    let val_str = &actual_args[i + 1];
                    let val = serde_json::from_str(val_str)
                        .unwrap_or_else(|_| serde_json::Value::String(val_str.clone()));
                    options_input = Some(val);
                    i += 1;
                }
            }
            "--duration" | "-d" => {
                if i + 1 < actual_args.len() {
                    options_duration = actual_args[i + 1].parse().ok();
                    i += 1;
                }
            }
            "--title" => {
                if i + 1 < actual_args.len() {
                    let val = actual_args[i + 1].clone();
                    options_title = Some(val.clone());
                    options_tile_title_explicit = Some(val);
                    i += 1;
                }
            }
            "--id" => {
                if i + 1 < actual_args.len() {
                    options_tile_id = Some(actual_args[i + 1].clone());
                    i += 1;
                }
            }
            "--value" | "-v" => {
                if i + 1 < actual_args.len() {
                    options_tile_value = Some(actual_args[i + 1].clone());
                    i += 1;
                }
            }
            "--subtitle" => {
                if i + 1 < actual_args.len() {
                    options_tile_subtitle = Some(actual_args[i + 1].clone());
                    i += 1;
                }
            }
            "--tone" => {
                if i + 1 < actual_args.len() {
                    options_tile_tone = Some(actual_args[i + 1].clone());
                    i += 1;
                }
            }
            "--progress" => {
                if i + 1 < actual_args.len() {
                    if let Ok(p) = actual_args[i + 1].parse::<f32>() {
                        if p.is_finite() {
                            let norm = if p > 1.0 && p <= 100.0 { p / 100.0 } else { p };
                            options_tile_progress = Some(norm.clamp(0.0, 1.0));
                        }
                    }
                    i += 1;
                }
            }
            "--ttl" => {
                if i + 1 < actual_args.len() {
                    options_tile_ttl = actual_args[i + 1].parse().ok();
                    i += 1;
                }
            }
            "--pin-mini" => {
                options_tile_pin_mini = Some(true);
            }
            pos if !pos.starts_with('-') => {
                if options_event.is_none() {
                    options_event = Some(pos.to_string());
                } else {
                    let ev_lower = options_event.as_deref().unwrap_or("").to_lowercase();
                    if matches!(
                        ev_lower.as_str(),
                        "tile" | "tile-set" | "tile_set" | "custom_tile" | "status_tile"
                    ) {
                        if options_tile_id.is_none() {
                            options_tile_id = Some(pos.to_string());
                            if options_tile_title.is_none() {
                                options_tile_title = Some(pos.to_string());
                            }
                        } else if options_tile_value.is_none() {
                            options_tile_value = Some(pos.to_string());
                        } else if options_tile_subtitle.is_none() {
                            options_tile_subtitle = Some(pos.to_string());
                        }
                    } else if matches!(
                        ev_lower.as_str(),
                        "tile-remove" | "tile_remove" | "tile-del" | "tile_del"
                    ) {
                        if options_tile_id.is_none() {
                            options_tile_id = Some(pos.to_string());
                        }
                    } else if options_tool.is_none()
                        && options_message.is_none()
                        && options_prompt.is_none()
                    {
                        if matches!(
                            ev_lower.as_str(),
                            "working" | "tool" | "busy" | "run" | "running" | "build" | "pretooluse"
                        ) {
                            options_tool = Some(pos.to_string());
                        } else if matches!(
                            ev_lower.as_str(),
                            "thinking" | "prompt" | "query" | "plan" | "userpromptsubmit"
                        ) {
                            options_prompt = Some(pos.to_string());
                        } else {
                            options_message = Some(pos.to_string());
                        }
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    let mut payload = stdin_payload;
    if !payload.is_object() {
        payload = serde_json::json!({});
    }

    if let Some(obj) = payload.as_object_mut() {
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
            obj.insert("session_title".to_string(), serde_json::Value::String(title.clone()));
            if options_tile_title.is_none() {
                options_tile_title = Some(title);
            }
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

        if let Some(id) = options_tile_id {
            obj.insert("tile_id".to_string(), serde_json::Value::String(id));
        }
        if let Some(title) = options_tile_title_explicit.or(options_tile_title) {
            obj.insert("tile_title".to_string(), serde_json::Value::String(title));
        }
        if let Some(val) = options_tile_value {
            obj.insert("tile_value".to_string(), serde_json::Value::String(val));
        }
        if let Some(sub) = options_tile_subtitle {
            obj.insert("tile_subtitle".to_string(), serde_json::Value::String(sub));
        }
        if let Some(tone) = options_tile_tone {
            obj.insert("tile_tone".to_string(), serde_json::Value::String(tone));
        }
        if let Some(prog) = options_tile_progress {
            obj.insert("tile_progress".to_string(), serde_json::json!(prog));
        }
        if let Some(ttl) = options_tile_ttl {
            obj.insert("tile_ttl_secs".to_string(), serde_json::json!(ttl));
        }
        if let Some(pin) = options_tile_pin_mini {
            obj.insert("tile_pin_mini".to_string(), serde_json::Value::Bool(pin));
        }
    }

    Some(payload)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cli_args = if args.len() > 1 { &args[1..] } else { &[] };
    let is_stdin_piped = !stdin().is_terminal();
    let stdin_payload: serde_json::Value = if is_stdin_piped {
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

    let payload = match build_payload_from_args(cli_args, stdin_payload) {
        Some(p) => p,
        None => {
            print_help();
            return;
        }
    };

    // Jika berjalan interaktif di terminal tanpa stdin dan tanpa hook_event_name, tampilkan bantuan
    if !is_stdin_piped && payload.get("hook_event_name").is_none() {
        print_help();
        return;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_flags_return_none() {
        assert!(build_payload_from_args(&["--help".into()], serde_json::json!({})).is_none());
        assert!(build_payload_from_args(&["-h".into()], serde_json::json!({})).is_none());
        assert!(build_payload_from_args(&["/?".into()], serde_json::json!({})).is_none());
        assert!(build_payload_from_args(&["noktah-hook.exe".into(), "--help".into()], serde_json::json!({})).is_none());
    }

    #[test]
    fn positional_tile_args() {
        let args = vec![
            "tile".into(),
            "cpu".into(),
            "85%".into(),
            "High load".into(),
        ];
        let p = build_payload_from_args(&args, serde_json::json!({})).expect("should build payload");
        assert_eq!(p["hook_event_name"], "tile");
        assert_eq!(p["tile_id"], "cpu");
        assert_eq!(p["tile_title"], "cpu");
        assert_eq!(p["tile_value"], "85%");
        assert_eq!(p["tile_subtitle"], "High load");
    }

    #[test]
    fn tile_flags_parsing_and_progress_normalization() {
        let args = vec![
            "--status".into(),
            "tile-set".into(),
            "--id".into(),
            "build".into(),
            "-v".into(),
            "Passed".into(),
            "--tone".into(),
            "success".into(),
            "--progress".into(),
            "85.0".into(),
            "--ttl".into(),
            "120".into(),
            "--pin-mini".into(),
        ];
        let p = build_payload_from_args(&args, serde_json::json!({})).expect("should build payload");
        assert_eq!(p["hook_event_name"], "tile-set");
        assert_eq!(p["tile_id"], "build");
        assert_eq!(p["tile_value"], "Passed");
        assert_eq!(p["tile_tone"], "success");
        assert!((p["tile_progress"].as_f64().unwrap() - 0.85).abs() < 1e-4);
        assert_eq!(p["tile_ttl_secs"], 120);
        assert_eq!(p["tile_pin_mini"], true);
    }

    #[test]
    fn tile_remove_and_clear() {
        let remove_args = vec!["tile-remove".into(), "cpu".into()];
        let p1 = build_payload_from_args(&remove_args, serde_json::json!({})).unwrap();
        assert_eq!(p1["hook_event_name"], "tile-remove");
        assert_eq!(p1["tile_id"], "cpu");

        let clear_args = vec!["tile-clear".into()];
        let p2 = build_payload_from_args(&clear_args, serde_json::json!({})).unwrap();
        assert_eq!(p2["hook_event_name"], "tile-clear");
    }

    #[test]
    fn merge_with_stdin_json() {
        let stdin = serde_json::json!({
            "cwd": "C:\\projects\\demo",
            "tile": {
                "id": "inner",
                "value": "initial"
            }
        });
        let args = vec!["tile".into(), "--id".into(), "override_id".into()];
        let p = build_payload_from_args(&args, stdin).unwrap();
        assert_eq!(p["tile_id"], "override_id");
        assert_eq!(p["tile"]["id"], "inner");
        assert_eq!(p["session_title"], "demo");
    }

    #[test]
    fn title_override_with_positional_tile() {
        let args = vec![
            "tile".into(),
            "cpu".into(),
            "50%".into(),
            "--title".into(),
            "CPU Load".into(),
        ];
        let p = build_payload_from_args(&args, serde_json::json!({})).unwrap();
        assert_eq!(p["hook_event_name"], "tile");
        assert_eq!(p["tile_id"], "cpu");
        assert_eq!(p["tile_value"], "50%");
        assert_eq!(p["tile_title"], "CPU Load");
        assert_eq!(p["session_title"], "CPU Load");
    }
}
