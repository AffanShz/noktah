//! Halaman Sistem — tiga kartu `.tile` dari prototipe `noktah-redesign.html`:
//! cuaca 1.2fr, CPU 1fr, memori 1fr, serta mode kartu status kustom dari noktah-hook.

use crate::app::Noktah;
use crate::presentation::{style, widgets};
use eframe::egui::{self, Rect, Rounding, Sense};
use noktah_core::format;

impl Noktah {
    /// Halaman 5: System & Weather Page
    ///
    /// Tiga tile pada satu baris: cuaca, CPU, memori.
    /// Atau switcher kartu status kustom bila ada data dari noktah-hook.
    pub(crate) fn draw_page_system(&mut self, ui: &mut egui::Ui, area: Rect) {
        let theme = self.theme;

        if !self.settings.custom_tiles_enabled || self.custom_tiles.is_empty() {
            self.system_tiles_mode = false;
        }

        let mut new_tiles_mode = self.system_tiles_mode;
        let mut clear_tiles = false;
        let show_switcher = self.settings.custom_tiles_enabled && !self.custom_tiles.is_empty();

        let lbl_metrics = self.tr("system.view_metrics").to_string();
        let lbl_tiles = format!("{} ({})", self.tr("system.view_tiles"), self.custom_tiles.len());
        let lbl_clear = self.tr("system.tiles_clear").to_string();

        let header_h = widgets::page_header(ui, "SYSTEM", &theme, |ui| {
            if show_switcher {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = style::S1;
                    let pad_x = ui.spacing().button_padding.x * 2.0;
                    let chip_w = |text: &str| {
                        ui.painter()
                            .layout_no_wrap(text.to_owned(), style::body(), theme.ink)
                            .size()
                            .x
                            + pad_x
                    };
                    let mut total_w = chip_w(&lbl_metrics) + style::S1 + chip_w(&lbl_tiles);
                    if self.system_tiles_mode {
                        total_w += style::S1 + chip_w(&lbl_clear);
                    }
                    let leading = (ui.available_width() - total_w).max(0.0);
                    if leading > 0.0 {
                        ui.add_space(leading);
                    }
                    if widgets::chip_button(ui, &lbl_metrics, !self.system_tiles_mode, &theme) {
                        new_tiles_mode = false;
                    }
                    if widgets::chip_button(ui, &lbl_tiles, self.system_tiles_mode, &theme) {
                        new_tiles_mode = true;
                    }
                    if self.system_tiles_mode {
                        if widgets::chip_button(ui, &lbl_clear, false, &theme) {
                            clear_tiles = true;
                        }
                    }
                });
            }
        });

        if clear_tiles {
            self.custom_tiles.clear();
            self.system_tiles_mode = false;
        } else {
            self.system_tiles_mode = new_tiles_mode;
        }

        const FOOTER_H: f32 = 18.0;
        let body_h = (area.height() - header_h - FOOTER_H - style::S1).clamp(50.0, 64.0);

        if self.system_tiles_mode {
            let mut id_to_remove: Option<String> = None;
            let n = self.custom_tiles.len();

            let draw_tile = |ui: &mut egui::Ui, tile: &noktah_core::CustomTile, card_rect: Rect, id_out: &mut Option<String>| {
                let pad = 8.0;
                ui.painter().rect_filled(card_rect, Rounding::same(style::R_CARD), theme.surface);

                let dot_r = 3.0;
                let dot_pos = egui::pos2(card_rect.left() + pad + dot_r, card_rect.top() + pad + dot_r + 1.0);
                let tone_col = crate::presentation::mini::tone_color(tile.tone, &theme);
                ui.painter().circle_filled(dot_pos, dot_r, tone_col);

                let close_size = egui::vec2(14.0, 14.0);
                let close_rect = Rect::from_min_size(
                    egui::pos2(card_rect.right() - pad - close_size.x, card_rect.top() + pad - 2.0),
                    close_size,
                );
                let close_id = ui.id().with(("tile_close", &tile.id));
                let close_resp = ui.interact(close_rect, close_id, Sense::click());
                let close_col = if close_resp.hovered() { theme.ink } else { theme.faint };
                ui.painter().text(
                    close_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "×",
                    egui::FontId::proportional(12.0),
                    close_col,
                );
                if close_resp.clicked() {
                    *id_out = Some(tile.id.clone());
                }

                let title_left = dot_pos.x + dot_r + 5.0;
                let title_rect = Rect::from_min_max(
                    egui::pos2(title_left, card_rect.top() + pad - 1.0),
                    egui::pos2(close_rect.left() - 2.0, card_rect.top() + pad + 13.0),
                );
                let title_galley = ui.painter().layout_no_wrap(tile.title.clone(), style::caption(), theme.dim);
                ui.painter().with_clip_rect(title_rect).galley(title_rect.min, title_galley, theme.dim);

                let val_font = egui::FontId::proportional(if tile.value.chars().count() > 10 { 14.0 } else { 16.5 });
                let val_rect = Rect::from_min_max(
                    egui::pos2(card_rect.left() + pad, card_rect.top() + pad + 14.0),
                    egui::pos2(card_rect.right() - pad, card_rect.bottom() - pad - 12.0),
                );
                let val_galley = ui.painter().layout_no_wrap(tile.value.clone(), val_font.clone(), tone_col);
                ui.painter().with_clip_rect(val_rect).galley(val_rect.min, val_galley, tone_col);

                if let Some(p) = tile.progress {
                    let bar_h = 2.0;
                    let bar_rect = Rect::from_min_size(
                        egui::pos2(card_rect.left() + pad, card_rect.bottom() - pad - bar_h),
                        egui::vec2(card_rect.width() - pad * 2.0, bar_h),
                    );
                    ui.painter().rect_filled(bar_rect, Rounding::same(1.0), theme.fill2);
                    let fill_w = (bar_rect.width() * p.clamp(0.0, 1.0)).max(0.0);
                    if fill_w > 0.0 {
                        let fill_rect = Rect::from_min_size(bar_rect.min, egui::vec2(fill_w, bar_h));
                        ui.painter().rect_filled(fill_rect, Rounding::same(1.0), tone_col);
                    }
                } else if let Some(sub) = &tile.subtitle {
                    let sub_rect = Rect::from_min_max(
                        egui::pos2(card_rect.left() + pad, card_rect.bottom() - pad - 12.0),
                        egui::pos2(card_rect.right() - pad, card_rect.bottom() - pad + 1.0),
                    );
                    let sub_galley = ui.painter().layout_no_wrap(sub.clone(), style::caption(), theme.faint);
                    ui.painter().with_clip_rect(sub_rect).galley(sub_rect.min, sub_galley, theme.faint);
                }
            };

            let tiles: Vec<noktah_core::CustomTile> = self.custom_tiles.items().to_vec();
            if tiles.is_empty() {
                let (empty_rect, _) = ui.allocate_exact_size(egui::vec2(area.width(), body_h), Sense::hover());
                ui.painter().rect_filled(empty_rect, Rounding::same(style::R_CARD), theme.surface);
                ui.painter().text(
                    empty_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    self.tr("system.tiles_empty"),
                    style::body(),
                    theme.faint,
                );
            } else if n > 3 {
                let tile_w = 140.0;
                egui::ScrollArea::horizontal()
                    .id_salt("custom_tiles_scroll")
                    .max_height(body_h)
                    .auto_shrink([false, false])
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = style::S2;
                            for tile in &tiles {
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(tile_w, body_h), Sense::hover());
                                draw_tile(ui, tile, rect, &mut id_to_remove);
                            }
                        });
                    });
            } else {
                let gap = style::S2;
                let tile_w = ((area.width() - gap * (n as f32 - 1.0).max(0.0)) / (n as f32)).floor();
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    for (i, tile) in tiles.iter().enumerate() {
                        if i > 0 {
                            ui.add_space(gap);
                        }
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(tile_w, body_h), Sense::hover());
                        draw_tile(ui, tile, rect, &mut id_to_remove);
                    }
                });
            }

            if let Some(id) = id_to_remove {
                self.custom_tiles.remove(&id);
                if self.custom_tiles.is_empty() {
                    self.system_tiles_mode = false;
                }
            }
        } else {
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
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::platform::Hover;
    use crate::presentation::layout;
    use noktah_core::tile::{CustomTile, TileTone};
    use noktah_core::Lang;

    #[test]
    fn system_header_switcher_buttons_are_left_to_right_ordered() {
        let ctx = egui::Context::default();
        let hover = Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);
        app.lang = Lang::En;
        app.settings.language = Lang::En;
        app.settings.custom_tiles_enabled = true;
        app.system_tiles_mode = true;
        app.custom_tiles.upsert(
            CustomTile {
                id: "t1".into(),
                title: "Build".into(),
                value: "OK".into(),
                tone: TileTone::Success,
                ..Default::default()
            },
            1_000,
        );

        let area = Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(layout::OPEN_W, layout::OPEN_H),
        );
        let content_rect = layout::content_rect(area, 16.0, style::S2);

        let output = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut page_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(content_rect)
                        .layout(egui::Layout::top_down(egui::Align::LEFT)),
                );
                page_ui.set_clip_rect(content_rect);
                app.draw_page_system(&mut page_ui, content_rect);
            });
        });

        let mut text_positions: Vec<(String, Rect)> = Vec::new();
        let mut rect_shapes: Vec<Rect> = Vec::new();

        fn collect_shapes(
            shape: &egui::epaint::Shape,
            text_positions: &mut Vec<(String, Rect)>,
            rect_shapes: &mut Vec<Rect>,
        ) {
            match shape {
                egui::epaint::Shape::Text(t) => {
                    text_positions.push((t.galley.text().to_owned(), t.visual_bounding_rect()));
                }
                egui::epaint::Shape::Rect(r) => {
                    rect_shapes.push(r.rect);
                }
                egui::epaint::Shape::Vec(v) => {
                    for s in v {
                        collect_shapes(s, text_positions, rect_shapes);
                    }
                }
                _ => {}
            }
        }

        for cs in &output.shapes {
            collect_shapes(&cs.shape, &mut text_positions, &mut rect_shapes);
        }

        let button_rect_for = |label: &str| -> Rect {
            let text_rect = text_positions
                .iter()
                .find(|(t, _)| t == label)
                .map(|(_, r)| *r)
                .unwrap_or_else(|| panic!("missing button label '{label}' in {text_positions:?}"));
            rect_shapes
                .iter()
                .copied()
                .filter(|r| r.contains(text_rect.center()) && r.width() < 200.0 && r.height() < 40.0)
                .min_by(|a, b| a.width().total_cmp(&b.width()))
                .unwrap_or(text_rect)
        };

        let system_btn = button_rect_for("System");
        let tiles_btn = button_rect_for("Tiles (1)");
        let clear_btn = button_rect_for("Clear");

        assert!(
            system_btn.min.x < tiles_btn.min.x && tiles_btn.min.x < clear_btn.min.x,
            "expected System < Tiles (1) < Clear, got system={:?}, tiles={:?}, clear={:?}",
            system_btn.min.x,
            tiles_btn.min.x,
            clear_btn.min.x
        );
        assert!(
            clear_btn.max.x <= content_rect.right() + 0.5,
            "Clear button overflows right edge: {} > {}",
            clear_btn.max.x,
            content_rect.right()
        );
    }
}
