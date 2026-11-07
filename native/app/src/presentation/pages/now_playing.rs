//! Halaman mini player satu kolom — dipakai sebagai tampilan ringkas ketika
//! pengguna tidak butuh kolom lirik.
//!
//! Geometrinya sengaja identik dengan kolom kiri halaman Musik, jadi tombol
//! transport di kedua halaman tidak pernah berbeda posisi.

use crate::app::{Noktah, NotchPage};
use crate::presentation::{icons::Icon, style, widgets};
use eframe::egui::{self, Color32, Rect, Rounding, Sense, Stroke};
use noktah_core::format;

impl Noktah {
    /// Mini player satu kolom (dipakai tombol lirik di halaman Musik).
    pub(crate) fn draw_page_now_playing(&mut self, ui: &mut egui::Ui, area: Rect) {
        let theme = self.theme;

        const ART: f32 = 44.0;
        const SCRUBBER: f32 = 12.0;
        const PLAY: f32 = 36.0;
        const TOTAL_H: f32 = ART + SCRUBBER + PLAY; // 92.0 px
        const IB: f32 = 28.0;
        let body_h = area.height().max(60.0);
        let gap_y = ((body_h - TOTAL_H) / 2.0).clamp(4.0, 10.0);

        ui.spacing_mut().item_spacing.y = 0.0;

        // ---- Baris album ----
        const ART_R: f32 = 10.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = style::S2;

            if let Some(art_tex) = &self.art {
                let img = egui::Image::new(art_tex)
                    .fit_to_exact_size(egui::vec2(ART, ART))
                    .rounding(Rounding::same(ART_R));
                let resp = ui.add(img);
                ui.painter().rect_stroke(
                    resp.rect,
                    Rounding::same(ART_R),
                    Stroke::new(0.5, Color32::from_white_alpha(64)),
                );
            } else {
                let (kotak, _) = ui.allocate_exact_size(egui::vec2(ART, ART), Sense::hover());
                ui.painter().rect_filled(
                    kotak,
                    Rounding::same(ART_R),
                    Color32::from_white_alpha(31),
                );
                widgets::glyph(
                    ui.painter(),
                    Icon::Music,
                    Rect::from_center_size(kotak.center(), egui::vec2(22.0, 22.0)),
                    Color32::from_white_alpha(235),
                    1.7,
                );
            }

            let meta_w = (area.width() - ART - style::S2).max(40.0);
            ui.allocate_ui_with_layout(
                egui::vec2(meta_w, ART),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.add_space(style::S1);
                    widgets::marquee_title(
                        ui,
                        &self.media.title,
                        egui::FontId::proportional(style::T_TITLE_LG),
                        theme.ink,
                        meta_w,
                    );

                    let sub = if self.media.album.is_empty() {
                        self.media.artist.clone()
                    } else {
                        format!("{} — {}", self.media.artist, self.media.album)
                    };
                    let sub = widgets::ellipsis_sized(ui, &sub, meta_w, style::T_SUBTITLE);
                    ui.label(style::subtitle(sub, theme.dim));
                },
            );
        });

        ui.add_space(gap_y);

        // ---- Scrubber ----
        let dur_s = (self.media.duration_ms as f32 / 1000.0).max(1.0);
        // Saat seek belum dikonfirmasi SMTC, tahan thumb di target supaya
        // tidak memantul ke posisi lama (siklus ≤ 500 ms).
        let pos_disp = if let Some((t, _)) = self.seek_at {
            (t as f32 / 1000.0).clamp(0.0, dur_s)
        } else {
            (self.media.position_ms as f32 / 1000.0).clamp(0.0, dur_s)
        };
        let mut pos_s = pos_disp;
        let elapsed = format::mmss(pos_s as i64);
        let remain = format::mmss((dur_s - pos_s).max(0.0) as i64);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = style::S1;
            ui.add_sized(
                egui::vec2(32.0, 12.0),
                egui::Label::new(style::micro_mono(elapsed.clone(), theme.dim)),
            );

            let track_w = (area.width() - 32.0 * 2.0 - style::S1 * 2.0).max(40.0);
            let (jalur_rect, resp) =
                ui.allocate_exact_size(egui::vec2(track_w, 12.0), Sense::click_and_drag());
            let frac = (pos_s / dur_s).clamp(0.0, 1.0);

            let hover = resp.hovered() || resp.dragged();
            let jalur_h = if hover { 5.0 } else { 3.5 };
            let jalur = Rect::from_center_size(
                jalur_rect.center(),
                egui::vec2(jalur_rect.width(), jalur_h),
            );

            ui.painter().rect_filled(
                jalur,
                Rounding::same(jalur_h / 2.0),
                Color32::from_white_alpha(51),
            );
            ui.painter().rect_filled(
                Rect::from_min_size(jalur.min, egui::vec2(jalur.width() * frac, jalur.height())),
                Rounding::same(jalur_h / 2.0),
                Color32::from_white_alpha(230),
            );

            if hover {
                ui.painter().circle_filled(
                    egui::pos2(jalur.left() + jalur.width() * frac, jalur.center().y),
                    5.0,
                    Color32::WHITE,
                );
            }

            if (resp.dragged() || resp.clicked()) && resp.interact_pointer_pos().is_some() {
                let p = resp.interact_pointer_pos().unwrap();
                let f = ((p.x - jalur.left()) / jalur.width()).clamp(0.0, 1.0);
                pos_s = f * dur_s;
                self.lyric_manual_at = std::time::Instant::now();
            }
            if resp.drag_stopped() || resp.clicked() {
                // Klik langsung DAN lepas drag sama-sama melompat ke posisi
                // kursor. Ditahan di `seek_at` supaya thumb tidak memantul
                // balik sebelum SMTC mengonfirmasi.
                let target_ms = (pos_s * 1000.0) as i64;
                self.seek_at = Some((target_ms, std::time::Instant::now()));
                self.media_cmd(crate::infrastructure::music::Cmd::Seek(target_ms));
                self.lyric_manual_at = std::time::Instant::now();
            }

            ui.add_sized(
                egui::vec2(32.0, 12.0),
                egui::Label::new(style::micro_mono(format!("-{remain}"), theme.dim)),
            );
        });

        ui.add_space(gap_y);

        // ---- Transport: 3-slot simetris bilateral ----
        let center_trio_w = IB + 16.0 + PLAY + 16.0 + IB;
        let slot_side_w = ((area.width() - center_trio_w) * 0.5).max(IB);
        let (baris, _) = ui.allocate_exact_size(egui::vec2(area.width(), PLAY), Sense::hover());

        let slot_left = Rect::from_min_size(baris.min, egui::vec2(slot_side_w, PLAY));
        let slot_center = Rect::from_min_size(
            egui::pos2(baris.left() + slot_side_w, baris.top()),
            egui::vec2(center_trio_w, PLAY),
        );
        let slot_right = Rect::from_min_size(
            egui::pos2(baris.right() - slot_side_w, baris.top()),
            egui::vec2(slot_side_w, PLAY),
        );

        // Slot kiri: Lyrics rata kiri
        let mut ui_left = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(slot_left)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        ui_left.spacing_mut().item_spacing.x = 0.0;
        let lyr_on = self.settings.lyrics_enabled;
        let lyr_icon = if lyr_on { Icon::LyricsOn } else { Icon::LyricsOff };
        if widgets::icon_button(&mut ui_left, lyr_icon, IB, lyr_on, &theme) {
            self.settings.lyrics_enabled = !lyr_on;
            self.touch_settings();
        }

        // Slot tengah: Prev, Play, Next dipusatkan di area.width() * 0.5
        let mut ui_center = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(slot_center)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        ui_center.spacing_mut().item_spacing.x = 16.0;

        if widgets::icon_button(&mut ui_center, Icon::Prev, IB, false, &theme) {
            self.media_cmd(crate::infrastructure::music::Cmd::Prev);
        }

        let play_icon = if self.media.playing {
            Icon::Pause
        } else {
            Icon::Play
        };
        if widgets::play_button(&mut ui_center, play_icon, PLAY, &theme) {
            self.media_cmd(crate::infrastructure::music::Cmd::PlayPause);
        }

        if widgets::icon_button(&mut ui_center, Icon::Next, IB, false, &theme) {
            self.media_cmd(crate::infrastructure::music::Cmd::Next);
        }

        // Slot kanan: Speaker rata kanan
        let mut ui_right = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(slot_right)
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
        );
        ui_right.spacing_mut().item_spacing.x = 0.0;
        if widgets::icon_button(&mut ui_right, Icon::Speaker, IB, false, &theme) {
            crate::infrastructure::os::toggle_volume_mute();
        }
    }
}
