//! Halaman Claude Code Agent & Interactive Permission Approval dengan Blob Mascot Noki.

use crate::app::Noktah;
use crate::presentation::{icons::Icon, markdown::render_markdown, mascot::MascotRenderer, widgets};
use eframe::egui::{self, Color32, FontId, Frame, Margin, Rect, Rounding, Stroke, Vec2};
use noktah_core::model::{AgentStatus, PermissionDecision};

impl Noktah {
    /// Menggambar halaman aktivitas Claude Code, maskot blob interaktif, dan dialog persetujuan.
    pub fn draw_page_agent(&mut self, ui: &mut egui::Ui, area: Rect) {
        let theme = self.theme;
        let now_ms = chrono::Utc::now().timestamp_millis();
        let anim_t = self.anim.elapsed().as_secs_f32();

        // 1. Tangani hotkey keyboard (Alt+Y untuk Allow, Alt+N untuk Deny)
        let is_alt = ui.input(|i| i.modifiers.alt);
        let key_y = ui.input(|i| i.key_pressed(egui::Key::Y));
        let key_n = ui.input(|i| i.key_pressed(egui::Key::N));

        if is_alt && key_y {
            if let Some((_, responder)) = self.pending_permission.take() {
                let _ = responder.send(PermissionDecision {
                    behavior: "allow".into(),
                    message: None,
                    updated_input: None,
                });
                self.agent.status = AgentStatus::PromptSubmitted {
                    prompt: "Approved by user via Alt+Y".into(),
                    timestamp: now_ms,
                };
                self.close_panel();
            }
        } else if is_alt && key_n {
            if let Some((_, responder)) = self.pending_permission.take() {
                let _ = responder.send(PermissionDecision {
                    behavior: "deny".into(),
                    message: Some("Rejected via Noktah Alt+N".into()),
                    updated_input: None,
                });
                self.agent.status = AgentStatus::Idle;
                self.close_panel();
            }
        }

        // Buffer padding aman: 8px kiri-kanan, 4px atas, 6px bawah agar tidak terpotong
        // kurva 28px di tepi atau tertabrak dock strip di bawah.
        let pad_x = 8.0;
        let pad_top = 4.0;
        let pad_bottom = 6.0;
        let inner_rect = Rect::from_min_max(
            egui::pos2(area.left() + pad_x, area.top() + pad_top),
            egui::pos2(area.right() - pad_x, area.bottom() - pad_bottom),
        );

        let mut page_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(inner_rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );
        page_ui.set_clip_rect(inner_rect);

        page_ui.vertical(|ui| {
            // Header Bar Pinned (Atas)
            ui.horizontal(|ui| {
                widgets::glyph(
                    ui.painter(),
                    Icon::Bolt,
                    Rect::from_center_size(ui.cursor().min + egui::vec2(8.0, 8.0), egui::vec2(13.0, 13.0)),
                    theme.accent,
                    1.5,
                );
                ui.add_space(18.0);
                ui.label(
                    egui::RichText::new("Claude Code")
                        .font(FontId::proportional(12.5))
                        .strong()
                        .color(theme.ink),
                );

                if !self.agent.session_title.is_empty() {
                    let title = if self.agent.session_title.len() > 30 {
                        format!("• {}…", &self.agent.session_title[..27])
                    } else {
                        format!("• {}", self.agent.session_title)
                    };
                    ui.label(
                        egui::RichText::new(title)
                            .font(FontId::proportional(10.5))
                            .color(theme.dim),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let AgentStatus::ToolRunning { start_time, .. } = &self.agent.status {
                        let elapsed_sec = (now_ms - start_time).max(0) / 1000;
                        ui.label(
                            egui::RichText::new(format!("{:02}:{:02}s", elapsed_sec / 60, elapsed_sec % 60))
                                .font(FontId::monospace(10.5))
                                .color(Color32::from_rgb(16, 185, 129)),
                        );
                    }
                });
            });

            ui.add_space(3.0);

            // Row Status Atas: Maskot Noki (kiri) + Status Card / Tombol Aksi (kanan)
            let mut allow_clicked = false;
            let mut always_clicked = false;
            let mut deny_clicked = false;

            ui.horizontal(|ui| {
                // 1. Maskot Interaktif Noki (38x38 px)
                let mascot_dim = 38.0;
                let (mascot_rect, mascot_resp) =
                    ui.allocate_exact_size(Vec2::new(mascot_dim, mascot_dim), egui::Sense::click());

                if mascot_resp.clicked() {
                    self.mascot.trigger_squish(anim_t);
                }

                MascotRenderer::draw_card(
                    ui.painter(),
                    mascot_rect,
                    &self.mascot,
                    &self.agent.status,
                    self.agent.active,
                    anim_t,
                );

                if mascot_resp.hovered() {
                    mascot_resp.on_hover_text("Noki • Maskot Noktah (Klik untuk memencet!)");
                }

                ui.add_space(8.0);

                // 2. Bar Status / Action Card di samping Maskot
                ui.vertical(|ui| {
                    match &self.agent.status {
                        AgentStatus::AwaitingApproval(req) => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("PERMISSION REQUIRED")
                                        .font(FontId::proportional(10.5))
                                        .strong()
                                        .color(Color32::from_rgb(245, 158, 11)),
                                );
                                ui.label(
                                    egui::RichText::new(format!("• Tool: {}", req.tool_name))
                                        .font(FontId::monospace(10.5))
                                        .color(Color32::from_rgb(226, 232, 240)),
                                );
                            });

                            ui.add_space(2.0);

                            ui.horizontal(|ui| {
                                let btn_allow = egui::Button::new(
                                    egui::RichText::new("Allow (Alt+Y)")
                                        .font(FontId::proportional(10.0))
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(16, 185, 129))
                                .rounding(Rounding::same(5.0));

                                if ui.add(btn_allow).clicked() {
                                    allow_clicked = true;
                                }

                                let btn_always = egui::Button::new(
                                    egui::RichText::new("Always")
                                        .font(FontId::proportional(10.0))
                                        .color(theme.dim),
                                )
                                .fill(Color32::from_white_alpha(20))
                                .rounding(Rounding::same(5.0));

                                if ui.add(btn_always).clicked() {
                                    always_clicked = true;
                                }

                                let btn_deny = egui::Button::new(
                                    egui::RichText::new("Deny (Alt+N)")
                                        .font(FontId::proportional(10.0))
                                        .color(Color32::from_rgb(248, 113, 113)),
                                )
                                .fill(Color32::from_rgba_unmultiplied(239, 68, 68, 40))
                                .rounding(Rounding::same(5.0));

                                if ui.add(btn_deny).clicked() {
                                    deny_clicked = true;
                                }
                            });
                        }
                        AgentStatus::ToolRunning { tool, .. } => {
                            ui.label(
                                egui::RichText::new(format!("Running Tool: [{}]", tool))
                                    .font(FontId::monospace(11.0))
                                    .strong()
                                    .color(Color32::from_rgb(16, 185, 129)),
                            );
                            ui.label(
                                egui::RichText::new("Sedang mengeksekusi di background...")
                                    .font(FontId::proportional(10.0))
                                    .color(theme.dim),
                            );
                        }
                        AgentStatus::PromptSubmitted { prompt, .. } => {
                            let preview = if prompt.len() > 36 {
                                format!("> {}…", &prompt[..33])
                            } else {
                                format!("> {}", prompt)
                            };
                            ui.label(
                                egui::RichText::new(preview)
                                    .font(FontId::proportional(11.0))
                                    .color(theme.ink),
                            );
                            ui.label(
                                egui::RichText::new("Claude sedang memproses...")
                                    .font(FontId::proportional(10.0))
                                    .color(theme.dim),
                            );
                        }
                        AgentStatus::Completed { summary, .. } => {
                            ui.label(
                                egui::RichText::new(format!("✔ {}", summary))
                                    .font(FontId::proportional(11.5))
                                    .strong()
                                    .color(Color32::from_rgb(16, 185, 129)),
                            );
                            ui.label(
                                egui::RichText::new("Jawaban lengkap tersedia di bawah (dapat di-scroll)")
                                    .font(FontId::proportional(10.0))
                                    .color(theme.dim),
                            );
                        }
                        AgentStatus::Subagents(subs) => {
                            ui.label(
                                egui::RichText::new(format!("Subagents ({} aktif)", subs.len()))
                                    .font(FontId::proportional(11.0))
                                    .color(theme.accent),
                            );
                        }
                        AgentStatus::Idle => {
                            ui.label(
                                egui::RichText::new("Agent Idle • Ready for prompts")
                                    .font(FontId::proportional(11.0))
                                    .color(theme.dim),
                            );
                            ui.label(
                                egui::RichText::new("Klik Noki untuk memencet karakternya!")
                                    .font(FontId::proportional(10.0))
                                    .color(theme.faint),
                            );
                        }
                    }
                });
            });

            // Eksekusi aksi approval di luar borrow
            if allow_clicked {
                if let Some((_, responder)) = self.pending_permission.take() {
                    let _ = responder.send(PermissionDecision {
                        behavior: "allow".into(),
                        message: None,
                        updated_input: None,
                    });
                    self.agent.status = AgentStatus::PromptSubmitted {
                        prompt: "Allowed by user".into(),
                        timestamp: now_ms,
                    };
                    self.close_panel();
                }
            } else if always_clicked {
                if let Some((_, responder)) = self.pending_permission.take() {
                    let _ = responder.send(PermissionDecision {
                        behavior: "allow".into(),
                        message: None,
                        updated_input: None,
                    });
                    self.agent.status = AgentStatus::PromptSubmitted {
                        prompt: "Always allowed by user".into(),
                        timestamp: now_ms,
                    };
                    self.close_panel();
                }
            } else if deny_clicked {
                if let Some((_, responder)) = self.pending_permission.take() {
                    let _ = responder.send(PermissionDecision {
                        behavior: "deny".into(),
                        message: Some("Rejected via Noktah UI".into()),
                        updated_input: None,
                    });
                    self.agent.status = AgentStatus::Idle;
                    self.close_panel();
                }
            }

            ui.add_space(4.0);

            // Scrollable Content Area (Dapat di-scroll vertikal dan merender Markdown Read Mode)
            let scroll_h = (inner_rect.bottom() - ui.cursor().top()).max(0.0);

            egui::ScrollArea::vertical()
                .id_salt("agent_markdown_scroll")
                .max_height(scroll_h)
                .min_scrolled_height(0.0)
                .auto_shrink([false, true])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.set_width(inner_rect.width());

                    match &self.agent.status.clone() {
                        AgentStatus::Completed { last_message, .. } => {
                            if !last_message.is_empty() {
                                Frame::none()
                                    .fill(Color32::from_rgba_unmultiplied(20, 22, 28, 160))
                                    .rounding(Rounding::same(8.0))
                                    .stroke(Stroke::new(1.0, Color32::from_white_alpha(18)))
                                    .inner_margin(Margin::same(8.0))
                                    .show(ui, |ui| {
                                        render_markdown(ui, last_message, &theme);
                                    });
                            } else {
                                ui.label(
                                    egui::RichText::new("Tidak ada pesan balasan.")
                                        .font(FontId::proportional(10.5))
                                        .color(theme.dim),
                                );
                            }
                        }
                        AgentStatus::AwaitingApproval(req) => {
                            Frame::none()
                                .fill(Color32::from_rgba_unmultiplied(28, 22, 14, 200))
                                .rounding(Rounding::same(8.0))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(245, 158, 11)))
                                .inner_margin(Margin::same(8.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new("COMMAND / INPUT DETAIL:")
                                            .font(FontId::monospace(9.5))
                                            .strong()
                                            .color(Color32::from_rgb(245, 158, 11)),
                                    );
                                    ui.add_space(2.0);

                                    let preview = if let Some(cmd) =
                                        req.tool_input.get("command").and_then(|v| v.as_str())
                                    {
                                        cmd.to_string()
                                    } else if let Some(path) =
                                        req.tool_input.get("file_path").and_then(|v| v.as_str())
                                    {
                                        format!("File Path: {}", path)
                                    } else {
                                        serde_json::to_string_pretty(&req.tool_input).unwrap_or_default()
                                    };

                                    ui.label(
                                        egui::RichText::new(preview)
                                            .font(FontId::monospace(10.0))
                                            .color(Color32::from_rgb(241, 245, 249)),
                                    );
                                });
                        }
                        AgentStatus::ToolRunning { input, .. } => {
                            Frame::none()
                                .fill(Color32::from_black_alpha(150))
                                .rounding(Rounding::same(8.0))
                                .stroke(Stroke::new(1.0, Color32::from_white_alpha(20)))
                                .inner_margin(Margin::same(8.0))
                                .show(ui, |ui| {
                                    let detail = if let Some(cmd) = input.get("command").and_then(|v| v.as_str()) {
                                        cmd
                                    } else if let Some(p) = input.get("file_path").and_then(|v| v.as_str()) {
                                        p
                                    } else {
                                        "sedang berjalan..."
                                    };
                                    ui.label(
                                        egui::RichText::new(detail)
                                            .font(FontId::monospace(10.5))
                                            .color(theme.ink),
                                    );
                                });
                        }
                        AgentStatus::PromptSubmitted { prompt, .. } => {
                            if prompt.len() > 36 {
                                Frame::none()
                                    .fill(Color32::from_rgba_unmultiplied(24, 20, 36, 180))
                                    .rounding(Rounding::same(8.0))
                                    .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(139, 92, 246, 60)))
                                    .inner_margin(Margin::same(8.0))
                                    .show(ui, |ui| {
                                        render_markdown(ui, prompt, &theme);
                                    });
                            }
                        }
                        AgentStatus::Subagents(subs) => {
                            for s in subs {
                                ui.label(
                                    egui::RichText::new(format!("• {} [{}]", s.agent_type, s.status))
                                        .font(FontId::monospace(10.5))
                                        .color(theme.accent),
                                );
                            }
                        }
                        AgentStatus::Idle => {
                            ui.label(
                                egui::RichText::new("Menunggu perintah CLI Claude Code...")
                                    .font(FontId::proportional(10.5))
                                    .color(theme.faint),
                            );
                        }
                    }
                });
        });
    }
}
