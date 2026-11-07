//! Permukaan kaca: gradien, pantulan tepi, cahaya bawah, dan telinga cair.
//!
//! egui tidak punya gradien di dalam bentuk membulat, jadi gradien digambar
//! sebagai pita tipis yang lebarnya menyusut mengikuti lengkung sudut. Dengan
//! 64 pita hasilnya tidak terlihat berjenjang.

use crate::presentation::theme::Theme;
use eframe::egui::{self, Color32, Painter, Pos2, Rect, Rounding};

/// Jarak tepi kiri/kanan yang harus dipotong pada ketinggian `y` agar pita
/// tetap berada di dalam lengkung sudut.
fn inset_at(y: f32, rect: Rect, r_top: f32, r_bot: f32) -> f32 {
    inset_at_n(y, rect, r_top, r_bot, 2.0)
}

/// Sama seperti `inset_at`, tetapi dengan eksponen: 2.0 = lingkaran biasa,
/// 4.5 = squircle (superellipse) yang sudutnya terasa lebih "padat".
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
    Color32::from_rgba_unmultiplied(
        f(a.r(), b.r()),
        f(a.g(), b.g()),
        f(a.b(), b.b()),
        f(a.a(), b.a()),
    )
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

/// Gradien vertikal di dalam bentuk membulat.
pub fn gradient(
    painter: &Painter,
    rect: Rect,
    r_top: f32,
    r_bot: f32,
    stops: &[(f32, Color32)],
) {
    gradient_n(painter, rect, r_top, r_bot, stops, 2.0)
}

pub fn gradient_n(
    painter: &Painter,
    rect: Rect,
    r_top: f32,
    r_bot: f32,
    stops: &[(f32, Color32)],
    n: f32,
) {
    const PITA: usize = 40;
    let tinggi = rect.height();
    if tinggi <= 0.0 {
        return;
    }

    for i in 0..PITA {
        let t0 = i as f32 / PITA as f32;
        let t1 = (i + 1) as f32 / PITA as f32;
        let y0 = rect.top() + tinggi * t0;
        let y1 = rect.top() + tinggi * t1;
        let inset = inset_at_n((y0 + y1) * 0.5, rect, r_top, r_bot, n);

        let pita = Rect::from_min_max(
            egui::pos2(rect.left() + inset, y0),
            egui::pos2(rect.right() - inset, y1 + 0.5),
        );
        painter.rect_filled(pita, 0.0, sample(stops, (t0 + t1) * 0.5));
    }
}

/// Gradien horizontal tipis — dipakai untuk garis cahaya di tepi bawah.
fn gradient_horizontal(painter: &Painter, rect: Rect, stops: &[(f32, Color32)]) {
    const PITA: usize = 28;
    let lebar = rect.width();
    if lebar <= 0.0 {
        return;
    }
    for i in 0..PITA {
        let t0 = i as f32 / PITA as f32;
        let t1 = (i + 1) as f32 / PITA as f32;
        let pita = Rect::from_min_max(
            egui::pos2(rect.left() + lebar * t0, rect.top()),
            egui::pos2(rect.left() + lebar * t1 + 0.5, rect.bottom()),
        );
        painter.rect_filled(pita, 0.0, sample(stops, (t0 + t1) * 0.5));
    }
}

/// Seluruh permukaan kaca.
///
/// `tembus` = true kalau ada blur di belakang jendela, sehingga warnanya dibuat
/// lebih bening supaya yang terlihat justru isi layar, bukan cat.
pub fn surface(
    painter: &Painter,
    rect: Rect,
    radius: f32,
    theme: &Theme,
    tembus: bool,
) {
    surface_n(painter, rect, radius, theme, tembus, 2.0)
}

pub fn surface_n(
    painter: &Painter,
    rect: Rect,
    radius: f32,
    theme: &Theme,
    tembus: bool,
    n: f32,
) {
    let alpha = if tembus { 246 } else { 255 };
    let fill = Color32::from_rgba_unmultiplied(
        theme.glass_top.r(),
        theme.glass_top.g(),
        theme.glass_top.b(),
        alpha,
    );

    let r = radius.min(rect.height()).min(rect.width() * 0.5);
    if r <= 0.5 {
        painter.rect_filled(rect, 0.0, fill);
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

    // Gambar poligon konveks mulus dengan antialiasing subpixel bawaan egui, tanpa outline (Stroke::NONE)
    painter.add(egui::Shape::convex_polygon(pts, fill, egui::Stroke::NONE));
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
pub fn ears(painter: &Painter, rect: Rect, size: f32, theme: &Theme) {
    let warna = Color32::from_rgba_unmultiplied(
        theme.glass_top.r(),
        theme.glass_top.g(),
        theme.glass_top.b(),
        255,
    );

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

/// Ambient aura: pendaran cahaya dinamis yang memancar dari tepi bawah notch ke desktop (ala Nusa).
pub fn ambient_aura(painter: &Painter, rect: Rect, color: Color32, alpha_mult: f32) {
    let alpha_mult = alpha_mult.clamp(0.0, 1.0);
    if alpha_mult <= 0.01 {
        return;
    }

    let aura_h = 24.0;
    let n_steps = 20;
    let cx = rect.center().x;
    let half_w = (rect.width() * 0.5 - 12.0).max(20.0);
    let bot_y = rect.bottom();

    let mut mesh = egui::Mesh::default();

    // Buat grid vertex 3 tingkat: Y0 (bawah notch), Y1 (tengah aura), Y2 (ujung luar aura)
    for i in 0..=n_steps {
        let frac = i as f32 / n_steps as f32;
        let x = rect.left() + 12.0 + (rect.width() - 24.0) * frac;
        let dist_ratio = ((x - cx) / half_w).abs().min(1.0);
        // Falloff horizontal berbentuk parabola halus (terang di tengah, meredup di tepi)
        let falloff_x = (1.0 - dist_ratio * dist_ratio).max(0.0);

        let a0 = ((95.0 * alpha_mult * falloff_x) as u8).min(255);
        let a1 = ((40.0 * alpha_mult * falloff_x) as u8).min(255);
        let a2 = 0;

        let c0 = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a0);
        let c1 = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a1);
        let c2 = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a2);

        let y0 = bot_y;
        let y1 = bot_y + aura_h * 0.40;
        let y2 = bot_y + aura_h;

        let v_idx = mesh.vertices.len() as u32;
        mesh.colored_vertex(egui::pos2(x, y0), c0);
        mesh.colored_vertex(egui::pos2(x, y1), c1);
        mesh.colored_vertex(egui::pos2(x, y2), c2);

        if i > 0 {
            let prev = v_idx - 3;
            // Baris atas: quad antara y0 dan y1 (2 segitiga)
            mesh.add_triangle(prev, prev + 1, v_idx);
            mesh.add_triangle(prev + 1, v_idx + 1, v_idx);
            // Baris bawah: quad antara y1 dan y2 (2 segitiga)
            mesh.add_triangle(prev + 1, prev + 2, v_idx + 1);
            mesh.add_triangle(prev + 2, v_idx + 2, v_idx + 1);
        }
    }

    painter.add(egui::Shape::mesh(mesh));
}

/// Sudut membulat penuh — dipakai saat mode mengambang.
pub fn rounding_all(radius: f32) -> Rounding {
    Rounding::same(radius)
}

/// Menggambar tekstur hasil tangkapan layar di dalam bentuk notch.
///
/// Dipakai mode kaca Penuh: isi layar di belakang notch ditangkap, diblur,
/// lalu ditempel sebagai latar sebelum lapisan gradien.
pub fn textured(
    painter: &Painter,
    rect: Rect,
    radius: f32,
    tex: egui::TextureId,
    alpha: u8,
    squircle: bool,
) {
    const PITA: usize = 40;
    let n = if squircle { 4.5 } else { 2.0 };
    let mut mesh = egui::Mesh::with_texture(tex);
    let tinggi = rect.height();
    let lebar = rect.width().max(1.0);
    let warna = Color32::from_white_alpha(alpha);

    for i in 0..PITA {
        let t0 = i as f32 / PITA as f32;
        let t1 = (i + 1) as f32 / PITA as f32;
        let y0 = rect.top() + tinggi * t0;
        let y1 = rect.top() + tinggi * t1;
        let inset = inset_at_n((y0 + y1) * 0.5, rect, 0.0, radius, n);

        let x0 = rect.left() + inset;
        let x1 = rect.right() - inset;
        let u0 = (x0 - rect.left()) / lebar;
        let u1 = (x1 - rect.left()) / lebar;

        let dasar = mesh.vertices.len() as u32;
        mesh.vertices.push(egui::epaint::Vertex { pos: egui::pos2(x0, y0), uv: egui::pos2(u0, t0), color: warna });
        mesh.vertices.push(egui::epaint::Vertex { pos: egui::pos2(x1, y0), uv: egui::pos2(u1, t0), color: warna });
        mesh.vertices.push(egui::epaint::Vertex { pos: egui::pos2(x1, y1), uv: egui::pos2(u1, t1), color: warna });
        mesh.vertices.push(egui::epaint::Vertex { pos: egui::pos2(x0, y1), uv: egui::pos2(u0, t1), color: warna });
        mesh.add_triangle(dasar, dasar + 1, dasar + 2);
        mesh.add_triangle(dasar, dasar + 2, dasar + 3);
    }

    painter.add(egui::Shape::mesh(mesh));
}

