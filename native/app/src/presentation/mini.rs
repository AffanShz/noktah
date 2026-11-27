//! Baris minimized: satu baris horizontal berisi potongan-potongan yang bisa
//! dinyalakan/dimatikan, dengan lebar yang dihitung dari isinya.

use crate::presentation::icons::{self, Icon};
use crate::presentation::theme::{self, Theme};
use eframe::egui::{self, Color32, FontId, Painter, Rect};
use noktah_core::model::MediaState;
use noktah_core::Settings;

#[allow(dead_code)]
pub struct MiniCtx<'a> {
    pub theme: &'a Theme,
    pub settings: &'a Settings,
    pub media: &'a MediaState,
    /// Apakah baris minimized menampilkan mode pemutar musik (bukan mode idle).
    pub show_media: bool,
    pub lyric: Option<&'a str>,
    pub prev_lyric: Option<&'a str>,
    pub lyric_trans: f32,
    /// Waktu sejak lirik aktif dimulai, untuk sinkronisasi scroll ping-pong dari awal.
    pub lyric_t: f32,
    /// Lagu ini punya lirik? Kalau tidak, slot lirik dibuang dari tata letak
    /// supaya pill mengecil selebar item yang dicentang saja.
    pub has_lyrics: bool,
    /// Persen pemakaian CPU dan RAM, untuk monitor di baris minimized.
    pub cpu: f32,
    pub ram: f32,
    pub art: Option<&'a egui::TextureHandle>,
    pub clock: String,
    pub weather: Option<String>,
    /// Waktu berjalan, untuk animasi equalizer dan marquee.
    pub t: f32,
    /// Energi empat pita dari audio sistem; nol berarti belum ada data.
    pub levels: [f32; 4],
    /// Faktor opasitas saat animasi transisi morph cair (0.0 .. 1.0).
    pub alpha: f32,
    pub is_charging: bool,
    pub battery_pct: u8,
    pub album_color: Color32,
    /// Animasi dimatikan pengguna. Aurora progres memakai ini untuk memilih
    /// denyut konstan, bukan sinus.
    pub reduce_motion: bool,
    pub agent: &'a noktah_core::model::AgentState,
    pub mascot: &'a crate::presentation::mascot::MascotState,
    pub banner: Option<&'a str>,
    pub pending_approval_count: usize,
    pub focus_dnd_rem: Option<u32>,
    pub custom_tile: Option<&'a noktah_core::CustomTile>,
}

/// Pemetaan TileTone ke Color32 yang sesuai tema atau semantik status.
pub fn tone_color(tone: noktah_core::TileTone, theme: &Theme) -> Color32 {
    match tone {
        noktah_core::TileTone::Neutral => theme.ink,
        noktah_core::TileTone::Accent => theme.accent,
        noktah_core::TileTone::Success => Color32::from_rgb(34, 197, 94),
        noktah_core::TileTone::Warning => Color32::from_rgb(245, 158, 11),
        noktah_core::TileTone::Error => Color32::from_rgb(239, 68, 68),
    }
}

const PAD: f32 = 20.0;
/// Ukuran teks lirik di slot tengah (sebelumnya hardcoded 12.0).
const LYR_FONT_SIZE: f32 = 11.5;
/// Setengah tinggi slot lirik; teks 11.5 px nyaman di dalam pita 20 px.
const LYR_HALF_H: f32 = 9.0;
/// Ruang napas di ujung kanan supaya baris terpotong tidak menyentuh pembatas.
const LYR_ELLIPSIS_PAD: f32 = 3.0;
/// Batas item pada grup trio tengah saat tidak ada lirik.
const NO_LYR_MAX_ITEMS: usize = 3;

pub fn ping_pong_offset(text_w: f32, max_w: f32, t: f32) -> f32 {
    if text_w <= max_w {
        return 0.0;
    }
    let max_scroll = text_w - max_w;
    let scroll_dur = (max_scroll / 26.0).max(0.6);
    const PAUSE: f32 = 1.8;
    let cycle_dur = PAUSE + scroll_dur + PAUSE + scroll_dur;
    let time_in_cycle = (t % cycle_dur).max(0.0);

    let offset = if time_in_cycle < PAUSE {
        0.0
    } else if time_in_cycle < PAUSE + scroll_dur {
        let progress = (time_in_cycle - PAUSE) / scroll_dur;
        let ease = (1.0 - (progress * std::f32::consts::PI).cos()) * 0.5;
        ease * max_scroll
    } else if time_in_cycle < PAUSE + scroll_dur + PAUSE {
        max_scroll
    } else {
        let progress = (time_in_cycle - (PAUSE + scroll_dur + PAUSE)) / scroll_dur;
        let ease = (1.0 - (progress * std::f32::consts::PI).cos()) * 0.5;
        (1.0 - ease) * max_scroll
    };

    offset.clamp(0.0, max_scroll)
}

/// Teks satu baris yang dipotong di dalam kotak tertentu; berjalan pelan kalau tidak muat.
fn clipped_text_in_rect(
    painter: &Painter,
    text: &str,
    font: FontId,
    color: Color32,
    x: f32,
    center_y: f32,
    max_w: f32,
    t: f32,
    clip_rect: Rect,
) {
    if max_w <= 4.0 || text.is_empty() {
        return;
    }
    let galley = painter.layout_no_wrap(text.to_owned(), font, color);
    let lebar = galley.size().x;
    let pos_x = x - ping_pong_offset(lebar, max_w, t);

    painter.with_clip_rect(clip_rect).galley(
        egui::pos2(pos_x, center_y - galley.size().y / 2.0),
        galley,
        color,
    );
}

/// Teks satu baris yang dipotong di dalam kotak; berjalan pelan kalau tidak muat.
fn clipped_text(
    painter: &Painter,
    text: &str,
    font: FontId,
    color: Color32,
    x: f32,
    center_y: f32,
    max_w: f32,
    t: f32,
) {
    if max_w <= 4.0 || text.is_empty() {
        return;
    }
    let galley = painter.layout_no_wrap(text.to_owned(), font.clone(), color);
    let clip = Rect::from_min_size(
        egui::pos2(x, center_y - galley.size().y / 2.0 - 2.0),
        egui::vec2(max_w, galley.size().y + 4.0),
    );
    clipped_text_in_rect(painter, text, font, color, x, center_y, max_w, t, clip);
}

/// Potong satu baris lirik ber-ellipsis CSS bila melebihi `max_w`.
///
/// Kalau teks muat, dikembalikan apa adanya. Kalau tidak, karakter di akhir
/// dibuang satu per satu sampai awalan + `...` muat di dalam `max_w`. Pemotongan
/// memakai batas karakter (`str::chars`) sehingga Hangul/Kanji tidak pernah
/// terbelah di tengah UTF-8, dan baris yang terlalu panjang selalu berakhir
/// `...` — tidak pernah kosong atau berubah jadi `?`.
fn ellipsize(painter: &Painter, line: &str, font: &FontId, color: Color32, max_w: f32) -> Option<String> {
    if line.is_empty() {
        return None;
    }

    const DOTS: &str = "...";
    let width_of = |s: &str| painter.layout_no_wrap(s.to_owned(), font.clone(), color).size().x;

    if width_of(line) <= max_w {
        return Some(line.to_owned());
    }

    // Kalau satu karakter pun tidak muat bersama `...`, tetap kembalikan
    // `...` saja di depan satu karakter itu — hasilnya tidak pernah kosong
    // dan tidak pernah `?`.
    let first: String = line.chars().take(1).collect();
    let first_plus = format!("{first}{DOTS}");
    if width_of(&first_plus) > max_w {
        return Some(first_plus);
    }

    // Buang karakter dari ujung sampai awalan + `...` pas di dalam max_w.
    let mut buf = String::with_capacity(line.len());
    let mut ok: Option<String> = None;
    for c in line.chars() {
        buf.push(c);
        let cand = format!("{buf}{DOTS}");
        if width_of(&cand) <= max_w {
            ok = Some(cand);
        } else {
            break;
        }
    }
    ok.or_else(|| Some(first_plus))
}

/// Pembersih nama tool Claude Code: menghapus prefix MCP berbelit
/// (misal `mcp__terminal__read_terminal` menjadi `read_terminal`).
pub(crate) fn clean_tool_name(tool: &str) -> &str {
    if let Some(stripped) = tool.strip_prefix("mcp__") {
        stripped.split("__").last().unwrap_or(stripped)
    } else if let Some(stripped) = tool.strip_prefix("mcp:") {
        stripped.split(':').last().unwrap_or(stripped)
    } else {
        tool
    }
}

/// Format label status agent atau jam untuk pill minimal.
pub(crate) fn format_agent_status_label(
    status: &noktah_core::model::AgentStatus,
    clock: &str,
) -> String {
    match status {
        noktah_core::model::AgentStatus::AwaitingApproval(_) => {
            "Izin [Alt+Y / Alt+N]".to_string()
        }
        noktah_core::model::AgentStatus::ToolRunning { tool, .. } => {
            let clean = clean_tool_name(tool);
            format!("Agent: {clean}")
        }
        noktah_core::model::AgentStatus::PromptSubmitted { .. } => {
            "Agent Berpikir...".to_string()
        }
        noktah_core::model::AgentStatus::Error { message, .. } => {
            format!("Error: {}", clean_tool_name(message))
        }
        _ => clock.to_string(),
    }
}

/// Menggambar baris minimal saat tidak memutar musik:
/// [ikon ▶/agent] + [jam/status] + [equalizer], mengikuti `.compact` di prototipe.
///
/// Prototipe selalu menampilkan lencana sampul kecil di kiri dan equalizer di
/// kanan; jam mengisi sisa ruang di tengah.
fn draw_minimal_pill(painter: &Painter, rect: Rect, ctx: &MiniCtx) {
    let theme = ctx.theme;
    let cy = rect.center().y;

    let (center_label, custom_tone) = if let Some(banner) = ctx.banner {
        (banner.to_string(), None)
    } else if ctx.agent.active {
        let label = if matches!(
            ctx.agent.status,
            noktah_core::model::AgentStatus::AwaitingApproval(_)
        ) && ctx.pending_approval_count > 1
        {
            format!("Izin (1/{}) [Alt+Y/N]", ctx.pending_approval_count)
        } else {
            format_agent_status_label(&ctx.agent.status, &ctx.clock)
        };
        (label, None)
    } else if let Some(rem) = ctx.focus_dnd_rem {
        (format!("DND {:02}:{:02} • {}", rem / 60, rem % 60, ctx.clock), None)
    } else if let Some(tile) = ctx.custom_tile {
        let label = if tile.value.is_empty() {
            tile.title.clone()
        } else {
            format!("{}: {}", tile.title, tile.value)
        };
        (label, Some(tile.tone))
    } else {
        (ctx.clock.clone(), None)
    };

    // 1. Lencana kecil di kiri (`.art.sm` 24×24, radius 8) - Maskot 2 mata interaktif Noki
    let badge_d = 24.0;
    let badge = Rect::from_center_size(
        egui::pos2(rect.left() + 20.0 + badge_d / 2.0, cy),
        egui::vec2(badge_d, badge_d),
    );
    crate::presentation::mascot::MascotRenderer::draw_mini(
        painter,
        badge,
        ctx.mascot,
        &ctx.agent.status,
        true,
        ctx.t,
    );

    // 2. Equalizer di kanan (`.eq`): 4 batang, jarak 2 px.
    let n_bars = 4;
    let bar_w = 3.0;
    let spacing = 2.0;
    let eq_right = rect.right() - 20.0;
    let eq_left = eq_right - (n_bars as f32 * (bar_w + spacing) - spacing);

    for i in 0..n_bars {
        let bx = eq_left + i as f32 * (bar_w + spacing) + bar_w / 2.0;
        let energy = ctx.levels[i];
        let h = if ctx.media.playing {
            (4.0 + energy * 10.0).clamp(4.0, 14.0)
        } else {
            let phase = ctx.t * 2.5 + i as f32 * 0.8;
            4.0 + phase.sin().abs() * 4.0
        };
        let bar_rect = Rect::from_center_size(egui::pos2(bx, cy), egui::vec2(bar_w, h));
        painter.rect_filled(bar_rect, egui::Rounding::same(2.0), theme.accent);
    }

    // 3. Jam / Status Claude mengisi ruang antara lencana dan equalizer.
    // Dibatasi ketat dengan padding aman agar tidak menabrak badge (kiri) maupun equalizer (kanan).
    let label_left = badge.right() + 8.0;
    let label_right = eq_left - 8.0;
    let label_rect = Rect::from_min_max(
        egui::pos2(label_left, rect.top()),
        egui::pos2(label_right, rect.bottom()),
    );
    let max_w = (label_right - label_left).max(0.0);
    let font = FontId::proportional(12.0);

    if max_w > 10.0 {
        let text_color = if ctx.banner.is_some() {
            theme.accent
        } else if let Some(tone) = custom_tone {
            tone_color(tone, theme)
        } else {
            theme.ink
        };
        let display_text = ellipsize(painter, &center_label, &font, text_color, max_w)
            .unwrap_or_else(|| center_label.clone());
        painter.with_clip_rect(label_rect).text(
            label_rect.center(),
            egui::Align2::CENTER_CENTER,
            &display_text,
            font,
            text_color,
        );
    }

    // Aura status agent di dasar pill saat minimal
    if ctx.agent.active {
        agent_aura(painter, rect, ctx);
    } else if ctx.banner.is_none() && ctx.focus_dnd_rem.is_none() {
        if let Some(tile) = ctx.custom_tile {
            if let Some(p) = tile.progress {
                if let Some((jalur, _)) = progress_geometry(rect) {
                    let color = tone_color(tile.tone, theme);
                    let w = (jalur.width() * p.clamp(0.0, 1.0)).max(0.0);
                    if w > 0.0 {
                        let prog_rect = Rect::from_min_size(jalur.min, egui::vec2(w, jalur.height()));
                        painter.rect_filled(
                            prog_rect,
                            egui::Rounding::same(1.5),
                            with_alpha(color, 220.0 * ctx.alpha),
                        );
                    }
                }
            }
        }
    }
}

pub fn draw(painter: &Painter, rect: Rect, ctx: &MiniCtx) {
    let theme = ctx.theme;
    let cy = rect.center().y;

    // Jika ada banner HUD aktif atau mode musik sedang disembunyikan (idle / pause):
    // tampilkan pill minimal / HUD banner.
    if ctx.banner.is_some() || !ctx.show_media {
        draw_minimal_pill(painter, rect, ctx);
        return;
    }

    // Wash ambient progres digambar di belakang konten baris supaya ikon/teks
    // tetap tajam di atasnya.
    if ctx.settings.mini_progress {
        progress_backdrop(painter, rect, ctx);
    }

    let lyrics_on = ctx.settings.mini_lyrics && ctx.has_lyrics;
    let trio_ada = ctx.settings.mini_art
        || ctx.settings.mini_wave
        || (ctx.agent.active && ctx.settings.mini_agent);

    // 3a. Tanpa lirik (dimatikan atau lagu memang tidak punya): yang tampil
    // hanya grup trio [sampul][eq][mascot] yang dikumpulkan di tengah pill.
    // Judul, pembatas, dan item kanan (kontrol/jam/monitor) tidak digambar.
    // Kalau tidak ada satu pun item trio, tengah pill dibiarkan kosong.
    if !lyrics_on {
        if trio_ada {
            // Urutan tetap: sampul, eq, mascot — masing-masing dengan lebar slot
            // yang sama seperti di mode normal.
            let mut item_w: Vec<f32> = Vec::new();
            if ctx.settings.mini_art {
                item_w.push(38.0);
            }
            if ctx.settings.mini_wave {
                item_w.push(38.0);
            }
            if ctx.agent.active && ctx.settings.mini_agent {
                item_w.push(34.0);
            }
            item_w.truncate(NO_LYR_MAX_ITEMS);

            let total: f32 = item_w.iter().sum::<f32>()
                + (item_w.len().saturating_sub(1) as f32) * 6.0;
            let mut slot_x = rect.center().x - total / 2.0;

            // Indeks dunia nyata yang sudah dipakai; urutan gambar sama dengan
            // daftar `item_w` (sampul, lalu eq, lalu mascot).
            let mut used = 0usize;

            if ctx.settings.mini_art && used < item_w.len() {
                // Sampul bulat — kode sama dengan langkah 1.
                let pusat = egui::pos2(slot_x + 14.0, cy);
                let r = 14.0;
                if let Some(tex) = ctx.art {
                    let art_alpha = (255.0 * ctx.alpha.clamp(0.0, 1.0)).round() as u8;
                    painter.circle_filled(pusat, r, Color32::from_rgba_unmultiplied(28, 32, 42, art_alpha));
                    let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                    let mut mesh = egui::Mesh::with_texture(tex.id());
                    let n = 24;
                    let art_color = Color32::from_white_alpha(art_alpha);
                    for i in 0..=n {
                        let a = std::f32::consts::TAU * i as f32 / n as f32;
                        let (s, c) = a.sin_cos();
                        mesh.vertices.push(egui::epaint::Vertex {
                            pos: pusat + egui::vec2(c * r, s * r),
                            uv: egui::pos2(
                                uv.center().x + c * 0.5,
                                uv.center().y + s * 0.5,
                            ),
                            color: art_color,
                        });
                    }
                    for i in 1..n {
                        mesh.add_triangle(0, i as u32, i as u32 + 1);
                    }
                    painter.add(egui::Shape::mesh(mesh));
                } else {
                    painter.circle_filled(pusat, r, theme.surface);
                }
                slot_x += item_w[used] + 6.0;
                used += 1;
            }

            if ctx.settings.mini_wave && used < item_w.len() {
                // Equalizer — kode sama dengan langkah 6. Kalau audio sistem
                // terbaca, tinggi balok mengikuti energinya; kalau tidak,
                // kembali ke animasi sinus.
                let hidup = ctx.levels.iter().any(|v| *v > 0.02);
                let fase = [0.0, 1.1, 2.2, 3.3];
                let kecepatan = [3.2, 4.0, 2.8, 3.6];
                for j in 0..4 {
                    let tinggi = if hidup {
                        4.0 + 18.0 * ctx.levels[j].clamp(0.0, 1.0)
                    } else {
                        4.0 + 14.0 * (0.5 + 0.5 * (ctx.t * kecepatan[j] + fase[j]).sin())
                    };
                    let bar = Rect::from_min_size(
                        egui::pos2(slot_x + 6.0 + j as f32 * 6.0, cy - tinggi / 2.0),
                        egui::vec2(3.0, tinggi),
                    );
                    painter.rect_filled(bar, egui::Rounding::same(1.5), ctx.album_color);
                }
                slot_x += item_w[used] + 6.0;
                used += 1;
            }

            if ctx.agent.active && ctx.settings.mini_agent && used < item_w.len() {
                // Avatar Mini Mascot — kode sama dengan langkah 6b.
                let mascot_box = Rect::from_center_size(
                    egui::pos2(slot_x + 11.0, cy),
                    egui::vec2(22.0, 22.0),
                );
                crate::presentation::mascot::MascotRenderer::draw_mini(
                    painter,
                    mascot_box,
                    ctx.mascot,
                    &ctx.agent.status,
                    true,
                    ctx.t,
                );
            }
        }

        if ctx.settings.mini_progress {
            progress_aura(painter, rect, ctx);
        }
        return;
    }

    // Dengan lirik: kolom kiri [sampul][judul] rata kiri mulai dari tepi pill.
    let mut x = rect.left() + PAD;

    // 1. Sampul bulat.
    if ctx.settings.mini_art {
        let pusat = egui::pos2(x + 14.0, cy);
        let r = 14.0;
        if let Some(tex) = ctx.art {
            let art_alpha = (255.0 * ctx.alpha.clamp(0.0, 1.0)).round() as u8;
            painter.circle_filled(pusat, r, Color32::from_rgba_unmultiplied(28, 32, 42, art_alpha));
            let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            let mut mesh = egui::Mesh::with_texture(tex.id());
            let n = 24;
            let art_color = Color32::from_white_alpha(art_alpha);
            for i in 0..=n {
                let a = std::f32::consts::TAU * i as f32 / n as f32;
                let (s, c) = a.sin_cos();
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: pusat + egui::vec2(c * r, s * r),
                    uv: egui::pos2(
                        uv.center().x + c * 0.5,
                        uv.center().y + s * 0.5,
                    ),
                    color: art_color,
                });
            }
            for i in 1..n {
                mesh.add_triangle(0, i as u32, i as u32 + 1);
            }
            painter.add(egui::Shape::mesh(mesh));
        } else {
            painter.circle_filled(pusat, r, theme.surface);
        }

        x += 38.0;
    }

    // 2. Judul dan artis.
    if ctx.settings.mini_title {
        let judul_w = 110.0;
        clipped_text(
            painter,
            &ctx.media.title,
            FontId::proportional(12.5),
            theme.ink,
            x,
            cy,
            judul_w,
            ctx.t,
        );

        let pemisah = " — ";
        let g = painter.layout_no_wrap(pemisah.into(), FontId::proportional(11.0), theme.faint);
        let px = x + judul_w.min(
            painter
                .layout_no_wrap(ctx.media.title.clone(), FontId::proportional(12.5), theme.ink)
                .size()
                .x,
        );
        painter.galley(egui::pos2(px, cy - g.size().y / 2.0), g, theme.faint);

        clipped_text(
            painter,
            &ctx.media.artist,
            FontId::proportional(11.5),
            theme.accent,
            px + 14.0,
            cy,
            48.0,
            ctx.t,
        );
        x += 175.0;
    }

    // 3. Slot lirik dinamis: ruang bebas antara item kiri (sampul/judul)
    //    dan item kanan (kontrol, equalizer, mascot, jam, cpu, ram).
    //    Mencegah teks lirik menabrak maskot atau kontrol pada rasio apa pun.
    let mut right_items_w = 0.0;
    if ctx.settings.mini_controls { right_items_w += 64.0 + 6.0; }
    if ctx.settings.mini_wave { right_items_w += 38.0 + 6.0; }
    if ctx.agent.active && ctx.settings.mini_agent { right_items_w += 34.0 + 6.0; }
    if ctx.settings.mini_clock { right_items_w += 64.0 + 6.0; }
    if ctx.settings.mini_cpu { right_items_w += 54.0 + 6.0; }
    if ctx.settings.mini_ram { right_items_w += 54.0 + 6.0; }

    let slot_left = if x > rect.left() + PAD {
        x + 8.0
    } else {
        rect.left() + PAD
    };

    let right_boundary = if right_items_w > 0.0 {
        rect.right() - PAD - right_items_w + 6.0
    } else {
        rect.right() - PAD
    };
    let slot_right = (right_boundary - 8.0).max(slot_left + 20.0);

    let slot_w = (slot_right - slot_left).max(20.0);
    let slot_x = slot_left;
    let slot_clip = Rect::from_min_max(
        egui::pos2(slot_left, cy - LYR_HALF_H),
        egui::pos2(slot_right, cy + LYR_HALF_H),
    )
    .intersect(painter.clip_rect());

    // Gambar satu baris lirik: kalau muat di dalam slot, teks dikumpulkan di
    // tengah slot; kalau tidak, dipotong `...` dan awalan digambar rata kiri
    // dari tepi kiri slot.
    let draw_line = |painter: &Painter,
                     text: &str,
                     color: Color32,
                     slot_x: f32,
                     slot_w: f32,
                     cy: f32,
                     font: FontId| {
        let full = painter.layout_no_wrap(text.to_owned(), font.clone(), color);
        let (pos_x, galley) = if full.size().x <= slot_w {
            // Muat utuh: kumpulkan di tengah slot.
            (slot_x + (slot_w - full.size().x) / 2.0, full)
        } else if let Some(line) = ellipsize(
            painter,
            text,
            &font,
            color,
            slot_w - LYR_ELLIPSIS_PAD,
        ) {
            // Terlalu panjang: awalan + `...` mulai dari tepi kiri slot.
            let g = painter.layout_no_wrap(line, font, color);
            (slot_x, g)
        } else {
            return;
        };
        painter.with_clip_rect(slot_clip).galley(
            egui::pos2(pos_x, cy - galley.size().y / 2.0),
            galley,
            color,
        );
    };

    if ctx.lyric_trans < 1.0 && ctx.prev_lyric.is_some() {
        let ease = (1.0 - (ctx.lyric_trans * std::f32::consts::PI).cos()) * 0.5;

        // prev_lyric: bergeser naik cy - ease * 7.0, alpha (1.0 - ease) * ctx.alpha
        if let Some(prev) = ctx.prev_lyric {
            let alpha_prev = ((1.0 - ease) * (theme.ink.a() as f32)).round() as u8;
            if alpha_prev > 0 {
                let prev_color = Color32::from_rgba_unmultiplied(
                    theme.ink.r(),
                    theme.ink.g(),
                    theme.ink.b(),
                    alpha_prev,
                );
                draw_line(
                    painter,
                    prev,
                    prev_color,
                    slot_x,
                    slot_w,
                    cy - ease * 7.0,
                    FontId::proportional(LYR_FONT_SIZE),
                );
            }
        }

        // curr_lyric: bergeser masuk dari bawah cy + (1.0 - ease) * 7.0, alpha ease * ctx.alpha
        if let Some(curr) = ctx.lyric {
            let alpha_curr = (ease * (theme.ink.a() as f32)).round() as u8;
            if alpha_curr > 0 {
                let curr_color = Color32::from_rgba_unmultiplied(
                    theme.ink.r(),
                    theme.ink.g(),
                    theme.ink.b(),
                    alpha_curr,
                );
                draw_line(
                    painter,
                    curr,
                    curr_color,
                    slot_x,
                    slot_w,
                    cy + (1.0 - ease) * 7.0,
                    FontId::proportional(LYR_FONT_SIZE),
                );
            }
        }
    } else {
        // Statis di cy dengan alpha normal
        if let Some(baris) = ctx.lyric {
            draw_line(
                painter,
                baris,
                theme.ink,
                slot_x,
                slot_w,
                cy,
                FontId::proportional(LYR_FONT_SIZE),
            );
        }
    }

    // 4. Kelompok kanan rata kanan: kontrol, equalizer, mascot, jam, lalu
    //    monitor sistem — menempel ke tepi pill dengan jarak 6 px antar item.
    let mut kanan_x = rect.right() - PAD;
    let mut geser = |lebar: f32| -> f32 {
        kanan_x -= lebar;
        let cx = kanan_x;
        kanan_x -= 6.0;
        cx
    };

    // 5. Kontrol mini.
    if ctx.settings.mini_controls {
        let cx = geser(64.0);
        let play = Rect::from_center_size(egui::pos2(cx + 12.0, cy), egui::vec2(24.0, 24.0));
        painter.circle_filled(play.center(), 12.0, theme.surface);

        painter.text(
            play.center(),
            egui::Align2::CENTER_CENTER,
            if ctx.media.playing { "❚❚" } else { "▶" },
            FontId::proportional(9.0),
            theme.ink,
        );

        let next = Rect::from_center_size(egui::pos2(cx + 42.0, cy), egui::vec2(24.0, 24.0));
        painter.circle_filled(next.center(), 12.0, theme.surface);

        painter.text(
            next.center(),
            egui::Align2::CENTER_CENTER,
            "⏭",
            FontId::proportional(9.0),
            theme.dim,
        );
    }

    // 6. Equalizer.
    if ctx.settings.mini_wave {
        let cx = geser(38.0);
        // Kalau audio sistem terbaca, tinggi balok mengikuti energinya.
        // Kalau tidak (tidak ada perangkat, loopback gagal), kembali ke
        // animasi sinus supaya tetap hidup.
        let hidup = ctx.levels.iter().any(|v| *v > 0.02);
        let fase = [0.0, 1.1, 2.2, 3.3];
        let kecepatan = [3.2, 4.0, 2.8, 3.6];
        for i in 0..4 {
            let tinggi = if hidup {
                4.0 + 18.0 * ctx.levels[i].clamp(0.0, 1.0)
            } else {
                4.0 + 14.0 * (0.5 + 0.5 * (ctx.t * kecepatan[i] + fase[i]).sin())
            };
            let bar = Rect::from_min_size(
                egui::pos2(cx + 6.0 + i as f32 * 6.0, cy - tinggi / 2.0),
                egui::vec2(3.0, tinggi),
            );
            painter.rect_filled(bar, egui::Rounding::same(1.5), ctx.album_color);
        }
    }

    // 6b. Avatar Mini Mascot jika agent aktif bersamaan dengan musik
    if ctx.agent.active && ctx.settings.mini_agent {
        let cx = geser(34.0);
        let mascot_box = Rect::from_center_size(egui::pos2(cx + 11.0, cy), egui::vec2(22.0, 22.0));
        crate::presentation::mascot::MascotRenderer::draw_mini(
            painter,
            mascot_box,
            ctx.mascot,
            &ctx.agent.status,
            true,
            ctx.t,
        );
    }

    // 7. Jam.
    if ctx.settings.mini_clock {
        let cx = geser(64.0);
        chip(painter, &ctx.clock, cx, cy, theme);
    }

    // 8. Monitor sistem. Hanya ada di baris minimized — panel terbuka tidak
    // menampilkannya, dan tiap metrik bisa dimatikan dari Setelan.
    if ctx.settings.mini_cpu {
        let cx = geser(54.0);
        metrik(painter, Icon::Cpu, ctx.cpu, cx, cy, theme);
    }
    if ctx.settings.mini_ram {
        let cx = geser(54.0);
        metrik(painter, Icon::Ram, ctx.ram, cx, cy, theme);
    }

    if ctx.settings.mini_progress {
        progress_aura(painter, rect, ctx);
    }
}

/// Ikon kecil + persen, dipakai monitor CPU/RAM di baris minimized.
fn metrik(painter: &Painter, icon: Icon, persen: f32, x: f32, cy: f32, theme: &Theme) {
    let glyph = egui::Rect::from_center_size(egui::pos2(x + 6.0, cy), egui::vec2(11.0, 11.0));
    icons::draw(painter, icon, glyph, theme.dim, 1.4);

    painter.text(
        egui::pos2(x + 15.0, cy),
        egui::Align2::LEFT_CENTER,
        format!("{:.0}%", persen.clamp(0.0, 100.0)),
        FontId::proportional(10.5),
        theme.dim,
    );
}

fn chip(painter: &Painter, teks: &str, x: f32, cy: f32, theme: &Theme) {
    let galley = painter.layout_no_wrap(
        teks.to_owned(),
        FontId::monospace(11.0),
        theme.ink,
    );
    let w = galley.size().x + 16.0;
    let rect = Rect::from_center_size(egui::pos2(x + w / 2.0, cy), egui::vec2(w, 19.0));
    painter.rect_filled(rect, egui::Rounding::same(999.0), theme.surface);
    painter.galley(
        egui::pos2(rect.center().x - galley.size().x / 2.0, cy - galley.size().y / 2.0),
        galley,
        theme.ink,
    );
}

/// Bagian progres yang digambar **di belakang** konten baris: track kosong
/// plus isian gradasi penuh tinggi pill pada bagian yang sudah diputar.
/// Dipanggil sebelum ikon/teks supaya konten tetap tajam di atasnya.
pub fn progress_backdrop(painter: &Painter, rect: Rect, ctx: &MiniCtx) {
    let alpha = ctx.alpha.clamp(0.0, 1.0);
    if alpha <= 0.01 {
        return;
    }
    let (jalur, _) = match progress_geometry(rect) {
        Some(g) => g,
        None => return,
    };

    // Track kosong: cahaya putih tipis untuk bagian yang belum diputar.
    painter.rect_filled(
        jalur,
        egui::Rounding::same(2.0),
        with_alpha(theme::mix(Color32::WHITE, ctx.album_color, 0.16), 20.0 * alpha),
    );

    let frac = ctx.media.progress().clamp(0.0, 1.0);
    if frac <= 0.001 {
        return;
    }
    let head_x = jalur.left() + jalur.width() * frac;
    // Isian gradasi vertikal penuh tinggi pill — inilah "aura menyala" yang
    // terlihat seperti badan pill terisi cahaya, bukan garis tipis di tepi.
    draw_fill_gradient(painter, rect, jalur, head_x, ctx.album_color, alpha);
}

/// Isian gradasi vertikal pada bagian pill yang sudah diputar, digambar di
/// belakang konten:
/// - Sumbu-X: konstan sepanjang bagian terisi (rata), hanya fade-in 4% di
///   kiri dan bleed lembut ke nol di kepala.
/// - Sumbu-Y: dari `rect.top()` samar hingga dasar pill pekat, bentuk
///   kuadratik yang naik perlahan (efek kaca terisi cahaya dari bawah).
fn draw_fill_gradient(
    painter: &Painter,
    rect: Rect,
    jalur: Rect,
    head_x: f32,
    color: Color32,
    alpha: f32,
) {
    const COLS: usize = 40;
    const ROWS: usize = 12;
    /// Seberapa jauh isian tumpah melewati kepala, dalam fraksi lebar jalur.
    const BLEED: f32 = 0.08;

    let left = jalur.left();
    let span_head = head_x - left;
    if span_head <= 0.5 {
        return;
    }
    let bleed_px = (jalur.width() * BLEED).clamp(8.0, 30.0);
    let span = span_head + bleed_px;
    let bleed_frac = bleed_px / span;

    // Isian mulai dari puncak pill sampai dasar jalur — badan penuh.
    let top = rect.top();
    let bottom = jalur.bottom();
    let height = (bottom - top).max(1.0);

    // Gradasi hampir linear, meniru referensi (`docs/pill-playing.png`):
    // atas samar, makin ke bawah makin terang, tanpa puncak panas di tepi
    // bawah. Alpha diset cukup supaya "menyala" terlihat jelas di atas kaca
    // gelap — sebelumnya (72) nyaris tak terbaca.
    const PEAK_BOTTOM: f32 = 118.0;
    const PEAK_TOP: f32 = 14.0;

    let mut mesh = egui::Mesh::default();
    for r in 0..=ROWS {
        let rv = r as f32 / ROWS as f32;
        let y = top + height * rv;
        // Ramp mendekati linear (pangkat 1.3) — tidak menumpuk terang di bawah
        // seperti kuadratik, tidak melandai terlalu lama seperti murni linear.
        let vgrad = rv.powf(1.3);
        let v_alpha = PEAK_TOP + (PEAK_BOTTOM - PEAK_TOP) * vgrad;
        for c in 0..=COLS {
            let cf = c as f32 / COLS as f32;
            let x = left + span * cf;
            let a = v_alpha * wash_horizontal_falloff(cf, bleed_frac) * alpha;
            mesh.colored_vertex(egui::pos2(x, y), with_alpha(color, a));
        }
    }

    let stride = (COLS + 1) as u32;
    for r in 0..ROWS {
        for c in 0..COLS {
            let i0 = r as u32 * stride + c as u32;
            let i1 = i0 + 1;
            let i2 = i0 + stride;
            let i3 = i2 + 1;
            mesh.add_triangle(i0, i1, i2);
            mesh.add_triangle(i1, i3, i2);
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// Bagian progres yang digambar **di depan** konten baris (setelah ikon/teks):
/// hanya penanda posisi di kepala — halo kecil berdenyut. Isian gradasi sudah
/// ditangani `progress_backdrop`.
fn progress_aura(painter: &Painter, rect: Rect, ctx: &MiniCtx) {
    let alpha = ctx.alpha.clamp(0.0, 1.0);
    if alpha <= 0.01 {
        return;
    }
    let (jalur, _) = match progress_geometry(rect) {
        Some(g) => g,
        None => return,
    };

    let frac = ctx.media.progress().clamp(0.0, 1.0);
    if frac <= 0.001 {
        return;
    }

    let head_x = jalur.left() + jalur.width() * frac;
    let pulse = if ctx.reduce_motion {
        1.0
    } else {
        1.0 + 0.10 * (ctx.t * 1.9).sin()
    };

    let hot = theme::lighten(ctx.album_color, 0.18);
    aura_bloom(painter, jalur, head_x, hot, alpha, pulse);
}

/// Kecerahan wash ambient pada posisi relatif `f` sepanjang bentang yang
/// diliputi wash (0 = awal lagu, 1 = ujung bleed melewati kepala progres).
///
/// `bleed_frac` = porsi bentang yang melewati kepala; kepala berada di
/// `1 - bleed_frac`. Tiga tujuan:
/// - Naik halus dari nol di awal lagu — tanpa ini wash berakhir sebagai **tepi
///   kiri keras**, yang pada pill hitam terbaca seperti balok cat.
/// - Tetap menyala sepanjang bagian yang sudah diputar, tidak cuma di ujung.
/// - Meluruh halus ke nol di ujung bleed — tidak ada tepi vertikal keras di
///   kepala; cahaya tumpah lembut ke area yang belum diputar.
pub(crate) fn wash_horizontal_falloff(f: f32, bleed_frac: f32) -> f32 {
    let f = f.clamp(0.0, 1.0);
    let head = 1.0 - bleed_frac.clamp(0.05, 0.9);
    // Pita rata: konstan 1.0 sepanjang bagian yang sudah diputar. Hanya ada
    // fade-in sangat pendek di sisi kiri (4% bentang) supaya progres tidak
    // "muncul" sebagai titik; bagian kiri tidak meredup ke arah awal lagi.
    const FADE_IN: f32 = 0.04;
    let masuk = (f / FADE_IN).clamp(0.0, 1.0);
    let masuk_ease = masuk * masuk * (3.0 - 2.0 * masuk); // smoothstep
    if f < head {
        masuk_ease
    } else {
        let turun = (1.0 - ((f - head) / (1.0 - head).max(1e-4)).clamp(0.0, 1.0)).max(0.0);
        turun
    }
}

fn agent_aura(painter: &Painter, rect: Rect, ctx: &MiniCtx) {
    let alpha = ctx.alpha.clamp(0.0, 1.0);
    if alpha <= 0.01 {
        return;
    }
    let (jalur, _) = match progress_geometry(rect) {
        Some(g) => g,
        None => return,
    };

    match &ctx.agent.status {
        noktah_core::model::AgentStatus::PromptSubmitted { .. } => {
            let pulse = if ctx.reduce_motion {
                1.0
            } else {
                1.0 + 0.16 * (ctx.t * 2.4).sin()
            };
            let color = theme::mix(
                Color32::from_rgb(139, 92, 246),
                Color32::from_rgb(56, 189, 248),
                (ctx.t * 0.8).sin().abs(),
            );
            painter.rect_filled(
                jalur,
                egui::Rounding::same(2.0),
                with_alpha(color, 160.0 * pulse * alpha),
            );
        }
        noktah_core::model::AgentStatus::ToolRunning { .. } => {
            let head_frac = if ctx.reduce_motion {
                0.5
            } else {
                (ctx.t * 0.6).fract()
            };
            let head_x = jalur.left() + jalur.width() * head_frac;
            let emerald = Color32::from_rgb(16, 185, 129);
            let cyan = Color32::from_rgb(6, 182, 212);
            comet_trail(painter, jalur, head_x, cyan, emerald, alpha);
            aura_bloom(painter, jalur, head_x, emerald, alpha, 1.15);
        }
        noktah_core::model::AgentStatus::AwaitingApproval(_) => {
            let pulse = if ctx.reduce_motion {
                1.0
            } else {
                1.0 + 0.28 * (ctx.t * 5.2).sin().abs()
            };
            let amber = Color32::from_rgb(245, 158, 11);
            let blue = Color32::from_rgb(59, 130, 246);
            let blend = theme::mix(amber, blue, 0.25);
            painter.rect_filled(
                jalur,
                egui::Rounding::same(2.0),
                with_alpha(blend, 210.0 * pulse * alpha),
            );
        }
        noktah_core::model::AgentStatus::Error { .. } => {
            let red = Color32::from_rgb(239, 68, 68);
            painter.rect_filled(
                jalur,
                egui::Rounding::same(2.0),
                with_alpha(red, 180.0 * alpha),
            );
        }
        _ => {}
    }
}

// ============================================================
// Geometri dan warna: fungsi murni yang bisa ditest
// ============================================================

/// Inset horizontal untuk jalur progress di dalam siluet pill.
///
/// Radius bawah pill = 19 px; sisi datar siluet dimulai dari `left+19`.
/// Inset 20 px mengaman-kan ujung kapsul agar tidak menonjol keluar kaca
/// (inset 18 akan membuatnya terpotong OS region).
const JALUR_INSET_HORIZ: f32 = 20.0;

/// Tinggi jalur kosong (`--track`) dan posisi vertikalnya.
/// Jalur = 4 px tebal, menempel di bottom edge (y 34..38 pada PILL_H=38).
const JALUR_TOP_OFFSET: f32 = 4.0;
const JALUR_BOTTOM_OFFSET: f32 = 1.0;

/// Hitung geometri jalur progress berdasarkan rect pill.
pub(crate) fn progress_geometry(rect: Rect) -> Option<(Rect, f32)> {
    let left = rect.left() + JALUR_INSET_HORIZ;
    let right = rect.right() - JALUR_INSET_HORIZ;
    if right <= left + 2.0 {
        return None;
    }
    let top = rect.bottom() - JALUR_TOP_OFFSET;
    let bottom = rect.bottom() - JALUR_BOTTOM_OFFSET;
    let jalur = Rect::from_min_max(egui::pos2(left, top), egui::pos2(right, bottom));
    Some((jalur, jalur.width()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{pos2, vec2};

    #[test]
    fn clean_tool_name_membersihkan_prefix_mcp_dan_server() {
        assert_eq!(clean_tool_name("mcp__terminal__read_terminal"), "read_terminal");
        assert_eq!(clean_tool_name("mcp__filesystem__read_file"), "read_file");
        assert_eq!(clean_tool_name("mcp:openbot:tool_call"), "tool_call");
        assert_eq!(clean_tool_name("Bash"), "Bash");
        assert_eq!(clean_tool_name("FileEdit"), "FileEdit");
    }

    #[test]
    fn format_agent_status_label_menampilkan_status_dan_tool_rapi() {
        let clock = "10:30";
        let status_tool = noktah_core::model::AgentStatus::ToolRunning {
            tool: "mcp__terminal__read_terminal".to_string(),
            input: serde_json::Value::Null,
            start_time: 0,
        };
        assert_eq!(format_agent_status_label(&status_tool, clock), "Agent: read_terminal");

        let status_think = noktah_core::model::AgentStatus::PromptSubmitted {
            prompt: "buatkan rencana".to_string(),
            timestamp: 0,
        };
        assert_eq!(format_agent_status_label(&status_think, clock), "Agent Berpikir...");

        let status_appr = noktah_core::model::AgentStatus::AwaitingApproval(noktah_core::model::PermissionRequest {
            request_id: "1".to_string(),
            tool_name: "Bash".to_string(),
            tool_input: serde_json::Value::Null,
            suggestions: Vec::new(),
            session_title: None,
        });
        assert_eq!(format_agent_status_label(&status_appr, clock), "Izin [Alt+Y / Alt+N]");

        let status_idle = noktah_core::model::AgentStatus::Idle;
        assert_eq!(format_agent_status_label(&status_idle, clock), "10:30");
    }

    #[test]
    fn status_agent_minimal_pill_tidak_pernah_menabrak_maskot_maupun_equalizer() {
        let rect = Rect::from_min_size(pos2(100.0, 0.0), vec2(260.0, 38.0));
        let badge_d = 24.0;
        let badge_right = rect.left() + 20.0 + badge_d;
        let eq_left = rect.right() - 20.0 - (4.0 * 5.0 - 2.0);

        let label_left = badge_right + 8.0;
        let label_right = eq_left - 8.0;
        let max_w = label_right - label_left;

        assert!(label_left > badge_right, "label harus memiliki margin dari badge");
        assert!(label_right < eq_left, "label harus memiliki margin dari equalizer");
        assert!(max_w > 100.0, "tersedia ruang baca yang proporsional");
    }

    #[test]
    fn slot_lirik_dinamis_tidak_pernah_menabrak_komponen_kanan_maupun_kiri() {
        let rect = Rect::from_min_size(pos2(100.0, 10.0), vec2(400.0, 38.0));
        let pad = PAD;
        // Simulasi item kiri OFF, tetapi maskot dan kontrol ON di kanan
        let x_kiri = rect.left() + pad; // tidak ada art / judul
        let mut right_w = 0.0;
        let mini_agent = true; // maskot ON
        if mini_agent { right_w += 34.0 + 6.0; }
        let mini_controls = true; // kontrol ON
        if mini_controls { right_w += 64.0 + 6.0; }

        let slot_left = x_kiri;
        let right_boundary = rect.right() - pad - right_w + 6.0;
        let slot_right = (right_boundary - 8.0).max(slot_left + 20.0);
        let slot_w = (slot_right - slot_left).max(20.0);

        assert!(slot_left >= rect.left() + pad);
        assert!(slot_right <= right_boundary - 8.0);
        assert!(slot_left + slot_w <= right_boundary);
    }

    #[test]
    fn progress_kosong_ketika_pill_lebih_sempit_dari_dua_inset() {
        // Pill selebar <= 2*inset + 2 px tidak menyisakan ruang untuk jalur;
        // fungsi harus mengembalikan None supaya progress_aura tidak menggambar
        // kapsul selebar 0 px (yang muncul sebagai titik aneh di tepi).
        let tiny = Rect::from_min_size(pos2(0.0, 0.0), vec2(41.0, 38.0));
        assert!(progress_geometry(tiny).is_none());
    }

    #[test]
    fn progress_geometri_konstan_lebar_mencerminkan_inset() {
        // Lebar jalur = lebar pill - 2 * inset; dengan inset 20 px, gap kiri-kanan
        // selalu 40 px apa pun lebar pil.
        let w = 300.0;
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(w, 38.0));
        let (jalur, _w) = progress_geometry(rect).unwrap();
        let expected_w = w - 2.0 * JALUR_INSET_HORIZ;
        assert!((jalur.width() - expected_w).abs() < 0.01, "lebar jalur salah");
        assert_eq!(jalur.left(), 20.0);
        assert_eq!(jalur.right(), w - 20.0);
    }

    #[test]
    fn progress_geometri_tinggi_selalu_3_px() {
        // Band 3 px, diangkat 1 px dari tepi bawah (bottom-4 .. bottom-1) supaya
        // halo ke bawah masih punya ruang di dalam kaca.
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 38.0));
        let (jalur, _) = progress_geometry(rect).unwrap();
        assert_eq!(jalur.height(), 3.0);
        assert_eq!(jalur.top(), 38.0 - 4.0);
        assert_eq!(jalur.bottom(), 38.0 - 1.0);
    }

    #[test]
    fn progress_head_x_posisi_proporsional_fleksibel() {
        let rect = Rect::from_min_size(pos2(50.0, 0.0), vec2(400.0, 38.0));
        let (jalur, _w) = progress_geometry(rect).unwrap();
        // Test: head_x dihitung dari jalur.left + jalur.width * frac
        let frac = 0.75;
        let head_x = jalur.left() + jalur.width() * frac;
        // left = 50 + 20 = 70; width = 400 - 2*20 = 360
        assert!((head_x - (70.0 + 360.0 * 0.75)).abs() < 0.01);
    }

    #[test]
    fn wash_horizontal_rata_sepanjang_progres_dengan_fade_pendek_di_ujung() {
        // Inti "progres rata": bagian yang sudah diputar menyala konstan (bukan
        // komet yang naik ke kepala). Hanya fade-in sangat pendek di kiri dan
        // bleed ke nol di kepala.
        for bleed in [0.05_f32, 0.12, 0.3] {
            let head = 1.0 - bleed;
            assert_eq!(wash_horizontal_falloff(0.0, bleed), 0.0, "awal = nol (fade-in)");
            // Setelah fade-in singkat (5%), pita harus konstan = 1.
            let mulai = wash_horizontal_falloff((head * 0.05).max(0.0), bleed);
            let tengah = wash_horizontal_falloff(head * 0.5, bleed);
            let kepala = wash_horizontal_falloff(head, bleed);
            assert!((tengah - 1.0).abs() < 0.001, "tengah harus konstan (bleed={bleed})");
            assert!((kepala - 1.0).abs() < 0.001, "kepala = penuh (bleed={bleed})");
            assert!(mulai > 0.9, "fade-in singkat: langsung hampir penuh");
        }
    }

    #[test]
    fn wash_horizontal_meluruh_ke_nol_melewati_kepala() {
        // Tanpa ini wash berakhir sebagai tepi vertikal keras di kepala dan
        // terbaca seperti balok warna, bukan cahaya.
        let bleed = 0.12_f32;
        let ujung = wash_horizontal_falloff(1.0, bleed);
        assert!(ujung < 0.001, "ujung bleed harus nol, dapat {ujung}");
        // Monoton turun sesudah kepala.
        let head = 1.0 - bleed;
        let a = wash_horizontal_falloff(head + 0.02, bleed);
        let b = wash_horizontal_falloff(head + 0.06, bleed);
        assert!(b < a, "harus turun monoton melewati kepala");
    }

    // ------------------------------------------------------------
    // Ellipsis lirik: potong aman UTF-8, tidak pernah `?` atau kosong
    // ------------------------------------------------------------

    /// Painter test dengan font egui bawaan (Ubuntu-Light) supaya lebar teks
    /// diukur dengan glif nyata — `__run_test_ctx` memakai `FontDefinitions::empty`
    /// yang membuat semua glyph selebar nol dan tidak bisa menguji pemotongan.
    fn test_painter(ctx: &egui::Context) -> egui::Painter {
        ctx.layer_painter(egui::LayerId::new(
            egui::Order::Middle,
            egui::Id::new("mini-test"),
        ))
    }

    fn run_test_ctx(f: impl FnOnce(&egui::Context)) {
        let ctx = egui::Context::default();
        // Font bawaan (default_fonts aktif) — bukan FontDefinitions::empty().
        // `Context::run` memakai `FnMut`, jadi `f` yang `FnOnce` dibungkus
        // `Option::take` supaya tetap bisa dipanggil tepat satu kali.
        let mut f = Some(f);
        let _ = ctx.run(Default::default(), |ctx| {
            if let Some(f) = f.take() {
                f(ctx);
            }
        });
    }

    #[test]
    fn ellipsize_teks_pendek_dikembalikan_apa_adanya() {
        run_test_ctx(|ctx| {
            let painter = test_painter(ctx);
            let font = FontId::proportional(LYR_FONT_SIZE);
            let out = ellipsize(&painter, "sepotong", &font, Color32::WHITE, 240.0);
            assert_eq!(out.as_deref(), Some("sepotong"), "teks muat harus utuh");
        });
    }

    #[test]
    fn ellipsize_teks_panjang_berakhir_titik_tiga_dan_lebar_terjaga() {
        run_test_ctx(|ctx| {
            let painter = test_painter(ctx);
            let font = FontId::proportional(LYR_FONT_SIZE);
            let max_w = 80.0;
            let out = ellipsize(&painter, "baris yang sangat panjang sekali", &font, Color32::WHITE, max_w)
                .expect("hasil tidak boleh kosong");
            assert!(out.ends_with("..."), "baris terlalu panjang harus berakhir ...");
            assert!(
                out.starts_with("baris"),
                "awalan asli harus dipertahankan, dapat {out}"
            );
            let w = painter.layout_no_wrap(out.clone(), font, Color32::WHITE).size().x;
            assert!(w <= max_w, "lebar {w} melebihi max_w {max_w}");
            assert!(!out.contains('?'), "tidak boleh ada glyph pengganti '?'");
        });
    }

    #[test]
    fn ellipsize_satu_karakter_tetap_ada_dan_bukan_pertanyaan() {
        run_test_ctx(|ctx| {
            let painter = test_painter(ctx);
            let font = FontId::proportional(LYR_FONT_SIZE);
            // max_w sangat kecil: bahkan satu karakter + `...` tidak muat.
            let max_w = 1.0;
            let out = ellipsize(&painter, "lirik panjang", &font, Color32::WHITE, max_w)
                .expect("hasil tidak boleh kosong");
            assert!(!out.is_empty(), "satu karakter pun harus tetap tampil");
            assert!(!out.contains('?'), "tidak boleh jadi '?'");
            assert!(out.starts_with('l'), "karakter pertama tetap ada: {out}");
            assert!(out.ends_with("..."));
        });
    }

    #[test]
    fn ellipsize_hangul_tidak_memecah_karakter_utf8() {
        run_test_ctx(|ctx| {
            let painter = test_painter(ctx);
            let font = FontId::proportional(LYR_FONT_SIZE);
            let max_w = 100.0;
            let asli = "낭만적인 fairy-tale 말도 안 되는 걸 난";
            let out = ellipsize(&painter, asli, &font, Color32::WHITE, max_w)
                .expect("hasil tidak boleh kosong");
            assert!(out.ends_with("..."), "harus terpotong: {out}");
            // `chars()` suku demi suku: bila ada byte pecah, to_string akan
            // menghasilkan karakter pengganti U+FFFD.
            assert!(
                !out.contains('\u{FFFD}'),
                "UTF-8 tidak boleh terbelah di tengah karakter"
            );
            assert!(
                asli.starts_with(&out[..out.len() - 3]),
                "hasil adalah awalan dari teks asli"
            );
            assert!(!out.contains('?'));
        });
    }
}

/// Ekor: gradasi dari dingin di awal lagu ke panas di kepala.
///
/// Panjang ekor tetap 84 px, jadi pada lagu panjang ia benar-benar bergerak
/// seperti komet, sedangkan di awal lagu seluruh bagian terlihat memanas
/// menjelang kepala.
fn comet_trail(
    painter: &Painter,
    jalur: Rect,
    head_x: f32,
    cold: Color32,
    hot: Color32,
    alpha: f32,
) {
    const TAIL_LEN: f32 = 84.0;
    const STEPS: usize = 14;

    let span = head_x - jalur.left();
    if span <= 0.5 {
        return;
    }

    let mut mesh = egui::Mesh::default();
    for i in 0..=STEPS {
        let f = i as f32 / STEPS as f32;
        let x = jalur.left() + span * f;
        let heat = (1.0 - (head_x - x) / TAIL_LEN).clamp(0.0, 1.0);
        let c = with_alpha(theme::mix(cold, hot, heat), (120.0 + 115.0 * heat) * alpha);

        mesh.colored_vertex(egui::pos2(x, jalur.top()), c);
        mesh.colored_vertex(egui::pos2(x, jalur.bottom()), c);

        if i > 0 {
            let cur = (mesh.vertices.len() - 2) as u32;
            let prev = cur - 2;
            mesh.add_triangle(prev, prev + 1, cur);
            mesh.add_triangle(prev + 1, cur + 1, cur);
        }
    }
    if mesh.is_valid() {
        painter.add(egui::Shape::mesh(mesh));
    }
}

/// Halo elips di ujung progres — titik terang yang membuat garis terbaca
/// sebagai cahaya yang bergerak, bukan panjang yang bertambah.
fn aura_bloom(
    painter: &Painter,
    jalur: Rect,
    head_x: f32,
    hot: Color32,
    alpha: f32,
    pulse: f32,
) {
    const WEDGES: usize = 18;
    let centre = egui::pos2(head_x, jalur.center().y);
    // Halo penanda posisi — kecil dan redup supaya tidak "memakan" area.
    let rx = 6.0 * pulse;
    // Digepengkan vertikal supaya halo menghormati bentuk pill yang tipis.
    let ry = 3.5 * pulse;

    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(centre, with_alpha(hot, 46.0 * alpha));
    for i in 0..WEDGES {
        let a = std::f32::consts::TAU * i as f32 / WEDGES as f32;
        let (s, c) = a.sin_cos();
        mesh.colored_vertex(
            egui::pos2(centre.x + c * rx, centre.y + s * ry),
            with_alpha(hot, 0.0),
        );
    }
    for i in 0..WEDGES {
        let rim = 1 + i as u32;
        let next = 1 + ((i + 1) % WEDGES) as u32;
        mesh.add_triangle(0, rim, next);
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// Alpha warna tanpa mengubah rona.
fn with_alpha(c: Color32, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a.clamp(0.0, 255.0) as u8)
}
