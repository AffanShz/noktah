//! Teks antarmuka dalam dua bahasa.
//!
//! Tabel, bukan `match` raksasa: menambah bahasa berarti menambah satu kolom,
//! dan tes memastikan tidak ada kunci yang kosong di bahasa mana pun.

use crate::settings::Lang;

/// (kunci, Indonesia, English)
pub const STRINGS: &[(&str, &str, &str)] = &[
    ("app.name", "Noktah", "Noktah"),
    ("tab.home", "Beranda", "Home"),
    ("tab.music", "Musik", "Music"),
    ("tab.tasks", "Tugas", "Tasks"),
    ("tab.focus", "Fokus", "Focus"),
    ("tab.system", "Sistem", "System"),
    ("tab.ai", "AI", "AI"),
    ("tab.ask", "Tanya", "Ask"),
    ("tab.agent", "Agent", "Agent"),
    ("tab.settings", "Setelan", "Settings"),
    ("common.close", "Tutup", "Close"),
    ("common.save", "Simpan", "Save"),
    ("common.cancel", "Batal", "Cancel"),
    ("common.delete", "Hapus", "Delete"),
    ("common.add", "Tambah", "Add"),
    ("common.loading", "Memuat…", "Loading…"),
    ("common.retry", "Coba lagi", "Retry"),
    ("common.refresh", "Segarkan", "Refresh"),
    ("common.empty", "Belum ada apa-apa", "Nothing yet"),
    ("common.today", "Hari ini", "Today"),
    ("common.tomorrow", "Besok", "Tomorrow"),
    ("common.not_yet", "Belum diporting", "Not ported yet"),
    ("common.reset", "Atur ulang", "Reset"),
    ("common.minutes_short", "mnt", "min"),
    ("home.greeting_morning", "Selamat pagi", "Good morning"),
    ("home.greeting_noon", "Selamat siang", "Good afternoon"),
    ("home.greeting_evening", "Selamat sore", "Good evening"),
    ("home.greeting_night", "Selamat malam", "Good evening"),
    ("home.next_task", "Tugas terdekat", "Next task"),
    ("home.no_task", "Tidak ada tugas", "No tasks"),
    ("home.start_focus", "Mulai fokus", "Start focus"),
    ("home.no_focus", "Belum ada sesi fokus", "No focus session"),
    ("home.weather.none", "Cuaca belum tersedia", "Weather unavailable"),
    ("home.weather.wind", "Angin", "Wind"),
    ("home.weather.humid", "Lembap", "Humidity"),
    ("home.weather.feels", "terasa", "feels"),
    ("home.now", "Kini", "Now"),
    ("home.forecast", "Prakiraan", "Forecast"),
    ("home.open_music", "Buka Musik untuk memutar lagu", "Open Music to play a song"),
    ("home.day_progress", "Hari ini", "Today"),
    //
    // Kuota Antigravity di Beranda. Kunci berakhiran `.err.` adalah **kunci
    // sebab**, bukan kalimat siap pakai: lapisan data (`quota.rs`) tidak tahu
    // bahasa antarmuka, jadi ia menyimpan salah satu kunci ini di
    // `QuotaSnapshot::error_key` dan halaman Home yang menerjemahkannya.
    ("home.quota.session", "5 jam", "5 hours"),
    ("home.quota.weekly", "minggu", "weekly"),
    ("home.quota.accounts", "akun", "accounts"),
    ("home.quota.stale", "lama", "stale"),
    ("home.quota.refresh_failed", "gagal", "failed"),
    ("home.quota.waiting", "9Router…", "9Router…"),
    ("home.quota.err.none", "9Router tidak ditemukan", "9Router not found"),
    (
        "home.quota.err.token",
        "token 9Router tidak terbaca",
        "9Router token unreadable",
    ),
    (
        "home.quota.err.no_account",
        "tidak ada akun aktif",
        "no active account",
    ),
    ("home.quota.err.db", "database 9Router rusak", "9Router database error"),
    (
        "home.quota.err.server",
        "9Router tidak merespons",
        "9Router not responding",
    ),
    (
        "home.quota.err.upstream",
        "kuota gagal diambil",
        "quota fetch failed",
    ),
    ("home.quota.err.empty", "kuota tidak terbaca", "quota unreadable"),
    ("music.now_playing", "Sedang diputar", "Now playing"),
    ("music.nothing", "Tidak ada yang diputar", "Nothing playing"),
    ("music.untitled", "Tanpa judul", "Untitled"),
    ("music.unknown_artist", "Artis tidak dikenal", "Unknown artist"),
    ("music.left", "sisa", "left"),
    ("music.lyrics", "Lirik", "Lyrics"),
    ("music.no_lyrics", "Lirik tidak ditemukan", "No lyrics found"),
    ("music.unsynced", "Lirik tanpa sinkronisasi", "Lyrics without sync"),
    ("music.searching", "Mencari lirik…", "Searching lyrics…"),
    ("tasks.title", "Tugas", "Tasks"),
    ("tasks.placeholder", "Apa yang perlu dikerjakan?", "What needs doing?"),
    ("tasks.due", "Pengingat", "Reminder"),
    ("tasks.overdue", "Terlewat", "Overdue"),
    ("tasks.done", "Selesai", "Done"),
    ("tasks.all", "Semua", "All"),
    ("tasks.active", "Aktif", "Active"),
    ("tasks.clear_done", "Hapus selesai", "Clear completed"),
    ("tasks.add_hint", "Tambah tugas, tekan Enter…", "Add task, press Enter…"),
    ("tasks.reminder_title", "Pengingat", "Reminder"),
    ("tasks.mode_tasks", "Tugas", "Tasks"),
    ("tasks.mode_notes", "Catatan", "Notes"),
    ("tasks.notes_hint", "Tulis catatan cepat di sini (tersimpan otomatis)…", "Write quick notes here (autosaved)…"),
    ("tasks.notes_copied", "Catatan disalin ke clipboard!", "Notes copied to clipboard!"),
    ("focus.title", "Fokus", "Focus"),
    ("focus.work", "Kerja", "Work"),
    ("focus.break", "Istirahat", "Break"),
    ("focus.start", "Mulai", "Start"),
    ("focus.pause", "Jeda", "Pause"),
    ("focus.resume", "Lanjut", "Resume"),
    ("focus.reset", "Ulang", "Reset"),
    ("focus.sessions", "Sesi hari ini", "Sessions today"),
    ("focus.work_long", "Fokus", "Focus"),
    ("focus.break_short", "Istirahat pendek", "Short break"),
    ("focus.break_long", "Istirahat panjang", "Long break"),
    ("focus.ready", "Siap", "Ready"),
    ("focus.running", "Berjalan", "Running"),
    ("focus.done_work", "Sesi fokus selesai! Waktunya istirahat.", "Focus session complete! Time for a break."),
    ("focus.done_break", "Waktu istirahat selesai! Siap kembali fokus?", "Break over! Ready to focus again?"),
    ("daily.reset_banner", "Hari baru! Progres harian direset.", "New day! Daily progress reset."),
    ("system.title", "Sistem", "System"),
    ("system.cpu", "CPU", "CPU"),
    ("system.ram", "RAM", "RAM"),
    ("system.uptime", "Aktif", "Uptime"),
    ("system.cores", "Inti", "Cores"),
    ("system.renderer", "Renderer", "Renderer"),
    ("system.view_metrics", "Sistem", "System"),
    ("system.view_tiles", "Tile", "Tiles"),
    ("system.tiles_clear", "Bersihkan", "Clear"),
    ("system.tiles_empty", "Belum ada tile kustom", "No custom tiles yet"),
    ("ai.title", "AI", "AI"),
    ("ai.placeholder", "Tanya apa saja…", "Ask anything…"),
    ("ai.send", "Kirim", "Send"),
    ("ai.stop", "Hentikan", "Stop"),
    ("ai.no_key", "Kunci API belum diatur", "API key not set"),
    ("ai.thinking", "Sedang berpikir…", "Thinking…"),
    ("ai.quick_translate", "ID↔EN", "ID↔EN"),
    ("ai.quick_summarize", "Ringkas", "Summarize"),
    ("ai.quick_explain", "Jelaskan", "Explain"),
    ("ai.quick_clear", "Bersihkan", "Clear"),
    ("ai.clipboard_empty", "Ketik teks atau salin teks ke clipboard lebih dulu", "Type text or copy text to clipboard first"),
    ("settings.title", "Setelan", "Settings"),
    ("settings.general", "Umum", "General"),
    ("settings.display", "Layar", "Display"),
    ("settings.music", "Musik & Lirik", "Music & Lyrics"),
    ("settings.todo", "Tugas", "To-do"),
    ("settings.pomodoro", "Pomodoro", "Pomodoro"),
    ("settings.language", "Bahasa", "Language"),
    ("settings.appearance", "Tampilan", "Appearance"),
    ("settings.accent", "Warna aksen", "Accent color"),
    ("settings.accent_desc", "Warna sorotan dan cincin fokus", "Highlight color and focus ring"),
    ("settings.glass_desc", "Ketebalan kaca panel", "Panel glass thickness"),
    ("settings.always_on_top_desc", "Noktah tidak tertutup jendela lain", "Noktah stays above other windows"),
    ("settings.start_expanded_desc", "Panel langsung besar saat aplikasi dibuka", "Panel opens large on launch"),
    ("settings.lyrics_enabled_desc", "Lirik tersinkron dari LRCLIB di tab Musik", "Synced lyrics from LRCLIB in Music tab"),
    ("settings.city_desc", "Dipakai untuk cuaca di Beranda", "Used for weather on Home"),
    ("settings.name_desc", "Nama yang dipanggil di sapaan", "Name used in the greeting"),
    ("settings.dynamic_accent", "Aksen dinamis album", "Dynamic album aura"),
    ("settings.opacity", "Opasitas", "Opacity"),
    ("settings.always_on_top", "Selalu di atas", "Always on top"),
    ("settings.custom_tiles", "Tile Status Kustom", "Custom Status Tiles"),
    (
        "settings.custom_tiles_desc",
        "Terima kartu metrik & status kustom lewat noktah-hook",
        "Accept custom metric & status tiles via noktah-hook",
    ),
    ("settings.start_expanded", "Mulai terbuka", "Start expanded"),
    ("settings.minimal_notch", "Notch minimal", "Minimal notch"),
    ("settings.lyrics", "Lirik", "Lyrics"),
    ("settings.lyrics_enabled", "Tampilkan lirik", "Show lyrics"),
    ("settings.lyric_offset", "Geser lirik", "Lyric offset"),
    ("settings.glass", "Efek kaca", "Glass effect"),
    ("settings.glass_effect", "Jenis kaca", "Glass type"),
    ("settings.ears", "Telinga cair", "Liquid ears"),
    ("settings.squircle", "Sudut squircle", "Squircle corners"),
    ("settings.animation", "Animasi morph", "Morph animation"),
    ("settings.sneak_peek", "Intip saat ganti lagu", "Peek on track change"),
    (
        "settings.hover_delay",
        "Jeda buka panel",
        "Panel open delay",
    ),
    ("settings.mini_row", "Baris minimal", "Mini row"),
    ("settings.mini_art", "Sampul album", "Album art"),
    ("settings.mini_title", "Judul lagu", "Track title"),
    ("settings.mini_lyrics", "Lirik", "Lyrics"),
    ("settings.mini_controls", "Kontrol putar", "Play controls"),
    ("settings.mini_wave", "Equalizer", "Equalizer"),
    ("settings.mini_progress", "Garis progres", "Progress line"),
    ("settings.mini_clock", "Jam", "Clock"),
    ("settings.mini_cpu", "Monitor CPU", "CPU monitor"),
    ("settings.mini_ram", "Monitor RAM", "RAM monitor"),
    ("settings.mini_agent", "Status Agent & Noki", "Agent Status & Noki"),
    ("settings.mini_idle_on_pause", "Kembali ke mode idle saat jeda", "Return to idle when paused"),
    (
        "settings.mini_idle_on_pause_desc",
        "Ciutkan pill ke tampilan jam & Noki saat musik di-pause atau tidak ada audio",
        "Collapse pill back to clock & Noki view when music is paused or silent",
    ),
    ("settings.mini_monitor_desc", "Hanya tampil di baris minimized", "Only shown in the minimized row"),
    ("settings.pill", "Pill", "Pill"),
    ("settings.pill_clock", "Jam di pill", "Clock in pill"),
    ("settings.pill_weather", "Suhu di pill", "Temperature in pill"),
    ("settings.pill_title", "Judul lagu di pill", "Track title in pill"),
    ("settings.pill_lyrics", "Lirik di pill", "Lyrics in pill"),
    ("settings.city", "Kota", "City"),
    ("settings.name", "Nama panggilan", "Nickname"),
    ("settings.ai", "AI", "AI"),
    ("settings.provider", "Penyedia", "Provider"),
    ("settings.model", "Model", "Model"),
    ("settings.api_key", "Kunci API", "API key"),
    ("settings.data", "Data", "Data"),
    ("settings.reset", "Hapus semua data", "Reset all data"),
    ("settings.about", "Tentang", "About"),
    ("settings.start_with_windows", "Mulai bersama Windows", "Start with Windows"),
    (
        "settings.start_with_windows_desc",
        "Jalankan Noktah otomatis saat komputer menyala",
        "Launch Noktah automatically on system startup",
    ),
    ("settings.version", "Versi", "Version"),
    ("settings.updates", "Pembaruan", "Updates"),
    ("settings.check_update", "Periksa Pembaruan", "Check for Updates"),
    ("settings.checking_update", "Memeriksa…", "Checking…"),
    ("settings.export", "Ekspor", "Export"),
    ("settings.import", "Impor", "Import"),
    ("settings.reset_confirm", "Yakin hapus? Klik lagi", "Confirm reset? Click again"),
    ("settings.reset_done", "Semua data berhasil direset", "All data has been reset"),
    ("settings.diagnostics", "Diagnostik & Log", "Diagnostics & Logs"),
    ("settings.open_data_dir", "Buka Folder Data", "Open Data Folder"),
    ("settings.view_logs", "Lihat Log", "View Logs"),
    ("settings.hide_logs", "Sembunyikan Log", "Hide Logs"),
    ("settings.copy_logs", "Salin Log", "Copy Logs"),
    ("settings.clear_logs", "Bersihkan Log", "Clear Logs"),
    ("settings.logs_copied", "Log diagnostik disalin ke clipboard!", "Diagnostic logs copied to clipboard!"),
    ("settings.logs_cleared", "Log diagnostik dibersihkan", "Diagnostic logs cleared"),
    ("settings.logs_empty", "Belum ada catatan log.", "No log entries yet."),
    ("settings.default_page", "Halaman bawaan saat dibuka", "Default page on open"),
    ("settings.default_page_desc", "Halaman yang tampil saat notch dibuka (pintasan Alt+1..7, Esc)", "Page shown when notch opens (shortcuts Alt+1..7, Esc)"),
    ("settings.focus_dnd", "Jangan Ganggu saat Fokus", "Do Not Disturb during Focus"),
    ("settings.focus_dnd_desc", "Tahan banner pengingat dan intip lagu selama sesi Fokus berjalan", "Hold reminder banners and track peek while Focus is running"),
    ("settings.self_update", "Unduh & Pasang Otomatis", "Download & Install Update"),
    ("settings.downloading_update", "Mengunduh & memverifikasi SHA-256…", "Downloading & verifying SHA-256…"),
    ("settings.restart_to_apply", "Pasang & Mulai Ulang Sekarang", "Install & Restart Now"),
    ("settings.fullscreen_autohide", "Auto-Hide Layar Penuh", "Fullscreen Auto-Hide"),
    ("settings.fullscreen_autohide_desc", "Sembunyikan notch otomatis saat bermain game atau video layar penuh", "Automatically hide notch when playing games or full-screen videos"),
    ("settings.monitor_mode", "Penempatan Layar", "Screen Placement"),
    ("settings.calendar_enabled", "Kalender .ics Lokal", "Local .ics Calendar"),
    ("settings.calendar_enabled_desc", "Tampilkan agenda rapat lokal dari berkas .ics di halaman Home", "Show local meeting agenda from .ics file on Home page"),
    ("settings.file_shelf", "Rak Berkas (Drop Shelf)", "File Drop Shelf"),
    ("settings.file_shelf_desc", "Tampung berkas sementara saat di-drag ke area notch", "Hold temporary files when dragged onto the notch"),
    ("settings.audio_hud", "HUD Perangkat Audio", "Audio Device HUD"),
    ("settings.audio_hud_desc", "Tampilkan notifikasi mikro saat perangkat audio beralih", "Show micro notification when audio device changes"),
    ("settings.presets", "Tema Preset", "Theme Presets"),
    ("settings.presets_desc", "Preset tampilan siap pakai", "Curated appearance presets"),
    ("settings.custom_accent", "Aksen Kustom (HEX)", "Custom Accent (HEX)"),
    ("settings.glass_specular", "Kilau Tepi (Specular)", "Edge Specular Rim"),
    ("settings.glass_blur", "Radius Blur Kaca", "Glass Blur Radius"),
    ("settings.glass_saturation", "Saturasi Latar Kaca", "Glass Saturation Boost"),
    ("settings.preset_export", "Ekspor Preset", "Export Preset"),
    ("settings.preset_import", "Impor Preset", "Import Preset"),
    ("settings.preset_exported", "Preset tema berhasil diekspor & disalin!", "Theme preset exported & copied to clipboard!"),
    ("settings.preset_imported", "Preset tema berhasil diterapkan!", "Theme preset applied successfully!"),
    ("settings.preset_import_err", "Gagal membaca preset: format JSON tidak valid", "Failed to import preset: invalid JSON format"),
    ("settings.custom_accent_desc", "Format #RRGGBB (contoh: #3B82F6)", "Format #RRGGBB (e.g. #3B82F6)"),
    ("settings.custom_accent_invalid", "HEX tidak valid", "Invalid HEX"),
    ("settings.dynamic_accent_desc", "Aksen dinamis mengikuti warna album musik", "Dynamic accent follows album artwork"),
    ("settings.monitor_mode_desc", "Pilih layar penempatan (Layar Utama atau Ikuti Kursor)", "Screen placement (Primary Screen or Follow Cursor)"),
    ("settings.minimal_notch_desc", "Ciutkan notch jadi garis tipis saat panel tertutup", "Collapse notch into thin line when panel is closed"),
    ("settings.calendar_ics_path", "Lokasi Berkas .ics", "Calendar .ics Path"),
    ("settings.calendar_ics_path_desc", "Path absolut atau relatif ke berkas kalender .ics", "Absolute or relative path to .ics calendar file"),
    ("settings.todo_tip", "💡 Tambahkan @HH:MM saat mengetik tugas untuk pengingat otomatis.", "💡 Add @HH:MM while typing a task for an automatic reminder."),
    ("settings.todo_clear_done", "Hapus tugas selesai", "Clear completed tasks"),
    ("settings.focus_length", "Durasi fokus", "Focus length"),
    ("settings.break_length", "Durasi istirahat", "Break length"),
    ("settings.section_startup", "Permulaan", "Startup"),
    ("settings.section_window", "Jendela", "Window"),
    ("settings.section_notch_motion", "Notch & Gerak", "Notch & Motion"),
    ("settings.section_nav_menu", "Menu Navigasi", "Navigation Menu"),
    ("settings.section_theme_accent", "Tema & Aksen", "Theme & Accent"),
    ("settings.section_tasks_reminders", "Tugas & Pengingat", "Tasks & Reminders"),
    ("settings.section_profile_location", "Profil & Lokasi", "Profile & Location"),
    ("settings.section_focus_session", "Sesi Fokus", "Focus Session"),
    ("settings.section_activity", "Aktivitas", "Activity"),
    ("settings.nav_home_desc", "Tampilkan halaman Beranda di menu navigasi", "Show Home dashboard in navigation menu"),
    ("settings.nav_music_desc", "Tampilkan pemutar musik & lirik di menu navigasi", "Show music player & lyrics in navigation menu"),
    ("settings.nav_focus", "Fokus (Pomodoro)", "Focus (Pomodoro)"),
    ("settings.nav_focus_desc", "Tampilkan timer fokus Pomodoro di menu navigasi", "Show Pomodoro focus timer in navigation menu"),
    ("settings.nav_tasks_desc", "Tampilkan daftar tugas to-do di menu navigasi", "Show to-do task list in navigation menu"),
    ("settings.nav_system_desc", "Tampilkan monitor sistem CPU & RAM di menu navigasi", "Show CPU & RAM system monitor in navigation menu"),
    ("settings.nav_ask", "Tanya (AI)", "Ask (AI)"),
    ("settings.nav_ask_desc", "Tampilkan obrolan AI di menu navigasi", "Show AI chat in navigation menu"),
    ("settings.nav_agent_desc", "Tampilkan agen Claude Code di menu navigasi", "Show Claude Code agent in navigation menu"),
    ("settings.calendar_ics_error", "Gagal memuat berkas .ics", "Failed to load .ics file"),
    ("settings.mini_row_desc", "Pilih komponen yang tampil di baris compact saat notch tertutup:", "Choose components shown in the compact row when the notch is closed:"),
    ("settings.mini_lyrics_desc", "Tampilkan lirik lagu berjalan satu baris", "Show single-line synced song lyrics"),
    ("settings.mini_wave_desc", "Bilah equalizer audio bergerak", "Animated audio equalizer bars"),
    ("settings.mini_agent_desc", "Avatar Maskot Noki & status aktivitas Claude Code", "Noki mascot avatar & Claude Code activity status"),
    ("settings.mini_art_desc", "Gambar sampul album melingkar", "Circular album cover artwork"),
    ("settings.mini_title_desc", "Nama lagu dan artis musik", "Song title and artist name"),
    ("settings.mini_progress_desc", "Pendar garis progres lagu di dasar pill", "Track progress glow line along the bottom of the pill"),
    ("settings.mini_controls_desc", "Tombol putar, jeda, dan lagu berikutnya", "Play, pause, and next track buttons"),
    ("settings.mini_clock_desc", "Penunjuk jam digital di pill", "Digital clock readout in the pill"),
    ("settings.todo_stats_active", "aktif", "active"),
    ("settings.todo_stats_reminders", "pengingat", "reminders"),
    ("settings.name_hint", "Kamu", "You"),
    ("settings.about_tagline", "Dynamic Island & Notch Native untuk Windows", "Native Windows Dynamic Island & Notch"),
    ("settings.update_idle", "Klik tombol di atas untuk memeriksa rilis terbaru di GitHub.", "Click the button above to check for the latest release on GitHub."),
    ("settings.update_checking_hint", "Menghubungi GitHub Releases…", "Contacting GitHub Releases…"),
    ("settings.update_latest", "✓ Noktah sudah menggunakan versi terbaru", "✓ Noktah is up to date"),
    ("settings.update_available", "★ Pembaruan Tersedia", "★ Update Available"),
    ("settings.update_open_github", "Buka Halaman Rilis GitHub ↗", "Open GitHub Releases ↗"),
    ("settings.update_ready_prefix", "✓ Paket", "✓ Package"),
    ("settings.update_ready_suffix", "terverifikasi SHA-256 dan siap dipasang.", "SHA-256 verified and ready to install."),
    ("settings.update_ready_unverified_suffix", "siap dipasang (checksum SHA-256 tidak tersedia).", "ready to install (SHA-256 checksum unavailable)."),
    ("settings.update_failed", "Gagal", "Failed"),
    ("settings.update_err.github_unreachable", "Tidak dapat menghubungi GitHub", "Unable to reach GitHub"),
    ("settings.update_err.http_status", "Server mengembalikan status HTTP", "Server returned HTTP status"),
    ("settings.update_err.invalid_json", "Respons rilis GitHub tidak valid", "Invalid GitHub release response"),
    ("settings.update_err.untrusted_url", "URL unduhan pembaruan tidak berasal dari repositori resmi.", "Update download URL is not from the official repository."),
    ("settings.update_err.asset_missing", "Berkas rilis tidak ditemukan", "Release asset missing"),
    ("settings.update_err.staging_dir", "Gagal menyiapkan folder staging pembaruan", "Failed to prepare update staging folder"),
    ("settings.update_err.download", "Gagal mengunduh paket pembaruan", "Failed to download update package"),
    ("settings.update_err.archive_read", "Gagal membaca arsip rilis", "Failed to read release archive"),
    ("settings.update_err.too_large", "Ukuran paket rilis melebihi batas aman", "Release package exceeds safe size limit"),
    ("settings.update_err.archive_write", "Gagal menyimpan arsip sementara", "Failed to save temporary archive"),
    ("settings.update_err.extract_status", "Ekstraksi paket gagal", "Package extraction failed"),
    ("settings.update_err.code", "kode", "code"),
    ("settings.update_err.extract_spawn", "Gagal menjalankan tar untuk ekstraksi", "Failed to launch tar for extraction"),
    ("settings.update_err.invalid_binary", "Berkas biner hasil ekstraksi tidak valid", "Extracted binary is invalid"),
    ("settings.update_err.binary_read", "Gagal membaca berkas untuk verifikasi", "Failed to read file for verification"),
    ("settings.update_err.sha256_mismatch", "Verifikasi SHA-256 gagal! Berkas unduhan rusak", "SHA-256 verification failed! Downloaded file is corrupted"),
    ("settings.update_err.install", "Gagal memasang pembaruan", "Failed to install update"),
    ("settings.update_err.hook_update", "Gagal memperbarui noktah-hook.exe", "Failed to update noktah-hook.exe"),
    ("settings.exported", "Setelan diekspor ke settings-export.json", "Settings exported to settings-export.json"),
    ("settings.imported", "Setelan diimpor", "Settings imported"),
    ("settings.import_err", "Gagal mengimpor setelan: berkas tidak ditemukan atau JSON tidak valid", "Failed to import settings: file missing or invalid JSON"),
    ("notch.hint", "Arahkan kursor untuk membuka", "Hover to open"),
];

/// Terjemahan satu kunci. Kunci tak dikenal dikembalikan apa adanya supaya
/// teks yang lupa diterjemahkan tetap terlihat, bukan jadi kosong.
pub fn t(lang: Lang, key: &str) -> &'static str {
    for (k, id, en) in STRINGS {
        if *k == key {
            return match lang {
                Lang::Id => id,
                Lang::En => en,
            };
        }
    }
    // Tidak bisa mengembalikan &'static str dari input, jadi pakai kunci
    // yang memang statis; pemanggil yang salah tetap kelihatan di UI.
    "(?)"
}

pub fn has_key(key: &str) -> bool {
    STRINGS.iter().any(|(k, _, _)| *k == key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn semua_kunci_punya_dua_bahasa() {
        for (k, id, en) in STRINGS {
            assert!(!k.is_empty(), "kunci kosong");
            assert!(!id.trim().is_empty(), "terjemahan ID kosong: {k}");
            assert!(!en.trim().is_empty(), "terjemahan EN kosong: {k}");
        }
    }

    #[test]
    fn tidak_ada_kunci_ganda() {
        let mut seen = HashSet::new();
        for (k, _, _) in STRINGS {
            assert!(seen.insert(*k), "kunci ganda: {k}");
        }
    }

    #[test]
    fn kunci_tab_lengkap() {
        for key in [
            "tab.home",
            "tab.music",
            "tab.tasks",
            "tab.system",
            "tab.ai",
            "tab.settings",
        ] {
            assert!(has_key(key), "kunci tab hilang: {key}");
        }
    }

    #[test]
    fn terjemahan_berbeda_antar_bahasa() {
        assert_eq!(t(Lang::Id, "tab.home"), "Beranda");
        assert_eq!(t(Lang::En, "tab.home"), "Home");
        assert_eq!(t(Lang::Id, "system.cpu"), "CPU");
        assert_eq!(t(Lang::En, "focus.work"), "Work");
    }

    #[test]
    fn kunci_tak_dikenal_tidak_panik() {
        assert_eq!(t(Lang::Id, "tidak.ada"), "(?)");
        assert!(!has_key("tidak.ada"));
    }
}
