//! Halaman Sistem — tiga kartu `.tile` dari prototipe `noktah-redesign.html`:
//! cuaca 1.2fr, CPU 1fr, memori 1fr.
//!
//! Perbedaan utama dari versi sebelumnya: cuaca mendapat kartunya sendiri
//! (sebelumnya baris mendatar yang menabrak tepi), dan kedua metrik memakai
//! sparkline area di dasar kartu alih-alih satu grafik bersama.

use crate::app::{Noktah, NotchPage};
use crate::presentation::{icons::Icon, style, widgets};
use eframe::egui::{self, Color32, Rect, Rounding, Sense, Stroke};
use noktah_core::format;

impl Noktah {
    /// Halaman 5: System & Weather Page
    ///
    /// Three tiles on one row: weather, CPU, memory. The weather card needs
    /// the extra width (temperature is the largest type on the page), so it
    /// gets 1.2fr while the two metrics share the rest evenly.
    pub(crate) fn draw_page_system(&mut self, ui: &mut egui::Ui, area: Rect) {
        let theme = self.theme;

        let header_h = widgets::page_header(ui, "SYSTEM", &theme, |ui| {
            if widgets::chip_button(ui, "Settings", false, &theme) {
                self.show_settings = true;
            }
        });

        const FOOTER_H: f32 = 18.0;
        let body_h = (area.height() - header_h - FOOTER_H - style::S1).clamp(50.0, 64.0);

        // `.sys{grid-template-columns:1.2fr 1fr 1fr;gap:10px}`
        let gap = style::S2;
        let unit = ((area.width() - gap * 2.0) / 3.2).max(60.0);
        let wx_w = (unit * 1.2).floor();
        let tile_w = unit.floor();

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            // ---- Kartu cuaca (`.tile.wx`) ----
            let (wx_rect, _) = ui.allocate_exact_size(egui::vec2(wx_w, body_h), Sense::hover());
            ui.painter()
                .rect_filled(wx_rect, Rounding::same(style::R_CARD), theme.surface);

            if let Some(cuaca) = &self.weather_now {
                let pad = 10.0;
                let left = wx_rect.left() + pad;

                // Ikon kondisi, warna aksen (`.wx .row1`).
                widgets::glyph(
                    ui.painter(),
                    crate::infrastructure::weather::icon_for(cuaca.code, cuaca.is_day),
                    Rect::from_min_size(
                        egui::pos2(left, wx_rect.top() + 8.0),
                        egui::vec2(18.0, 18.0),
                    ),
                    theme.accent,
                    1.5,
                );

                // Suhu (`.temp`), di samping ikon.
                ui.painter().text(
                    egui::pos2(left + 24.0, wx_rect.top() + 6.0),
                    egui::Align2::LEFT_TOP,
                    format!("{:.0}°", cuaca.temp_c),
                    egui::FontId::proportional(style::T_STAT),
                    theme.ink,
                );

                // Kondisi 12 px (`.cond`).
                ui.painter().text(
                    egui::pos2(left, wx_rect.top() + 32.0),
                    egui::Align2::LEFT_TOP,
                    crate::infrastructure::weather::label(cuaca.code),
                    egui::FontId::proportional(style::T_SUBTITLE).into(),
                    theme.ink,
                );

                // Detail tempat di dasar kartu
                ui.painter().text(
                    egui::pos2(left, wx_rect.top() + 48.0),
                    egui::Align2::LEFT_TOP,
                    &cuaca.place,
                    style::caption(),
                    theme.dim,
                );
            } else {
                ui.painter().text(
                    wx_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "—",
                    egui::FontId::proportional(style::T_STAT),
                    theme.faint,
                );
            }

            ui.add_space(gap);

            // ---- Kartu CPU ----
            let data: Vec<f32> = self.sys.history.iter().copied().collect();
            widgets::spark_tile(
                ui,
                tile_w,
                body_h,
                "CPU",
                &format!("{:.0}%", self.sys.cpu),
                None,
                &data,
                0.0,
                100.0,
                theme.accent,
                &theme,
            );

            ui.add_space(gap);

            // ---- Kartu memori ----
            let ram_used = self.sys.ram_used as f32 / 1_073_741_824.0;
            let ram_total = self.sys.ram_total as f32 / 1_073_741_824.0;
            // Riwayat RAM tidak disimpan sampler; pakai garis datar supaya
            // kartunya tidak kosong tanpa mengarang data.
            let datar = vec![ram_used; 2];
            widgets::spark_tile(
                ui,
                tile_w,
                body_h,
                "Memory",
                &format!("{ram_used:.1}"),
                Some(&format!("/ {ram_total:.1} GB")),
                &datar,
                (ram_used - 1.0).max(0.0),
                (ram_used + 1.0).max(1.0),
                theme.accent,
                &theme,
            );
        });

        ui.add_space(style::S1);

        // ---- Footer: fakta sekunder, satu baris ----
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = style::S3;
            ui.label(style::cap(
                format!("Uptime {}", format::uptime_text(self.sys.uptime_secs)),
                theme.faint,
            ));
            ui.label(style::cap(format!("{} cores", self.sys.cores), theme.faint));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut renderer = self.renderer.clone();
                if renderer.chars().count() > 22 {
                    renderer = renderer.chars().take(22).collect::<String>() + "…";
                }
                ui.label(style::cap(renderer, theme.faint));
            });
        });
    }
}
