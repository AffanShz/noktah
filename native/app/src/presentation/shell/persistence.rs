//! Sinkronisasi data dan penyimpanan persisten untuk Noktah.
//!
//! Mengatur penyimpanan ter-debounce untuk setelan (400 ms setelah disentuh)
//! serta penyimpanan langsung untuk tugas, obrolan, air, dan pomodoro.

use crate::app::Noktah;
use crate::infrastructure::store;
use crate::presentation::theme::Theme;
use noktah_core::reminders;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

impl Noktah {
    /// Tandai setelan berubah; penulisan dilakukan 400 ms kemudian supaya
    /// menggeser slider tidak menulis file ratusan kali.
    pub fn touch_settings(&mut self) {
        self.lang = self.settings.language;
        self.theme = Theme::from_settings(&self.settings);
        self.glass_dirty = true;
        self.hover
            .minimal
            .store(self.settings.minimal_notch, Ordering::Relaxed);
        self.save_at = Some(Instant::now());
    }

    pub fn save_todos(&mut self) {
        store::save("tasks.json", &self.todos);
    }

    pub fn save_chat(&mut self) {
        store::save("chat.json", &self.chat);
    }

    pub fn save_pomodoro(&mut self) {
        store::save("pomodoro.json", &self.pomodoro);
    }

    pub fn touch_notes(&mut self) {
        self.scratchpad.updated_at = chrono::Local::now().timestamp_millis();
        self.notes_save_at = Some(Instant::now());
    }

    /// Jumlah tugas yang belum selesai, untuk badge di tab Tugas.
    pub fn todo_open_count(&self) -> usize {
        self.todos.iter().filter(|t| !t.done).count()
    }

    pub(crate) fn flush_saves(&mut self, now: Instant) {
        if let Some(at) = self.save_at {
            if now.duration_since(at) > Duration::from_millis(400) {
                store::save("settings.json", &self.settings);
                self.save_at = None;
            }
        }
        if let Some(at) = self.notes_save_at {
            if now.duration_since(at) > Duration::from_millis(400) {
                store::save("notes.json", &self.scratchpad);
                self.notes_save_at = None;
            }
        }
    }

    pub(crate) fn check_reminders(&mut self, now_ms: i64) {
        if self.is_focus_dnd_active() {
            return;
        }
        let mut changed = false;
        for task in reminders::missed_reminders(&self.todos, now_ms) {
            let pesan = format!("{}: {}", self.tr("tasks.reminder_title"), task.text);
            self.banner = Some((pesan, Instant::now()));
            if let Some(t) = self.todos.iter_mut().find(|t| t.id == task.id) {
                t.notified_at = Some(now_ms);
                changed = true;
                crate::infrastructure::logger::info("reminders", "Task reminder triggered");
            }
        }
        if changed {
            self.save_todos();
        }
    }

    pub(crate) fn check_daily_reset(&mut self) {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let mut reset_occurred = false;
        if self.pomodoro.check_daily_reset(&today) {
            self.save_pomodoro();
            reset_occurred = true;
        }
        if reset_occurred {
            self.banner = Some((self.tr("daily.reset_banner").to_string(), Instant::now()));
        }
    }
}
