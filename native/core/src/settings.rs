//! Setelan aplikasi dan model penyedia AI.
//!
//! Kunci API **tidak** disimpan di sini — di Windows ia masuk Credential
//! Manager, di tempat lain ke file terpisah dengan izin ketat. Yang disimpan
//! di setelan hanya nama model dan alamat dasar.

use serde::{Deserialize, Serialize};

/// Palet aksen. Nilainya diambil dari prototipe desain `noktah-redesign.html`
/// (`const SW`), jadi UI native dan prototipe memakai warna yang sama persis.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Accent {
    /// `#5CD6CB` — bawaan prototipe.
    Teal,
    /// `#4DA3FF`
    #[serde(alias = "ice")]
    Blue,
    /// `#FF6B9D`
    #[serde(alias = "violet")]
    Pink,
    /// `#FF9F43`
    #[serde(alias = "amber")]
    Orange,
    /// `#5BD98C`
    #[serde(alias = "mint")]
    Green,
}

impl Default for Accent {
    fn default() -> Self {
        Accent::Teal
    }
}

impl Accent {
    pub const ALL: [Accent; 5] = [
        Accent::Teal,
        Accent::Blue,
        Accent::Pink,
        Accent::Orange,
        Accent::Green,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Accent::Teal => "Teal",
            Accent::Blue => "Blue",
            Accent::Pink => "Pink",
            Accent::Orange => "Orange",
            Accent::Green => "Green",
        }
    }

    /// Warna utama sebagai RGB.
    pub fn rgb(self) -> [u8; 3] {
        match self {
            Accent::Teal => [0x5C, 0xD6, 0xCB],
            Accent::Blue => [0x4D, 0xA3, 0xFF],
            Accent::Pink => [0xFF, 0x6B, 0x9D],
            Accent::Orange => [0xFF, 0x9F, 0x43],
            Accent::Green => [0x5B, 0xD9, 0x8C],
        }
    }

    /// Nama lama yang mungkin masih tersimpan di `settings.json` pengguna.
    ///
    /// Palet berganti dari Amber/Mint/Violet/Ice ke palet prototipe; tanpa
    /// pemetaan ini, berkas setelan lama gagal di-deserialize dan seluruh
    /// setelan pengguna kembali ke bawaan.
    pub fn from_legacy(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "amber" => Some(Accent::Orange),
            "mint" => Some(Accent::Green),
            "violet" => Some(Accent::Pink),
            "ice" => Some(Accent::Blue),
            _ => None,
        }
    }
}

/// Seberapa "kaca" permukaannya.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GlassEffect {
    /// Tanpa blur; cat lebih pekat.
    Off,
    /// Bening tanpa blur — aman di semua mesin.
    Light,
    /// Blur asli lewat DWM acrylic.
    Full,
}

impl Default for GlassEffect {
    fn default() -> Self {
        GlassEffect::Light
    }
}

impl GlassEffect {
    pub const ALL: [GlassEffect; 3] = [GlassEffect::Off, GlassEffect::Light, GlassEffect::Full];

    pub fn label(self) -> &'static str {
        match self {
            GlassEffect::Off => "Mati",
            GlassEffect::Light => "Ringan",
            GlassEffect::Full => "Penuh",
        }
    }

    /// Mode Penuh memakai tangkap-layar + blur sendiri, bukan acrylic DWM
    /// (yang membuat jendela ber-alpha menjadi kotak hitam).
    pub fn uses_blur(self) -> bool {
        matches!(self, GlassEffect::Full)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    Id,
    En,
}

impl Default for Lang {
    fn default() -> Self {
        Lang::Id
    }
}

impl Lang {
    pub fn toggle(self) -> Self {
        match self {
            Lang::Id => Lang::En,
            Lang::En => Lang::Id,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::Id => "id",
            Lang::En => "en",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Openai,
    Gemini,
    Openrouter,
    Groq,
    Deepseek,
    Ollama,
}

impl Default for Provider {
    fn default() -> Self {
        Provider::Gemini
    }
}

impl Provider {
    pub const ALL: [Provider; 6] = [
        Provider::Openai,
        Provider::Gemini,
        Provider::Openrouter,
        Provider::Groq,
        Provider::Deepseek,
        Provider::Ollama,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Provider::Openai => "OpenAI",
            Provider::Gemini => "Google Gemini",
            Provider::Openrouter => "OpenRouter",
            Provider::Groq => "Groq",
            Provider::Deepseek => "DeepSeek",
            Provider::Ollama => "Ollama (lokal)",
        }
    }

    pub fn base_url(self) -> &'static str {
        match self {
            Provider::Openai => "https://api.openai.com/v1",
            Provider::Gemini => "https://generativelanguage.googleapis.com/v1beta/openai",
            Provider::Openrouter => "https://openrouter.ai/api/v1",
            Provider::Groq => "https://api.groq.com/openai/v1",
            Provider::Deepseek => "https://api.deepseek.com/v1",
            Provider::Ollama => "http://localhost:11434/v1",
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            Provider::Openai => "gpt-4o-mini",
            Provider::Gemini => "gemini-2.0-flash",
            Provider::Openrouter => "google/gemini-2.0-flash-001",
            Provider::Groq => "llama-3.3-70b-versatile",
            Provider::Deepseek => "deepseek-chat",
            Provider::Ollama => "llama3.2",
        }
    }

    /// Ollama berjalan lokal dan tidak butuh kunci.
    pub fn keyless(self) -> bool {
        matches!(self, Provider::Ollama)
    }
}

fn default_true() -> bool {
    true
}

/// Bawaan jeda hover: 240 ms, cukup membedakan "lewat sambil menunjuk" dari
/// "sengaja membuka" tanpa terasa lambat.
fn default_hover_delay() -> u32 {
    240
}

/// Pilihan halaman bawaan saat panel notch dibuka.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DefaultPage {
    LastOpened,
    Home,
    Music,
    Pomodoro,
    Tasks,
    System,
    Ask,
    Agent,
}

impl Default for DefaultPage {
    fn default() -> Self {
        DefaultPage::LastOpened
    }
}

/// Mode penempatan monitor untuk notch.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MonitorMode {
    Primary,
    Cursor,
    Index(u32),
}

impl Default for MonitorMode {
    fn default() -> Self {
        MonitorMode::Primary
    }
}

impl MonitorMode {
    pub const ALL: [MonitorMode; 2] = [
        MonitorMode::Primary,
        MonitorMode::Cursor,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MonitorMode::Primary => "Layar Utama",
            MonitorMode::Cursor => "Ikuti Kursor",
            MonitorMode::Index(_) => "Monitor Tertentu",
        }
    }
}

impl DefaultPage {
    pub const ALL: [DefaultPage; 8] = [
        DefaultPage::LastOpened,
        DefaultPage::Home,
        DefaultPage::Music,
        DefaultPage::Pomodoro,
        DefaultPage::Tasks,
        DefaultPage::System,
        DefaultPage::Ask,
        DefaultPage::Agent,
    ];

    pub fn label(&self, lang: Lang) -> &'static str {
        match (self, lang) {
            (DefaultPage::LastOpened, Lang::Id) => "Terakhir dibuka",
            (DefaultPage::LastOpened, Lang::En) => "Last opened",
            (DefaultPage::Home, Lang::Id) => "Beranda",
            (DefaultPage::Home, Lang::En) => "Home",
            (DefaultPage::Music, Lang::Id) => "Musik",
            (DefaultPage::Music, Lang::En) => "Music",
            (DefaultPage::Pomodoro, Lang::Id) => "Fokus",
            (DefaultPage::Pomodoro, Lang::En) => "Focus",
            (DefaultPage::Tasks, Lang::Id) => "Tugas",
            (DefaultPage::Tasks, Lang::En) => "Tasks",
            (DefaultPage::System, Lang::Id) => "Sistem",
            (DefaultPage::System, Lang::En) => "System",
            (DefaultPage::Ask, Lang::Id) => "Tanya AI",
            (DefaultPage::Ask, Lang::En) => "Ask AI",
            (DefaultPage::Agent, _) => "Agent",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub city: String,
    pub accent: Accent,
    #[serde(default = "default_true")]
    pub dynamic_accent: bool,
    pub opacity: f32,
    pub always_on_top: bool,
    pub start_expanded: bool,
    pub provider: Provider,
    pub model: String,
    pub base_url: String,
    pub user_name: String,
    pub lyrics_enabled: bool,
    pub lyric_offset_ms: i64,
    pub pill_show_clock: bool,
    pub pill_show_weather: bool,
    pub pill_show_title: bool,
    pub pill_show_lyrics: bool,
    pub language: Lang,
    pub minimal_notch: bool,
    /// Telinga cair di kiri-kanan panel.
    pub ears: bool,
    /// Sudut superellipse (squircle) alih-alih lingkaran biasa.
    pub squircle: bool,
    /// Paksa animasi walau Windows punya "Animation effects" dimatikan.
    ///
    /// Default false = ikut Windows. Banyak mesin Windows dimatikan efeknya
    /// (atau dimatikan oleh tweak "performance"), dan tanpa saklar ini notch
    /// tidak pernah beranimasi sama sekali — terlihat seperti aplikasi rusak,
    /// bukan seperti aplikasi yang menghormati setelan aksesibilitas.
    #[serde(default)]
    pub force_animation: bool,
    /// Pill melebar sesaat menampilkan judul+artis saat lagu berganti.
    /// Pola "sneak peek" yang terbukti di Boring Notch dan OpenNook.
    #[serde(default = "default_true")]
    pub sneak_peek: bool,
    /// Jeda hover sebelum panel membuka, dalam milidetik. 0 = buka langsung.
    /// Mencegah expand tidak sengaja — keluhan terbanyak pengguna aplikasi
    /// notch (riset Droppy, OpenNook, Notchy).
    #[serde(default = "default_hover_delay")]
    pub hover_delay_ms: u32,
    pub glass_effect: GlassEffect,
    // Baris minimized: item mana yang tampil.
    pub mini_art: bool,
    pub mini_title: bool,
    #[serde(default = "default_true")]
    pub mini_lyrics: bool,
    pub mini_controls: bool,
    pub mini_wave: bool,
    pub mini_progress: bool,
    pub mini_clock: bool,
    /// Monitor CPU di baris minimized. Tidak ditampilkan di panel terbuka.
    #[serde(default = "default_true")]
    pub mini_cpu: bool,
    /// Monitor RAM di baris minimized. Tidak ditampilkan di panel terbuka.
    #[serde(default = "default_true")]
    pub mini_ram: bool,
    /// Maskot / status Agent Claude Code di baris minimized.
    #[serde(default = "default_true")]
    pub mini_agent: bool,
    /// Kembali ke mode idle (`[Noki] + [Jam] + [||||]`) saat musik di-pause
    /// atau tidak ada audio yang diputar. Bila `false`, pill tetap menahan
    /// tampilan lagu terakhir selama sesi SMTC aktif.
    #[serde(default = "default_true")]
    pub mini_idle_on_pause: bool,
    /// Halaman bawaan yang ditampilkan setiap kali panel notch dibuka.
    #[serde(default)]
    pub default_page: DefaultPage,
    /// Tahan banner non-esensial saat timer Fokus sedang berjalan.
    #[serde(default = "default_true")]
    pub focus_dnd: bool,
    pub start_with_windows: bool,
    /// Durasi pomodoro dalam menit.
    pub focus_minutes: u32,
    pub break_minutes: u32,
    // Navigasi dock: halaman mana yang tampil di menu navigasi.
    #[serde(default = "default_true")]
    pub nav_home: bool,
    #[serde(default = "default_true")]
    pub nav_music: bool,
    #[serde(default = "default_true")]
    pub nav_pomodoro: bool,
    #[serde(default = "default_true")]
    pub nav_tasks: bool,
    #[serde(default = "default_true")]
    pub nav_system: bool,
    #[serde(default = "default_true")]
    pub nav_ask: bool,
    #[serde(default = "default_true")]
    pub nav_agent: bool,
    /// Sembunyikan notch otomatis saat mendeteksi jendela layar penuh (game, video).
    #[serde(default = "default_true")]
    pub fullscreen_autohide: bool,
    /// Mode penempatan monitor (Layar Utama atau Ikuti Kursor).
    #[serde(default)]
    pub monitor_mode: MonitorMode,
    /// Mengaktifkan kalender lokal .ics.
    #[serde(default = "default_true")]
    pub calendar_enabled: bool,
    /// Lokasi custom berkas kalender .ics (kosong = bawaan %APPDATA%\Noktah\calendar.ics).
    #[serde(default)]
    pub calendar_ics_path: String,
    /// Mengaktifkan File Drop Shelf (rak penampung berkas sementara).
    #[serde(default = "default_true")]
    pub file_shelf_enabled: bool,
    /// Menampilkan HUD banner saat perangkat audio beralih / tersambung.
    #[serde(default = "default_true")]
    pub audio_hud_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            city: "Jakarta".into(),
            accent: Accent::Teal,
            dynamic_accent: true,
            opacity: 0.94,
            always_on_top: true,
            start_expanded: false,
            provider: Provider::Gemini,
            model: Provider::Gemini.default_model().into(),
            base_url: Provider::Gemini.base_url().into(),
            user_name: String::new(),
            lyrics_enabled: true,
            lyric_offset_ms: 0,
            pill_show_clock: true,
            pill_show_weather: true,
            pill_show_title: true,
            pill_show_lyrics: true,
            language: Lang::Id,
            minimal_notch: false,
            ears: false,
            squircle: true,
            force_animation: false,
            sneak_peek: true,
            hover_delay_ms: 240,
            glass_effect: GlassEffect::Light,
            mini_art: true,
            mini_title: true,
            mini_lyrics: true,
            mini_controls: false,
            mini_wave: true,
            mini_progress: true,
            mini_clock: true,
            mini_cpu: true,
            mini_ram: true,
            mini_agent: true,
            mini_idle_on_pause: true,
            default_page: DefaultPage::LastOpened,
            focus_dnd: true,
            start_with_windows: false,
            focus_minutes: 25,
            break_minutes: 5,
            nav_home: true,
            nav_music: true,
            nav_pomodoro: true,
            nav_tasks: true,
            nav_system: true,
            nav_ask: true,
            nav_agent: true,
            fullscreen_autohide: true,
            monitor_mode: MonitorMode::Primary,
            calendar_enabled: true,
            calendar_ics_path: String::new(),
            file_shelf_enabled: true,
            audio_hud_enabled: true,
        }
    }
}

/// Susunan baris minimized yang siap pakai.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MiniPreset {
    Standar,
    ModeLirik,
    FokusLirik,
    Lengkap,
}

impl MiniPreset {
    pub const ALL: [MiniPreset; 4] = [
        MiniPreset::Standar,
        MiniPreset::ModeLirik,
        MiniPreset::FokusLirik,
        MiniPreset::Lengkap,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MiniPreset::Standar => "Standar",
            MiniPreset::ModeLirik => "Mode Lirik",
            MiniPreset::FokusLirik => "Fokus Lirik",
            MiniPreset::Lengkap => "Lengkap",
        }
    }
}

impl Settings {
    /// Memakai penyedia baru sekaligus menimpa model dan alamat dasarnya.
    pub fn switch_provider(&mut self, provider: Provider) {
        self.provider = provider;
        self.model = provider.default_model().into();
        self.base_url = provider.base_url().into();
    }

    /// Menentukan apakah baris minimized menampilkan mode pemutar musik
    /// atau kembali ke mode idle.
    pub fn show_media_in_mini(&self, media_active: bool, media_playing: bool) -> bool {
        if !media_active {
            return false;
        }
        if self.mini_idle_on_pause {
            media_playing
        } else {
            true
        }
    }

    pub fn apply_preset(&mut self, preset: MiniPreset) {
        let (art, title, lyrics, controls, wave, progress, clock, agent) = match preset {
            MiniPreset::Standar => (true, true, false, false, true, true, true, true),
            MiniPreset::ModeLirik => (true, false, true, false, true, true, false, true),
            MiniPreset::FokusLirik => (false, false, true, false, false, true, false, false),
            MiniPreset::Lengkap => (true, true, true, true, true, true, true, true),
        };
        self.mini_art = art;
        self.mini_title = title;
        self.mini_lyrics = lyrics;
        self.mini_controls = controls;
        self.mini_wave = wave;
        self.mini_progress = progress;
        self.mini_clock = clock;
        self.mini_agent = agent;
    }

    /// Lebar baris minimized, dihitung dari item yang menyala saja.
    ///
    /// `has_lyrics` penting: kalau lagu tidak punya lirik, slot lirik ikut
    /// dibuang dari hitungan — kalau tidak, pill menyisakan ruang kosong
    /// selebar 245 px untuk teks yang tidak akan pernah muncul.
    ///
    /// Geometri baru (Mode Compact yang diminta):
    /// - Saat ada lirik: slot lirik tetap **240 px di tengah** pill; kiri
    ///   ditempati art+judul, kanan oleh item kanan yang disejajarkan ke tepi.
    /// - Saat tidak ada lirik: hanya trio ter-pusat (art, equalizer, maskot,
    ///   maksimal 3) yang tampil; judul/kontrol/jam/CPU/RAM disembunyikan.
    pub fn mini_width(&self, playing: bool, has_lyrics: bool) -> f32 {
        if !playing {
            // Pill idle: jam di tengah, equalizer di kanan. Tanpa indikator
            // daya, jadi hanya dua slot — lebar tetap yang pas, bukan
            // akumulasi yang menyisakan celah kosong.
            let mut w: f32 = 96.0;
            if self.pill_show_weather {
                w += 50.0;
            }
            return w.clamp(120.0, 180.0);
        }

        let lyrics_on = self.mini_lyrics && has_lyrics;
        if lyrics_on {
            // Kiri (art + judul) + slot lirik 240 px + kanan. Item kanan
            // disejajarkan ke tepi kanan pill (right-20), jadi lebar = jumlah
            // slotnya saja; margin 20+20 kiri/kanan sudah di dalam "44 + 12".
            let mut w: f32 = 44.0;
            if self.mini_art {
                w += 38.0;
            }
            if self.mini_title {
                w += 175.0;
            }
            w += 240.0;
            if self.mini_controls {
                w += 64.0;
            }
            if self.mini_wave {
                w += 38.0;
            }
            if self.mini_agent {
                w += 34.0;
            }
            if self.mini_clock {
                w += 64.0;
            }
            if self.mini_cpu {
                w += 54.0;
            }
            if self.mini_ram {
                w += 54.0;
            }
            w += 12.0;
            w.max(160.0).min(720.0)
        } else {
            // Mode trio: hanya art (38), equalizer (38), maskot (34) yang
            // dipusatkan, maksimal 3 item, gap antar item 6 px. Tanpa salah
            // satu pun: pill sempit hanya berisi aura progres.
            let mut n: usize = 0;
            if self.mini_art {
                n += 1;
            }
            if self.mini_wave {
                n += 1;
            }
            if self.mini_agent {
                n += 1;
            }
            if n == 0 {
                return 120.0;
            }
            let slots: f32 = [self.mini_art, self.mini_wave, self.mini_agent]
                .iter()
                .zip([38.0, 38.0, 34.0])
                .filter(|(on, _)| **on)
                .map(|(_, w)| w)
                .sum();
            let gaps = (n - 1) as f32 * 6.0;
            (20.0 + slots + gaps + 20.0 + 12.0).clamp(120.0, 720.0)
        }
    }

    /// Membetulkan nilai yang tidak masuk akal setelah dibaca dari disk.
    pub fn sanitize(&mut self) {

        self.opacity = self.opacity.clamp(0.5, 1.0);
        self.lyric_offset_ms = self.lyric_offset_ms.clamp(-10_000, 10_000);
        self.focus_minutes = self.focus_minutes.clamp(1, 180);
        self.break_minutes = self.break_minutes.clamp(1, 60);
        if self.model.trim().is_empty() {
            self.model = self.provider.default_model().into();
        }
        if self.base_url.trim().is_empty() {
            self.base_url = self.provider.base_url().into();
        }
        if self.city.trim().is_empty() {
            self.city = "Jakarta".into();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setelan_bawaan_masuk_akal() {
        let s = Settings::default();
        assert_eq!(s.city, "Jakarta");
        assert_eq!(s.language, Lang::Id);
        assert!(s.lyrics_enabled);
        assert_eq!(s.focus_minutes, 25);
    }

    #[test]
    fn json_kosong_memakai_bawaan() {
        let s: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(s, Settings::default());
    }

    #[test]
    fn pill_menyusut_kalau_lagu_tanpa_lirik() {
        let mut s = Settings::default();
        s.mini_art = true;
        s.mini_title = true;
        s.mini_lyrics = true;
        s.mini_controls = true;
        // Item kanan lain dimatikan supaya lebar tidak menyentuh batas 720,
        // yang akan menutupi selisih yang sedang diuji.
        s.mini_wave = false;
        s.mini_clock = false;
        s.mini_cpu = false;
        s.mini_ram = false;

        let dengan_lirik = s.mini_width(true, true);
        let tanpa_lirik = s.mini_width(true, false);

        // Geometri baru: dengan lirik = kiri (art+judul) + slot lirik 240 px
        // di tengah + kanan (kontrol + agent); tanpa lirik = trio ter-pusat
        // (art + maskot, karena wave/monitor dimatikan di sini).
        assert_eq!(dengan_lirik, 44.0 + 38.0 + 175.0 + 240.0 + 64.0 + 34.0 + 12.0);
        assert_eq!(tanpa_lirik, 20.0 + 38.0 + 6.0 + 34.0 + 20.0 + 12.0);
        assert!(tanpa_lirik < dengan_lirik, "tanpa lirik harus jauh lebih sempit");
    }

    #[test]
    fn pill_tanpa_lirik_sama_dengan_lirik_dimatikan() {
        let mut s = Settings::default();
        s.mini_art = true;
        s.mini_title = true;
        s.mini_lyrics = true;
        s.mini_controls = true;

        // Menyembunyikan slot karena tak ada lirik harus identik dengan
        // mematikan centang liriknya — satu sumber kebenaran untuk geometri.
        assert_eq!(s.mini_width(true, false), {
            let mut tanpa = s;
            tanpa.mini_lyrics = false;
            tanpa.mini_width(true, true)
        });
    }

    #[test]
    fn monitor_sistem_bisa_dimatikan() {
        // Geometri baru: CPU/RAM tidak lagi tampil di baris compact TANPA lirik
        // (mode trio ter-pusat). Untuk tetap menguji toggling monitor, pakai
        // lagu BER-lirik supaya slot kanan penuh dihitung.
        let mut s = Settings::default();
        s.mini_art = true;
        s.mini_title = true;
        s.mini_lyrics = true;
        s.mini_controls = true;
        s.mini_wave = false;
        s.mini_clock = false;

        let keduanya = s.mini_width(true, true);
        s.mini_cpu = false;
        let ram_saja = s.mini_width(true, true);
        s.mini_ram = false;
        let tanpa_keduanya = s.mini_width(true, true);

        assert_eq!(keduanya - ram_saja, 54.0);
        assert_eq!(ram_saja - tanpa_keduanya, 54.0);
    }

    #[test]
    fn trio_tanpa_lirik_lebarnya_pas_dengan_item_yang_menyala() {
        let mut s = Settings::default();
        s.mini_title = false;
        s.mini_lyrics = false;
        s.mini_controls = false;
        s.mini_clock = false;
        s.mini_cpu = false;
        s.mini_ram = false;
        s.mini_art = false;
        s.mini_wave = false;
        s.mini_agent = false;

        // Tidak ada satu pun item trio: pill sempit, hanya aura progres.
        assert_eq!(s.mini_width(true, false), 120.0);

        // Hanya equalizer: 20 + 38 + 20 + 12 = 90, tapi dipaksa ke minimum 120
        // supaya pill tidak menyusut ke ukuran yang mustahil diklik.
        s.mini_wave = true;
        assert_eq!(s.mini_width(true, false), 120.0);

        // Equalizer + maskot (gap 6 px di antara).
        s.mini_agent = true;
        assert_eq!(
            s.mini_width(true, false),
            20.0 + 38.0 + 6.0 + 34.0 + 20.0 + 12.0
        );

        // Trio penuh art + wave + agent (2 gap).
        s.mini_art = true;
        assert_eq!(
            s.mini_width(true, false),
            20.0 + 38.0 + 6.0 + 38.0 + 6.0 + 34.0 + 20.0 + 12.0
        );
        // Judul dan kontrol di luar trio tidak menambah lebar tanpa lirik.
        s.mini_title = true;
        s.mini_controls = true;
        assert_eq!(
            s.mini_width(true, false),
            20.0 + 38.0 + 6.0 + 38.0 + 6.0 + 34.0 + 20.0 + 12.0
        );
    }

    #[test]
    fn lirik_aktif_memakai_slot_tengah_240_dan_item_kanan() {
        let mut s = Settings::default();
        s.mini_lyrics = true;
        s.mini_art = true;
        s.mini_title = true;
        s.mini_controls = true;
        s.mini_wave = false;
        s.mini_clock = false;
        s.mini_cpu = false;
        s.mini_ram = false;
        s.mini_agent = false;

        // 44 + art 38 + judul 175 + slot 240 + kontrol 64 + 12.
        assert_eq!(
            s.mini_width(true, true),
            44.0 + 38.0 + 175.0 + 240.0 + 64.0 + 12.0
        );
        // Lagu tanpa lirik jauh lebih sempit (trio: art saja, di-clamp ke 120).
        assert_eq!(s.mini_width(true, false), 120.0);
    }

    #[test]
    fn monitor_sistem_bawaan_menyala() {
        let s = Settings::default();
        assert!(s.mini_cpu && s.mini_ram);
    }

    #[test]
    fn field_baru_tidak_merusak_setelan_lama() {
        // Setelan versi lama tanpa focus_minutes harus tetap terbaca.
        let raw = r#"{"city":"Bandung","accent":"mint","language":"en"}"#;
        let s: Settings = serde_json::from_str(raw).unwrap();
        assert_eq!(s.city, "Bandung");
        assert_eq!(s.language, Lang::En);
        assert_eq!(s.focus_minutes, 25);
    }

    #[test]
    fn aksen_lama_dipetakan_ke_palet_baru() {
        // Palet berganti dari Amber/Mint/Violet/Ice ke palet prototipe
        // (Teal/Blue/Pink/Orange/Green). Setelan pengguna yang sudah ada di
        // disk memakai nama lama; tanpa alias serde, seluruh berkas gagal
        // di-deserialize dan semua preferensi hilang.
        for (lama, baru) in [
            ("amber", Accent::Orange),
            ("mint", Accent::Green),
            ("violet", Accent::Pink),
            ("ice", Accent::Blue),
        ] {
            let raw = format!(r#"{{"accent":"{lama}"}}"#);
            let s: Settings = serde_json::from_str(&raw)
                .unwrap_or_else(|e| panic!("aksen lama {lama:?} gagal dibaca: {e}"));
            assert_eq!(s.accent, baru, "aksen {lama:?} dipetakan salah");
        }

        // Nama baru juga harus tetap terbaca.
        for (nama, v) in [
            ("teal", Accent::Teal),
            ("blue", Accent::Blue),
            ("pink", Accent::Pink),
            ("orange", Accent::Orange),
            ("green", Accent::Green),
        ] {
            let raw = format!(r#"{{"accent":"{nama}"}}"#);
            let s: Settings = serde_json::from_str(&raw).unwrap();
            assert_eq!(s.accent, v);
        }
    }

    #[test]
    fn palet_aksen_sesuai_prototipe() {
        // Warna diambil dari `const SW` di noktah-redesign.html.
        assert_eq!(Accent::Teal.rgb(), [0x5C, 0xD6, 0xCB]);
        assert_eq!(Accent::Blue.rgb(), [0x4D, 0xA3, 0xFF]);
        assert_eq!(Accent::Pink.rgb(), [0xFF, 0x6B, 0x9D]);
        assert_eq!(Accent::Orange.rgb(), [0xFF, 0x9F, 0x43]);
        assert_eq!(Accent::Green.rgb(), [0x5B, 0xD9, 0x8C]);
        assert_eq!(Accent::default(), Accent::Teal);
    }

    #[test]
    fn ganti_penyedia_menimpa_model_dan_url() {
        let mut s = Settings::default();
        s.switch_provider(Provider::Groq);
        assert_eq!(s.model, "llama-3.3-70b-versatile");
        assert_eq!(s.base_url, "https://api.groq.com/openai/v1");
    }

    #[test]
    fn sanitize_membatasi_nilai_liar() {
        let mut s = Settings::default();
        s.opacity = 5.0;
        s.focus_minutes = 0;
        s.break_minutes = 999;
        s.lyric_offset_ms = 99_999;
        s.model = "   ".into();
        s.sanitize();
        assert_eq!(s.opacity, 1.0);
        assert_eq!(s.focus_minutes, 1);
        assert_eq!(s.break_minutes, 60);
        assert_eq!(s.lyric_offset_ms, 10_000);
        assert!(!s.model.trim().is_empty());
    }

    #[test]
    fn accent_punya_warna_berbeda() {
        let warna: Vec<[u8; 3]> = Accent::ALL.iter().map(|a| a.rgb()).collect();
        assert_eq!(warna.len(), 5);
        // Tidak ada dua aksen yang warnanya sama — kalau ada, tombol swatch
        // di setelan akan tampak duplikat.
        for i in 0..warna.len() {
            for j in (i + 1)..warna.len() {
                assert_ne!(warna[i], warna[j], "aksen {i} dan {j} berwarna sama");
            }
        }
    }

    #[test]
    fn ollama_tanpa_kunci() {
        assert!(Provider::Ollama.keyless());
        assert!(!Provider::Openai.keyless());
    }

    #[test]
    fn bahasa_bisa_dibolak_balik() {
        assert_eq!(Lang::Id.toggle(), Lang::En);
        assert_eq!(Lang::En.toggle(), Lang::Id);
    }

    #[test]
    fn show_media_in_mini_mengikuti_setelan_idle_saat_pause() {
        let mut s = Settings::default();
        assert!(s.mini_idle_on_pause);
        let dari_kosong: Settings = serde_json::from_str("{}").unwrap();
        assert!(dari_kosong.mini_idle_on_pause);

        // Bawaan: kembali ke mode idle saat tidak aktif atau pause
        assert!(!s.show_media_in_mini(false, false));
        assert!(s.show_media_in_mini(true, true));
        assert!(!s.show_media_in_mini(true, false));

        // Jika dimatikan: tetap pertahankan tampilan lagu terakhir saat pause
        s.mini_idle_on_pause = false;
        assert!(!s.show_media_in_mini(false, false));
        assert!(s.show_media_in_mini(true, true));
        assert!(s.show_media_in_mini(true, false));
    }

    #[test]
    fn default_page_dan_focus_dnd_bawaan() {
        let s = Settings::default();
        assert_eq!(s.default_page, DefaultPage::LastOpened);
        assert!(s.focus_dnd);

        let raw = serde_json::to_string(&DefaultPage::Pomodoro).unwrap();
        assert_eq!(raw, "\"pomodoro\"");
        let back: DefaultPage = serde_json::from_str(&raw).unwrap();
        assert_eq!(back, DefaultPage::Pomodoro);

        let dari_kosong: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(dari_kosong.default_page, DefaultPage::LastOpened);
        assert!(dari_kosong.focus_dnd);
    }
}
