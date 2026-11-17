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
    ("ai.title", "AI", "AI"),
    ("ai.placeholder", "Tanya apa saja…", "Ask anything…"),
    ("ai.send", "Kirim", "Send"),
    ("ai.stop", "Hentikan", "Stop"),
    ("ai.no_key", "Kunci API belum diatur", "API key not set"),
    ("ai.thinking", "Sedang berpikir…", "Thinking…"),
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
