//! Halaman Ask — tata letak `.askbody` + `.askform` dari prototipe
//! `noktah-redesign.html`.
//!
//! Perbedaan dari versi sebelumnya: kolom input dan tombol kirim memakai
//! bentuk kapsul yang sama dengan halaman lain (bukan kotak 8 px), dan saat
//! percakapan kosong ada lencana aksen di tengah — bukan teks polos yang
//! mengambang tanpa jangkar.

use crate::app::Noktah;
use crate::presentation::{icons::Icon, style, widgets};
use eframe::egui::{self, Color32, Rect, Rounding, Sense};

impl Noktah {
    /// Ask: streaming chat against the user's own key.
    ///
    /// Keeps the key entry inline rather than behind a settings trip — the
    /// page is useless without a key, so the one thing that unblocks it should
    /// be the thing on screen. Bubbles are distinguished by alignment and
    /// fill alone; no per-message frames, which at this width turn a
    /// three-message exchange into a stack of boxes.
    pub(crate) fn draw_page_ai(&mut self, ui: &mut egui::Ui, area: Rect) {
        let theme = self.theme;

        let header_h = widgets::page_header(ui, "ASK", &theme, |ui| {
            // `.status`: titik status + label, bukan teks polos.
            let (dot_col, status) = if self.ai_busy {
                (Color32::from_rgb(0x4C, 0xD9, 0x64), "Thinking…")
            } else if self.api_key.is_empty() && !self.settings.provider.keyless() {
                (Color32::from_rgb(0xFF, 0x6B, 0x5E), "No key")
            } else {
                (Color32::from_rgb(0x4C, 0xD9, 0x64), self.settings.model.as_str())
            };
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = style::S1;
                let (d, _) = ui.allocate_exact_size(egui::vec2(7.0, 7.0), Sense::hover());
                ui.painter().circle_filled(d.center(), 3.5, dot_col);
                ui.label(style::cap(status, theme.dim));
            });
        });

        // ---- Jika belum ada kunci API, tampilkan kartu penyiapan kunci ----
        let butuh_kunci = self.api_key.is_empty() && !self.settings.provider.keyless();
        if butuh_kunci {
            let body_h = (area.height() - header_h).max(60.0);
            ui.allocate_ui_with_layout(
                egui::vec2(area.width(), body_h),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.add_space(style::S1);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = style::S2;
                        let (badge, _) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), Sense::hover());
                        ui.painter().rect_filled(
                            badge,
                            Rounding::same(6.0),
                            Color32::from_rgba_unmultiplied(
                                theme.accent.r(),
                                theme.accent.g(),
                                theme.accent.b(),
                                210,
                            ),
                        );
                        widgets::glyph(
                            ui.painter(),
                            Icon::Sparkles,
                            Rect::from_center_size(badge.center(), egui::vec2(14.0, 14.0)),
                            theme.on_accent,
                            1.5,
                        );
                        ui.label(style::title("Set up API key", theme.ink));
                    });

                    ui.add_space(style::S1);
                    ui.label(style::cap(
                        "Stored encrypted with DPAPI on this machine.",
                        theme.dim,
                    ));
                    ui.add_space(style::S1);

                    const FIELD_H: f32 = 32.0;
                    let save_w = 64.0;
                    let field_w = (area.width() * 0.8).min(380.0);
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(field_w, FIELD_H), Sense::hover());
                    ui.painter().rect_filled(
                        rect,
                        Rounding::same(style::R_CONTROL),
                        theme.surface,
                    );

                    let inner = Rect::from_min_max(
                        egui::pos2(rect.left() + 10.0, rect.top() + 4.0),
                        egui::pos2(rect.right() - save_w - 6.0, rect.bottom() - 4.0),
                    );
                    ui.put(
                        inner,
                        egui::TextEdit::singleline(&mut self.api_key_input)
                            .password(true)
                            .hint_text("sk-…")
                            .desired_width(inner.width())
                            .frame(false)
                            .vertical_align(egui::Align::Center),
                    );

                    let btn_rect = Rect::from_min_size(
                        egui::pos2(rect.right() - save_w - 3.0, rect.top() + 3.0),
                        egui::vec2(save_w, FIELD_H - 6.0),
                    );
                    let btn_resp = ui.allocate_rect(btn_rect, Sense::click());
                    ui.painter().rect_filled(
                        btn_rect,
                        Rounding::same(style::R_PILL),
                        if btn_resp.hovered() { theme.rim } else { theme.accent },
                    );
                    ui.painter().text(
                        btn_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Save",
                        egui::FontId::proportional(style::T_CAPTION),
                        theme.on_accent,
                    );
                    if btn_resp.clicked() && !self.api_key_input.trim().is_empty() {
                        let key = self.api_key_input.trim().to_string();
                        self.set_api_key(key);
                        self.api_key_input.clear();
                    }
                },
            );
            return;
        }

        // ---- Transkrip ----
        const INPUT_H: f32 = 32.0;
        let list_h = (area.bottom() - ui.cursor().top() - INPUT_H - style::S1).max(20.0);

        #[derive(Clone, Copy)]
        enum QuickAction {
            Translate,
            Summarize,
            Explain,
            Clear,
        }
        let mut quick_action: Option<QuickAction> = None;
        let lbl_translate = self.tr("ai.quick_translate").to_string();
        let lbl_summarize = self.tr("ai.quick_summarize").to_string();
        let lbl_explain = self.tr("ai.quick_explain").to_string();
        let lbl_clear = self.tr("ai.quick_clear").to_string();

        egui::ScrollArea::vertical()
            .id_salt("ask_chat")
            .max_height(list_h)
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.set_width(area.width());
                ui.spacing_mut().item_spacing.y = style::S1;

                if !self.ai_busy {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(style::S1, style::S1);
                        if widgets::chip_button(ui, &lbl_translate, false, &theme) {
                            quick_action = Some(QuickAction::Translate);
                        }
                        if widgets::chip_button(ui, &lbl_summarize, false, &theme) {
                            quick_action = Some(QuickAction::Summarize);
                        }
                        if widgets::chip_button(ui, &lbl_explain, false, &theme) {
                            quick_action = Some(QuickAction::Explain);
                        }
                        if !self.chat.is_empty() && widgets::chip_button(ui, &lbl_clear, false, &theme) {
                            quick_action = Some(QuickAction::Clear);
                        }
                    });
                }

                if self.chat.is_empty() {
                    // `.askbody` + `.spk`: lencana aksen, judul, keterangan.
                    ui.add_space(style::S1);
                    ui.vertical_centered(|ui| {
                        let (badge, _) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), Sense::hover());
                        ui.painter().rect_filled(
                            badge,
                            Rounding::same(8.0),
                            Color32::from_rgba_unmultiplied(
                                theme.accent.r(),
                                theme.accent.g(),
                                theme.accent.b(),
                                210,
                            ),
                        );
                        widgets::glyph(
                            ui.painter(),
                            Icon::Sparkles,
                            Rect::from_center_size(badge.center(), egui::vec2(14.0, 14.0)),
                            theme.on_accent,
                            1.5,
                        );
                        ui.add_space(style::S1);
                        ui.label(style::title("Ask anything", theme.ink));
                        ui.label(style::cap(
                            "Uses your own key. Nothing leaves this machine but the request.",
                            theme.dim,
                        ));
                    });
                    return;
                }

                let mut salin: Option<String> = None;

                for m in self.chat.iter() {
                    let dari_saya = m.role == noktah_core::model::Role::User;

                    // `.bub`: gelembung dengan latar, bukan hanya perataan.
                    // Lebar maksimum 86% seperti prototipe.
                    let max_w = area.width() * 0.86;
                    let teks = if m.content.is_empty() {
                        "…".to_string()
                    } else {
                        m.content.clone()
                    };

                    let galley = ui.painter().layout(
                        teks.clone(),
                        egui::FontId::proportional(style::T_SUBTITLE),
                        if dari_saya { theme.on_accent } else { theme.ink },
                        max_w - 24.0,
                    );
                    let bub_w = (galley.size().x + 24.0).min(max_w);
                    let bub_h = galley.size().y + 14.0;

                    ui.horizontal(|ui| {
                        if dari_saya {
                            let gap = (area.width() - bub_w).max(0.0);
                            ui.add_space(gap);
                        }
                        let (r, resp) = ui.allocate_exact_size(
                            egui::vec2(bub_w, bub_h),
                            Sense::hover(),
                        );
                        ui.painter().rect_filled(
                            r,
                            Rounding::same(16.0),
                            if dari_saya { theme.accent } else { theme.fill2 },
                        );
                        ui.painter().galley(
                            egui::pos2(r.left() + 12.0, r.top() + 7.0),
                            galley,
                            if dari_saya { theme.on_accent } else { theme.ink },
                        );

                        if !dari_saya && !m.content.is_empty() && resp.hovered() {
                            if widgets::chip_button(ui, "Copy", false, &theme) {
                                salin = Some(m.content.clone());
                            }
                        }
                    });
                }

                if let Some(teks) = salin {
                    ui.output_mut(|o| o.copied_text = teks);
                }
            });

        if let Some(act) = quick_action {
            match act {
                QuickAction::Clear => {
                    self.chat.clear();
                    self.save_chat();
                }
                QuickAction::Translate | QuickAction::Summarize | QuickAction::Explain => {
                    let typed = self.chat_input.trim().to_string();
                    let source = if !typed.is_empty() {
                        self.chat_input.clear();
                        Some(typed)
                    } else {
                        crate::infrastructure::os::read_clipboard_text()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                    };
                    match source {
                        Some(src) => {
                            let prompt = match act {
                                QuickAction::Translate => format!(
                                    "Translate the following text between Indonesian and English naturally and concisely:\n\n{src}"
                                ),
                                QuickAction::Summarize => format!(
                                    "Summarize the key points of the following text concisely (respond in the same language as the input):\n\n{src}"
                                ),
                                QuickAction::Explain => format!(
                                    "Explain the following text or code clearly and concisely (respond in the same language as the input):\n\n{src}"
                                ),
                                QuickAction::Clear => unreachable!(),
                            };
                            self.send_prompt(prompt);
                        }
                        None => {
                            self.banner = Some((
                                self.tr("ai.clipboard_empty").to_string(),
                                std::time::Instant::now(),
                            ));
                        }
                    }
                }
            }
        }

        // ---- Baris tulis (`.askform`) ----
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = style::S2;
            let send_w = INPUT_H;
            let field_w = (area.width() - send_w - style::S2).max(80.0);
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(field_w, INPUT_H), Sense::hover());
            ui.painter().rect_filled(
                rect,
                Rounding::same(style::R_PILL),
                theme.surface,
            );

            let inner = Rect::from_min_max(
                egui::pos2(rect.left() + 16.0, rect.top() + 8.0),
                egui::pos2(rect.right() - 16.0, rect.bottom() - 8.0),
            );
            let resp = ui.put(
                inner,
                egui::TextEdit::singleline(&mut self.chat_input)
                    .hint_text("Ask anything…")
                    .desired_width(inner.width())
                    .frame(false)
                    .vertical_align(egui::Align::Center),
            );
            let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            let bisa = !self.ai_busy && !self.chat_input.trim().is_empty();
            let btn_rect = Rect::from_min_size(
                egui::pos2(rect.right() + style::S2, rect.top()),
                egui::vec2(send_w, INPUT_H),
            );
            let btn_resp = ui.allocate_rect(btn_rect, Sense::click());

            // `.ib.send`: lingkaran aksen penuh dengan ikon kirim.
            ui.painter().circle_filled(
                btn_rect.center(),
                INPUT_H / 2.0,
                if bisa {
                    if btn_resp.hovered() { theme.rim } else { theme.accent }
                } else {
                    theme.surface
                },
            );
            widgets::glyph(
                ui.painter(),
                Icon::ArrowUpRight,
                Rect::from_center_size(btn_rect.center(), egui::vec2(18.0, 18.0)),
                if bisa { theme.on_accent } else { theme.faint },
                1.8,
            );

            if (btn_resp.clicked() || enter) && bisa {
                self.send_chat();
                resp.request_focus();
            }
        });
    }
}
