//! Noktah native — titik masuk.
//!
//! Jendela transparan tanpa dekorasi, selalu di atas, selebar panel. Yang
//! terlihat hanya bentuk notch yang digambar sendiri.
//!
//! # Arsitektur
//!
//! Empat lapisan, dari dalam ke luar. Panah ketergantungan hanya boleh
//! menunjuk ke dalam — tidak pernah sebaliknya.
//!
//! ```text
//!   presentation/   cara menggambar (egui, warna, token)
//!        ↓
//!   application/    use case, port (trait), keadaan aplikasi
//!        ↓
//!   infrastructure/ implementasi nyata tiap port (disk, Win32, HTTP)
//!        ↓
//!   noktah_core     aturan murni, tanpa I/O, tanpa egui
//! ```
//!
//! `infrastructure` bergantung pada trait yang dideklarasikan `application`,
//! bukan sebaliknya — itu yang membuat use case bisa diuji dengan fake.

// Tanpa ini Windows membuka jendela konsol hitam di samping notch.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod application;
mod infrastructure;
mod presentation;

// Modul yang belum dipindah ke lapisannya. Setiap penghapusan di daftar ini
// adalah satu langkah refactor yang selesai.
mod app;
mod tabs;

#[cfg(test)]
mod layout_test;

#[cfg(test)]
mod page_fit_test;

// Re-export supaya path lama (`crate::presentation::style`, `crate::infrastructure::store`, ...) tetap
// berfungsi selama migrasi bertahap. Setelah semua pemanggil memakai path
// berlapis, blok ini dihapus.
pub(crate) use application as app_layer;
pub(crate) use infrastructure as infra;
pub(crate) use presentation as ui;

use eframe::egui;
use eframe::glow::HasContext;

fn load_app_icon() -> Option<egui::IconData> {
    let png_bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(png_bytes).ok()?.to_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    // Satu instance saja: notch kedua akan menimpa yang pertama.
    if !infrastructure::os::claim_single_instance() {
        return Ok(());
    }

    let x = infrastructure::platform::initial_position();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([
            presentation::layout::WINDOW_W,
            presentation::layout::WINDOW_H,
        ])
        .with_position([x, 0.0])
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top()
        .with_resizable(false)
        .with_taskbar(false);

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Noktah",
        options,
        Box::new(|cc| {
            let renderer = cc
                .gl
                .as_ref()
                .map(|gl| unsafe { gl.get_parameter_string(eframe::glow::RENDERER) })
                .unwrap_or_else(|| "tanpa GL".to_string());

            let hover = infrastructure::platform::spawn_watcher(cc.egui_ctx.clone(), false);
            infrastructure::os::spawn_tray_and_hotkey(cc.egui_ctx.clone());

            Ok(Box::new(app::Noktah::new(renderer, hover, &cc.egui_ctx)))
        }),
    )
}
