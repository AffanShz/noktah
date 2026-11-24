//! Bentuk data yang dipakai bersama UI dan penyimpanan.

use serde::{Deserialize, Serialize};

/// Satu tugas. `due_at` dan `notified_at` dalam milidetik epoch.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Todo {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notified_at: Option<i64>,
}

impl Todo {
    pub fn new(id: impl Into<String>, text: impl Into<String>, created_at: i64) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            done: false,
            created_at,
            due_at: None,
            notified_at: None,
        }
    }
}

/// Catatan cepat (scratchpad) persisten dengan stempel waktu pembaruan terakhir.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Scratchpad {
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ChatMessage {
    pub id: String,
    pub role: Role,
    pub content: String,
    pub at: i64,
}

/// Keadaan pemutar media, diisi dari SMTC.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MediaState {
    pub active: bool,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub source: String,
    pub playing: bool,
    pub position_ms: i64,
    pub duration_ms: i64,
}

impl MediaState {
    /// Bagian "Artis — Judul" untuk ditampilkan di pill.
    pub fn label(&self) -> String {
        if self.artist.is_empty() {
            self.title.clone()
        } else if self.title.is_empty() {
            self.artist.clone()
        } else {
            format!("{} — {}", self.artist, self.title)
        }
    }

    /// 0.0–1.0; nol kalau durasi belum diketahui.
    pub fn progress(&self) -> f32 {
        if self.duration_ms <= 0 {
            0.0
        } else {
            (self.position_ms as f32 / self.duration_ms as f32).clamp(0.0, 1.0)
        }
    }
}

/// Cuaca terkini, sudah diterjemahkan dari kode Open-Meteo.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeatherNow {
    pub place: String,
    pub temp_c: f32,
    pub feels_c: f32,
    pub humidity: u8,
    pub wind_kmh: f32,
    pub code: u8,
    pub is_day: bool,
    pub fetched_at: i64,
    /// Prakiraan harian ringkas (maks 5 hari) untuk kartu forecast.
    pub days: Vec<ForecastDay>,
    /// Strip per jam (maks 6 titik) mulai dari jam sekarang.
    pub hours: Vec<ForecastHour>,
}

/// Satu baris prakiraan harian.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ForecastDay {
    /// Label hari singkat, mis. "Sen".
    pub label: String,
    pub code: u8,
    pub max_c: f32,
    pub min_c: f32,
}

/// Satu titik di strip per jam.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ForecastHour {
    /// "Kini" untuk jam berjalan, selain itu "19", "20", ...
    pub label: String,
    pub code: u8,
    pub temp_c: f32,
    pub is_now: bool,
}

/// Mode Pomodoro 
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PomodoroMode {
    Focus,
    ShortBreak,
    LongBreak,
}

impl PomodoroMode {
    pub fn duration_secs(&self) -> u32 {
        match self {
            PomodoroMode::Focus => 25 * 60,
            PomodoroMode::ShortBreak => 5 * 60,
            PomodoroMode::LongBreak => 15 * 60,
        }
    }

    pub fn duration_secs_custom(&self, focus_m: u32, break_m: u32) -> u32 {
        match self {
            PomodoroMode::Focus => focus_m.max(1) * 60,
            PomodoroMode::ShortBreak => break_m.max(1) * 60,
            PomodoroMode::LongBreak => (break_m * 3).max(15) * 60,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            PomodoroMode::Focus => "Fokus",
            PomodoroMode::ShortBreak => "Istirahat",
            PomodoroMode::LongBreak => "Panjang",
        }
    }
}

/// Keadaan timer fokus / Pomodoro
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PomodoroState {
    pub mode: PomodoroMode,
    pub running: bool,
    pub remaining_secs: u32,
    pub sessions_today: u32,
    #[serde(default)]
    pub last_date: String,
    /// Riwayat jumlah sesi selesai harian (maks 7 hari terakhir: `(YYYY-MM-DD, count)`).
    #[serde(default)]
    pub history: Vec<(String, u32)>,
}

impl Default for PomodoroState {
    fn default() -> Self {
        Self {
            mode: PomodoroMode::Focus,
            running: false,
            remaining_secs: 25 * 60,
            sessions_today: 0,
            last_date: String::new(),
            history: Vec::new(),
        }
    }
}

impl PomodoroState {
    pub fn set_mode(&mut self, mode: PomodoroMode) {
        self.mode = mode;
        self.running = false;
        self.remaining_secs = mode.duration_secs();
    }

    pub fn set_mode_with(&mut self, mode: PomodoroMode, focus_m: u32, break_m: u32) {
        self.mode = mode;
        self.running = false;
        self.remaining_secs = mode.duration_secs_custom(focus_m, break_m);
    }

    pub fn toggle_run(&mut self) {
        self.running = !self.running;
    }

    pub fn reset(&mut self) {
        self.running = false;
        self.remaining_secs = self.mode.duration_secs();
    }

    pub fn reset_with(&mut self, focus_m: u32, break_m: u32) {
        self.running = false;
        self.remaining_secs = self.mode.duration_secs_custom(focus_m, break_m);
    }

    pub fn tick(&mut self) -> bool {
        if self.running && self.remaining_secs > 0 {
            self.remaining_secs -= 1;
            if self.remaining_secs == 0 {
                self.running = false;
                if self.mode == PomodoroMode::Focus {
                    self.sessions_today += 1;
                }
                return true; // Completed
            }
        }
        false
    }

    /// Reset counter sesi harian saat tanggal kalender berganti sambil
    /// mengarsipkan total sesi hari sebelumnya ke riwayat 7 hari.
    pub fn check_daily_reset(&mut self, today: &str) -> bool {
        if !self.last_date.is_empty() && self.last_date != today {
            if let Some(entry) = self.history.iter_mut().find(|(d, _)| d == &self.last_date) {
                entry.1 = self.sessions_today;
            } else {
                self.history.push((self.last_date.clone(), self.sessions_today));
            }
            if self.history.len() > 7 {
                let drop_n = self.history.len() - 7;
                self.history.drain(0..drop_n);
            }
            self.sessions_today = 0;
            self.last_date = today.to_string();
            true
        } else if self.last_date.is_empty() {
            self.last_date = today.to_string();
            false
        } else {
            false
        }
    }

    /// Total sesi fokus selesai dalam jendela 7 hari terakhir (termasuk hari ini).
    pub fn weekly_total(&self) -> u32 {
        let past: u32 = self.history.iter().rev().take(6).map(|(_, n)| *n).sum();
        self.sessions_today.saturating_add(past)
    }
}

/// Keadaan pengingat hidrasi. Semua volume dalam mililiter.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WaterState {
    pub goal_ml: u32,
    pub taken_ml: u32,
    pub bottle_ml: u32,
    #[serde(default)]
    pub last_date: String,
}

impl Default for WaterState {
    fn default() -> Self {
        Self {
            goal_ml: 2000,
            taken_ml: 0,
            bottle_ml: 250,
            last_date: String::new(),
        }
    }
}

impl WaterState {
    /// Satu botol diminum. Tidak dibatasi target: melewati target itu wajar.
    pub fn drink(&mut self) {
        self.taken_ml = self.taken_ml.saturating_add(self.bottle_ml);
    }

    pub fn reset(&mut self) {
        self.taken_ml = 0;
    }

    /// Reset catatan minum harian saat tanggal kalender berganti.
    pub fn check_daily_reset(&mut self, today: &str) -> bool {
        if !self.last_date.is_empty() && self.last_date != today {
            self.taken_ml = 0;
            self.last_date = today.to_string();
            true
        } else if self.last_date.is_empty() {
            self.last_date = today.to_string();
            false
        } else {
            false
        }
    }

    /// 0.0–1.0 untuk cincin progres. Target 0 tidak boleh membagi nol.
    pub fn progress(&self) -> f32 {
        if self.goal_ml == 0 {
            return 0.0;
        }
        (self.taken_ml as f32 / self.goal_ml as f32).clamp(0.0, 1.0)
    }

    /// Jumlah botol utuh yang sudah diminum.
    pub fn bottles(&self) -> u32 {
        if self.bottle_ml == 0 {
            return 0;
        }
        self.taken_ml / self.bottle_ml
    }
}

/// Status subagent yang sedang berjalan.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SubagentInfo {
    pub agent_id: String,
    pub agent_type: String,
    pub status: String,
}

/// Permintaan izin eksekusi tool dari Claude Code.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PermissionRequest {
    pub request_id: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    #[serde(default)]
    pub suggestions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_title: Option<String>,
}

/// Status lifecycle agent Claude Code.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum AgentStatus {
    Idle,
    PromptSubmitted {
        prompt: String,
        timestamp: i64,
    },
    ToolRunning {
        tool: String,
        input: serde_json::Value,
        start_time: i64,
    },
    AwaitingApproval(PermissionRequest),
    Subagents(Vec<SubagentInfo>),
    Completed {
        summary: String,
        last_message: String,
        timestamp: i64,
    },
    Error {
        message: String,
        timestamp: i64,
    },
}

impl AgentStatus {
    pub fn is_awaiting_approval(&self) -> bool {
        matches!(self, AgentStatus::AwaitingApproval(_))
    }
}

impl Default for AgentStatus {
    fn default() -> Self {
        AgentStatus::Idle
    }
}

/// Keputusan izin yang dikirim kembali ke Claude Code.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PermissionDecision {
    pub behavior: String, // "allow" | "deny"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_input: Option<serde_json::Value>,
}

/// Keadaan agent secara keseluruhan yang disimpan di Noktah.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct AgentState {
    pub active: bool,
    pub session_id: String,
    pub session_title: String,
    pub status: AgentStatus,
    pub current_agent_type: String,
    pub permission_mode: String,
    pub subagents: Vec<SubagentInfo>,
    pub logs: Vec<String>,
    pub updated_at: i64,
}

/// Payload mentah yang diterima dari stdin hook Claude Code.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AgentHookPayload {
    pub hook_event_name: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub session_title: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,
    #[serde(default)]
    pub tool_use_id: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub permission_mode: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub agent_type: Option<String>,
    #[serde(default)]
    pub last_assistant_message: Option<String>,
    #[serde(default)]
    pub permission_suggestions: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile: Option<crate::tile::CustomTile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile_subtitle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile_tone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile_progress: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile_ttl_secs: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile_pin_mini: Option<bool>,
}

impl AgentHookPayload {
    /// Konversi payload hook menjadi CustomTile jika memuat informasi tile.
    ///
    /// Mendukung format bersarang (`tile`) dengan override opsional dari field `tile_*`,
    /// maupun format flat `tile_*` langsung, dengan fallback `value` dari
    /// `last_assistant_message` atau `prompt`.
    pub fn into_custom_tile(&self, now_ms: i64) -> Option<crate::tile::CustomTile> {
        let ev = self.hook_event_name.trim().to_lowercase();
        let is_tile_event = matches!(
            ev.as_str(),
            "tile" | "tile-set" | "tile_set" | "custom_tile" | "status_tile"
        );

        let has_tile_indicator = self.tile.is_some()
            || self.tile_id.is_some()
            || self.tile_title.is_some()
            || self.tile_value.is_some()
            || self.tile_subtitle.is_some()
            || self.tile_tone.is_some()
            || self.tile_progress.is_some()
            || self.tile_ttl_secs.is_some()
            || self.tile_pin_mini.is_some()
            || is_tile_event;

        if !has_tile_indicator {
            return None;
        }

        let mut tile = self.tile.clone().unwrap_or_default();

        if let Some(id) = &self.tile_id {
            tile.id = id.clone();
        }
        if let Some(title) = &self.tile_title {
            tile.title = title.clone();
        }
        if let Some(val) = &self.tile_value {
            tile.value = val.clone();
        }
        if let Some(sub) = &self.tile_subtitle {
            tile.subtitle = Some(sub.clone());
        }
        if let Some(tone_str) = &self.tile_tone {
            tile.tone = crate::tile::TileTone::parse(tone_str);
        }
        if let Some(prog) = self.tile_progress {
            tile.progress = Some(prog);
        }
        if let Some(ttl) = self.tile_ttl_secs {
            tile.ttl_secs = Some(ttl);
        }
        if let Some(pin) = self.tile_pin_mini {
            tile.pin_mini = pin;
        }

        if tile.value.is_empty() {
            if let Some(msg) = self
                .last_assistant_message
                .as_deref()
                .or(self.prompt.as_deref())
            {
                if !msg.trim().is_empty() {
                    tile.value = msg.trim().to_string();
                }
            }
        }

        Some(tile.normalized(now_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_menggabungkan_artis_dan_judul() {
        let mut m = MediaState::default();
        m.artist = "Alex Crichton".into();
        m.title = "Stuck on You".into();
        assert_eq!(m.label(), "Alex Crichton — Stuck on You");
    }

    #[test]
    fn label_tanpa_artis_hanya_judul() {
        let mut m = MediaState::default();
        m.title = "Instrumental".into();
        assert_eq!(m.label(), "Instrumental");
    }

    #[test]
    fn progres_nol_saat_durasi_belum_ada() {
        let mut m = MediaState::default();
        m.position_ms = 30_000;
        assert_eq!(m.progress(), 0.0);
    }

    #[test]
    fn progres_dibatasi_satu() {
        let mut m = MediaState::default();
        m.position_ms = 300_000;
        m.duration_ms = 200_000;
        assert_eq!(m.progress(), 1.0);
    }

    #[test]
    fn todo_baru_belum_selesai_dan_tanpa_tenggat() {
        let t = Todo::new("a", "beli susu", 1);
        assert!(!t.done);
        assert!(t.due_at.is_none());
    }

    #[test]
    fn todo_bolak_balik_json() {
        let mut t = Todo::new("a", "beli susu", 1);
        t.due_at = Some(1_700_000_000_000);
        let raw = serde_json::to_string(&t).unwrap();
        let back: Todo = serde_json::from_str(&raw).unwrap();
        assert_eq!(t, back);
    }

    #[test]
    fn field_kosong_tidak_ikut_disimpan() {
        let t = Todo::new("a", "x", 1);
        let raw = serde_json::to_string(&t).unwrap();
        assert!(!raw.contains("due_at"));
    }

    #[test]
    fn minum_menambah_satu_botol() {
        let mut w = WaterState::default();
        assert_eq!(w.taken_ml, 0);
        w.drink();
        assert_eq!(w.taken_ml, w.bottle_ml);
        w.drink();
        assert_eq!(w.bottles(), 2);
    }

    #[test]
    fn reset_mengosongkan_konsumsi() {
        let mut w = WaterState::default();
        w.drink();
        w.reset();
        assert_eq!(w.taken_ml, 0);
        assert_eq!(w.progress(), 0.0);
    }

    #[test]
    fn progres_dibatasi_satu_dan_tahan_target_nol() {
        let mut w = WaterState::default();
        w.taken_ml = w.goal_ml * 2;
        assert_eq!(w.progress(), 1.0);

        w.goal_ml = 0;
        assert_eq!(w.progress(), 0.0);
    }

    #[test]
    fn botol_nol_tidak_membagi_nol() {
        let mut w = WaterState::default();
        w.bottle_ml = 0;
        w.taken_ml = 500;
        assert_eq!(w.bottles(), 0);
    }

    #[test]
    fn water_bolak_balik_json() {
        let mut w = WaterState::default();
        w.drink();
        let raw = serde_json::to_string(&w).unwrap();
        let back: WaterState = serde_json::from_str(&raw).unwrap();
        assert_eq!(w, back);
    }

    #[test]
    fn agent_status_dan_permission_serialization() {
        let req = PermissionRequest {
            request_id: "req-123".into(),
            tool_name: "Bash".into(),
            tool_input: serde_json::json!({"command": "cargo test"}),
            suggestions: vec!["Bash(cargo test)".into()],
            session_title: Some("noktah-workspace".into()),
        };
        let status = AgentStatus::AwaitingApproval(req);
        let raw = serde_json::to_string(&status).unwrap();
        let back: AgentStatus = serde_json::from_str(&raw).unwrap();
        assert_eq!(status, back);

        let err_status = AgentStatus::Error {
            message: "Build failed".into(),
            timestamp: 123456789,
        };
        let err_raw = serde_json::to_string(&err_status).unwrap();
        let err_back: AgentStatus = serde_json::from_str(&err_raw).unwrap();
        assert_eq!(err_status, err_back);

        let decision = PermissionDecision {
            behavior: "allow".into(),
            message: None,
            updated_input: None,
        };
        let dec_raw = serde_json::to_string(&decision).unwrap();
        assert!(dec_raw.contains("\"behavior\":\"allow\""));
        assert!(!dec_raw.contains("message"));
    }

    #[test]
    fn pomodoro_durasi_kustom_dan_reset_with() {
        let mut p = PomodoroState::default();
        assert_eq!(p.remaining_secs, 25 * 60);

        p.set_mode_with(PomodoroMode::Focus, 50, 10);
        assert_eq!(p.remaining_secs, 50 * 60);

        p.set_mode_with(PomodoroMode::ShortBreak, 50, 10);
        assert_eq!(p.remaining_secs, 10 * 60);

        p.set_mode_with(PomodoroMode::LongBreak, 50, 10);
        assert_eq!(p.remaining_secs, 30 * 60); // 10 * 3 = 30 min

        p.toggle_run();
        p.tick();
        assert_eq!(p.remaining_secs, 30 * 60 - 1);
        p.reset_with(50, 10);
        assert_eq!(p.remaining_secs, 30 * 60);
        assert!(!p.running);
    }

    #[test]
    fn reset_harian_otomatis_hidrasi_dan_pomodoro() {
        let mut w = WaterState::default();
        w.drink();
        w.drink();
        assert_eq!(w.taken_ml, 500);

        // Hari pertama diinisialisasi
        assert!(!w.check_daily_reset("2026-11-15"));
        assert_eq!(w.taken_ml, 500);

        // Hari sama tidak mereset
        assert!(!w.check_daily_reset("2026-11-15"));
        assert_eq!(w.taken_ml, 500);

        // Hari baru (2026-11-16) mereset kuota air ke 0
        assert!(w.check_daily_reset("2026-11-16"));
        assert_eq!(w.taken_ml, 0);
        assert_eq!(w.last_date, "2026-11-16");

        let mut p = PomodoroState::default();
        p.sessions_today = 4;
        assert!(!p.check_daily_reset("2026-11-15"));
        assert_eq!(p.sessions_today, 4);

        // Hari baru mereset sessions_today ke 0 dan menyimpan riwayat
        assert!(p.check_daily_reset("2026-11-16"));
        assert_eq!(p.sessions_today, 0);
        assert_eq!(p.last_date, "2026-11-16");
        assert_eq!(p.history, vec![("2026-11-15".to_string(), 4)]);
        assert_eq!(p.weekly_total(), 4);
    }

    #[test]
    fn riwayat_fokus_7_hari_dan_scratchpad_serde() {
        let mut p = PomodoroState::default();
        p.last_date = "2026-11-10".into();
        for day in 11..=18 {
            p.sessions_today = day - 1;
            assert!(p.check_daily_reset(&format!("2026-11-{day:02}")));
        }
        assert_eq!(p.history.len(), 7);
        assert_eq!(p.history[0].0, "2026-11-11");
        p.sessions_today = 2;
        // 6 hari sebelumnya (12..=17) + hari ini (2)
        assert_eq!(p.weekly_total(), 12 + 13 + 14 + 15 + 16 + 17 + 2);

        let note = Scratchpad {
            content: "Catatan cepat".into(),
            updated_at: 1_700_000_000_000,
        };
        let raw = serde_json::to_string(&note).unwrap();
        let back: Scratchpad = serde_json::from_str(&raw).unwrap();
        assert_eq!(note, back);
    }

    #[test]
    fn into_custom_tile_parsing_and_fallbacks() {
        use crate::tile::TileTone;

        // 1. Nested tile object
        let payload1 = AgentHookPayload {
            hook_event_name: "custom_tile".into(),
            tile: Some(crate::tile::CustomTile {
                id: "build".into(),
                title: "Build".into(),
                value: "Running".into(),
                subtitle: Some("job #12".into()),
                tone: TileTone::Warning,
                progress: Some(0.4),
                ttl_secs: Some(120),
                updated_at_ms: 100,
                pin_mini: true,
            }),
            ..Default::default()
        };
        let tile1 = payload1.into_custom_tile(200).expect("should parse nested tile");
        assert_eq!(tile1.id, "build");
        assert_eq!(tile1.title, "Build");
        assert_eq!(tile1.value, "Running");
        assert_eq!(tile1.subtitle.as_deref(), Some("job #12"));
        assert_eq!(tile1.tone, TileTone::Warning);
        assert_eq!(tile1.progress, Some(0.4));
        assert_eq!(tile1.ttl_secs, Some(120));
        assert_eq!(tile1.updated_at_ms, 100);
        assert!(tile1.pin_mini);

        // 2. Nested tile with flat overrides
        let payload2 = AgentHookPayload {
            hook_event_name: "tile".into(),
            tile: Some(tile1.clone()),
            tile_value: Some("Success".into()),
            tile_tone: Some("success".into()),
            tile_progress: Some(1.0),
            ..Default::default()
        };
        let tile2 = payload2.into_custom_tile(300).expect("should apply overrides");
        assert_eq!(tile2.id, "build");
        assert_eq!(tile2.value, "Success");
        assert_eq!(tile2.tone, TileTone::Success);
        assert_eq!(tile2.progress, Some(1.0));

        // 3. Flat fields with fallback to last_assistant_message for value
        let payload3 = AgentHookPayload {
            hook_event_name: "tile".into(),
            tile_id: Some("deploy".into()),
            last_assistant_message: Some("Deployed to prod".into()),
            tile_tone: Some("accent".into()),
            tile_pin_mini: Some(true),
            ..Default::default()
        };
        let tile3 = payload3.into_custom_tile(400).expect("should parse flat fields");
        assert_eq!(tile3.id, "deploy");
        assert_eq!(tile3.title, "deploy");
        assert_eq!(tile3.value, "Deployed to prod");
        assert_eq!(tile3.tone, TileTone::Accent);
        assert!(tile3.pin_mini);
        assert_eq!(tile3.updated_at_ms, 400);

        // 4. Non-tile payload returns None
        let non_tile = AgentHookPayload {
            hook_event_name: "userpromptsubmit".into(),
            prompt: Some("hello".into()),
            ..Default::default()
        };
        assert!(non_tile.into_custom_tile(500).is_none());
    }
}
