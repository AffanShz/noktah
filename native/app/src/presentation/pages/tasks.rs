//! Halaman Tugas — tata letak `.field` + `.tlist` dari prototipe
//! `noktah-redesign.html`.
//!
//! Kolom input berupa kapsul 42 px dengan latar `fill`; setiap baris tugas
//! 38 px dipisah garis rambut 0.5 px, bukan kartu terpisah. Pemisah tipis
//! itu yang membuat daftar panjang terbaca sebagai satu blok, bukan tumpukan
//! kotak.

use crate::app::{Noktah, NotchPage};
use crate::presentation::{icons::Icon, style, widgets};
use eframe::egui::{self, Color32, Id, Rect, Rounding, Sense, Stroke};
use noktah_core::{format, model::Todo};

impl Noktah {
    /// Halaman 2: Todo / Tasks Page.
    ///
    /// The list previously painted raw task text with no clip, so a long task
    /// ran under the delete glyph and off the card. Rows now reserve the
    /// trailing affordance first and let the label take exactly what is left.
    pub(crate) fn draw_page_todo(&mut self, ui: &mut egui::Ui, area: Rect) {
        let theme = self.theme;
        let now_ms = chrono::Local::now().timestamp_millis();
        let open_count = self.todo_open_count();
        let done_count = self.todos.iter().filter(|t| t.done).count();

        let header_title = {
            let name = self.settings.user_name.trim();
            if name.is_empty() || name.eq_ignore_ascii_case("you") || name.eq_ignore_ascii_case("kamu") {
                "TASKS".to_string()
            } else {
                format!("TASKS • {}", name.to_uppercase())
            }
        };

        let lbl_tasks = self.tr("tasks.mode_tasks").to_string();
        let lbl_notes = self.tr("tasks.mode_notes").to_string();
        let mut next_mode = self.notes_mode;
        let mut copy_notes = false;
        let has_notes = !self.scratchpad.content.trim().is_empty();

        let _header_h = widgets::page_header(ui, &header_title, &theme, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                if !next_mode {
                    let teks = if self.todos.is_empty() {
                        String::new()
                    } else if open_count > 0 {
                        format!("{open_count} left")
                    } else {
                        "All done".to_string()
                    };
                    if !teks.is_empty() {
                        ui.label(style::cap(teks, theme.dim));
                        ui.add_space(4.0);
                    }
                } else if has_notes && widgets::chip_button(ui, "Copy", false, &theme) {
                    copy_notes = true;
                }
                if widgets::chip_button(ui, &lbl_tasks, !next_mode, &theme) {
                    next_mode = false;
                }
                if widgets::chip_button(ui, &lbl_notes, next_mode, &theme) {
                    next_mode = true;
                }
            });
        });

        self.notes_mode = next_mode;
        if copy_notes {
            ui.output_mut(|o| o.copied_text = self.scratchpad.content.clone());
            self.banner = Some((
                self.tr("tasks.notes_copied").to_string(),
                std::time::Instant::now(),
            ));
        }

        if self.notes_mode {
            let notes_h = (area.bottom() - ui.cursor().top()).max(32.0);
            let box_rect = Rect::from_min_size(ui.cursor().min, egui::vec2(area.width(), notes_h));
            ui.painter()
                .rect_filled(box_rect, Rounding::same(style::R_CONTROL), theme.surface);

            let hint = self.tr("tasks.notes_hint").to_string();
            let mut changed = false;

            ui.add_space(6.0);
            let scroll_h = (notes_h - 12.0).max(20.0);
            egui::ScrollArea::vertical()
                .id_salt("page_notes_scroll")
                .max_height(scroll_h)
                .auto_shrink([false, false])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        let resp = ui.add(
                            egui::TextEdit::multiline(&mut self.scratchpad.content)
                                .id_salt("tasks_scratchpad_input")
                                .hint_text(hint)
                                .desired_width((area.width() - 20.0).max(60.0))
                                .desired_rows(4)
                                .font(egui::FontId::proportional(style::T_BODY))
                                .text_color(theme.ink)
                                .frame(false),
                        );
                        changed = resp.changed();
                    });
                });

            if changed {
                self.touch_notes();
            }
            return;
        }

        // ---- Kolom input (`.field` 32 px, radius 14) ----
        const FIELD_H: f32 = 32.0;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(area.width(), FIELD_H), Sense::hover());
        ui.painter()
            .rect_filled(rect, Rounding::same(style::R_CONTROL), theme.surface);

        widgets::glyph(
            ui.painter(),
            Icon::Plus,
            Rect::from_center_size(
                egui::pos2(rect.left() + 16.0, rect.center().y),
                egui::vec2(14.0, 14.0),
            ),
            theme.accent,
            1.5,
        );

        let inner = Rect::from_min_max(
            egui::pos2(rect.left() + 34.0, rect.top() + 4.0),
            egui::pos2(rect.right() - 12.0, rect.bottom() - 4.0),
        );
        let text_resp = ui.put(
            inner,
            egui::TextEdit::singleline(&mut self.todo_input)
                .hint_text("Add a task, press Enter…")
                .desired_width(inner.width())
                .frame(false)
                .vertical_align(egui::Align::Center),
        );
        if text_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let text = self.todo_input.trim().to_string();
            if !text.is_empty() {
                self.todos
                    .insert(0, Todo::new(format!("t_{now_ms}"), text, now_ms));
                self.todo_input.clear();
                self.save_todos();
            }
        }

        ui.add_space(style::S1);

        // ---- Daftar (`.tlist li` 30 px, garis bawah 0.5) ----
        let list_h = (area.bottom() - ui.cursor().top()).max(0.0);
        let row_h = 30.0;
        let check_d = 18.0;
        let delete_d = 24.0;

        egui::ScrollArea::vertical()
            .id_salt("page_todo_scroll")
            .max_height(list_h)
            .min_scrolled_height(0.0)
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.set_width(area.width());
                ui.spacing_mut().item_spacing.y = 0.0;

                if self.todos.is_empty() {
                    // `.empty` di prototipe: dua baris, keduanya redup.
                    ui.vertical_centered(|ui| {
                        ui.add_space(style::S1);
                        let greeting = {
                            let name = self.settings.user_name.trim();
                            if name.is_empty() || name.eq_ignore_ascii_case("you") || name.eq_ignore_ascii_case("kamu") {
                                "Nothing here yet".to_string()
                            } else {
                                format!("Semua beres, {name}!")
                            }
                        };
                        ui.label(style::subtitle(greeting, theme.dim));
                        ui.label(style::cap("Add your first task above", theme.faint));
                    });
                    return;
                }

                let mut to_toggle: Option<String> = None;
                let mut to_delete: Option<String> = None;

                let visible: Vec<&Todo> = self.todos.iter().take(8).collect();
                for (idx, todo) in visible.iter().enumerate() {
                    let (row_rect, row_resp) =
                        ui.allocate_exact_size(egui::vec2(area.width(), row_h), Sense::hover());
                    let hover = row_resp.hovered();

                    // `.tlist li{border-bottom:.5px solid var(--sep)}`
                    ui.painter().line_segment(
                        [
                            egui::pos2(row_rect.left(), row_rect.bottom() - 0.5),
                            egui::pos2(row_rect.right(), row_rect.bottom() - 0.5),
                        ],
                        Stroke::new(1.0, theme.stroke),
                    );

                    // `.chk`: lingkaran 20 px, kosong saat aktif, terisi aksen
                    // dengan centang saat selesai.
                    let box_rect = Rect::from_center_size(
                        egui::pos2(row_rect.left() + 6.0 + check_d / 2.0, row_rect.center().y),
                        egui::vec2(check_d, check_d),
                    );
                    let check_resp =
                        ui.interact(box_rect, Id::new("todo_chk").with(idx), Sense::click());
                    if check_resp.clicked() {
                        to_toggle = Some(todo.id.clone());
                    }

                    if todo.done {
                        ui.painter()
                            .circle_filled(box_rect.center(), check_d / 2.0, theme.accent);
                        widgets::glyph(
                            ui.painter(),
                            Icon::Check,
                            Rect::from_center_size(box_rect.center(), egui::vec2(12.0, 12.0)),
                            theme.on_accent,
                            2.4,
                        );
                    } else {
                        ui.painter().circle_stroke(
                            box_rect.center(),
                            check_d / 2.0 - 0.75,
                            Stroke::new(
                                1.5,
                                if check_resp.hovered() { theme.ink } else { theme.faint },
                            ),
                        );
                    }

                    // Sisa lebar setelah checkbox dan tombol hapus.
                    let text_left = box_rect.right() + style::S3;
                    let text_right = row_rect.right() - delete_d - style::S1;
                    let text_w = (text_right - text_left).max(20.0);

                    let text_col = if todo.done { theme.faint } else { theme.ink };
                    let label = widgets::ellipsis(ui, &todo.text, text_w);

                    let job = egui::text::LayoutJob::simple_singleline(
                        label,
                        egui::FontId::proportional(style::T_BODY),
                        text_col,
                    );
                    let galley = ui.painter().layout_job(job);
                    ui.painter().galley(
                        egui::pos2(text_left, row_rect.center().y - galley.size().y / 2.0),
                        galley.clone(),
                        text_col,
                    );

                    if todo.done {
                        let y = row_rect.center().y;
                        let w = galley.size().x.min(text_w);
                        ui.painter().line_segment(
                            [egui::pos2(text_left, y), egui::pos2(text_left + w, y)],
                            Stroke::new(1.0, theme.faint),
                        );
                    }

                    // Jam tenggat, rata kanan di kolomnya sendiri.
                    if let Some(due) = todo.due_at {
                        ui.painter().text(
                            egui::pos2(text_right, row_rect.center().y),
                            egui::Align2::RIGHT_CENTER,
                            format::due_label(due, now_ms),
                            style::caption(),
                            theme.accent,
                        );
                    }

                    // `.del{opacity:0}` — hanya muncul saat baris di-hover.
                    if hover {
                        let del_rect = Rect::from_center_size(
                            egui::pos2(
                                row_rect.right() - delete_d / 2.0 - 4.0,
                                row_rect.center().y,
                            ),
                            egui::vec2(delete_d, delete_d),
                        );
                        let del_resp = ui.interact(
                            del_rect,
                            Id::new("todo_del").with(idx),
                            Sense::click(),
                        );
                        if del_resp.clicked() {
                            to_delete = Some(todo.id.clone());
                        }
                        widgets::glyph(
                            ui.painter(),
                            Icon::Trash,
                            Rect::from_center_size(del_rect.center(), egui::vec2(15.0, 15.0)),
                            if del_resp.hovered() {
                                Color32::from_rgb(229, 72, 77)
                            } else {
                                theme.dim
                            },
                            1.5,
                        );
                    }
                }

                if let Some(id) = to_toggle {
                    if let Some(t) = self.todos.iter_mut().find(|t| t.id == id) {
                        t.done = !t.done;
                    }
                    self.save_todos();
                }
                if let Some(id) = to_delete {
                    self.todos.retain(|t| t.id != id);
                    self.save_todos();
                }

                // Katakan apa yang disembunyikan, jangan diam-diam dipotong.
                if self.todos.len() > 8 {
                    ui.add_space(style::S2);
                    ui.label(style::cap(
                        format!("+{} more", self.todos.len() - 8),
                        theme.faint,
                    ));
                }

                if done_count > 0 && open_count == 0 {
                    ui.add_space(style::S2);
                    ui.label(style::cap("All done for today", theme.accent));
                }
            });
    }
}
