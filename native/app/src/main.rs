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

#[cfg(test)]
mod layout_test;

#[cfg(test)]
mod page_fit_test;

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

fn setup_panic_logger() {
    std::panic::set_hook(Box::new(|info| {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "unknown panic payload"
        };
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("unnamed");

        let log_line = format!(
            "[{}] PANIC in thread '{}' at {}:\n  {}\n\n",
            timestamp, thread_name, location, payload
        );

        eprintln!("{}", log_line);

        let logs_dir = infrastructure::store::data_dir().join("logs");
        let _ = std::fs::create_dir_all(&logs_dir);
        let crash_file = logs_dir.join("crash.log");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(crash_file)
        {
            use std::io::Write;
            let _ = f.write_all(log_line.as_bytes());
        }

        infrastructure::logger::error(
            "panic",
            &format!("thread '{}' at {}: {}", thread_name, location, payload),
        );
    }));
}

fn main() -> eframe::Result<()> {
    // Pastikan working directory selalu berada di folder executable,
    // bukan di C:\Windows\System32 saat dijalankan via registry Run.
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let _ = std::env::set_current_dir(exe_dir);
        }
    }

    setup_panic_logger();

    // Satu instance saja: notch kedua akan menimpa yang pertama.
    if !infrastructure::os::claim_single_instance() {
        return Ok(());
    }

    infrastructure::logger::info(
        "app",
        &format!("Noktah v{} started", env!("CARGO_PKG_VERSION")),
    );

    let settings: noktah_core::settings::Settings = infrastructure::store::load("settings.json");
    let x = infrastructure::platform::initial_position();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([
            presentation::layout::WINDOW_W,
            presentation::layout::WINDOW_H,
        ])
        .with_position([x, 0.0])
        .with_decorations(false)
        .with_transparent(true)
        .with_resizable(false)
        .with_taskbar(false);

    if settings.always_on_top {
        viewport = viewport.with_always_on_top();
    }

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
