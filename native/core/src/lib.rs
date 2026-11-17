//! Logika murni Noktah.
//!
//! Tidak ada `egui` dan tidak ada `windows` di sini, supaya seluruh aturan
//! permainan bisa diuji di mesin apa pun tanpa membuka jendela.

pub mod format;
pub mod i18n;
pub mod lrc;
pub mod lyrics;
pub mod model;
pub mod pill;
pub mod reminders;
pub mod settings;

pub use model::{
    ChatMessage, MediaState, PomodoroMode, PomodoroState, Scratchpad, Todo, WaterState, WeatherNow,
};
pub use settings::{Accent, DefaultPage, Lang, MiniPreset, Provider, Settings};
