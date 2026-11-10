//! Keadaan utama aplikasi Noktah dan definisi struktur data.
//!
//! Logika per-metode telah didelegasikan ke submodul di `crate::presentation::shell`:
//! - `persistence`: sinkronisasi berkas data & penyimpanan ter-debounce
//! - `sync`: polling worker SMTC, AI, lirik, cuaca
//! - `render`: shell notch, panel, jendela setelan
//! - `frame`: siklus hidup per-frame (`eframe::App`)

use crate::infrastructure::ai::{self, AiHandle};
use crate::infrastructure::audio::AudioHandle;
use crate::infrastructure::lyrics_client::{self, LyricsHandle};
use crate::infrastructure::music::{self, MusicHandle};
use crate::infrastructure::platform::Hover;
use crate::infrastructure::secret;
use crate::infrastructure::store;
use crate::infrastructure::sys::SysSampler;
use crate::infrastructure::weather::{self, WeatherHandle};
use crate::presentation::icons;
use crate::presentation::layout;
use crate::presentation::spring::*;
use crate::presentation::theme::Theme;
use eframe::egui;
use noktah_core::{
    i18n, lrc,
    model::{ChatMessage, MediaState, PomodoroState, Todo, WaterState},
    settings::{Lang, Settings},
};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsTab {
    General,
    Display,
    Appearance,
    Music,
    Todo,
    Pomodoro,
    Ai,
    About,
}

pub struct Noktah {
    pub open: Spring,
    /// Tinggi punya pegas sendiri yang lebih lambat — inilah yang membuat
    /// bentuknya terasa melar dan mengendur, bukan kaku seperti satu kotak.
    pub open_h: Spring,
    pub(crate) hovered: bool,
    pub hover: Arc<Hover>,
    pub renderer: String,
    pub frames: u32,
    pub(crate) last_frame: Instant,
    pub anim: Instant,
    /// Target seek yang belum dikonfirmasi SMTC. `Some((target_ms, saat))`:
    /// slider ditahan di target ini supaya tidak memantul ke posisi lama
    /// sebelum update posisi baru datang (siklus SMTC ≤ 500 ms).
    pub seek_at: Option<(i64, std::time::Instant)>,

    pub settings: Settings,
    pub theme: Theme,

    pub sys: SysSampler,
    pub(crate) stats_at: Instant,

    pub todos: Vec<Todo>,
    pub todo_input: String,
    pub(crate) todo_due: String,

    pub media: MediaState,
    pub lyrics: Vec<lrc::LrcLine>,
    pub lyrics_synced: bool,
    pub lyrics_status: LyricsStatus,
    pub lyrics_provider: String,
    pub music: MusicHandle,
    pub lyrics_client: LyricsHandle,
    pub art: Option<egui::TextureHandle>,
    pub(crate) art_for: String,
    pub(crate) lyrics_for: String,
    pub chat: Vec<ChatMessage>,
    pub chat_input: String,
    pub ai: AiHandle,
    pub api_key: String,
    pub api_key_input: String,
    pub ai_busy: bool,
    pub weather: WeatherHandle,
    pub weather_now: Option<noktah_core::model::WeatherNow>,
    pub(crate) weather_city: String,
    /// Kuota Antigravity dari 9Router. `None` sampai pembacaan pertama selesai
    /// — dibedakan dari snapshot kosong, yang berarti "sudah dibaca, gagal".
    pub quota: Option<crate::infrastructure::quota::QuotaSnapshot>,
    pub quota_worker: crate::infrastructure::quota::QuotaHandle,
    pub(crate) applied_glass: Option<noktah_core::settings::GlassEffect>,
    pub reduce_motion: bool,
    /// 0 = baris minimized biasa, 1 = garis tipis. Dianimasikan, bukan boolean,
    /// supaya transisi antar-lagu bisa mengecil lalu membesar kembali.
    pub minimal_mix: Spring,
    /// Kapan transisi antar-lagu mulai. `None` = tidak sedang transisi.
    pub(crate) transisi_at: Option<Instant>,
    /// Sneak peek: sampai kapan pill ditahan menampilkan lagu baru.
    /// `None` = tidak sedang peek.
    pub(crate) peek_until: Option<Instant>,
    /// Kapan kursor mulai diam di atas notch. `None` = tidak hover.
    /// Hover intent: panel hanya membuka setelah jeda ini terpenuhi —
    /// keluhan #1 pengguna aplikasi notch adalah expand tidak sengaja.
    pub(crate) hover_since: Option<Instant>,
    pub(crate) glass_tex: Option<egui::TextureHandle>,
    /// Kapan pengguna terakhir menggeser atau memilih lirik.
    pub lyric_manual_at: Instant,
    pub album_accent: Option<egui::Color32>,
    pub scrubber_expand: Spring,
    pub lyrics_anim: Spring,
    /// Lebar baris minimized ikut dianimasikan: saat lagu mulai diputar
    /// isinya bertambah dan lebarnya melompat 193 → 489 px. Tanpa pegas,
    /// lompatan itu terlihat kasar.
    pub pill: Spring,
    pub last_active_lyric: Option<usize>,
    pub lyric_line_active_at: Instant,
    pub lyric_scroll_pos: f32,
    pub lyric_target_scroll: f32,
    /// Posisi baris aktif lirik sebagai indeks pecahan (mis. 6.3 = 30% dari
    /// baris 6 menuju baris 7). Menggantikan lompatan t_trans karena spring
    /// membawa kecepatan: posisi mengalir lewat batas baris, sehingga pusat
    /// kolom dan target gulir ikut bergerak mulus.
    pub lyric_anim_pos: Spring,
    pub mini_prev_lyric: Option<String>,
    pub mini_curr_lyric: Option<String>,
    pub mini_lyric_at: Instant,
    pub(crate) bands: AudioHandle,
    pub levels: [f32; 4],

    pub banner: Option<(String, Instant)>,
    pub(crate) save_at: Option<Instant>,
    pub lang: Lang,
    /// Buka jendela/screen setelan baru ala Nusa (Image 4).
    pub show_settings: bool,
    pub(crate) settings_tab: SettingsTab,
    /// 0 = dock menganggur (tiga garis), 1 = dock terbuka penuh.
    ///
    /// Dianimasikan, bukan boolean: tinggi menu navigasi tumbuh ke bawah
    /// dari shell dasar tanpa menggeser atau menyempitkan isi halaman.
    pub dock_open: Spring,
    /// Kapan kursor mulai masuk ke daerah dock. `None` = tidak hover.
    pub(crate) dock_hover_since: Option<Instant>,
    /// Kapan kursor terakhir terlihat di daerah dock. Dipakai menahan dock
    /// tetap terbuka selama kursor bergerak dari strip menuju ikon.
    pub(crate) dock_linger: Instant,
    /// Halaman aktif di notch (0: Music & Lyrics Split, 1: Tasks, 2: Pomodoro, 3: System).
    pub current_page: NotchPage,
    pub pomodoro: PomodoroState,
    pub water: WaterState,
    pub is_charging: bool,
    pub battery_pct: u8,
    pub last_power_check: Instant,
    pub last_pomodoro_tick: Instant,
    pub(crate) idle_since: Option<Instant>,
    pub(crate) trimmed: bool,
    pub(crate) suppress_hover: bool,
    pub agent: noktah_core::model::AgentState,
    pub mascot: crate::presentation::mascot::MascotState,
    pub pending_permission: Option<(
        String,
        std::sync::mpsc::SyncSender<noktah_core::model::PermissionDecision>,
    )>,
    pub ipc: crate::infrastructure::ipc::IpcHandle,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NotchPage {
    Music = 0,
    Tasks = 1,
    Pomodoro = 2,
    System = 3,
    NowPlaying = 4,
    Water = 5,
    Ask = 6,
    Home = 7,
    Agent = 8,
}

impl NotchPage {
    /// Halaman yang bisa dicapai lewat tombol chevron, **dalam urutan yang
    /// sama persis dengan ikon di bilah bawah**.
    ///
    /// Sebelumnya ada dua sumber urutan yang berbeda: bilah bawah menampilkan
    /// Music, Focus, Tasks, System, Ask sementara `prev()`/`next()` memutar
    /// Music, Tasks, Pomodoro, System. Menekan chevron kanan dari Music
    /// melompati Focus dan mendarat di Tasks — terlihat seperti tombol yang
    /// mengacak urutan. Satu daftar, satu sumber kebenaran.
    pub const SWITCHER: [NotchPage; 7] = [
        NotchPage::Home,
        NotchPage::Music,
        NotchPage::Pomodoro,
        NotchPage::Tasks,
        NotchPage::System,
        NotchPage::Ask,
        NotchPage::Agent,
    ];

    /// Jumlah halaman yang ikut siklus chevron.
    pub const COUNT: usize = Self::SWITCHER.len();

    /// Posisi halaman di dalam `SWITCHER`. Halaman sekunder (NowPlaying,
    /// Water) tidak ada di daftar; `None` membuat chevron memperlakukannya
    /// sebagai "kembali ke Music".
    fn switcher_index(&self) -> Option<usize> {
        Self::SWITCHER.iter().position(|p| p == self)
    }

    /// Halaman sebelumnya dalam siklus. Halaman sekunder kembali ke Music.
    pub fn prev(&self) -> Self {
        match self.switcher_index() {
            Some(0) | None => NotchPage::Music,
            Some(i) => Self::SWITCHER[i - 1],
        }
    }

    /// Halaman berikutnya dalam siklus. Halaman sekunder kembali ke Music.
    pub fn next(&self) -> Self {
        match self.switcher_index() {
            Some(i) if i + 1 < Self::SWITCHER.len() => Self::SWITCHER[i + 1],
            _ => NotchPage::Music,
        }
    }

    pub fn from_index(idx: usize) -> Self {
        Self::SWITCHER.get(idx).copied().unwrap_or(NotchPage::Music)
    }

    /// Indeks halaman di dalam `SWITCHER`, untuk menyorot ikon yang aktif.
    pub fn index_of(&self) -> Option<usize> {
        self.switcher_index()
    }

    pub fn icon(&self) -> icons::Icon {
        match self {
            NotchPage::Music => icons::Icon::Music,
            NotchPage::Tasks => icons::Icon::Tasks,
            NotchPage::Pomodoro => icons::Icon::Focus,
            NotchPage::System => icons::Icon::System,
            NotchPage::NowPlaying => icons::Icon::Music,
            NotchPage::Water => icons::Icon::Water,
            NotchPage::Home => icons::Icon::Home,
            NotchPage::Ask => icons::Icon::AI,
            NotchPage::Agent => icons::Icon::Bolt,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LyricsStatus {
    Idle,
    Searching,
    Found,
    NotFound,
}

impl Noktah {
    pub fn is_lyrics_synced(&self) -> bool {
        !self.lyrics.is_empty() && (self.lyrics_synced || self.lyrics.iter().any(|l| l.time >= 0.0))
    }
    pub fn new(renderer: String, hover: Arc<Hover>, ctx: &egui::Context) -> Self {
        let mut settings: Settings = store::load("settings.json");
        let force_anim = settings.force_animation;
        let settings_minimal = settings.minimal_notch;
        settings.sanitize();
        let theme = Theme::new(settings.accent, settings.opacity);
        let lang = settings.language;
        let start_open = settings.start_expanded;
        let todos: Vec<Todo> = store::load("tasks.json");
        let chat: Vec<ChatMessage> = store::load("chat.json");
        let pomodoro: PomodoroState = store::load("pomodoro.json");
        let water: WaterState = store::load("water.json");
        let (is_charging, battery_pct) = crate::infrastructure::os::power_status();
        let pill_w = settings.mini_width(false, false);
        let lyrics_enabled = settings.lyrics_enabled;

        // Pastikan entri autostart di registry selalu mutakhir jika diaktifkan.
        if settings.start_with_windows {
            let _ = crate::infrastructure::os::set_autostart(true);
        }

        hover.expanded.store(start_open, Ordering::Relaxed);

        Self {
            open: Spring::from_spec(if start_open { 1.0 } else { 0.0 }, MORPH_OPEN_W),
            open_h: Spring::from_spec(if start_open { 1.0 } else { 0.0 }, MORPH_OPEN_H),
            hovered: start_open,
            hover,
            renderer,
            frames: 0,
            last_frame: Instant::now(),
            anim: Instant::now(),
            seek_at: None,
            settings,
            theme,
            show_settings: false,
            settings_tab: SettingsTab::General,
            dock_open: Spring::from_spec(0.0, MORPH_DOCK),
            dock_hover_since: None,
            dock_linger: Instant::now(),
            current_page: NotchPage::Music,
            pomodoro,
            water,
            is_charging,
            battery_pct,
            last_power_check: Instant::now(),
            last_pomodoro_tick: Instant::now(),
            sys: SysSampler::default(),
            stats_at: Instant::now(),
            todos,
            todo_input: String::new(),
            todo_due: String::new(),
            media: MediaState::default(),
            lyrics: Vec::new(),
            lyrics_synced: false,
            lyrics_status: LyricsStatus::Idle,
            lyrics_provider: String::new(),
            music: music::spawn(ctx.clone()),
            lyrics_client: lyrics_client::spawn(ctx.clone()),
            art: None,
            art_for: String::new(),
            lyrics_for: String::new(),
            chat,
            chat_input: String::new(),
            ai: ai::spawn(ctx.clone()),
            api_key: secret::load(),
            api_key_input: String::new(),
            ai_busy: false,
            weather: weather::spawn(ctx.clone()),
            weather_now: None,
            weather_city: String::new(),
            quota: None,
            quota_worker: crate::infrastructure::quota::spawn(ctx.clone()),
            applied_glass: None,
            reduce_motion: !crate::infrastructure::os::animations_enabled() && !force_anim,
            minimal_mix: Spring::from_spec(
                if settings_minimal { 1.0 } else { 0.0 },
                MORPH_MINIMAL_OUT,
            ),
            transisi_at: None,
            peek_until: None,
            hover_since: None,
            glass_tex: None,
            lyric_manual_at: Instant::now() - std::time::Duration::from_secs(10),
            album_accent: None,
            scrubber_expand: Spring::with(0.0, 280.0, 22.0),
            lyrics_anim: Spring::from_spec(
                if lyrics_enabled { 1.0 } else { 0.0 },
                MORPH_LYRICS,
            ),
            pill: Spring::from_spec(pill_w, MORPH_PILL),
            last_active_lyric: None,
            lyric_line_active_at: Instant::now() - std::time::Duration::from_secs(10),
            lyric_scroll_pos: 0.0,
            lyric_target_scroll: 0.0,
            lyric_anim_pos: Spring::from_spec(0.0, MORPH_LYRIC_LINE),
            mini_prev_lyric: None,
            mini_curr_lyric: None,
            mini_lyric_at: Instant::now() - std::time::Duration::from_secs(10),
            bands: crate::infrastructure::audio::spawn(ctx.clone()),
            levels: [0.0; 4],
            banner: None,
            save_at: None,
            lang,
            idle_since: None,
            trimmed: false,
            suppress_hover: false,
            agent: noktah_core::model::AgentState::default(),
            mascot: crate::presentation::mascot::MascotState::new(),
            pending_permission: None,
            ipc: crate::infrastructure::ipc::spawn(ctx.clone()),
        }
    }

    pub fn tr(&self, key: &str) -> &'static str {
        i18n::t(self.lang, key)
    }
}
