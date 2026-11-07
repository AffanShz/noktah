//! Renderer Markdown Read Mode untuk egui.
//!
//! Mengonversi teks Markdown (dari hook Claude Code) menjadi teks kaya berformat:
//! - Heading (# H1, ## H2, ### H3) dengan ukuran tipografi hierarkis dan warna aksen
//! - Divider (--- atau ***) sebagai garis pemisah horizontal
//! - Code blocks (```lang ... ```) dalam kotak berlatar gelap dengan font monospace
//! - Unordered list (- atau *) dan Ordered list (1.) dengan bullet dot
//! - Inline formatting: **bold**, `inline code`, dan [link](url)
//! - Line wrapping dan spasi vertikal yang rapi di dalam panel notch

use crate::presentation::theme::Theme;
use eframe::egui::{self, Color32, FontId, Frame, Margin, RichText, Rounding, Stroke};

#[derive(Clone, Debug, PartialEq)]
enum InlineToken {
    Text(String),
    Bold(String),
    Code(String),
    Link { label: String, url: String },
}

/// Parsing sebaris teks menjadi token-token inline (Plain, Bold, Code, Link).
fn parse_inline_tokens(line: &str) -> Vec<InlineToken> {
    let mut tokens = Vec::new();
    let mut chars = line.chars().peekable();
    let mut buffer = String::new();

    while let Some(&ch) = chars.peek() {
        if ch == '*' {
            chars.next();
            if chars.peek() == Some(&'*') {
                chars.next();
                // Simpan teks biasa sebelumnya
                if !buffer.is_empty() {
                    tokens.push(InlineToken::Text(std::mem::take(&mut buffer)));
                }
                // Ambil isi bold sampai '**'
                let mut bold_text = String::new();
                while let Some(b) = chars.next() {
                    if b == '*' && chars.peek() == Some(&'*') {
                        chars.next();
                        break;
                    }
                    bold_text.push(b);
                }
                tokens.push(InlineToken::Bold(bold_text));
            } else {
                buffer.push('*');
            }
        } else if ch == '`' {
            chars.next();
            if !buffer.is_empty() {
                tokens.push(InlineToken::Text(std::mem::take(&mut buffer)));
            }
            let mut code_text = String::new();
            while let Some(c) = chars.next() {
                if c == '`' {
                    break;
                }
                code_text.push(c);
            }
            tokens.push(InlineToken::Code(code_text));
        } else if ch == '[' {
            chars.next();
            let mut label = String::new();
            let mut closed = false;
            while let Some(l) = chars.next() {
                if l == ']' {
                    closed = true;
                    break;
                }
                label.push(l);
            }
            if closed && chars.peek() == Some(&'(') {
                chars.next();
                let mut url = String::new();
                while let Some(u) = chars.next() {
                    if u == ')' {
                        break;
                    }
                    url.push(u);
                }
                if !buffer.is_empty() {
                    tokens.push(InlineToken::Text(std::mem::take(&mut buffer)));
                }
                tokens.push(InlineToken::Link { label, url });
            } else {
                buffer.push('[');
                buffer.push_str(&label);
                if closed {
                    buffer.push(']');
                }
            }
        } else {
            buffer.push(ch);
            chars.next();
        }
    }

    if !buffer.is_empty() {
        tokens.push(InlineToken::Text(buffer));
    }

    tokens
}

/// Render sebaris token inline dalam flow horizontal_wrapped.
fn render_inline_line(ui: &mut egui::Ui, line: &str, theme: &Theme) {
    let tokens = parse_inline_tokens(line);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for token in tokens {
            match token {
                InlineToken::Text(text) => {
                    ui.label(
                        RichText::new(text)
                            .font(FontId::proportional(11.0))
                            .color(theme.ink),
                    );
                }
                InlineToken::Bold(text) => {
                    ui.label(
                        RichText::new(text)
                            .font(FontId::proportional(11.0))
                            .strong()
                            .color(Color32::WHITE),
                    );
                }
                InlineToken::Code(code) => {
                    Frame::none()
                        .fill(Color32::from_rgba_unmultiplied(30, 41, 59, 220))
                        .rounding(Rounding::same(3.0))
                        .stroke(Stroke::new(0.8, Color32::from_white_alpha(30)))
                        .inner_margin(Margin::symmetric(4.0, 1.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(code)
                                    .font(FontId::monospace(10.0))
                                    .color(Color32::from_rgb(56, 189, 248)),
                            );
                        });
                }
                InlineToken::Link { label, url } => {
                    let link_label = if label.is_empty() { &url } else { &label };
                    let resp = ui
                        .link(
                            RichText::new(link_label)
                                .font(FontId::proportional(11.0))
                                .underline()
                                .color(theme.accent),
                        )
                        .on_hover_text(&url);
                    if resp.clicked() {
                        #[cfg(windows)]
                        {
                            let _ = std::process::Command::new("cmd")
                                .args(["/C", "start", "", &url])
                                .spawn();
                        }
                    }
                }
            }
        }
    });
}

/// Merender dokumen markdown lengkap dalam mode read mode yang rapi.
pub fn render_markdown(ui: &mut egui::Ui, markdown: &str, theme: &Theme) {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim_end();

        // 1. Code Block (``` ... ```)
        if line.starts_with("```") {
            let lang = line.trim_start_matches('`').trim();
            i += 1;
            let mut code_lines = Vec::new();
            while i < lines.len() && !lines[i].starts_with("```") {
                code_lines.push(lines[i]);
                i += 1;
            }
            if i < lines.len() && lines[i].starts_with("```") {
                i += 1;
            }
            let code_content = code_lines.join("\n");

            ui.add_space(2.0);
            Frame::none()
                .fill(Color32::from_rgb(18, 20, 26))
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0, Color32::from_white_alpha(24)))
                .inner_margin(Margin::same(6.0))
                .show(ui, |ui| {
                    if !lang.is_empty() {
                        ui.label(
                            RichText::new(lang.to_uppercase())
                                .font(FontId::monospace(9.0))
                                .color(theme.dim),
                        );
                        ui.add_space(2.0);
                    }
                    ui.label(
                        RichText::new(code_content)
                            .font(FontId::monospace(10.0))
                            .color(Color32::from_rgb(226, 232, 240)),
                    );
                });
            ui.add_space(3.0);
            continue;
        }

        // 2. Horizontal Divider (--- atau ***)
        if line.starts_with("---") || line.starts_with("***") {
            ui.add_space(3.0);
            ui.separator();
            ui.add_space(3.0);
            i += 1;
            continue;
        }

        // 3. Headings (# H1, ## H2, ### H3)
        if let Some(h1) = line.strip_prefix("# ") {
            ui.add_space(4.0);
            ui.label(
                RichText::new(h1)
                    .font(FontId::proportional(13.5))
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.add_space(2.0);
            i += 1;
            continue;
        }
        if let Some(h2) = line.strip_prefix("## ") {
            ui.add_space(3.0);
            ui.label(
                RichText::new(h2)
                    .font(FontId::proportional(12.5))
                    .strong()
                    .color(theme.accent),
            );
            ui.add_space(2.0);
            i += 1;
            continue;
        }
        if let Some(h3) = line.strip_prefix("### ") {
            ui.add_space(3.0);
            ui.label(
                RichText::new(h3)
                    .font(FontId::proportional(11.5))
                    .strong()
                    .color(Color32::from_rgb(56, 189, 248)),
            );
            ui.add_space(1.0);
            i += 1;
            continue;
        }

        // 4. Bullet list (- item atau * item)
        if let Some(item) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("•")
                        .font(FontId::proportional(11.0))
                        .color(theme.accent),
                );
                render_inline_line(ui, item, theme);
            });
            i += 1;
            continue;
        }

        // 5. Ordered list (1. item, 2. item, dsb.)
        let is_ordered_list = line.chars().next().map_or(false, |c| c.is_ascii_digit())
            && line.find(". ").is_some();

        if is_ordered_list {
            if let Some(dot_idx) = line.find(". ") {
                let num = &line[..=dot_idx];
                let item = &line[dot_idx + 2..];
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(num)
                            .font(FontId::proportional(10.5))
                            .color(theme.dim),
                    );
                    render_inline_line(ui, item, theme);
                });
                i += 1;
                continue;
            }
        }

        // 6. Baris Kosong
        if line.trim().is_empty() {
            ui.add_space(3.0);
            i += 1;
            continue;
        }

        // 7. Paragraf biasa dengan formatting inline
        render_inline_line(ui, line, theme);
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_inline_tokens() {
        let line = "Halo **dunia** dengan `kode` dan [Coucou](https://coucou.app)";
        let tokens = parse_inline_tokens(line);
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0], InlineToken::Text("Halo ".into()));
        assert_eq!(tokens[1], InlineToken::Bold("dunia".into()));
        assert_eq!(tokens[2], InlineToken::Text(" dengan ".into()));
        assert_eq!(tokens[3], InlineToken::Code("kode".into()));
        assert_eq!(tokens[4], InlineToken::Text(" dan ".into()));
        assert_eq!(
            tokens[5],
            InlineToken::Link {
                label: "Coucou".into(),
                url: "https://coucou.app".into()
            }
        );
    }
}
