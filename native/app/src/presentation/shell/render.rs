//! Penggambaran kerangka luar (shell) notch, panel isi, dan jendela setelan.
//!
//! Bertanggung jawab atas:
//! - Latar belakang kaca/blur (`glass`)
//! - Bentuk squircle notch dinamis yang bermetamorfosis sesuai spring
//! - Penempatan baris minimized (`mini`)
//! - Kontainer panel isi notch (`draw_panel`)
//! - Viewport jendela setelan sekunder dengan tab UI  (`draw_settings_window`)

use crate::app::{Noktah, SettingsTab};
use crate::infrastructure::glass;
use crate::presentation::{icons::Icon, layout, mini, style, widgets};
use eframe::egui::{self, Color32, Rect, Rounding, Sense, Stroke};
use noktah_core::{i18n, lrc};
use std::time::Instant;

impl Noktah {
    /// Jendela setelan terpisah (Viewport egui). Mengembalikan `true` saat
    /// pengguna menutup jendela — pemanggil yang mematikan `show_settings`.
    ///
    /// Isinya memakai ulang `draw_settings_tab_content` supaya tidak ada dua daftar
    /// setelan yang bisa berbeda; yang ditambahkan di sini hanya judul,
    /// tombol tutup, dan bingkai gulir.
    /// Jendela setelan terpisah (Viewport egui) dengan tab UI dengan navigasi sidebar
    /// — header dengan title bar, tab switcher horizontal di atas, konten scroll di bawah.
    ///
    /// Tab design principles:
    /// - Single row of tabs below title (max ~5 visible before scrolling)
    /// - Active tab fills with accent_soft + stroke
    /// - Smooth transitions via spring animation (future optimization)
    pub(crate) fn draw_settings_window(&mut self, ctx: &egui::Context) -> bool {
        let theme = self.theme;
        let mut close = false;

        // 1. Sidebar Kiri : lebar 200px, warna #161619
        egui::SidePanel::left("settings_sidebar")
            .resizable(false)
            .exact_width(200.0)
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(22, 22, 25))
                    .inner_margin(egui::Margin {
                        left: 14.0,
                        right: 14.0,
                        top: 14.0,
                        bottom: 14.0,
                    }),
            )
            .show(ctx, |ui| {
                // Tombol close merah bulat ala macOS di pojok kiri atas
                let (btn_rect, btn_resp) =
                    ui.allocate_exact_size(egui::vec2(12.0, 12.0), Sense::click());
                let circle_color = if btn_resp.hovered() {
                    Color32::from_rgb(255, 110, 103)
                } else {
                    Color32::from_rgb(255, 95, 86)
                };
                ui.painter().circle_filled(btn_rect.center(), 6.0, circle_color);
                ui.painter().circle_stroke(
                    btn_rect.center(),
                    6.0,
                    Stroke::new(0.5, Color32::from_rgb(224, 68, 62)),
                );
                if btn_resp.hovered() {
                    let center = btn_rect.center();
                    let d = 2.4;
                    let stroke_x = Stroke::new(1.2, Color32::from_rgb(74, 0, 2));
                    ui.painter().line_segment(
                        [
                            egui::pos2(center.x - d, center.y - d),
                            egui::pos2(center.x + d, center.y + d),
                        ],
                        stroke_x,
                    );
                    ui.painter().line_segment(
                        [
                            egui::pos2(center.x - d, center.y + d),
                            egui::pos2(center.x + d, center.y - d),
                        ],
                        stroke_x,
                    );
                }
                if btn_resp.clicked() {
                    close = true;
                }

                ui.add_space(16.0);

                let item_h = 36.0;
                let item_w = ui.available_width();

                // Tab About disematkan di paling bawah (pinned bottom)
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    let about_tab = (
                        SettingsTab::About,
                        Icon::About,
                        i18n::t(self.lang, "settings.about"),
                        Color32::from_rgb(100, 116, 139),
                    );
                    let (tab_enum, icon, label, badge_bg) = about_tab;
                    let (rect, resp) =
                        ui.allocate_exact_size(egui::vec2(item_w, item_h), Sense::click());
                    let is_active = self.settings_tab == tab_enum;

                    if is_active {
                        ui.painter().rect_filled(
                            rect,
                            Rounding::same(8.0),
                            Color32::from_rgb(39, 39, 44),
                        );
                    } else if resp.hovered() {
                        ui.painter().rect_filled(
                            rect,
                            Rounding::same(8.0),
                            Color32::from_white_alpha(12),
                        );
                    }

                    let badge_rect = Rect::from_center_size(
                        egui::pos2(rect.left() + 19.0, rect.center().y),
                        egui::vec2(24.0, 24.0),
                    );
                    ui.painter().rect_filled(badge_rect, Rounding::same(6.0), badge_bg);
                    widgets::glyph(
                        ui.painter(),
                        icon,
                        Rect::from_center_size(badge_rect.center(), egui::vec2(13.0, 13.0)),
                        Color32::WHITE,
                        1.5,
                    );

                    let text_color = if is_active {
                        Color32::WHITE
                    } else if resp.hovered() {
                        Color32::from_rgb(230, 230, 235)
                    } else {
                        Color32::from_rgb(160, 160, 168)
                    };
                    ui.painter().text(
                        egui::pos2(rect.left() + 38.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        label,
                        egui::FontId::proportional(13.0),
                        text_color,
                    );

                    if resp.clicked() {
                        self.settings_tab = tab_enum;
                    }

                    ui.add_space(8.0);

                    // Navigasi Tab Utama
                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        let main_tabs = [
                            (
                                SettingsTab::General,
                                Icon::Gear,
                                i18n::t(self.lang, "settings.general"),
                                Color32::from_rgb(113, 113, 122),
                            ),
                            (
                                SettingsTab::Display,
                                Icon::Display,
                                i18n::t(self.lang, "settings.display"),
                                Color32::from_rgb(37, 99, 235),
                            ),
                            (
                                SettingsTab::Appearance,
                                Icon::Palette,
                                i18n::t(self.lang, "settings.appearance"),
                                theme.accent,
                            ),
                            (
                                SettingsTab::Music,
                                Icon::MusicNote,
                                i18n::t(self.lang, "settings.music"),
                                Color32::from_rgb(244, 63, 94),
                            ),
                            (
                                SettingsTab::Todo,
                                Icon::Todo,
                                i18n::t(self.lang, "settings.todo"),
                                Color32::from_rgb(34, 197, 94),
                            ),
                            (
                                SettingsTab::Pomodoro,
                                Icon::Focus,
                                i18n::t(self.lang, "settings.pomodoro"),
                                Color32::from_rgb(249, 115, 22),
                            ),
                            (
                                SettingsTab::Ai,
                                Icon::AI,
                                i18n::t(self.lang, "settings.ai"),
                                Color32::from_rgb(168, 85, 247),
                            ),
                        ];

                        for (tab_enum, icon, label, badge_bg) in main_tabs {
                            let (rect, resp) =
                                ui.allocate_exact_size(egui::vec2(item_w, item_h), Sense::click());
                            let is_active = self.settings_tab == tab_enum;

                            if is_active {
                                ui.painter().rect_filled(
                                    rect,
                                    Rounding::same(8.0),
                                    Color32::from_rgb(39, 39, 44),
                                );
                            } else if resp.hovered() {
                                ui.painter().rect_filled(
                                    rect,
                                    Rounding::same(8.0),
                                    Color32::from_white_alpha(12),
                                );
                            }

                            let badge_rect = Rect::from_center_size(
                                egui::pos2(rect.left() + 19.0, rect.center().y),
                                egui::vec2(24.0, 24.0),
                            );
                            ui.painter().rect_filled(badge_rect, Rounding::same(6.0), badge_bg);
                            widgets::glyph(
                                ui.painter(),
                                icon,
                                Rect::from_center_size(badge_rect.center(), egui::vec2(13.0, 13.0)),
                                Color32::WHITE,
                                1.5,
                            );

                            let text_color = if is_active {
                                Color32::WHITE
                            } else if resp.hovered() {
                                Color32::from_rgb(230, 230, 235)
                            } else {
                                Color32::from_rgb(160, 160, 168)
                            };
                            ui.painter().text(
                                egui::pos2(rect.left() + 38.0, rect.center().y),
                                egui::Align2::LEFT_CENTER,
                                label,
                                egui::FontId::proportional(13.0),
                                text_color,
                            );

                            if resp.clicked() {
                                self.settings_tab = tab_enum;
                            }

                            ui.add_space(3.0);
                        }
                    });
                });
            });

        // 2. Central Panel (Konten Utama di Sebelah Kanan)
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(28, 28, 31))
                    .inner_margin(egui::Margin {
                        left: 24.0,
                        right: 24.0,
                        top: 20.0,
                        bottom: 16.0,
                    }),
            )
            .show(ctx, |ui| {
                let panel_rect = ui.max_rect().intersect(ctx.screen_rect());
                // Judul Tab di bagian atas konten
                let tab_title = match self.settings_tab {
                    SettingsTab::General => i18n::t(self.lang, "settings.general"),
                    SettingsTab::Display => i18n::t(self.lang, "settings.display"),
                    SettingsTab::Appearance => i18n::t(self.lang, "settings.appearance"),
                    SettingsTab::Music => i18n::t(self.lang, "settings.music"),
                    SettingsTab::Todo => i18n::t(self.lang, "settings.todo"),
                    SettingsTab::Pomodoro => i18n::t(self.lang, "settings.pomodoro"),
                    SettingsTab::Ai => i18n::t(self.lang, "settings.ai"),
                    SettingsTab::About => i18n::t(self.lang, "settings.about"),
                };

                ui.label(
                    egui::RichText::new(tab_title)
                        .size(22.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.add_space(16.0);

                // Area gulir konten pengaturan
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.draw_settings_tab_content(ui);
                    });

                // Toast banner mengambang di bagian bawah viewport Settings
                if let Some((ref text, at)) = self.banner {
                    let elapsed = at.elapsed().as_secs_f32();
                    if elapsed < 4.0 {
                        ctx.request_repaint();
                        let alpha = if elapsed > 3.5 {
                            ((4.0 - elapsed) / 0.5).clamp(0.0, 1.0)
                        } else if elapsed < 0.2 {
                            (elapsed / 0.2).clamp(0.0, 1.0)
                        } else {
                            1.0
                        };

                        let pad_x = style::S3;
                        let pad_y = style::S2;
                        let max_toast_w = (panel_rect.width() - style::S4 * 2.0)
                            .min(520.0)
                            .max(panel_rect.width().min(160.0));
                        let max_text_w = (max_toast_w - pad_x * 2.0).max(80.0);

                        let text_color = theme.ink.gamma_multiply(alpha);
                        let mut job = egui::text::LayoutJob::single_section(
                            text.clone(),
                            egui::TextFormat {
                                font_id: style::body(),
                                color: text_color,
                                ..Default::default()
                            },
                        );
                        job.wrap = egui::text::TextWrapping {
                            max_width: max_text_w,
                            max_rows: 4,
                            break_anywhere: true,
                            overflow_character: Some('…'),
                        };
                        let text_galley = ui.painter().layout_job(job);

                        let min_h = style::S4 + style::S2;
                        let max_h = (panel_rect.height() - style::S4).max(min_h);
                        let toast_w = (text_galley.size().x + pad_x * 2.0)
                            .clamp(140.0_f32.min(max_toast_w), max_toast_w);
                        let toast_h = (text_galley.size().y + pad_y * 2.0).clamp(min_h, max_h);
                        let rounding = if text_galley.rows.len() <= 1 {
                            style::R_PILL
                        } else {
                            style::R_GROUP
                        };

                        let center_x = panel_rect.center().x;
                        let bottom_y = panel_rect.bottom() - (style::S2 * 2.0);
                        let raw_rect = Rect::from_center_size(
                            egui::pos2(center_x, bottom_y - toast_h * 0.5),
                            egui::vec2(toast_w, toast_h),
                        );
                        let min_x = (panel_rect.left() + style::S2)
                            .min((panel_rect.right() - toast_w).max(panel_rect.left()));
                        let max_x = (panel_rect.right() - style::S2 - toast_w).max(min_x);
                        let min_y = (panel_rect.top() + style::S2)
                            .min((panel_rect.bottom() - toast_h).max(panel_rect.top()));
                        let max_y = (panel_rect.bottom() - style::S2 - toast_h).max(min_y);
                        let toast_rect = Rect::from_min_size(
                            egui::pos2(
                                raw_rect.left().clamp(min_x, max_x),
                                raw_rect.top().clamp(min_y, max_y),
                            ),
                            egui::vec2(toast_w, toast_h),
                        );

                        let bg_color =
                            Color32::from_rgba_unmultiplied(24, 24, 28, (235.0 * alpha) as u8);
                        let stroke_color = theme.accent.gamma_multiply(0.82 * alpha);
                        ui.painter().rect(
                            toast_rect,
                            Rounding::same(rounding),
                            bg_color,
                            Stroke::new(1.0, stroke_color),
                        );
                        ui.painter().galley(
                            egui::pos2(
                                toast_rect.center().x - text_galley.size().x * 0.5,
                                toast_rect.center().y - text_galley.size().y * 0.5,
                            ),
                            text_galley,
                            text_color,
                        );
                    }
                }
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
        self.settings.lyrics_enabled && self.is_lyrics_synced()
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
        self.hover_since = None;
        self.suppress_hover = true;
        self.show_settings = false;
        self.open.target = 0.0;
        self.open_h.target = 0.0;
        self.open_h.set_spec(crate::presentation::spring::MORPH_CLOSE_H);
        self.open.set_spec(crate::presentation::spring::MORPH_CLOSE_W);
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
                            ctx.screen_rect(),
                            radius,
                            tekstur.id(),
                            232,
                            self.settings.squircle,
                        );
                    }
                }

                let has_blur = self.settings.glass_effect.uses_blur() && self.glass_tex.is_some();
                glass::surface_n(
                    painter,
                    outer_rect,
                    radius,
                    &self.theme,
                    has_blur,
                    n_sudut,
                );

                // Telinga selalu ada selama panel menempel di tepi atas.
                if mix < 0.99 && self.settings.ears {
                    let telinga = layout::EAR * (0.35 + 0.65 * t_w.clamp(0.0, 1.0));
                    glass::ears(painter, outer_rect, telinga, &self.theme, has_blur);
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

                    // Klik pada badge maskot saat baris minimized untuk efek squish
                    if mini_alpha > 0.5 && ui.input(|i| i.pointer.primary_clicked()) {
                        let badge_center = egui::pos2(outer_rect.left() + 32.0, outer_rect.center().y);
                        let badge_rect = Rect::from_center_size(badge_center, egui::vec2(28.0, 28.0));
                        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                            if badge_rect.contains(pos) {
                                self.mascot.trigger_squish(self.anim.elapsed().as_secs_f32());
                                ui.ctx().request_repaint();
                            }
                        }
                    }

                    let mini_ctx = mini::MiniCtx {
                        theme: &self.theme.with_opacity_multiplier(mini_alpha),
                        settings: &self.settings,
                        media: &self.media,
                        show_media: self.show_media_in_mini(),
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
                        banner: self.banner.as_ref().and_then(|(msg, at)| {
                            if at.elapsed().as_secs_f32() < 4.0 {
                                Some(msg.as_str())
                            } else {
                                None
                            }
                        }),
                        pending_approval_count: self.pending_permissions.len(),
                        focus_dnd_rem: if self.is_focus_dnd_active() {
                            Some(self.pomodoro.remaining_secs)
                        } else {
                            None
                        },
                        custom_tile: if self.settings.custom_tiles_enabled {
                            self.custom_tiles.pinned_mini()
                        } else {
                            None
                        },
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

                        // HUD Floating Toast Banner saat panel terbuka
                        if let Some((ref banner_text, at)) = self.banner {
                            let elapsed = at.elapsed().as_secs_f32();
                            if elapsed < 4.0 {
                                let banner_alpha = if elapsed < 0.25 {
                                    (elapsed / 0.25).clamp(0.0, 1.0)
                                } else if elapsed > 3.5 {
                                    ((4.0 - elapsed) / 0.5).clamp(0.0, 1.0)
                                } else {
                                    1.0
                                } * alpha;
                                let toast_w = 320.0f32.min(outer_rect.width() - 40.0);
                                let toast_h = 30.0;
                                let toast_rect = Rect::from_center_size(
                                    egui::pos2(outer_rect.center().x, outer_rect.bottom() - 24.0),
                                    egui::vec2(toast_w, toast_h),
                                );
                                let p = ui.painter();
                                p.rect_filled(
                                    toast_rect,
                                    Rounding::same(toast_h / 2.0),
                                    Color32::from_rgba_unmultiplied(20, 21, 26, (245.0 * banner_alpha) as u8),
                                );
                                p.rect_stroke(
                                    toast_rect,
                                    Rounding::same(toast_h / 2.0),
                                    Stroke::new(1.0, self.theme.accent.gamma_multiply(banner_alpha)),
                                );
                                p.text(
                                    toast_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    banner_text,
                                    egui::FontId::proportional(11.5),
                                    Color32::from_white_alpha((255.0 * banner_alpha) as u8),
                                );
                            }
                        }
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
