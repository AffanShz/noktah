//! Halaman Fokus — tata letak `.focus` dari prototipe `noktah-redesign.html`:
//! cincin 120 px di kiri, status + kontrol di kanan.
//!
//! Prototipe menaruh pilihan mode sebagai **segmented control** di baris yang
//! sama dengan tombol Mulai/Reset, bukan deretan chip terpisah. Satu baris
//! aksi berarti mata membaca "mode apa" dan "mulai" sebagai satu kalimat.

use crate::app::{Noktah, NotchPage};
use crate::presentation::{icons::Icon, style, widgets};
use eframe::egui::{self, Color32, Rect, Rounding, Sense};
use noktah_core::model::PomodoroMode;

impl Noktah {
    /// Halaman 4: Pomodoro Focus Timer Page.
    ///
    /// The countdown is the reason this page exists, so it gets the only
    /// display-size type on the page and everything else supports it.
    pub(crate) fn draw_page_pomodoro(&mut self, ui: &mut egui::Ui, area: Rect) {
        let theme = self.theme;
        let rem = self.pomodoro.remaining_secs;
        let total = self.pomodoro.mode.duration_secs().max(1);
        let frac = 1.0 - (rem as f32 / total as f32).clamp(0.0, 1.0);

        let header_h = widgets::page_header(ui, "FOCUS", &theme, |ui| {
            ui.label(style::cap(
                format!("{} today", self.pomodoro.sessions_today),
                theme.dim,
            ));
        });

        let body_h = (area.height() - header_h).max(60.0);

        // `.ring` menyusut supaya pas di dalam panel kompak baru.
        let ring = body_h.min(76.0).max(64.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = style::S3;

            // Warna cincin menandai paruh siklus: fokus = aksen, istirahat =
            // hijau tenang. Satu isyarat, tanpa label tambahan.
            let ring_color = match self.pomodoro.mode {
                PomodoroMode::Focus => theme.accent,
                _ => Color32::from_rgb(0x4C, 0xD9, 0x64),
            };

            widgets::progress_ring(ui, ring, frac, ring_color, &theme, |ui| {
                let sisa = format!("{:02}:{:02}", rem / 60, rem % 60);
                // `.clock` angka tabular.
                ui.label(
                    egui::RichText::new(sisa)
                        .size((ring * 0.235).max(17.0))
                        .strong()
                        .monospace()
                        .color(theme.ink),
                );
            });

            // Kolom kanan: status di atas, baris aksi di dasar.
            let info_w = (area.width() - ring - style::S3).max(120.0);
            ui.allocate_ui_with_layout(
                egui::vec2(info_w, ring),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.add_space(1.0);

                    // `.fstat` status teks.
                    let status = if rem == 0 {
                        "Session complete"
                    } else if self.pomodoro.running {
                        match self.pomodoro.mode {
                            PomodoroMode::Focus => "Focusing",
                            _ => "On a break",
                        }
                    } else if rem < total {
                        "Paused"
                    } else {
                        "Ready when you are"
                    };
                    ui.label(
                        egui::RichText::new(status)
                            .size(17.0)
                            .strong()
                            .color(theme.ink),
                    );

                    // `.fsub` 13 px label2.
                    let mode_label = match self.pomodoro.mode {
                        PomodoroMode::Focus => "Focus session",
                        PomodoroMode::ShortBreak => "Short break",
                        PomodoroMode::LongBreak => "Long break",
                    };
                    ui.label(style::subtitle(mode_label, theme.dim));

                    // Baris aksi didorong ke dasar supaya sejajar dengan dasar cincin.
                    ui.add_space((ring - 66.0).max(0.0));

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = style::S2;

                        // Segmented control mode.
                        let seg_w = widgets::segmented(
                            ui,
                            &[
                                ("Focus", self.pomodoro.mode == PomodoroMode::Focus),
                                ("Short", self.pomodoro.mode == PomodoroMode::ShortBreak),
                                ("Long", self.pomodoro.mode == PomodoroMode::LongBreak),
                            ],
                            &theme,
                        );
                        if let Some(i) = seg_w {
                            let mode = match i {
                                0 => PomodoroMode::Focus,
                                1 => PomodoroMode::ShortBreak,
                                _ => PomodoroMode::LongBreak,
                            };
                            self.pomodoro.set_mode(mode);
                            self.save_pomodoro();
                        }

                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                let label =
                                    if self.pomodoro.running { "Pause" } else { "Start" };
                                if widgets::pill_button(ui, label, true, &theme) {
                                    self.pomodoro.toggle_run();
                                    self.save_pomodoro();
                                }
                                ui.add_space(style::S2);
                                if widgets::pill_button(ui, "Reset", false, &theme) {
                                    self.pomodoro.reset();
                                    self.save_pomodoro();
                                }
                            },
                        );
                    });
                },
            );
        });

        // Sisa ruang diisi catatan kecil supaya panel tidak terlihat kosong
        // ketika baris aksi tidak berada di dasar.
        let _ = body_h;
        let _ = Rounding::same(style::R_CARD);
        let _ = Sense::hover();
        let _ = Icon::Timer;
    }
}
