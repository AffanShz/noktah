//! Geometri notch. Satu tempat untuk semua angka, supaya UI dan hit-test
//! tidak pernah berbeda pendapat soal di mana bentuknya berada.

use egui::{pos2, vec2, Rect};

/// Lebar telinga cair di kiri dan kanan panel.
pub const EAR: f32 = 28.0;

// --- Geometri dari prototipe `noktah-redesign.html` -----------------------
//
// Prototipe memakai: pil 260×38 (radius 19), pil live 390×38, panel terbuka
// 660×252 (radius 38). Noktah tetap **notch** — menempel di tepi atas layar
// dengan telinga cair — bukan dynamic island menggantung, jadi angkanya
// dipakai sebagai skala, bukan disalin bulat-bulat.

/// Lebar pil saat tidak ada media yang diputar (`.island` `--cw:260px`).
///
/// Noktah tetap notch, jadi lebar ini adalah lebar notch saat tertutup —
/// menempel di tepi atas dengan telinga cair, bukan pulau menggantung.
#[allow(dead_code)]
pub const PILL_W: f32 = 260.0;
/// Lebar pil saat memutar lagu (`.island[data-live="1"]` `--cw:390px`).
#[allow(dead_code)]
pub const PILL_W_LIVE: f32 = 390.0;
/// Tinggi pil (`.island` `height:38px`).
pub const PILL_H: f32 = 38.0;
/// Radius pil (`.island` `border-radius:19px`). Hanya sudut bawah yang
/// dibulatkan — sudut atas menyatu ke tepi layar.
pub const RADIUS_PILL: f32 = 19.0;

/// Panel terbuka (`.island[data-open="true"]` `min(500px) × 176px`).
///
/// Dimensi kompak ala Apple: lebar 500 px dan tinggi 176 px saat menganggur
/// memberi proporsi notch yang ramping, tidak memakan separuh layar, dan tetap
/// memberikan ruang yang cukup bagi sampul musik, kontrol, serta lirik tersinkron.
pub const OPEN_W: f32 = 500.0;
pub const OPEN_H: f32 = 148.0;
/// Radius panel terbuka (28 px kontinu).
pub const RADIUS_OPEN: f32 = 28.0;

/// Nama lama `ISLAND_*` dipertahankan supaya pemanggil yang belum diperbarui
/// tetap mengacu ke bentuk terbuka yang sama. Noktah tidak punya "island"
/// terpisah — yang terbuka adalah panel notch yang sama.
pub const ISLAND_W: f32 = OPEN_W;
pub const ISLAND_H: f32 = OPEN_H;
pub const RADIUS_ISLAND: f32 = RADIUS_OPEN;

/// Lebar maksimum baris minimized (preset Lengkap bisa selebar ini).
pub const PILL_MAX: f32 = 720.0;

/// Jendela harus memuat pill terlebar, bukan hanya panel — kalau tidak,
/// preset Lengkap terpotong di tepi.
pub const WINDOW_W: f32 = PILL_MAX + EAR * 2.0;

/// Notch minimal: garis tipis seperti poni.
pub const MINIMAL_W: f32 = 168.0;
pub const MINIMAL_H: f32 = 6.0;
pub const RADIUS_MINIMAL: f32 = 3.0;

/* ---------------------------------------------------------------- dock */

/// Tinggi dock saat menganggur: hanya memuat strip penanda tiga garis (16 px).
pub const DOCK_STRIP_H: f32 = 16.0;
/// Tambahan tinggi ke bawah saat dock di-hover untuk menampilkan deretan menu tab (46 px).
pub const DOCK_MENU_H: f32 = 46.0;
/// Tinggi total dock saat dibuka penuh.
#[allow(dead_code)]
pub const DOCK_H: f32 = DOCK_STRIP_H + DOCK_MENU_H;
/// Tinggi shell terluar saat menu dock terbuka penuh (148 + 46 = 194 px).
pub const OUTER_OPEN_H: f32 = OPEN_H + DOCK_MENU_H;
/// Jendela harus memuat shell terluar, bukan hanya panel dasar.
pub const WINDOW_H: f32 = OUTER_OPEN_H + 70.0;
/// Lebar kelompok tab saat terbuka: satu tab 36 px ditambah padding 4 px
/// kiri-kanan. Dikalikan jumlah halaman di `NotchPage::SWITCHER` oleh
/// pemakainya — dulu angkanya ditulis 5×36+8 dan diam-diam jadi salah begitu
/// jumlah tab berubah.
pub const DOCK_TAB_W: f32 = 36.0;
/// Padding kiri-kanan kapsul dock di luar baris tab.
pub const DOCK_PAD: f32 = 8.0;
/// Lebar kapsul dock untuk `n` tab.
pub const fn dock_group_w(n: usize) -> f32 {
    DOCK_TAB_W * n as f32 + DOCK_PAD
}
/// Lebar strip saat menganggur — tiga garis, bukan selebar isinya.
#[allow(dead_code)]
pub const DOCK_STRIP_W: f32 = 38.0;
/// Berapa lama dock bertahan terbuka setelah kursor terakhir terlihat di
/// daerahnya. Menutup jendela kecil antara strip dan ikon yang bergerak.
pub const DOCK_LINGER: std::time::Duration = std::time::Duration::from_millis(250);

/// Batas bawah isi halaman: selalu tetap pada 132 px, baik dock menganggur
/// maupun membuka. Ini menjamin kontrol musik tidak pernah bergeser.
pub const CONTENT_BOTTOM: f32 = OPEN_H - DOCK_STRIP_H;

/// Rect notch untuk nilai morph `t` (0 = baris minimized, 1 = panel penuh/island).
///
/// `pill_w` datang dari setelan: lebarnya mengikuti item mana yang aktif,
/// bukan angka tetap.
///
/// `minimal_mix` (0..1) menyusutkan baris minimized ke garis tipis. Dulu ini
/// boolean dari setelan; sekarang pecahan supaya transisi antar-lagu bisa
/// mengecil lalu membesar kembali dengan mulus, bukan melompat.
pub fn notch_rect(
    t_w: f32,
    t_h: f32,
    pill_w: f32,
    minimal_mix: f32,
    screen: Rect,
    open_w: f32,
    open_h: f32,
) -> Rect {
    let t_w = t_w.clamp(-0.08, 1.15);
    let t_h = t_h.clamp(-0.08, 1.15);
    let m = minimal_mix.clamp(0.0, 1.0);

    let w_pill = pill_w + (MINIMAL_W - pill_w) * m;
    let h_pill = PILL_H + (MINIMAL_H - PILL_H) * m;

    let w = (w_pill + (open_w - w_pill) * t_w).max(40.0);
    let h = (h_pill + (open_h - h_pill) * t_h).max(MINIMAL_H);

    Rect::from_min_size(pos2(screen.center().x - w / 2.0, screen.top()), vec2(w, h))
}

/// Rect isi halaman di dalam panel terbuka.
///
/// Tinggi cadangan dock **tidak** bergantung pada pegas `dock_open`. Kalau
/// bergantung, isi halaman memanjang ke dalam pita dock saat dock menganggur:
/// kapsul menimpa baris transport, dan kursor yang mendekati tombol itu malah
/// membuka dock — yang menggeser tombolnya naik — yang menutup dock lagi.
/// Ayunan itu tidak mungkin terjadi kalau isi halaman selalu berakhir di
/// `CONTENT_BOTTOM` (132 px).
pub fn content_rect(area: Rect, gutter: f32, top_pad: f32) -> Rect {
    Rect::from_min_max(
        pos2(area.left() + gutter, area.top() + top_pad),
        pos2(area.right() - gutter, area.top() + CONTENT_BOTTOM),
    )
}

/// Pita tempat ikon dock strip digambar saat dock menganggur (132..148 px).
pub fn dock_band(area: Rect) -> Rect {
    Rect::from_min_max(
        pos2(area.left(), area.top() + CONTENT_BOTTOM),
        pos2(area.right(), area.bottom()),
    )
}

/// Shell terluar: panel dasar yang selalu utuh (148 px), ditambah ekstensi menu
/// dock yang tumbuh ke bawah hingga 194 px saat di-hover.
pub fn outer_shell_rect(base: Rect, dock_open: f32) -> Rect {
    let extra = DOCK_MENU_H * dock_open.clamp(0.0, 1.0);
    Rect::from_min_size(
        base.min,
        vec2(base.width(), base.height() + extra),
    )
}

/// Pita menu yang ditambahkan di bawah panel dasar selama dock membuka.
pub fn dock_menu_band(base: Rect, dock_open: f32) -> Rect {
    let top = base.bottom();
    let bottom = base.bottom() + DOCK_MENU_H * dock_open.clamp(0.0, 1.0);
    Rect::from_min_max(pos2(base.left(), top), pos2(base.right(), bottom))
}

/// Zona hover dock mencakup pita strip di dalam panel dasar dan seluruh
/// menu yang sudah muncul di bawahnya, tanpa memasuki isi halaman.
pub fn dock_hover_zone(base: Rect, dock_open: f32) -> Rect {
    let top = base.top() + CONTENT_BOTTOM;
    let outer = outer_shell_rect(base, dock_open);
    Rect::from_min_max(pos2(base.left(), top), outer.max)
}

/// Pita tipis di dasar panel yang membuka dock saat kursor menyentuhnya.
#[allow(dead_code)]
pub fn strip_band(area: Rect) -> Rect {
    Rect::from_min_max(
        pos2(area.left(), area.bottom() - DOCK_STRIP_H),
        pos2(area.right(), area.bottom()),
    )
}

pub fn radius(t: f32, minimal_mix: f32, target_r: f32) -> f32 {
    let t = t.clamp(0.0, 1.15);
    let m = minimal_mix.clamp(0.0, 1.0);
    let dari = RADIUS_PILL + (RADIUS_MINIMAL - RADIUS_PILL) * m;
    dari + (target_r - dari) * t
}

/// Bentuk yang boleh menerima klik. Saat tertutup hanya baris minimized,
/// saat terbuka seluruh panel/island.
pub fn hit_shape(expanded: bool, minimal_mix: f32, pill_w: f32, open_w: f32, open_h: f32) -> Rect {
    if expanded {
        let x = (WINDOW_W - open_w) / 2.0;
        Rect::from_min_size(pos2(x, 0.0), vec2(open_w, open_h))
    } else {
        let m = minimal_mix.clamp(0.0, 1.0);
        let w = pill_w + (MINIMAL_W - pill_w) * m;
        let h = PILL_H + (MINIMAL_H - PILL_H) * m;
        Rect::from_center_size(pos2(WINDOW_W / 2.0, h / 2.0), vec2(w, h))
    }
}
