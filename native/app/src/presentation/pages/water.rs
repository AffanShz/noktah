//! Halaman hidrasi — pola `.focus` dari prototipe: cincin di kiri,
//! status + baris aksi pil di kanan.

use crate::app::{Noktah, NotchPage};
use crate::presentation::{icons::Icon, style, widgets};
use eframe::egui::{self, Color32, Rect, Rounding, Sense};
use noktah_core::format;

impl Noktah {
    /// Halaman 3: Water Reminder Page.
    ///
    /// Hydration. The ring now actually shows progress — previously two
    /// `circle_stroke` calls drew a full circle in both the track and the
    /// accent colour, so the ring looked complete at any intake.
    pub(crate) fn draw_page_water(&mut self, ui: &mut egui::Ui, area: Rect) {
        let theme = self.theme;
        let goal = self.water.goal_ml.max(1);
        let taken = self.water.taken_ml;
        let frac = self.water.progress();
        let bottles = self.water.bottles();
        let bottle_ml = self.water.bottle_ml.max(1);
        let blue = Color32::from_rgb(0x4D, 0xA3, 0xFF);

        let header_h = widgets::page_header(ui, "HYDRATION", &theme, |ui| {
            ui.label(style::cap(format!("Goal {goal} ml"), theme.dim));
        });

        let body_h = (area.height() - header_h).max(60.0);
        let ring = body_h.min(76.0).max(64.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = style::S3;

            widgets::progress_ring(ui, ring, frac, blue, &theme, |ui| {
                let (tengah, _) = ui.allocate_exact_size(
                    egui::vec2(ring * 0.42, ring * 0.42),
                    Sense::hover(),
                );
                widgets::glyph(ui.painter(), Icon::Water, tengah, blue, 1.8);
            });

            let info_w = (area.width() - ring - style::S3).max(120.0);
            ui.allocate_ui_with_layout(
                egui::vec2(info_w, ring),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.add_space(1.0);

                    // Angka utama status air.
                    ui.label(
                        egui::RichText::new(format!("{} / {} ml", taken, goal))
                            .size(17.0)
                            .strong()
                            .color(theme.ink),
                    );

                    // Baris keterangan berubah mengikuti progres: di bawah
                    // target ia memberi tahu sisa; setelah tercapai berhenti
                    // menagih.
                    let sub = if frac >= 1.0 {
                        "Goal reached for today".to_string()
                    } else {
                        format!("{} × {} ml · {} ml to go", bottles, bottle_ml, (goal - taken).max(0))
                    };
                    ui.label(style::subtitle(sub, theme.dim));

                    ui.add_space((ring - 66.0).max(0.0));

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = style::S2;
                        if widgets::pill_button(
                            ui,
                            &format!("Drink {} ml", bottle_ml),
                            true,
                            &theme,
                        ) {
                            self.water.drink();
                            self.save_water();
                        }

                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if widgets::pill_button(ui, "Reset", false, &theme) {
                                    self.water.reset();
                                    self.save_water();
                                }
                            },
                        );
                    });
                },
            );
        });
    }
}
