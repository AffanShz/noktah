//! Permukaan kaca: gradien, pantulan tepi, cahaya bawah, dan telinga cair.
//!
//! egui tidak punya gradien di dalam bentuk membulat, jadi gradien digambar
//! sebagai pita tipis yang lebarnya menyusut mengikuti lengkung sudut. Dengan
//! 64 pita hasilnya tidak terlihat berjenjang.

use crate::presentation::theme::Theme;
use eframe::egui::{self, Color32, Painter, Pos2, Rect};

/// Jarak tepi kiri/kanan yang harus dipotong pada ketinggian `y` agar pita
/// tetap berada di dalam lengkung sudut, dengan eksponen `n`:
/// 2.0 = lingkaran biasa, 4.5 = squircle (superellipse).
fn inset_at_n(y: f32, rect: Rect, r_top: f32, r_bot: f32, n: f32) -> f32 {
    let d_top = (y - rect.top()).max(0.0);
    let d_bot = (rect.bottom() - y).max(0.0);

    let mut inset: f32 = 0.0;

    if r_top > 0.5 && d_top < r_top {
        let dy = (r_top - d_top) / r_top;
        inset = inset.max(r_top * (1.0 - (1.0 - dy.powf(n)).max(0.0).powf(1.0 / n)));
    }
    if r_bot > 0.5 && d_bot < r_bot {
        let dy = (r_bot - d_bot) / r_bot;
        inset = inset.max(r_bot * (1.0 - (1.0 - dy.powf(n)).max(0.0).powf(1.0 / n)));
    }

    inset
}

fn lerp(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_premultiplied(
        f(a.r(), b.r()),
        f(a.g(), b.g()),
        f(a.b(), b.b()),
        f(a.a(), b.a()),
    )
}

fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    let [r, g, b, _] = c.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied(r, g, b, alpha)
}

/// Warna pada posisi `t` (0–1) di antara beberapa titik henti.
fn sample(stops: &[(f32, Color32)], t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    for i in 0..stops.len() {
        if t <= stops[i].0 {
            if i == 0 {
                return stops[0].1;
            }
            let (t0, c0) = stops[i - 1];
            let (t1, c1) = stops[i];
            let local = if (t1 - t0).abs() < f32::EPSILON {
                0.0
            } else {
                (t - t0) / (t1 - t0)
            };
            return lerp(c0, c1, local);
        }
    }
    stops.last().map(|s| s.1).unwrap_or(Color32::TRANSPARENT)
}

/// Seluruh permukaan kaca.
///
/// `tembus` = true kalau ada blur di belakang jendela, sehingga warnanya dibuat
/// lebih bening supaya yang terlihat justru isi layar, bukan cat.
pub fn surface_n(
    painter: &Painter,
    rect: Rect,
    radius: f32,
    theme: &Theme,
    tembus: bool,
    n: f32,
) {
    let alpha = if tembus {
        (125.0 + (theme.panel.a() as f32 / 255.0) * 95.0).round().clamp(125.0, 220.0) as u8
    } else if theme.edge.a() > 0 {
        theme.panel.a().max(215)
    } else {
        255
    };
    let fill = with_alpha(theme.glass_top, alpha);

    let r = radius.min(rect.height()).min(rect.width() * 0.5);
    if r <= 0.5 {
        painter.rect_filled(rect, 0.0, fill);
        if theme.edge.a() > 0 {
            painter.line_segment(
                [rect.left_bottom(), rect.right_bottom()],
                egui::Stroke::new(1.0, theme.edge),
            );
        }
        return;
    }

    let n = n.max(1.0);
    let exp = 2.0 / n;
    const SAMPLES: usize = 28;

    let mut pts = Vec::with_capacity(SAMPLES * 2 + 6);
    // 1. Sudut kiri atas (menempel tepi layar)
    pts.push(egui::pos2(rect.left(), rect.top()));
    // 2. Sudut kanan atas (menempel tepi layar)
    pts.push(egui::pos2(rect.right(), rect.top()));

    // 3. Sisi kanan turun ke awal lengkung bawah-kanan
    let cx_r = rect.right() - r;
    let cy_r = rect.bottom() - r;
    pts.push(egui::pos2(rect.right(), cy_r));

    // 4. Lengkung squircle bawah-kanan (theta 0 -> pi/2)
    for i in 1..SAMPLES {
        let th = std::f32::consts::FRAC_PI_2 * (i as f32 / SAMPLES as f32);
        let c = th.cos();
        let s = th.sin();
        let x = cx_r + r * c.powf(exp);
        let y = cy_r + r * s.powf(exp);
        pts.push(egui::pos2(x, y));
    }
    pts.push(egui::pos2(cx_r, rect.bottom()));

    // 5. Sisi bawah menuju awal lengkung bawah-kiri
    let cx_l = rect.left() + r;
    let cy_l = rect.bottom() - r;
    pts.push(egui::pos2(cx_l, rect.bottom()));

    // 6. Lengkung squircle bawah-kiri (alpha pi/2 -> 0)
    for i in (1..SAMPLES).rev() {
        let al = std::f32::consts::FRAC_PI_2 * (i as f32 / SAMPLES as f32);
        let c = al.cos();
        let s = al.sin();
        let x = cx_l - r * c.powf(exp);
        let y = cy_l + r * s.powf(exp);
        pts.push(egui::pos2(x, y));
    }
    pts.push(egui::pos2(rect.left(), cy_l));

    if theme.glass_top == theme.glass_mid && theme.glass_mid == theme.glass_bot {
        // Gambar poligon konveks mulus dengan antialiasing subpixel bawaan egui, tanpa outline (Stroke::NONE)
        painter.add(egui::Shape::convex_polygon(pts.clone(), fill, egui::Stroke::NONE));
    } else {
        // Render single filled pass: smooth vertical mesh dengan PITA = 40 horizontal strips
        // Baris bersebelahan berbagi y yang tepat (tanpa overlap y1 + 0.5) agar zero double-alpha dan zero seams!
        const PITA: usize = 40;
        let tinggi = rect.height();
        if tinggi > 0.0 {
            let stops = [
                (0.0, with_alpha(theme.glass_top, alpha)),
                (0.5, with_alpha(theme.glass_mid, alpha)),
                (1.0, with_alpha(theme.glass_bot, alpha)),
            ];

            let mut mesh = egui::Mesh::default();
            for k in 0..=PITA {
                let t_k = k as f32 / PITA as f32;
                let y_k = rect.top() + tinggi * t_k;
                let inset_k = inset_at_n(y_k, rect, 0.0, r, n);
                let x0_k = rect.left() + inset_k;
                let x1_k = rect.right() - inset_k;
                let c_k = sample(&stops, t_k);

                mesh.colored_vertex(egui::pos2(x0_k, y_k), c_k);
                mesh.colored_vertex(egui::pos2(x1_k, y_k), c_k);
            }

            for k in 0..PITA {
                let row0 = (k * 2) as u32;
                let row1 = ((k + 1) * 2) as u32;
                mesh.add_triangle(row0, row0 + 1, row1 + 1);
                mesh.add_triangle(row0, row1 + 1, row1);
            }

            painter.add(egui::Shape::mesh(mesh));
        }
    }

    if theme.edge.a() > 0 && pts.len() > 1 {
        let mut rim_pts = pts[1..].to_vec();
        rim_pts.push(pts[0]);
        painter.add(egui::Shape::line(rim_pts, egui::Stroke::new(1.0, theme.edge)));
    }

    if theme.edge_bright.a() > 0 {
        let bot_y = rect.bottom() - 0.5;
        let hl_left = egui::pos2(rect.left() + r, bot_y);
        let hl_right = egui::pos2(rect.right() - r, bot_y);
        if hl_right.x > hl_left.x {
            painter.line_segment([hl_left, hl_right], egui::Stroke::new(1.0, theme.edge_bright));
        }
    }
}

/// Telinga cair: potongan sudut yang menyambung panel ke tepi layar.
///
/// Bentuknya diambil dari prototipe: kurva bezier dari (0,0) ke (28,28) dengan
/// titik kendali (15.464, 0) dan (28, 12.536), lalu ditutup ke sudut kanan atas.
pub fn ear(painter: &Painter, anchor: Pos2, size: f32, flip: bool, color: Color32, edge: Color32) {
    let skala = size / 28.0;
    let mut titik: Vec<Pos2> = Vec::with_capacity(24);

    // Bezier kubik, 22 sampel.
    const N: usize = 22;
    for i in 0..=N {
        let t = i as f32 / N as f32;
        let mt = 1.0 - t;
        let x = 3.0 * mt * mt * t * 15.464 + 3.0 * mt * t * t * 28.0 + t * t * t * 28.0;
        let y = 3.0 * mt * t * t * 12.536 + t * t * t * 28.0;
        let px = if flip { size - x * skala } else { x * skala };
        titik.push(egui::pos2(anchor.x + px, anchor.y + y * skala));
    }

    let sudut = if flip {
        egui::pos2(anchor.x, anchor.y)
    } else {
        egui::pos2(anchor.x + size, anchor.y)
    };

    // Kipas dari sudut atas: setiap pasangan titik kurva membentuk segitiga.
    let mut mesh = egui::Mesh::default();
    let warna = color;
    for i in 0..titik.len().saturating_sub(1) {
        let a = titik[i];
        let b = titik[i + 1];
        let ia = mesh.vertices.len() as u32;
        mesh.colored_vertex(a, warna);
        mesh.colored_vertex(b, warna);
        mesh.colored_vertex(sudut, warna);
        mesh.add_triangle(ia, ia + 1, ia + 2);
    }
    painter.add(egui::Shape::mesh(mesh));

    // Tanpa garis tepi kurva untuk menyatu mulus ke layar
    let _ = edge;
}

/// Telinga kiri dan kanan sekaligus, mengapit panel.
pub fn ears(painter: &Painter, rect: Rect, size: f32, theme: &Theme, tembus: bool) {
    let alpha = if tembus {
        (125.0 + (theme.panel.a() as f32 / 255.0) * 95.0).round().clamp(125.0, 220.0) as u8
    } else if theme.edge.a() > 0 {
        theme.panel.a().max(215)
    } else {
        255
    };
    let warna = with_alpha(theme.glass_top, alpha);

    ear(
        painter,
        egui::pos2(rect.left() - size, rect.top()),
        size,
        false,
        warna,
        theme.edge,
    );
    ear(
        painter,
        egui::pos2(rect.right(), rect.top()),
        size,
        true,
        warna,
        theme.edge,
    );
}

/// Menggambar tekstur hasil tangkapan layar di dalam bentuk notch.
///
/// Dipakai mode kaca Penuh: isi layar di belakang notch ditangkap, diblur,
/// lalu ditempel sebagai latar sebelum lapisan gradien.
pub fn textured(
    painter: &Painter,
    rect: Rect,
    screen_rect: Rect,
    radius: f32,
    tex: egui::TextureId,
    alpha: u8,
    squircle: bool,
) {
    const PITA: usize = 40;
    let n = if squircle { 4.5 } else { 2.0 };
    let mut mesh = egui::Mesh::with_texture(tex);
    let tinggi = rect.height();
    let warna = Color32::from_white_alpha(alpha);
    let sw = screen_rect.width().max(1.0);
    let sh = screen_rect.height().max(1.0);

    for i in 0..PITA {
        let t0 = i as f32 / PITA as f32;
        let t1 = (i + 1) as f32 / PITA as f32;
        let y0 = rect.top() + tinggi * t0;
        let y1 = rect.top() + tinggi * t1;
        let inset = inset_at_n((y0 + y1) * 0.5, rect, 0.0, radius, n);

        let x0 = rect.left() + inset;
        let x1 = rect.right() - inset;
        let u0 = ((x0 - screen_rect.left()) / sw).clamp(0.0, 1.0);
        let u1 = ((x1 - screen_rect.left()) / sw).clamp(0.0, 1.0);
        let v0 = ((y0 - screen_rect.top()) / sh).clamp(0.0, 1.0);
        let v1 = ((y1 - screen_rect.top()) / sh).clamp(0.0, 1.0);

        let dasar = mesh.vertices.len() as u32;
        mesh.vertices.push(egui::epaint::Vertex { pos: egui::pos2(x0, y0), uv: egui::pos2(u0, v0), color: warna });
        mesh.vertices.push(egui::epaint::Vertex { pos: egui::pos2(x1, y0), uv: egui::pos2(u1, v0), color: warna });
        mesh.vertices.push(egui::epaint::Vertex { pos: egui::pos2(x1, y1), uv: egui::pos2(u1, v1), color: warna });
        mesh.vertices.push(egui::epaint::Vertex { pos: egui::pos2(x0, y1), uv: egui::pos2(u0, v1), color: warna });
        mesh.add_triangle(dasar, dasar + 1, dasar + 2);
        mesh.add_triangle(dasar, dasar + 2, dasar + 3);
    }

    painter.add(egui::Shape::mesh(mesh));
}

#[cfg(test)]
mod tests {
    use super::*;
    use noktah_core::{settings::GlassEffect, Settings};

    #[test]
    fn gradient_stops_not_double_premultiplied() {
        let mut s = Settings::default();
        s.glass_effect = GlassEffect::Full;
        s.glass_tint_rgb = [159, 126, 200];
        s.opacity = 0.5;
        let theme = Theme::from_settings(&s);

        // With tembus = true and opacity = 0.5 -> panel.a = 127 -> alpha = 172.
        let expected_alpha = (125.0 + (theme.panel.a() as f32 / 255.0) * 95.0)
            .round()
            .clamp(125.0, 220.0) as u8;
        assert_eq!(expected_alpha, 172);

        let expected_top = Color32::from_rgba_unmultiplied(
            theme.glass_top.r(),
            theme.glass_top.g(),
            theme.glass_top.b(),
            expected_alpha,
        );
        let expected_mid = Color32::from_rgba_unmultiplied(
            theme.glass_mid.r(),
            theme.glass_mid.g(),
            theme.glass_mid.b(),
            expected_alpha,
        );
        let expected_bot = Color32::from_rgba_unmultiplied(
            theme.glass_bot.r(),
            theme.glass_bot.g(),
            theme.glass_bot.b(),
            expected_alpha,
        );
        // Verify single premultiplication (double premultiplication would darken RGB further)
        let double_premult = Color32::from_rgba_unmultiplied(
            expected_mid.r(),
            expected_mid.g(),
            expected_mid.b(),
            expected_alpha,
        );
        assert_ne!(expected_mid, double_premult);
        assert!(expected_mid.r() > double_premult.r());

        let ctx = egui::Context::default();
        let rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(500.0, 148.0));
        let output = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                surface_n(ui.painter(), rect, 28.0, &theme, true, 4.5);
            });
        });

        let mesh = output
            .shapes
            .iter()
            .find_map(|cs| match &cs.shape {
                egui::epaint::Shape::Mesh(m) if m.vertices.len() == 82 => Some(m),
                _ => None,
            })
            .expect("expected 41-row gradient mesh from surface_n");

        // Row 0 -> t = 0.0 (vertices 0, 1)
        assert_eq!(mesh.vertices[0].color.to_array(), expected_top.to_array());
        assert_eq!(mesh.vertices[1].color.to_array(), expected_top.to_array());
        // Row 20 -> t = 0.5 (vertices 40, 41)
        assert_eq!(mesh.vertices[40].color.to_array(), expected_mid.to_array());
        assert_eq!(mesh.vertices[41].color.to_array(), expected_mid.to_array());
        // Row 40 -> t = 1.0 (vertices 80, 81)
        assert_eq!(mesh.vertices[80].color.to_array(), expected_bot.to_array());
        assert_eq!(mesh.vertices[81].color.to_array(), expected_bot.to_array());
    }
}

