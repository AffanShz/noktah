//! Integrasi Windows: tray icon, hotkey global, dan mulai bersama Windows.
//!
//! Tray dan hotkey butuh jendela tersembunyi dengan message loop sendiri,
//! karena `egui` tidak menyediakan tempat menangkap `WM_HOTKEY`.

/// Menyalakan/mematikan entri autostart di registry pengguna.
#[cfg(windows)]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
        KEY_SET_VALUE, REG_SZ,
    };

    let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0"
        .encode_utf16()
        .collect();
    let approved_subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run\0"
        .encode_utf16()
        .collect();
    let nama: Vec<u16> = "Noktah\0".encode_utf16().collect();

    unsafe {
        let mut key = HKEY::default();
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            0,
            KEY_SET_VALUE,
            &mut key,
        )
        .ok()
        .map_err(|e| e.to_string())?;

        let hasil = if enabled {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let exe_str = exe.to_string_lossy();
            let clean_path = exe_str.strip_prefix(r"\\?\").unwrap_or(&exe_str);
            let value: Vec<u16> = format!("\"{}\"", clean_path)
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let bytes = std::slice::from_raw_parts(
                value.as_ptr() as *const u8,
                value.len() * std::mem::size_of::<u16>(),
            );
            let set_res = RegSetValueExW(key, PCWSTR(nama.as_ptr()), 0, REG_SZ, Some(bytes))
                .ok()
                .map_err(|e| e.to_string());

            // Hapus pencekalan disabled di StartupApproved\Run jika user pernah mematikan via Task Manager
            let mut app_key = HKEY::default();
            if RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(approved_subkey.as_ptr()),
                0,
                KEY_SET_VALUE,
                &mut app_key,
            )
            .is_ok()
            {
                let _ = RegDeleteValueW(app_key, PCWSTR(nama.as_ptr()));
                let _ = RegCloseKey(app_key);
            }

            set_res
        } else {
            let del_res = RegDeleteValueW(key, PCWSTR(nama.as_ptr()));

            let mut app_key = HKEY::default();
            if RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(approved_subkey.as_ptr()),
                0,
                KEY_SET_VALUE,
                &mut app_key,
            )
            .is_ok()
            {
                let _ = RegDeleteValueW(app_key, PCWSTR(nama.as_ptr()));
                let _ = RegCloseKey(app_key);
            }

            let _ = del_res;
            Ok(())
        };

        let _ = RegCloseKey(key);
        hasil
    }
}

#[cfg(not(windows))]
pub fn set_autostart(_enabled: bool) -> Result<(), String> {
    Err("hanya didukung di Windows".into())
}

#[cfg(windows)]
pub fn autostart_enabled() -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ, REG_BINARY,
    };

    let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0"
        .encode_utf16()
        .collect();
    let approved_subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run\0"
        .encode_utf16()
        .collect();
    let nama: Vec<u16> = "Noktah\0".encode_utf16().collect();

    unsafe {
        let mut key = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            0,
            KEY_READ,
            &mut key,
        )
        .is_err()
        {
            return false;
        }

        let mut ukuran = 0u32;
        let hasil = RegQueryValueExW(
            key,
            PCWSTR(nama.as_ptr()),
            None,
            None,
            None,
            Some(&mut ukuran),
        );
        let _ = RegCloseKey(key);

        if hasil.is_err() || ukuran == 0 {
            return false;
        }

        // Periksa apakah status autostart dinonaktifkan oleh Task Manager / Settings Windows
        let mut app_key = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(approved_subkey.as_ptr()),
            0,
            KEY_READ,
            &mut app_key,
        )
        .is_ok()
        {
            let mut buf = [0u8; 12];
            let mut buf_size = buf.len() as u32;
            let mut val_type = REG_BINARY;
            if RegQueryValueExW(
                app_key,
                PCWSTR(nama.as_ptr()),
                None,
                Some(&mut val_type),
                Some(buf.as_mut_ptr()),
                Some(&mut buf_size),
            )
            .is_ok()
                && buf_size > 0
            {
                // Nilai byte ganjil (misal 0x01 atau 0x03) menandakan status Disabled di Windows Explorer
                if buf[0] & 1 != 0 {
                    let _ = RegCloseKey(app_key);
                    return false;
                }
            }
            let _ = RegCloseKey(app_key);
        }

        true
    }
}

/// Mendeteksi status daya: (sedang mengisi daya / colok charger, persentase baterai 0..100)
#[cfg(windows)]
pub fn power_status() -> (bool, u8) {
    #[repr(C)]
    struct SystemPowerStatus {
        ac_line_status: u8,
        battery_flag: u8,
        battery_life_percent: u8,
        system_status_flag: u8,
        battery_life_time: u32,
        battery_full_life_time: u32,
    }
    extern "system" {
        fn GetSystemPowerStatus(lp: *mut SystemPowerStatus) -> i32;
    }
    let mut status = SystemPowerStatus {
        ac_line_status: 255,
        battery_flag: 255,
        battery_life_percent: 255,
        system_status_flag: 0,
        battery_life_time: 0,
        battery_full_life_time: 0,
    };
    if unsafe { GetSystemPowerStatus(&mut status) } != 0 {
        let is_charging = status.ac_line_status == 1;
        let pct = if status.battery_life_percent <= 100 {
            status.battery_life_percent
        } else {
            100
        };
        (is_charging, pct)
    } else {
        (true, 100)
    }
}

#[cfg(not(windows))]
pub fn power_status() -> (bool, u8) {
    (true, 100)
}

/// Beralih mute audio Windows
#[cfg(windows)]
pub fn toggle_volume_mute() {
    unsafe {
        windows::Win32::UI::Input::KeyboardAndMouse::keybd_event(
            0xAD, // VK_VOLUME_MUTE
            0,
            windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
            0,
        );
        windows::Win32::UI::Input::KeyboardAndMouse::keybd_event(
            0xAD,
            0,
            windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_KEYUP,
            0,
        );
    }
}

#[cfg(not(windows))]
pub fn toggle_volume_mute() {}

#[cfg(not(windows))]
pub fn autostart_enabled() -> bool {
    false
}

/// Menempelkan jendela di atas semua aplikasi lain, atau melepasnya.
#[cfg(windows)]
pub fn set_always_on_top(on: bool) {
    use windows::Win32::UI::WindowsAndMessaging::{
        FindWindowW, SetWindowPos, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE,
        SWP_NOSIZE,
    };

    unsafe {
        if let Ok(h) = FindWindowW(None, windows::core::w!("Noktah")) {
            let urutan = if on { HWND_TOPMOST } else { HWND_NOTOPMOST };
            let _ = SetWindowPos(
                h,
                urutan,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }
}

#[cfg(not(windows))]
pub fn set_always_on_top(_on: bool) {}

/// Mengosongkan Working Set proses Windows saat menganggur agar halaman memori fisik
/// yang tidak terpakai (cache driver GPU, temporary compiler, dsb.) dikembalikan ke OS.
#[cfg(windows)]
pub fn trim_working_set() {
    use windows::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};
    unsafe {
        let _ = SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
    }
}

#[cfg(not(windows))]
pub fn trim_working_set() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_working_set() {
        use windows::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};
        let res = unsafe { SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX) };
        assert!(res.is_ok(), "SetProcessWorkingSetSize failed: {:?}", res);
    }
}



#[cfg(windows)]
mod tray {
    use std::sync::OnceLock;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, MOD_ALT};
    use windows::Win32::UI::Shell::{
        Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
        DispatchMessageW, FindWindowW, GetCursorPos, GetMessageW, GetSystemMetrics, LoadIconW,
        LoadImageW, PostQuitMessage, RegisterClassW, SetForegroundWindow, ShowWindow, TrackPopupMenu,
        TranslateMessage, HICON, IDI_APPLICATION, IMAGE_ICON, LR_DEFAULTCOLOR, LR_SHARED,
        MF_CHECKED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED, MSG, SM_CXSMICON, SM_CYSMICON,
        SW_HIDE, SW_SHOW, TPM_LEFTALIGN, TPM_RIGHTBUTTON, WINDOW_EX_STYLE, WNDCLASSW, WM_APP,
        WM_COMMAND, WM_DESTROY, WM_HOTKEY, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_RBUTTONUP,
        WS_OVERLAPPEDWINDOW,
    };

    const HOTKEY_ID: i32 = 1;
    const TRAY_ID: u32 = 1;
    const MENU_SHOW: usize = 1001;
    const MENU_HIDE: usize = 1002;
    const MENU_AUTOSTART: usize = 1003;
    const MENU_QUIT: usize = 1004;
    const WM_TRAY: u32 = WM_APP + 1;

    static CONTEXT: OnceLock<egui::Context> = OnceLock::new();

    fn show_main(show: bool) {
        if let Ok(h) = unsafe { FindWindowW(None, windows::core::w!("Noktah")) } {
            unsafe {
                let _ = ShowWindow(h, if show { SW_SHOW } else { SW_HIDE });
            }
        }
    }

    unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_HOTKEY => {
                show_main(true);
                if let Some(ctx) = CONTEXT.get() {
                    ctx.request_repaint();
                }
                LRESULT(0)
            }
            WM_TRAY => {
                let event = (lparam.0 as u32) & 0xffff;
                if event == WM_LBUTTONUP || event == WM_LBUTTONDBLCLK {
                    show_main(true);
                    if let Some(ctx) = CONTEXT.get() {
                        ctx.request_repaint();
                    }
                } else if event == WM_RBUTTONUP {
                    if let Ok(menu) = CreatePopupMenu() {
                        let auto_on = super::autostart_enabled();
                        let auto_flags = MF_STRING | if auto_on { MF_CHECKED } else { MF_UNCHECKED };

                        let _ = AppendMenuW(menu, MF_STRING, MENU_SHOW, windows::core::w!("Tampilkan"));
                        let _ = AppendMenuW(menu, MF_STRING, MENU_HIDE, windows::core::w!("Sembunyikan"));
                        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
                        let _ = AppendMenuW(menu, auto_flags, MENU_AUTOSTART, windows::core::w!("Mulai bersama Windows"));
                        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
                        let _ = AppendMenuW(menu, MF_STRING, MENU_QUIT, windows::core::w!("Keluar"));

                        let mut point = POINT::default();
                        let _ = GetCursorPos(&mut point);
                        let _ = SetForegroundWindow(hwnd);
                        let _ = TrackPopupMenu(
                            menu,
                            TPM_LEFTALIGN | TPM_RIGHTBUTTON,
                            point.x,
                            point.y,
                            0,
                            hwnd,
                            None,
                        );
                        let _ = DestroyMenu(menu);
                    }
                }
                LRESULT(0)
            }
            WM_COMMAND => {
                match wparam.0 & 0xffff {
                    MENU_SHOW => show_main(true),
                    MENU_HIDE => show_main(false),
                    MENU_AUTOSTART => {
                        let cur = super::autostart_enabled();
                        let _ = super::set_autostart(!cur);
                    }
                    MENU_QUIT => {
                        PostQuitMessage(0);
                        std::process::exit(0);
                    }
                    _ => {}
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    pub fn spawn(ctx: egui::Context) {
        let _ = CONTEXT.set(ctx);

        let _ = std::thread::Builder::new()
            .name("noktah-tray".into())
            .stack_size(256 * 1024)
            .spawn(move || unsafe {
                let nama_kelas: Vec<u16> = "NoktahTray\0".encode_utf16().collect();
            // Judul HARUS berbeda dari jendela utama. Kalau sama,
            // FindWindowW bisa mengembalikan jendela host ini, dan semua
            // pengaturan gaya (klik-tembus, posisi) salah sasaran.
            let judul: Vec<u16> = "NoktahTrayHost\0".encode_utf16().collect();

            let module = GetModuleHandleW(None).unwrap_or_default();
            let instance = HINSTANCE(module.0);

            let wc = WNDCLASSW {
                lpfnWndProc: Some(wndproc),
                hInstance: instance,
                lpszClassName: PCWSTR(nama_kelas.as_ptr()),
                ..Default::default()
            };
            RegisterClassW(&wc);

            let hwnd = match CreateWindowExW(
                WINDOW_EX_STYLE(0),
                PCWSTR(nama_kelas.as_ptr()),
                PCWSTR(judul.as_ptr()),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                0,
                0,
                None,
                None,
                instance,
                None,
            ) {
                Ok(h) => h,
                Err(_) => return,
            };

            let _ = RegisterHotKey(hwnd, HOTKEY_ID, MOD_ALT, 'N' as u32);

            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: TRAY_ID,
                uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
                uCallbackMessage: WM_TRAY,
                ..Default::default()
            };
            let cx = GetSystemMetrics(SM_CXSMICON);
            let cy = GetSystemMetrics(SM_CYSMICON);
            let h_icon = LoadImageW(
                instance,
                PCWSTR(1 as usize as *const u16),
                IMAGE_ICON,
                cx,
                cy,
                LR_DEFAULTCOLOR | LR_SHARED,
            )
            .map(|h| HICON(h.0))
            .or_else(|_| LoadIconW(instance, PCWSTR(1 as usize as *const u16)))
            .or_else(|_| LoadIconW(None, IDI_APPLICATION))
            .unwrap_or_default();
            nid.hIcon = h_icon;
            for (i, c) in "Noktah".encode_utf16().take(127).enumerate() {
                nid.szTip[i] = c;
            }
            let _ = Shell_NotifyIconW(NIM_ADD, &nid);

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
        });
    }
}

#[cfg(windows)]
pub fn spawn_tray_and_hotkey(ctx: egui::Context) {
    tray::spawn(ctx);
}

#[cfg(not(windows))]
pub fn spawn_tray_and_hotkey(_ctx: egui::Context) {}

/// Mengeklaim hak menjalankan satu instance saja.
///
/// Memakai named mutex: kalau sudah ada proses lain yang memegang nama itu,
/// fungsi ini mengembalikan false dan pemanggil langsung keluar — bukan
/// menampilkan notch kedua yang saling menimpa.
#[cfg(windows)]
pub fn claim_single_instance() -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, LPARAM, WPARAM,
    };
    use windows::Win32::System::Threading::CreateMutexW;
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, PostMessageW, WM_COMMAND};

    let nama: Vec<u16> = "Local\\NoktahSingleInstance\0".encode_utf16().collect();

    unsafe {
        match CreateMutexW(None, true, PCWSTR(nama.as_ptr())) {
            Ok(handle) => {
                if GetLastError() != ERROR_ALREADY_EXISTS {
                    // Handle sengaja tidak ditutup: harus hidup selama proses.
                    let _ = handle;
                    return true;
                }
                let _ = CloseHandle(handle);

                // Jika ada jendela tray Noktah di sesi desktop saat ini, minta keluar
                // dengan rapi supaya instance baru dapat langsung menggantikannya.
                if let Ok(tray_hwnd) = FindWindowW(
                    windows::core::w!("NoktahTray"),
                    windows::core::w!("NoktahTrayHost"),
                ) {
                    if !tray_hwnd.0.is_null() {
                        let _ = PostMessageW(tray_hwnd, WM_COMMAND, WPARAM(1004), LPARAM(0));
                        for _ in 0..15 {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            if let Ok(new_handle) = CreateMutexW(None, true, PCWSTR(nama.as_ptr()))
                            {
                                if GetLastError() != ERROR_ALREADY_EXISTS {
                                    let _ = new_handle;
                                    return true;
                                }
                                let _ = CloseHandle(new_handle);
                            }
                        }
                    }
                }

                crate::infrastructure::logger::warn(
                    "app",
                    "Another Noktah instance is already active; exiting",
                );
                false
            }
            Err(_) => true,
        }
    }
}

#[cfg(not(windows))]
pub fn claim_single_instance() -> bool {
    true
}

/// Apakah animasi antarmuka Windows menyala.
///
/// Kalau pengguna mematikan "Animation effects" di Windows, notch tidak boleh
/// tetap beranimasi — itu bisa memicu pusing dan melanggar setelan sistem.
#[cfg(windows)]
pub fn animations_enabled() -> bool {
    use windows::Win32::Foundation::BOOL;
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPI_GETCLIENTAREAANIMATION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    };

    let mut aktif = BOOL::from(true);
    unsafe {
        let _ = SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            Some(&mut aktif as *mut _ as *mut core::ffi::c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
    }
    aktif.as_bool()
}

#[cfg(not(windows))]
pub fn animations_enabled() -> bool {
    true
}

/// Membaca teks Unicode dari clipboard Windows secara aman dan terbatas.
#[cfg(windows)]
pub fn read_clipboard_text() -> Option<String> {
    use windows::Win32::Foundation::{HANDLE, HGLOBAL};
    use windows::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
    use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};

    const CF_UNICODETEXT_FMT: u32 = 13;
    const MAX_CHARS: usize = 1_000_000;

    unsafe {
        if OpenClipboard(None).is_err() {
            return None;
        }
        let result = (|| -> Option<String> {
            let handle: HANDLE = GetClipboardData(CF_UNICODETEXT_FMT).ok()?;
            if handle.is_invalid() || handle.0.is_null() {
                return None;
            }
            let hglobal = HGLOBAL(handle.0);
            let ptr = GlobalLock(hglobal) as *const u16;
            if ptr.is_null() {
                return None;
            }
            let mut len = 0usize;
            while len < MAX_CHARS && *ptr.add(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(ptr, len);
            let text = String::from_utf16_lossy(slice);
            let _ = GlobalUnlock(hglobal);
            if text.trim().is_empty() {
                None
            } else {
                Some(text)
            }
        })();
        let _ = CloseClipboard();
        result
    }
}

#[cfg(not(windows))]
pub fn read_clipboard_text() -> Option<String> {
    None
}
