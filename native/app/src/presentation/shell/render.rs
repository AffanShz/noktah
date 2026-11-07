//! Penggambaran kerangka luar (shell) notch, panel isi, dan jendela setelan.
//!
//! Bertanggung jawab atas:
//! - Latar belakang kaca/blur (`glass`)
//! - Bentuk squircle notch dinamis yang bermetamorfosis sesuai spring
//! - Penempatan baris minimized (`mini`)
//! - Kontainer panel isi notch (`draw_panel`)
//! - Viewport jendela setelan sekunder dengan tab UI ala Nusa (`draw_settings_window`)

use crate::app::{Noktah, SettingsTab};
use crate::infrastructure::glass;
use crate::presentation::{icons::Icon, layout, mini, style, widgets};
use eframe::egui::{self, Color32, Id, Rect, Rounding, Sense, Stroke};
use noktah_core::lrc;
use std::time::Instant;

impl Noktah {
    /// Jendela setelan terpisah (Viewport egui). Mengembalikan `true` saat
    /// pengguna menutup jendela — pemanggil yang mematikan `show_settings`.
    ///
    /// Isinya memakai ulang `tab_settings` supaya tidak ada dua daftar
    /// setelan yang bisa berbeda; yang ditambahkan di sini hanya judul,
    /// tombol tutup, dan bingkai gulir.
    /// Jendela setelan terpisah (Viewport egui) dengan tab UI ala Nusa / Apple
    /// — header dengan title bar, tab switcher horizontal di atas, konten scroll di bawah.
    ///
    /// Tab design principles:
    /// - Single row of tabs below title (max ~5 visible before scrolling)
    /// - Active tab fills with accent_soft + stroke
    /// - Smooth transitions via spring animation (future optimization)
    pub(crate) fn draw_settings_window(&mut self, ctx: &egui::Context) -> bool {
        let theme = self.theme;
        let mut close = false;

        // Tab bar height
        const TAB_H: f32 = 40.0;
        // Tab width for icon-only tabs (no labels shown for space efficiency)
        const TAB_ITEM_W: f32 = 44.0;

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(18, 19, 24))
                    .inner_margin(egui::Margin::same(style::PAD)),
            )
            .show(ctx, |ui| {
                // ---- Title bar ----
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(self.tr("settings.title"))
                            .size(style::T_TITLE)
                            .strong()
                            .color(egui::Color32::WHITE),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if widgets::chip_button(ui, self.tr("common.close"), false, &theme) {
                            close = true;
                        }
                    });
                });
                ui.add_space(style::S2);

                // ---- Tab switcher ----
                let tab_rect = egui::Rect::from_min_size(
                    egui::pos2(ui.cursor().min.x, ui.cursor().min.y),
                    egui::vec2(ui.available_width(), TAB_H),
                );

                // Tab icons in order matching SettingsTab enum
                let tabs = [
                    (SettingsTab::General, Icon::Gear, "General"),
                    (SettingsTab::Display, Icon::Display, "Display"),
                    (SettingsTab::Appearance, Icon::Palette, "Appearance"),
                    (SettingsTab::Music, Icon::MusicNote, "Music"),
                    (SettingsTab::Todo, Icon::Todo, "Tasks"),
                    (SettingsTab::Pomodoro, Icon::Focus, "Focus"),
                    (SettingsTab::Ai, Icon::AI, "AI"),
                    (SettingsTab::About, Icon::About, "About"),
                ];

                let group_w = TAB_ITEM_W * tabs.len() as f32;
                let group_left = tab_rect.left() + (tab_rect.width() - group_w) * 0.5;

                for (idx, (tab_enum, icon, _label)) in tabs.iter().enumerate() {
                    let r = egui::Rect::from_min_size(
                        egui::pos2(group_left + idx as f32 * TAB_ITEM_W, tab_rect.top()),
                        egui::vec2(TAB_ITEM_W, TAB_H),
                    );
                    let resp = ui.allocate_rect(r, Sense::click());
                    let aktif = *tab_enum == self.settings_tab;

                    if aktif {
                        ui.painter().rect_filled(
                            r.shrink2(egui::vec2(4.0, 3.0)),
                            Rounding::same(style::concentric(style::R_CONTROL, style::S1)),
                            theme.accent_soft,
                        );
                        ui.painter().rect_stroke(
                            r.shrink2(egui::vec2(4.0, 3.0)),
                            Rounding::same(style::concentric(style::R_CONTROL, style::S1)),
                            Stroke::new(1.0_f32, theme.accent),
                        );
                    } else if resp.hovered() {
                        ui.painter().rect_filled(
                            r.shrink2(egui::vec2(4.0, 3.0)),
                            Rounding::same(style::R_SMALL),
                            Color32::from_white_alpha(18),
                        );
                    }

                    widgets::glyph(
                        ui.painter(),
                        *icon,
                        Rect::from_center_size(r.center(), egui::vec2(16.0, 16.0)),
                        if aktif {
                            theme.accent
                        } else if resp.hovered() {
                            theme.ink
                        } else {
                            theme.dim
                        },
                        1.7,
                    );

                    if resp.clicked() {
                        self.settings_tab = *tab_enum;
                    }
                }

                ui.add_space(style::S3);
                ui.separator();
                ui.add_space(style::S2);

                // ---- Tab content ----
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.draw_settings_tab_content(ui);
                    });
            });

        // Tombol X di titlebar jendela juga menutup, sama seperti Tombol Tutup.
        if ctx.input(|i| i.viewport().close_requested()) {
            close = true;
        }

        close
    }

    /// Lagu yang sedang diputar punya baris lirik untuk ditampilkan?
    ///
    /// Dipakai untuk menyusutkan pill: slot lirik hanya ikut dihitung kalau
    /// memang ada teks yang akan mengisinya.
    pub(crate) fn has_lyrics(&self) -> bool {
        self.settings.lyrics_enabled && !self.lyrics.is_empty()
    }

    /// Dimensi target saat terbuka (selalu pulau kompak 560x160 px ala Apple Dynamic Island).
    pub fn target_open_size(&self) -> (f32, f32) {
        (layout::ISLAND_W, layout::ISLAND_H)
    }

    /// Radius sudut target saat terbuka.
    pub fn target_radius(&self) -> f32 {
        layout::RADIUS_ISLAND
    }

    /// Menutup panel kembali ke pill, sama seperti kursor menjauh.
    pub fn close_panel(&mut self) {
        self.hovered = false;
        self.show_settings = false;
        self.open.target = 0.0;
        self.open_h.target = 0.0;
    }

    pub(crate) fn draw_shell(&mut self, ctx: &egui::Context, outer_rect: Rect) {
        let t_w = self.open.pos.clamp(-0.10, 1.15);
        let t_h = self.open_h.pos.clamp(-0.10, 1.15);
        let mix = self.minimal_mix.pos;
        let target_r = self.target_radius();
        let radius = layout::radius(t_h, mix, target_r);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                let painter = ui.painter();

                let n_sudut =
                    2.0 + (if self.settings.squircle { 2.5 } else { 0.5 }) * t_h.clamp(0.0, 1.0);

                if self.settings.glass_effect.uses_blur() {
                    if let Some(tekstur) = &self.glass_tex {
                        glass::textured(
                            painter,
                            outer_rect,
                            radius,
                            tekstur.id(),
                            232,
                            self.settings.squircle,
                        );
                    }
                }

                glass::surface_n(
                    painter,
                    outer_rect,
                    radius,
                    &self.theme,
                    self.settings.glass_effect.uses_blur(),
                    n_sudut,
                );

                // Telinga selalu ada selama panel menempel di tepi atas.
                if mix < 0.99 && self.settings.ears {
                    let telinga = layout::EAR * (0.35 + 0.65 * t_w.clamp(0.0, 1.0));
                    glass::ears(painter, outer_rect, telinga, &self.theme);
                }

                // Isi baris minimized hanya terlihat saat hampir tertutup.
                let mini_alpha = ((0.35 - t_h) / 0.35).clamp(0.0, 1.0) * (1.0 - mix);
                if mini_alpha > 0.01 {
                    let lyric = lrc::active_line(
                        &self.lyrics,
                        self.media.position_ms + self.settings.lyric_offset_ms,
                    )
                    .and_then(|i| self.lyrics.get(i))
                    .map(|l| l.text.as_str());

                    if lyric != self.mini_curr_lyric.as_deref() {
                        self.mini_prev_lyric = self.mini_curr_lyric.take();
                        self.mini_curr_lyric = lyric.map(|s| s.to_string());
                        self.mini_lyric_at = Instant::now();
                    }

                    let elapsed = self.mini_lyric_at.elapsed().as_secs_f32();
                    let lyric_trans = if self.reduce_motion {
                        1.0
                    } else {
                        (elapsed / 0.32).clamp(0.0, 1.0)
                    };
                    if lyric_trans < 1.0
                        || (self.media.active && self.media.playing && self.settings.mini_lyrics)
                    {
                        ui.ctx().request_repaint();
                    }

                    let mini_ctx = mini::MiniCtx {
                        theme: &self.theme.with_opacity_multiplier(mini_alpha),
                        settings: &self.settings,
                        media: &self.media,
                        lyric,
                        prev_lyric: self.mini_prev_lyric.as_deref(),
                        lyric_trans,
                        lyric_t: if self.reduce_motion { 0.0 } else { elapsed },
                        has_lyrics: self.has_lyrics(),
                        cpu: self.sys.cpu,
                        ram: self.sys.ram_fraction() * 100.0,
                        art: self.art.as_ref(),
                        clock: chrono::Local::now().format("%H:%M").to_string(),
                        weather: self
                            .weather_now
                            .as_ref()
                            .map(|w| format!("{:.0}°", w.temp_c)),
                        t: if self.reduce_motion {
                            0.0
                        } else {
                            self.anim.elapsed().as_secs_f32()
                        },
                        levels: self.levels,
                        alpha: mini_alpha,
                        is_charging: self.is_charging,
                        battery_pct: self.battery_pct,
                        album_color: self
                            .album_accent
                            .map(|c| self.theme.with_accent(c).accent)
                            .unwrap_or(self.theme.accent),
                        reduce_motion: self.reduce_motion,
                        agent: &self.agent,
                        mascot: &self.mascot,
                    };
                    // Klip aura dilonggarkan 6 px ke bawah saja (rect tata
                    // letak tetap sama supaya isi baris tidak bergeser).
                    // 6 px masih di dalam margin region jendela Windows saat
                    // telinga mati (8 px), jadi pendar tidak dipotong OS.
                    let aura_clip = Rect::from_min_max(
                        outer_rect.min,
                        egui::pos2(outer_rect.max.x, outer_rect.max.y + 6.0),
                    )
                    .intersect(painter.clip_rect());
                    mini::draw(&painter.with_clip_rect(aura_clip), outer_rect, &mini_ctx);
                }
            });
    }

    pub(crate) fn draw_panel(
        &mut self,
        ctx: &egui::Context,
        base_rect: Rect,
        outer_rect: Rect,
        alpha: f32,
    ) {
        let slide_y = (1.0 - alpha.clamp(0.0, 1.0)) * -8.0;
        let area_pos = egui::pos2(outer_rect.min.x, outer_rect.min.y + slide_y);

        egui::Area::new(egui::Id::new("isi-panel"))
            .fixed_pos(area_pos)
            .order(egui::Order::Foreground)
            .constrain(false)
            .show(ctx, |ui| {
                ui.set_clip_rect(outer_rect);
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.allocate_ui_with_layout(
                    egui::vec2(outer_rect.width(), outer_rect.height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.multiply_opacity(alpha);
                        self.draw_notch_pages(ui, base_rect, outer_rect);
                        // Halaman digambar lewat `new_child`, yang tidak
                        // mengalokasikan ruang di UI induk. Tanpa baris ini
                        // `min_rect` Area menyusut jadi 0x0.
                        //
                        // Yang bergantung pada luas itu adalah pemeriksaan
                        // berbasis layer: `Context::rect_contains_pointer`
                        // menuntut pointer berada di layer Area ini, dan
                        // `ui.rect_contains_pointer` (music.rs:362) serta
                        // `ScrollArea::end()` (egui scroll_area.rs:921)
                        // memakainya. Saat rect 0x0, roda mouse tidak pernah
                        // dikonsumsi sehingga lirik tidak bisa digulir.
                        //
                        // Klik tombol dan baris lirik TIDAK terpengaruh: itu
                        // lewat `ui.interact` yang memakai `interact_rect`
                        // per-widget di layer yang sama.
                        ui.expand_to_include_rect(outer_rect);
                    },
                );
            });
    }
}
