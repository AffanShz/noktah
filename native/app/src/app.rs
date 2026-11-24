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
    model::{ChatMessage, MediaState, PomodoroState, Scratchpad, Todo},
    settings::{DefaultPage, Lang, Settings},
};
use std::collections::VecDeque;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

/// Permintaan izin tool dari satu sesi agent yang mengantre secara FIFO.
pub struct QueuedPermission {
    pub request: noktah_core::model::PermissionRequest,
    pub responder: std::sync::mpsc::SyncSender<noktah_core::model::PermissionDecision>,
}

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
    pub scratchpad: Scratchpad,
    pub(crate) notes_save_at: Option<Instant>,
    pub(crate) notes_mode: bool,

    pub media: MediaState,
    /// Waktu sejak sesi media aktif mulai berstatus pause (`!playing`), untuk
    /// meredam jeda singkat `< 350 ms` saat lompat lagu (`Next`/`Prev`).
    pub(crate) media_paused_since: Option<Instant>,
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
    pub updater: crate::infrastructure::updater::UpdaterHandle,
    pub update_status: crate::infrastructure::updater::UpdateStatus,
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
    /// Buka jendela/screen setelan baru sekunder.
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
    /// Halaman aktif di notch (mengikuti varian `NotchPage`: Home, Music, Pomodoro, Tasks, System, Ask, Agent).
    pub current_page: NotchPage,
    pub pomodoro: PomodoroState,
    pub is_charging: bool,
    pub battery_pct: u8,
    pub last_power_check: Instant,
    pub last_pomodoro_tick: Instant,
    pub(crate) idle_since: Option<Instant>,
    pub(crate) trimmed: bool,
    pub(crate) suppress_hover: bool,
    pub(crate) confirm_reset_at: Option<Instant>,
    pub(crate) show_diag_logs: bool,
    pub(crate) diag_logs_cache: String,
    pub agent: noktah_core::model::AgentState,
    pub mascot: crate::presentation::mascot::MascotState,
    pub pending_permissions: VecDeque<QueuedPermission>,
    pub ipc: crate::infrastructure::ipc::IpcHandle,
    pub calendar_events: Vec<noktah_core::CalendarEvent>,
    pub last_calendar_check: Instant,
    pub file_shelf: Vec<crate::presentation::widgets::ShelfFile>,
    pub last_audio_dev_check: Instant,
    pub last_audio_dev_name: Option<String>,
    pub custom_tiles: noktah_core::TileStore,
    pub(crate) system_tiles_mode: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NotchPage {
    Home = 0,
    Music = 1,
    Pomodoro = 2,
    Tasks = 3,
    System = 4,
    Ask = 5,
    Agent = 6,
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
    #[cfg(test)]
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
    #[cfg(test)]
    pub const COUNT: usize = Self::SWITCHER.len();

    pub fn icon(&self) -> icons::Icon {
        match self {
            NotchPage::Music => icons::Icon::Music,
            NotchPage::Tasks => icons::Icon::Tasks,
            NotchPage::Pomodoro => icons::Icon::Focus,
            NotchPage::System => icons::Icon::System,
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
    /// Mengembalikan daftar halaman navigasi yang diaktifkan oleh pengguna di setelan.
    /// Selalu menjamin minimal 1 halaman aktif (fallback ke Home).
    pub fn active_nav_pages(&self) -> Vec<NotchPage> {
        let mut pages = Vec::with_capacity(7);
        if self.settings.nav_home {
            pages.push(NotchPage::Home);
        }
        if self.settings.nav_music {
            pages.push(NotchPage::Music);
        }
        if self.settings.nav_pomodoro {
            pages.push(NotchPage::Pomodoro);
        }
        if self.settings.nav_tasks {
            pages.push(NotchPage::Tasks);
        }
        if self.settings.nav_system {
            pages.push(NotchPage::System);
        }
        if self.settings.nav_ask {
            pages.push(NotchPage::Ask);
        }
        if self.settings.nav_agent {
            pages.push(NotchPage::Agent);
        }

        if pages.is_empty() {
            pages.push(NotchPage::Home);
        }
        pages
    }

    /// Halaman sebelumnya dalam siklus navigasi aktif.
    pub fn prev_nav_page(&self) -> NotchPage {
        let pages = self.active_nav_pages();
        if let Some(pos) = pages.iter().position(|p| *p == self.current_page) {
            if pos == 0 {
                pages[pages.len() - 1]
            } else {
                pages[pos - 1]
            }
        } else {
            pages[0]
        }
    }

    /// Halaman berikutnya dalam siklus navigasi aktif.
    pub fn next_nav_page(&self) -> NotchPage {
        let pages = self.active_nav_pages();
        if let Some(pos) = pages.iter().position(|p| *p == self.current_page) {
            if pos + 1 < pages.len() {
                pages[pos + 1]
            } else {
                pages[0]
            }
        } else {
            pages[0]
        }
    }

    pub fn is_lyrics_synced(&self) -> bool {
        !self.lyrics.is_empty() && (self.lyrics_synced || self.lyrics.iter().any(|l| l.time >= 0.0))
    }

    /// Menentukan halaman tujuan berdasarkan `settings.default_page` dengan
    /// menghormati daftar halaman aktif (`active_nav_pages`). Mengembalikan `None`
    /// jika pengguna memilih mempertahankan halaman terakhir dibuka.
    pub fn resolve_default_page(&self) -> Option<NotchPage> {
        let desired = match self.settings.default_page {
            DefaultPage::LastOpened => return None,
            DefaultPage::Home => NotchPage::Home,
            DefaultPage::Music => NotchPage::Music,
            DefaultPage::Pomodoro => NotchPage::Pomodoro,
            DefaultPage::Tasks => NotchPage::Tasks,
            DefaultPage::System => NotchPage::System,
            DefaultPage::Ask => NotchPage::Ask,
            DefaultPage::Agent => NotchPage::Agent,
        };
        let active = self.active_nav_pages();
        if active.contains(&desired) {
            Some(desired)
        } else {
            Some(active[0])
        }
    }

    /// Apakah mode Fokus DND sedang menahan banner non-esensial.
    pub fn is_focus_dnd_active(&self) -> bool {
        self.settings.focus_dnd
            && self.pomodoro.running
            && self.pomodoro.mode == noktah_core::model::PomodoroMode::Focus
    }

    /// Menyelesaikan permintaan izin terdepan pada antrean FIFO dan mempromosikan
    /// antrean berikutnya (jika ada) tanpa menutup panel.
    pub fn resolve_front_permission(
        &mut self,
        decision: noktah_core::model::PermissionDecision,
        approved_prompt: Option<&str>,
        now_ms: i64,
    ) {
        let Some(front) = self.pending_permissions.pop_front() else {
            return;
        };
        let _ = front.responder.send(decision);

        if let Some(next) = self.pending_permissions.front() {
            self.agent.status =
                noktah_core::model::AgentStatus::AwaitingApproval(next.request.clone());
        } else {
            self.agent.status = match approved_prompt {
                Some(prompt) => noktah_core::model::AgentStatus::PromptSubmitted {
                    prompt: prompt.to_string(),
                    timestamp: now_ms,
                },
                None => noktah_core::model::AgentStatus::Idle,
            };
            self.close_panel();
        }
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
        let scratchpad: Scratchpad = store::load("notes.json");
        let chat: Vec<ChatMessage> = store::load("chat.json");
        let pomodoro: PomodoroState = store::load("pomodoro.json");
        let (is_charging, battery_pct) = crate::infrastructure::os::power_status();
        let pill_w = settings.mini_width(false, false);
        let lyrics_enabled = settings.lyrics_enabled;

        // Autostart sengaja TIDAK disinkronkan di sini: konstruktor ini juga
        // dipanggil harness test, dan dulu itu menulis path harness ke registry
        // `Run`. Sinkronisasi ada di `main.rs` (entri produksi) lewat
        // `os::sync_autostart`.

        hover.expanded.store(start_open, Ordering::Relaxed);

        let mut app = Self {
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
            is_charging,
            battery_pct,
            last_power_check: Instant::now(),
            last_pomodoro_tick: Instant::now(),
            sys: SysSampler::default(),
            stats_at: Instant::now(),
            todos,
            todo_input: String::new(),
            scratchpad,
            notes_save_at: None,
            notes_mode: false,
            media: MediaState::default(),
            media_paused_since: None,
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
            updater: crate::infrastructure::updater::spawn(ctx.clone()),
            update_status: crate::infrastructure::updater::UpdateStatus::Idle,
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
            confirm_reset_at: None,
            show_diag_logs: false,
            diag_logs_cache: String::new(),
            agent: noktah_core::model::AgentState::default(),
            mascot: crate::presentation::mascot::MascotState::new(),
            pending_permissions: VecDeque::new(),
            ipc: crate::infrastructure::ipc::spawn(ctx.clone()),
            calendar_events: Vec::new(),
            last_calendar_check: Instant::now() - std::time::Duration::from_secs(60),
            file_shelf: Vec::new(),
            last_audio_dev_check: Instant::now() - std::time::Duration::from_secs(10),
            last_audio_dev_name: None,
            custom_tiles: noktah_core::TileStore::default(),
            system_tiles_mode: false,
        };
        if let Some(p) = app.resolve_default_page() {
            app.current_page = p;
        }
        app
    }

    pub fn tr(&self, key: &str) -> &'static str {
        i18n::t(self.lang, key)
    }
}
