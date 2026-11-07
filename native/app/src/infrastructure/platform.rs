//! Bagian yang menyentuh Windows: posisi jendela, pemantauan kursor, dan
//! klik-tembus.
//!
//! Jendela transparan yang tidak aktif **tidak** mengirim event mouse, jadi
//! `egui` tidak akan pernah melihat `hover_pos`. Semua interaksi hover di sini
//! dibaca langsung dari `GetCursorPos` di thread terpisah — pendekatan yang
//! sama dipakai versi Tauri untuk masalah yang sama.

use crate::presentation::layout;
use egui::Context;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

#[cfg(windows)]
use windows::Win32::Foundation::{HWND, POINT, RECT};
#[cfg(windows)]
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
#[cfg(windows)]
use windows::Win32::UI::HiDpi::{GetDpiForSystem, GetDpiForWindow};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetCursorPos, GetSystemMetrics, GetWindowLongPtrW, GetWindowRect,
    SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST, SM_CXSCREEN, SWP_NOACTIVATE,
    SWP_NOSIZE, WS_EX_NOACTIVATE, WS_EX_TRANSPARENT,
};

pub struct Hover {
    pub pos: Mutex<(f32, f32)>,
    pub expanded: AtomicBool,
    pub minimal: AtomicBool,
    /// true = jendela sedang menembus klik (di luar bentuk notch).
    pub through: AtomicBool,
    /// Lebar baris minimized saat ini, disimpan sebagai bit f32.
    pub pill_w: AtomicU32,
    /// Telinga cair aktif — ikut menentukan luas region jendela.
    pub ears: AtomicBool,
    /// Rect notch yang sedang dianimasikan (kiri, atas, kanan, bawah).
    pub cur: Mutex<(f32, f32, f32, f32)>,
    /// Radius sudut saat ini.
    pub cur_r: AtomicU32,
    /// true = bentuk sedang bergerak. Dipakai pemantau untuk mempercepat
    /// pembacaan kursor, dan oleh `shape` untuk memilih margin region.
    pub animating: AtomicBool,
    /// Region jendela dipasang dari thread UI (bukan dari pemantau): nilainya
    /// harus identik dengan frame yang barusan digambar.
    #[cfg(windows)]
    pub hwnd: Mutex<Option<isize>>,
    /// Region terakhir yang berhasil dipasang, supaya `SetWindowRgn` tidak
    /// dipanggil setiap frame saat bentuknya diam.
    #[cfg(windows)]
    pub last_region: Mutex<Option<(i32, i32, i32, i32)>>,
    /// DPI jendela × 100 (mis. 150 untuk 150%), dibaca pemantau tiap tick.
    pub scale: AtomicU32,
    /// Seberapa "minimal" bentuknya sekarang, 0..1 sebagai bit f32.
    /// Ditulis UI setiap frame; `shape()` memakainya untuk hit-test saat
    /// `cur` belum terisi.
    pub minimal_mix: AtomicU32,
    /// Target dimensi terbuka (bisa OPEN_W/OPEN_H atau ISLAND_W/ISLAND_H).
    pub open_w: AtomicU32,
    pub open_h: AtomicU32,
    /// Derajat bukaan menu dock sebagai bit f32, dipakai fallback `shape()`.
    pub dock_open: AtomicU32,
}

impl Hover {
    pub fn new(minimal: bool) -> Arc<Self> {
        Arc::new(Self {
            pos: Mutex::new((-1000.0, -1000.0)),
            expanded: AtomicBool::new(false),
            minimal: AtomicBool::new(minimal),
            through: AtomicBool::new(true),
            pill_w: AtomicU32::new(193f32.to_bits()),
            ears: AtomicBool::new(false),
            cur: Mutex::new((0.0, 0.0, 0.0, 0.0)),
            cur_r: AtomicU32::new(24f32.to_bits()),
            animating: AtomicBool::new(false),
            #[cfg(windows)]
            hwnd: Mutex::new(None),
            #[cfg(windows)]
            last_region: Mutex::new(None),
            scale: AtomicU32::new(100),
            minimal_mix: AtomicU32::new(0.0f32.to_bits()),
            open_w: AtomicU32::new(layout::OPEN_W.to_bits()),
            open_h: AtomicU32::new(layout::OPEN_H.to_bits()),
            dock_open: AtomicU32::new(0.0f32.to_bits()),
        })
    }

    pub fn get(&self) -> (f32, f32) {
        // Tanpa `unwrap()`: kalau thread lain panik saat memegang lock, panik
        // di sini akan ikut mematikan pemanggilnya — dan notch membeku.
        self.pos.lock().map(|g| *g).unwrap_or((-1000.0, -1000.0))
    }

    /// Bentuk notch yang sedang dianimasikan, dalam koordinat lokal jendela.
    pub fn shape(&self) -> egui::Rect {
        let (x0, y0, x1, y1) = self.cur.lock().map(|g| *g).unwrap_or_default();
        if x1 > x0 {
            egui::Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(x1, y1))
        } else {
            let pill_w = f32::from_bits(self.pill_w.load(Ordering::Relaxed));
            let mix = f32::from_bits(self.minimal_mix.load(Ordering::Relaxed));
            let ow = f32::from_bits(self.open_w.load(Ordering::Relaxed));
            let ow = if ow > 0.0 { ow } else { layout::OPEN_W };
            let oh = f32::from_bits(self.open_h.load(Ordering::Relaxed));
            let oh = if oh > 0.0 { oh } else { layout::OPEN_H };
            let base =
                layout::hit_shape(self.expanded.load(Ordering::Relaxed), mix, pill_w, ow, oh);
            let dock_open = f32::from_bits(self.dock_open.load(Ordering::Relaxed));
            if self.expanded.load(Ordering::Relaxed) {
                layout::outer_shell_rect(base, dock_open)
            } else {
                base
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Hover;
    use crate::presentation::layout;
    use std::sync::atomic::Ordering;

    #[test]
    fn fallback_shape_memakai_tinggi_shell_luar_sesuai_dock() {
        let hover = Hover::new(false);
        hover.expanded.store(true, Ordering::Relaxed);

        hover.dock_open.store(0.0f32.to_bits(), Ordering::Relaxed);
        assert_eq!(hover.shape().height(), layout::OPEN_H);

        hover.dock_open.store(1.0f32.to_bits(), Ordering::Relaxed);
        assert_eq!(hover.shape().height(), layout::OUTER_OPEN_H);
    }
}

/// Tebakan awal posisi; angka pastinya ditentukan `center_window`.
#[cfg(windows)]
pub fn initial_position() -> f32 {
    unsafe {
        let scale = GetDpiForSystem() as f32 / 96.0;
        let screen_w = GetSystemMetrics(SM_CXSCREEN) as f32;
        ((screen_w / scale) - layout::WINDOW_W) / 2.0
    }
}

#[cfg(not(windows))]
pub fn initial_position() -> f32 {
    0.0
}

#[cfg(windows)]
fn center_window(hwnd: HWND) {
    unsafe {
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return;
        }

        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return;
        }

        let work = info.rcWork;
        let width = rect.right - rect.left;
        let x = work.left + ((work.right - work.left) - width) / 2;
        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            x,
            work.top,
            0,
            0,
            SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

/// Memotong jendela menjadi bentuk notch.
///
/// `WS_EX_TRANSPARENT` saja tidak cukup untuk jendela ber-alpha per piksel —
/// klik tetap tertelan di area transparan. Region membuat Windows sendiri yang
/// menolak klik di luar bentuk, tanpa bergantung pada gaya jendela.
///
/// Dipanggil dari THREAD UI setiap frame (lihat `Hover::shape`), bukan dari
/// thread pemantau. Dulu region disegarkan lewat polling 40 ms, dan selama
/// morph panel tumbuh ~13 px per frame — tepinya terpotong garis lurus
/// beberapa frame di awal expand.
#[cfg(windows)]
fn set_region(hwnd: HWND, rect: egui::Rect, scale: f32, margin: f32) -> bool {
    use windows::Win32::Graphics::Gdi::{CreateRectRgn, SetWindowRgn};

    let x1 = ((rect.left() - margin) * scale).round() as i32;
    let y1 = ((rect.top() - margin) * scale).round() as i32;
    let x2 = ((rect.right() + margin) * scale).round() as i32;
    let y2 = ((rect.bottom() + margin) * scale).round() as i32;
    // Sengaja PERSEGI, bukan membulat. Region Windows tidak punya antialias
    // dan bentuknya lingkaran, sedangkan panel digambar sebagai squircle —
    // sudut squircle keluar dari region sehingga terpotong bergerigi.

    unsafe {
        let rgn = CreateRectRgn(x1, y1, x2, y2);
        if rgn.is_invalid() {
            return false;
        }
        // SetWindowRgn mengambil alih kepemilikan region; jangan dihapus.
        SetWindowRgn(hwnd, rgn, true) != 0
    }
}

/// Memasang region jendela agar identik dengan bentuk yang baru digambar.
///
/// Dipanggil dari thread UI tepat setelah spring di-step. Satu-satunya tugas
/// fungsi ini adalah menerjemahkan `shape` + margin ke `SetWindowRgn`, jadi
/// tidak ada lagi jeda antara bentuk di layar dan bentuk yang menerima klik.
///
/// Margin hanya perlu menutupi apa yang DIGAMBAR di luar rect: bayangan dan
/// telinga cair. Tanpa telinga, tidak ada piksel di luar rect, jadi margin
/// kecil sudah cukup — margin besar hanya memperluas area yang bisa menelan
/// klik aplikasi lain.
#[cfg(windows)]
pub fn sync_region(hover: &Hover, shape: egui::Rect, scale: f32) {
    let hwnd = match hover.hwnd.lock() {
        Ok(guard) => match *guard {
            Some(h) => HWND(h as *mut core::ffi::c_void),
            None => return,
        },
        Err(_) => return,
    };

    let telinga = hover.ears.load(Ordering::Relaxed);
    let margin = if telinga { layout::EAR + 8.0 } else { 8.0 };

    // Ukuran region jarang berubah; hitung kuncinya dulu supaya SetWindowRgn
    // tidak dipanggil 60 kali per detik tanpa alasan.
    let kunci = (
        ((shape.left() - margin) * scale).round() as i32,
        ((shape.top() - margin) * scale).round() as i32,
        ((shape.right() + margin) * scale).round() as i32,
        ((shape.bottom() + margin) * scale).round() as i32,
    );
    let sudah_sama = hover
        .last_region
        .lock()
        .map(|g| *g == Some(kunci))
        .unwrap_or(false);
    if sudah_sama {
        return;
    }

    if set_region(hwnd, shape, scale, margin) {
        if let Ok(mut slot) = hover.last_region.lock() {
            *slot = Some(kunci);
        }
    } else if let Ok(mut slot) = hover.last_region.lock() {
        // Gagal: kosongkan supaya percobaan berikutnya memasang dari nol.
        *slot = None;
    }
}

#[cfg(not(windows))]
pub fn sync_region(_hover: &Hover, _shape: egui::Rect, _scale: f32) {}

/// Menyalakan/mematikan klik-tembus untuk bentuk yang baru digambar.
///
/// Dulu ini dikerjakan thread pemantau tiap 40 ms, dan itu punya dua cacat
/// yang keduanya terasa oleh pengguna:
///   - kursor baru masuk notch, gaya masih "tembus" → klik pertama hilang ke
///     aplikasi di bawah;
///   - kursor baru keluar, gaya masih "tangkap" → klik di sekitar notch
///     ditelan Noktah tanpa efek apa pun.
///
/// Sekarang dipanggil dari thread UI setiap frame, jadi jaraknya paling jauh
/// satu frame (~6 ms pada 165 fps) — di bawah ambang yang bisa disadari
/// manusia, dan tanpa perlu mengganti window procedure milik winit.
#[cfg(windows)]
pub fn set_click_through(hover: &Hover, through: bool) {
    if hover.through.swap(through, Ordering::Relaxed) == through {
        return;
    }
    let hwnd = match hover.hwnd.lock() {
        Ok(guard) => match *guard {
            Some(h) => HWND(h as *mut core::ffi::c_void),
            None => return,
        },
        Err(_) => return,
    };
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let flag = WS_EX_TRANSPARENT.0 as isize;
        let noact = WS_EX_NOACTIVATE.0 as isize;
        let want = if through { style | flag } else { style & !flag } | noact;
        if want != style {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, want);
        }
    }
}

#[cfg(not(windows))]
pub fn set_click_through(_hover: &Hover, _through: bool) {}

/// Menjalankan pemantau kursor. Mengembalikan handle yang dipakai UI.
pub fn spawn_watcher(ctx: Context, minimal: bool) -> Arc<Hover> {
    let hover = Hover::new(minimal);

    #[cfg(windows)]
    {
        let hover_thread = hover.clone();
        let _ = std::thread::Builder::new()
            .name("noktah-watcher".into())
            .stack_size(256 * 1024)
            .spawn(move || {
                let mut last_near = false;
                let mut hwnd: Option<HWND> = None;
            let mut last_width: Option<i32> = None;

            loop {
                // Saat morph berjalan, region jendela harus ikut bergerak:
                // kalau tertinggal, tepi panel yang sedang membesar terpotong
                // garis lurus. 16 ms adalah granularitas timer Windows yang
                // bisa diandalkan; 4 ms hanya akan dibulatkan ke ~15 ms.
                let jeda = if hover_thread.animating.load(Ordering::Relaxed) {
                    std::time::Duration::from_millis(16)
                } else {
                    std::time::Duration::from_millis(40)
                };
                std::thread::sleep(jeda);

                if hwnd.is_none() {
                    match unsafe { FindWindowW(None, windows::core::w!("Noktah")) } {
                        Ok(h) => {
                            // Pastikan ini jendela utama, bukan jendela lain
                            // yang kebetulan berjudul sama: ukurannya harus
                            // selebar jendela notch.
                            let mut cek = RECT::default();
                            if unsafe { GetWindowRect(h, &mut cek) }.is_err()
                                || (cek.right - cek.left) < 300
                            {
                                continue;
                            }
                            hwnd = Some(h);
                            if let Ok(mut slot) = hover_thread.hwnd.lock() {
                                *slot = Some(h.0 as isize);
                            }
                            unsafe {
                                let style = GetWindowLongPtrW(h, GWL_EXSTYLE);
                                SetWindowLongPtrW(
                                    h,
                                    GWL_EXSTYLE,
                                    style | WS_EX_NOACTIVATE.0 as isize,
                                );
                            }
                            center_window(h);
                            last_width = Some(cek.right - cek.left);
                        }
                        Err(_) => continue,
                    }
                }
                let Some(h) = hwnd else { continue };

                let mut point = POINT::default();
                if unsafe { GetCursorPos(&mut point) }.is_err() {
                    continue;
                }

                // Koordinat lokal dari rect sebenarnya, bukan dari posisi yang
                // kita minta — supaya tidak meleset saat DPI berubah.
                let mut rect = RECT::default();
                if unsafe { GetWindowRect(h, &mut rect) }.is_err() {
                    // HWND sudah mati (jendela dibuat ulang, mis. DPI change).
                    // Tanpa ini seluruh sistem region membeku di nilai terakhir.
                    hwnd = None;
                    if let Ok(mut slot) = hover_thread.hwnd.lock() {
                        *slot = None;
                    }
                    continue;
                }

                // Saat start, ukuran jendela belum final ketika pertama kali
                // dipusatkan. Pusatkan ulang setiap kali lebarnya berubah.
                let lebar = rect.right - rect.left;
                if last_width != Some(lebar) {
                    last_width = Some(lebar);
                    center_window(h);
                    if unsafe { GetWindowRect(h, &mut rect) }.is_err() {
                        continue;
                    }
                }

                let dpi = unsafe { GetDpiForWindow(h) };
                hover_thread.scale.store(dpi, Ordering::Relaxed);
                let scale = dpi as f32 / 96.0;
                let scale = if scale <= 0.0 { 1.0 } else { scale };

                let local_x = (point.x - rect.left) as f32 / scale;
                let local_y = (point.y - rect.top) as f32 / scale;

                if let Ok(mut guard) = hover_thread.pos.lock() {
                    *guard = (local_x, local_y);
                }

                let cursor = egui::pos2(local_x, local_y);
                let expanded = hover_thread.expanded.load(Ordering::Relaxed);
                let minimal_now = hover_thread.minimal.load(Ordering::Relaxed);
                let pill_w = f32::from_bits(hover_thread.pill_w.load(Ordering::Relaxed));

                // Bentuk yang dipakai untuk uji "dekat" adalah bentuk yang
                // SEDANG dianimasikan, bukan bentuk tujuan. Kalau memakai
                // bentuk tujuan, panel terpotong di tengah animasi.
                let shape = hover_thread.shape();
                let _ = (&expanded, minimal_now, pill_w);
                // Region jendela DAN gaya klik-tembus tidak lagi diurus di
                // sini: keduanya dipasang thread UI lewat `sync_region` dan
                // `set_click_through`, tepat setelah spring di-step.

                // Repaint hanya kalau kursor dekat notch atau status berubah.
                let zone = if expanded { shape } else { shape.expand(24.0) };
                let near = zone.contains(cursor);
                if near || near != last_near {
                    ctx.request_repaint();
                }
                last_near = near;
            }
        });
    }

    hover
}

/// Posisi dan ukuran jendela notch dalam piksel layar.
#[cfg(windows)]
pub fn window_rect() -> Option<(i32, i32, i32, i32)> {
    unsafe {
        let h = FindWindowW(None, windows::core::w!("Noktah")).ok()?;
        let mut r = RECT::default();
        GetWindowRect(h, &mut r).ok()?;
        Some((r.left, r.top, r.right - r.left, r.bottom - r.top))
    }
}

#[cfg(not(windows))]
pub fn window_rect() -> Option<(i32, i32, i32, i32)> {
    None
}
