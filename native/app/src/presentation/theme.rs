//! Warna dan ukuran yang diturunkan dari setelan.

use egui::Color32;
use noktah_core::Accent;

/// Luminans relatif WCAG (0 = hitam, 1 = putih).
pub fn luminance(c: Color32) -> f32 {
    let f = |v: u8| {
        let s = v as f32 / 255.0;
        if s <= 0.039_28 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
}

/// Campur dua warna, `t` 0 = `a`, 1 = `b`. Alpha ikut dicampur.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(
        l(a.r(), b.r()),
        l(a.g(), b.g()),
        l(a.b(), b.b()),
        l(a.a(), b.a()),
    )
}

/// Skala alpha warna; `k` 1.0 = apa adanya, 0.0 = transparan.
pub fn alpha_scale(c: Color32, k: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), ((c.a() as f32) * k.clamp(0.0, 1.0)) as u8)
}

/// Deret `n` warna dari `a` ke `b` — dipakai menggambar busur bergradasi
/// sebagai potongan garis, karena `egui::Shape::line` hanya menerima satu warna.
pub fn gradient(a: Color32, b: Color32, n: usize) -> Vec<Color32> {
    if n <= 1 {
        return vec![a];
    }
    (0..n)
        .map(|i| mix(a, b, i as f32 / (n - 1) as f32))
        .collect()
}

/// Naikkan *lightness* di ruang HSL tanpa membuang saturasi.
///
/// Kenapa bukan `mix(c, WHITE, t)`: mencampur ke putih menurunkan saturasi
/// sekaligus, jadi aksen emas/mint berubah jadi putih pucat — dan gradasi yang
/// ujung terangnya putih justru terbaca "kotor", bukan "menyala", karena ia
/// mendekati warna jalur dasarnya sendiri. Menaikkan L di HSL mempertahankan
/// rona, jadi emas tetap emas dan merah tetap merah.
pub fn lighten(c: Color32, amount: f32) -> Color32 {
    let (h, s, l) = rgb_to_hsl(c);
    hsl_to_rgb(h, s, (l + amount).clamp(0.0, 1.0))
}

/// (hue 0..360, saturation 0..1, lightness 0..1)
pub fn rgb_to_hsl(c: Color32) -> (f32, f32, f32) {
    let r = c.r() as f32 / 255.0;
    let g = c.g() as f32 / 255.0;
    let b = c.b() as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f32::EPSILON {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if max == r {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };

    (h * 360.0, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color32 {
    if s <= 0.0 {
        let v = (l * 255.0).round() as u8;
        return Color32::from_rgb(v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let h = (h / 360.0).rem_euclid(1.0);

    let ke = |mut t: f32| {
        t = t.rem_euclid(1.0);
        let v = if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        };
        (v.clamp(0.0, 1.0) * 255.0).round() as u8
    };

    Color32::from_rgb(ke(h + 1.0 / 3.0), ke(h), ke(h - 1.0 / 3.0))
}

/// Warna teks yang tetap terbaca di atas latar `bg`.
///
/// Ambang 0.18 adalah titik di mana teks hitam dan putih sama-sama mencapai
/// rasio kontras ~4.5:1, jadi pilihannya tidak pernah jatuh di zona abu-abu.
pub fn on_color(bg: Color32) -> Color32 {
    if luminance(bg) > 0.18 {
        Color32::from_rgb(14, 13, 12)
    } else {
        Color32::from_rgb(252, 252, 253)
    }
}

/// Mengangkat aksen yang terlalu gelap agar tetap terbaca sebagai teks di atas
/// panel hitam, tanpa membuang ronanya.
///
/// Warna cover lagu bisa nyaris hitam (mis. sampul biru dongker). Kalau dipakai
/// apa adanya, judul lagu dan label jadi tak terlihat. Ambang 0.26 setara
/// kontras ~6:1 terhadap latar hitam.
fn legible_accent(c: Color32) -> Color32 {
    let l = luminance(c);
    if l < 0.26 {
        // Semakin gelap asalnya, semakin banyak putih yang dicampur.
        mix(c, Color32::WHITE, ((0.26 - l) / 0.26) * 0.78)
    } else {
        c
    }
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub accent: Color32,
    pub accent_soft: Color32,
    /// Warna teks di atas latar aksen penuh (tombol terisi, badge, centang).
    pub on_accent: Color32,
    pub ink: Color32,
    pub dim: Color32,
    pub faint: Color32,
    pub stroke: Color32,
    /// `--fill`: rgba(255,255,255,.08) — latar kartu dan kolom input.
    pub surface: Color32,
    /// `--fill2`: rgba(255,255,255,.14) — kontrol sekunder, jalur progres,
    /// latar baris setelan yang sedang dipilih.
    pub fill2: Color32,
    pub panel: Color32,

    /// Tiga titik henti gradien kaca.
    pub glass_top: Color32,
    pub glass_mid: Color32,
    pub glass_bot: Color32,
    /// Warna highlight aksen / pendar.
    pub rim: Color32,
    /// Garis tepi tipis (transparan untuk pure borderless).
    pub edge: Color32,
    /// Garis tepi bawah (transparan untuk pure borderless).
    pub edge_bright: Color32,
}

impl Theme {
    pub fn new(accent: Accent, opacity: f32) -> Self {
        let [r, g, b] = accent.rgb();
        let alpha = (opacity.clamp(0.5, 1.0) * 255.0) as u8;
        let aksen = legible_accent(Color32::from_rgb(r, g, b));

        // Pendar aksen terang untuk vokal, equalizer bar, dan highlight
        let rim = Color32::from_rgb(
            (aksen.r() as f32 * 0.85 + 50.0).min(255.0) as u8,
            (aksen.g() as f32 * 0.90 + 70.0).min(255.0) as u8,
            (aksen.b() as f32 * 0.95 + 90.0).min(255.0) as u8,
        );

        Self {
            accent: aksen,
            // `--accent-soft`: aksen 24% — dipakai latar tab aktif dan
            // tombol ikon yang menyala di prototipe.
            accent_soft: Color32::from_rgba_unmultiplied(aksen.r(), aksen.g(), aksen.b(), 61),
            on_accent: on_color(aksen),
            // `--label` .94, `--label2` .62, `--label3` .32 di atas hitam.
            ink: Color32::from_rgba_unmultiplied(255, 255, 255, 240),
            dim: Color32::from_rgba_unmultiplied(235, 235, 245, 158),
            faint: Color32::from_rgba_unmultiplied(235, 235, 245, 82),
            // `--sep` .10, `--fill` .08, `--fill2` .14.
            stroke: Color32::from_rgba_unmultiplied(255, 255, 255, 26),
            surface: Color32::from_rgba_unmultiplied(255, 255, 255, 20),
            fill2: Color32::from_rgba_unmultiplied(255, 255, 255, 36),
            panel: Color32::from_rgba_unmultiplied(0, 0, 0, alpha),

            // Pure black obsidian aesthetic ala Apple Dynamic Island
            glass_top: Color32::from_rgb(0, 0, 0),
            glass_mid: Color32::from_rgb(0, 0, 0),
            glass_bot: Color32::from_rgb(0, 0, 0),
            rim,
            edge: Color32::TRANSPARENT,
            edge_bright: Color32::TRANSPARENT,
        }
    }

    /// Menghasilkan tema dengan aksen yang ditimpa (misal aksen dinamis dari album art).
    ///
    /// Aksen diangkat dulu ke ambang terbaca: warna cover bisa nyaris hitam,
    /// dan tanpa ini judul lagu serta label di atasnya menghilang ke latar.
    pub fn with_accent(&self, c: Color32) -> Self {
        let c = legible_accent(c);
        let mut t = *self;
        t.accent = c;
        t.accent_soft = Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 65);
        t.on_accent = on_color(c);
        t.rim = Color32::from_rgb(
            (c.r() as f32 * 0.85 + 50.0).min(255.0) as u8,
            (c.g() as f32 * 0.90 + 70.0).min(255.0) as u8,
            (c.b() as f32 * 0.95 + 90.0).min(255.0) as u8,
        );
        t
    }

    /// Menghasilkan tema dengan faktor opasitas untuk transisi crossfade cair.
    pub fn with_opacity_multiplier(&self, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        let mod_a = |c: Color32| {
            Color32::from_rgba_unmultiplied(
                c.r(),
                c.g(),
                c.b(),
                ((c.a() as f32) * factor).round() as u8,
            )
        };
        Self {
            accent: mod_a(self.accent),
            accent_soft: mod_a(self.accent_soft),
            on_accent: mod_a(self.on_accent),
            ink: mod_a(self.ink),
            dim: mod_a(self.dim),
            faint: mod_a(self.faint),
            stroke: mod_a(self.stroke),
            surface: mod_a(self.surface),
            fill2: mod_a(self.fill2),
            panel: mod_a(self.panel),
            glass_top: self.glass_top,
            glass_mid: self.glass_mid,
            glass_bot: self.glass_bot,
            rim: mod_a(self.rim),
            edge: mod_a(self.edge),
            edge_bright: mod_a(self.edge_bright),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kontras(a: Color32, b: Color32) -> f32 {
        let (la, lb) = (luminance(a), luminance(b));
        let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
        (hi + 0.05) / (lo + 0.05)
    }

    #[test]
    fn teks_di_atas_aksen_selalu_terbaca() {
        // Sampul album bisa warna apa saja; teks di atas tombol terisi harus
        // tetap lolos ambang keterbacaan.
        for c in [
            Color32::from_rgb(0, 0, 0),
            Color32::from_rgb(18, 20, 60),
            Color32::from_rgb(120, 60, 20),
            Color32::from_rgb(230, 190, 90),
            Color32::from_rgb(255, 255, 255),
        ] {
            let aksen = legible_accent(c);
            assert!(
                kontras(on_color(aksen), aksen) >= 4.5,
                "kontras kurang untuk {c:?}"
            );
        }
    }

    #[test]
    fn aksen_gelap_diangkat_agar_terbaca_di_latar_hitam() {
        let gelap = Color32::from_rgb(12, 16, 48);
        let aksen = legible_accent(gelap);
        assert!(
            kontras(aksen, Color32::BLACK) >= 4.5,
            "aksen cover gelap harus diangkat"
        );
        // Rona tetap: biru harus masih dominan.
        assert!(aksen.b() > aksen.r() && aksen.b() > aksen.g());
    }

    #[test]
    fn aksen_terang_tidak_diubah() {
        let terang = Color32::from_rgb(240, 180, 60);
        assert_eq!(legible_accent(terang), terang);
    }
}
