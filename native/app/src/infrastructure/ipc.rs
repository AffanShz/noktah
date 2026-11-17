//! Server IPC Windows Named Pipe untuk Noktah.
//! Menerima event lifecycle dari Claude Code via `noktah-hook.exe` dan
//! menyediakan kanal approval dua arah untuk PermissionRequest.

use eframe::egui;
use noktah_core::model::{AgentHookPayload, PermissionDecision};
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};
use std::sync::Arc;
use std::time::{Duration, Instant};
use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, LocalFree, ERROR_IO_PENDING, ERROR_PIPE_CONNECTED,
    HANDLE, HLOCAL, INVALID_HANDLE_VALUE,
};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::{GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER};
use windows::Win32::Storage::FileSystem::{
    ReadFile as ReadFileOverlapped, WriteFile as WriteFileOverlapped,
    FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAG_OVERLAPPED, PIPE_ACCESS_DUPLEX,
};
use windows::Win32::System::IO::{
    CancelIoEx, GetOverlappedResult, OVERLAPPED,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PeekNamedPipe,
    PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};
use windows::Win32::System::Threading::{
    CreateEventW, GetCurrentProcess, OpenProcessToken, SetEvent, WaitForMultipleObjects,
    WaitForSingleObject,
};

const PIPE_BUF_SIZE: u32 = 65536;

#[derive(Clone, Copy)]
struct SendHandle(usize);

/// Event yang dikirim dari IPC background thread ke UI thread Noktah.
pub enum IpcEvent {
    HookReceived(AgentHookPayload),
    PermissionRequest {
        request_id: String,
        tool_name: String,
        tool_input: serde_json::Value,
        suggestions: Vec<String>,
        session_title: Option<String>,
        responder: SyncSender<PermissionDecision>,
    },
    PermissionCanceled(String),
}

/// Handle yang dipegang oleh struct `Noktah` di UI thread.
pub struct IpcHandle {
    pub rx: Receiver<IpcEvent>,
    pub running: Arc<AtomicBool>,
    stop_event: SendHandle,
}

impl Drop for IpcHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        unsafe {
            let handle = HANDLE(self.stop_event.0 as *mut c_void);
            if !handle.is_invalid() {
                let _ = SetEvent(handle);
                let _ = CloseHandle(handle);
            }
        }
    }
}

/// Mendapatkan nama pipe unik per-user session Windows.
pub fn get_user_pipe_name() -> String {
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

/// Membaca buffer secara overlapped dengan dukungan pembatalan `stop_event`.
unsafe fn read_overlapped(pipe: HANDLE, buf: &mut [u8], stop_event: HANDLE) -> bool {
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
        let handles = [stop_event, event];
        let wait_res = WaitForMultipleObjects(&handles, false, 120_000);
        if wait_res.0 == 1 {
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

/// Menulis buffer secara overlapped dengan dukungan pembatalan `stop_event`.
unsafe fn write_overlapped(pipe: HANDLE, data: &[u8], stop_event: HANDLE) -> bool {
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
        written as usize == data.len()
    } else if GetLastError() == ERROR_IO_PENDING {
        let handles = [stop_event, event];
        let wait_res = WaitForMultipleObjects(&handles, false, 10_000);
        if wait_res.0 == 1 {
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

/// Menulis frame respon (4 byte panjang + payload).
fn send_response(pipe: HANDLE, data: &[u8], stop_event: HANDLE) -> bool {
    let len = (data.len() as u32).to_le_bytes();
    unsafe {
        if !write_overlapped(pipe, &len, stop_event) {
            return false;
        }
        if !data.is_empty() {
            return write_overlapped(pipe, data, stop_event);
        }
    }
    true
}

/// Menangani satu koneksi client di thread worker terpisah.
fn handle_client_connection(
    pipe_handle: SendHandle,
    tx: Sender<IpcEvent>,
    ctx: egui::Context,
    stop_event_handle: SendHandle,
) {
    let pipe = HANDLE(pipe_handle.0 as *mut c_void);
    let stop_event = HANDLE(stop_event_handle.0 as *mut c_void);
    let mut len_buf = [0u8; 4];
    if !unsafe { read_overlapped(pipe, &mut len_buf, stop_event) } {
        unsafe {
            let _ = DisconnectNamedPipe(pipe);
            let _ = CloseHandle(pipe);
        }
        return;
    }

    let payload_len = u32::from_le_bytes(len_buf) as usize;
    if payload_len == 0 || payload_len > 10 * 1024 * 1024 {
        unsafe {
            let _ = DisconnectNamedPipe(pipe);
            let _ = CloseHandle(pipe);
        }
        return;
    }

    let mut payload_buf = vec![0u8; payload_len];
    if !unsafe { read_overlapped(pipe, &mut payload_buf, stop_event) } {
        unsafe {
            let _ = DisconnectNamedPipe(pipe);
            let _ = CloseHandle(pipe);
        }
        return;
    }

    let hook_payload: AgentHookPayload = match serde_json::from_slice(&payload_buf) {
        Ok(p) => p,
        Err(_) => {
            unsafe {
                let _ = DisconnectNamedPipe(pipe);
                let _ = CloseHandle(pipe);
            }
            return;
        }
    };

    let ev_lower = hook_payload.hook_event_name.trim().to_lowercase();
    if ev_lower == "permissionrequest" || ev_lower == "permission" || ev_lower == "approval" || ev_lower == "ask" {
        let (resp_tx, resp_rx) = sync_channel::<PermissionDecision>(1);
        let request_id = hook_payload
            .tool_use_id
            .clone()
            .unwrap_or_else(|| format!("req_{}", chrono::Utc::now().timestamp_millis()));
        let tool_name = hook_payload.tool_name.clone().unwrap_or_default();
        let tool_input = hook_payload.tool_input.clone().unwrap_or_default();
        let suggestions = hook_payload
            .permission_suggestions
            .clone()
            .unwrap_or_default();
        let session_title = hook_payload
            .session_title
            .clone()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| hook_payload.session_id.clone().filter(|s| !s.trim().is_empty()));

        let _ = tx.send(IpcEvent::PermissionRequest {
            request_id: request_id.clone(),
            tool_name,
            tool_input,
            suggestions,
            session_title,
            responder: resp_tx,
        });
        ctx.request_repaint();

        // Loop tunggu respon dengan deteksi pipe broken (misal user tekan Ctrl+C di terminal)
        let start = Instant::now();
        let timeout = Duration::from_secs(115);
        let mut final_decision = None;

        while start.elapsed() < timeout {
            // 1. Apakah user sudah klik tombol di GUI?
            if let Ok(decision) = resp_rx.try_recv() {
                final_decision = Some(decision);
                break;
            }

            // 2. Apakah client masih hidup? (Deteksi Ctrl+C terminal)
            let mut bytes_avail = 0u32;
            let peek_ok = unsafe {
                PeekNamedPipe(pipe, None, 0, None, Some(&mut bytes_avail), None).is_ok()
            };
            if !peek_ok {
                // Client terputus mendadak! Batalkan modal di GUI
                let _ = tx.send(IpcEvent::PermissionCanceled(request_id.clone()));
                ctx.request_repaint();
                break;
            }

            // 3. Apakah Noktah sedang shutdown?
            if unsafe { WaitForSingleObject(stop_event, 0) }.0 == 0 {
                break;
            }

            std::thread::sleep(Duration::from_millis(25));
        }

        if let Some(decision) = final_decision {
            let verdict_obj = serde_json::json!({
                "hookSpecificOutput": {
                    "hookEventName": "PermissionRequest",
                    "decision": decision
                }
            });
            let verdict_bytes = serde_json::to_vec(&verdict_obj).unwrap_or_default();
            send_response(pipe, &verdict_bytes, stop_event);
        } else {
            // Timeout / dibatalkan / gagal: kirim respon kosong (fail-open)
            send_response(pipe, &[], stop_event);
        }
    } else {
        // Event notifikasi satu arah: kirim ke UI dan segera ACK 0-byte
        let _ = tx.send(IpcEvent::HookReceived(hook_payload));
        ctx.request_repaint();
        send_response(pipe, &[], stop_event);
    }

    unsafe {
        let _ = DisconnectNamedPipe(pipe);
        let _ = CloseHandle(pipe);
    }
}

/// Menjalankan server Named Pipe Noktah di background thread.
pub fn spawn(ctx: egui::Context) -> IpcHandle {
    let (tx_event, rx_event) = channel::<IpcEvent>();
    let running = Arc::new(AtomicBool::new(true));
    let pipe_name = get_user_pipe_name();

    let stop_event_handle = unsafe { CreateEventW(None, true, false, PCWSTR::null()).unwrap() };
    let send_stop = SendHandle(stop_event_handle.0 as usize);
    let send_stop_thread = send_stop;

    let running_thread = running.clone();
    let tx_thread = tx_event;

    let _ = std::thread::Builder::new()
        .name("noktah-ipc".into())
        .stack_size(512 * 1024)
        .spawn(move || {
            let pipe_w = HSTRING::from(&pipe_name);
            let mut is_first = true;
            let stop_event = HANDLE(send_stop_thread.0 as *mut c_void);

            while running_thread.load(Ordering::Relaxed) {
                let open_mode = if is_first {
                    PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE
                } else {
                    PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED
                };

                let pipe = unsafe {
                    CreateNamedPipeW(
                        PCWSTR(pipe_w.as_ptr()),
                        open_mode,
                        PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                        PIPE_UNLIMITED_INSTANCES,
                        PIPE_BUF_SIZE,
                        PIPE_BUF_SIZE,
                        0,
                        None,
                    )
                };

                if pipe.is_invalid() {
                    is_first = false;
                    std::thread::sleep(Duration::from_millis(50));
                    continue;
                }
                is_first = false;

                let mut connect_ol = OVERLAPPED::default();
                let connect_event = match unsafe { CreateEventW(None, true, false, PCWSTR::null()) } {
                    Ok(h) => h,
                    Err(_) => {
                        unsafe { let _ = CloseHandle(pipe); }
                        continue;
                    }
                };
                connect_ol.hEvent = connect_event;

                let connected = unsafe { ConnectNamedPipe(pipe, Some(&mut connect_ol)) };
                let last_err = unsafe { GetLastError() };

                let client_ready = if connected.is_ok() {
                    true
                } else if last_err == ERROR_PIPE_CONNECTED {
                    true
                } else if last_err == ERROR_IO_PENDING {
                    let wait_handles = [stop_event, connect_event];
                    let wait_res = unsafe { WaitForMultipleObjects(&wait_handles, false, 0xFFFFFFFF) };
                    if wait_res.0 == 0 {
                        // Sinyal shutdown
                        unsafe {
                            let _ = CancelIoEx(pipe, Some(&connect_ol));
                            let _ = CloseHandle(connect_event);
                            let _ = CloseHandle(pipe);
                        }
                        break;
                    }
                    let mut transferred = 0u32;
                    unsafe { GetOverlappedResult(pipe, &connect_ol, &mut transferred, false).is_ok() }
                } else {
                    false
                };

                unsafe { let _ = CloseHandle(connect_event); }

                if client_ready {
                    // Spawn thread worker terpisah untuk menangani client ini
                    // sehingga server listener tetap bebas menerima client lain (misal subagents)
                    let client_tx = tx_thread.clone();
                    let client_ctx = ctx.clone();
                    let send_pipe = SendHandle(pipe.0 as usize);
                    let send_stop_client = send_stop_thread;
                    let _ = std::thread::Builder::new()
                        .name("noktah-ipc-client".into())
                        .stack_size(256 * 1024)
                        .spawn(move || {
                            handle_client_connection(send_pipe, client_tx, client_ctx, send_stop_client);
                        });
                } else {
                    unsafe {
                        let _ = DisconnectNamedPipe(pipe);
                        let _ = CloseHandle(pipe);
                    }
                }
            }
        });

    IpcHandle {
        rx: rx_event,
        running,
        stop_event: send_stop,
    }
}
