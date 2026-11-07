//! Permintaan lompat posisi pemutar.
//!
//! Dipindah dari `tabs.rs` tanpa perubahan perilaku. Satu halaman, satu berkas.

use crate::app::{Noktah, NotchPage};
use crate::presentation::{icons::Icon, style, widgets};
use eframe::egui::{self, Color32, FontId, Id, Rect, Rounding, Sense, Stroke};
use noktah_core::{format, i18n, model::PomodoroMode, model::Todo};

impl Noktah {
    /// Placeholder sampai lapisan SMTC tersedia.
    pub fn request_seek(&mut self, target_ms: i64) {
    self.media_cmd(crate::infrastructure::music::Cmd::Seek(target_ms));
    }
}
