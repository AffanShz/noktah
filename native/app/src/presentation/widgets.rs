//! Komponen kecil yang digambar sendiri, mengikuti bentuk di prototipe.
//!
//! Semua angka spasi, ukuran huruf, dan radius datang dari `crate::presentation::style`.
//! Jangan menulis angka mentah di sini — itulah yang dulu membuat panel
//! terlihat tidak rata.

use crate::presentation::icons::{self, Icon};
use crate::presentation::style::{
    self, R_CARD, R_CONTROL, R_PILL, S1, S2, S3, T_BODY, T_MICRO,
};
use crate::presentation::theme::Theme;
use eframe::egui::{self, Color32, Rect, Rounding, Stroke};

/// Saklar 36×20 dengan kenop yang meluncur.
///
/// Ukuran sengaja tetap: ini kontrol yang paling sering disentuh, dan
/// mengecilkannya mengikuti skala tipografi hanya membuatnya sulit diklik.
pub fn toggle(ui: &mut egui::Ui, value: &mut bool, theme: &Theme) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(36.0, 20.0), egui::Sense::click());

    if resp.clicked() {
        *value = !*value;
        return true;
    }

    let nyala = *value;
    let track = if nyala {
        Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 200)
    } else {
        Color32::from_white_alpha(36)
    };

    ui.painter().rect_filled(rect, Rounding::same(R_PILL), track);

    let r = 7.0;
    let cx = if nyala {
        rect.right() - 3.0 - r
    } else {
        rect.left() + 3.0 + r
    };
    ui.painter().circle_filled(
        egui::pos2(cx, rect.center().y),
        r,
        if nyala {
            Color32::WHITE
        } else {
            Color32::from_white_alpha(220)
        },
    );

    false
}

/// Judul bagian di dalam panel setelan.
///
/// Dulu: HURUF KAPITAL SEMUA ukuran 9,5 dengan garis pemisah di bawahnya.
/// Kapital semua memperlambat membaca, dan garis itu menambah satu elemen
/// visual untuk setiap bagian — padahal jarak sudah cukup memisahkan.
/// Sekarang: teks biasa yang lebih tebal, tanpa garis.
pub fn section(ui: &mut egui::Ui, judul: &str, _theme: &Theme) {
    ui.add_space(S2);
    ui.label(
        egui::RichText::new(judul)
            .size(11.5)
            .color(Color32::from_rgb(142, 142, 147)),
    );
    ui.add_space(S1);
}

/// Kontainer kartu setelan modern (dark rounded card).
pub fn card_group(ui: &mut egui::Ui, isi: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none()
        .fill(Color32::from_rgb(37, 37, 41))
        .rounding(Rounding::same(style::R_GROUP))
        .stroke(Stroke::new(1.0, Color32::from_white_alpha(10)))
        .inner_margin(egui::Margin::symmetric(16.0, 14.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            isi(ui);
        });
    ui.add_space(S2);
}

/// Garis pemisah halus antar baris di dalam kartu.
pub fn card_divider(ui: &mut egui::Ui) {
    ui.add_space(6.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.0, Color32::from_white_alpha(10)),
    );
    ui.add_space(6.0);
}

/// Baris label + kontrol di kanan.
pub fn row(ui: &mut egui::Ui, label: &str, theme: &Theme, isi: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.label(style::text(label, theme.dim));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            isi(ui);
        });
    });
    ui.add_space(S2);
}

/// Baris label + keterangan kecil di bawahnya, kontrol di kanan.
///
/// Judul memakai warna `ink` (bukan `dim`) supaya hierarki datang dari
/// kontras, bukan dari ukuran — aturan Apple: bangun hierarki dari bobot,
/// ukuran, dan warna sebagai satu set.
pub fn row_desc(
    ui: &mut egui::Ui,
    label: &str,
    ket: &str,
    theme: &Theme,
    isi: impl FnOnce(&mut egui::Ui),
) {
    let row_id = ui.next_auto_id().with(label);
    let avail_w = ui.available_width();
    let cached_ctrl_w = ui.ctx().data(|d| d.get_temp::<f32>(row_id));
    let reserve_w = cached_ctrl_w.unwrap_or_else(|| (avail_w * 0.45).clamp(36.0, 180.0));
    let left_max_w = (avail_w - reserve_w - S3).max(80.0);

    let mut ctrl_w = 0.0_f32;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_max_width(left_max_w);
            ui.label(style::text(label, theme.ink));
            if !ket.is_empty() {
                ui.label(style::cap(ket, theme.faint));
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            isi(ui);
            ctrl_w = ui.min_rect().width();
        });
    });

    if ctrl_w > 0.0 && cached_ctrl_w.map(|c| (c - ctrl_w).abs() > 0.5).unwrap_or(true) {
        ui.ctx().data_mut(|d| d.insert_temp(row_id, ctrl_w));
        ui.ctx().request_repaint();
    }
    ui.add_space(S3);
}

/// Bulatan warna untuk memilih aksen.
pub fn swatch(ui: &mut egui::Ui, warna: Color32, dipilih: bool, theme: &Theme) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
    let r = 8.0;
    ui.painter().circle_filled(rect.center(), r, warna);
    if dipilih {
        ui.painter()
            .circle_stroke(rect.center(), r + 3.0, egui::Stroke::new(1.5_f32, theme.ink));
    } else if resp.hovered() {
        ui.painter()
            .circle_stroke(rect.center(), r + 3.0, egui::Stroke::new(1.0_f32, theme.dim));
    }
    resp.clicked()
}

/// Tombol kecil bergaya kaca.
pub fn chip_button(ui: &mut egui::Ui, label: &str, aktif: bool, theme: &Theme) -> bool {
    let teks = egui::RichText::new(label)
        .size(T_BODY)
        .color(if aktif { theme.ink } else { theme.dim });

    let tombol = egui::Button::new(teks)
        .fill(if aktif {
            Color32::from_rgba_unmultiplied(
                theme.accent.r(),
                theme.accent.g(),
                theme.accent.b(),
                80,
            )
        } else {
            Color32::from_white_alpha(22)
        })
        .rounding(Rounding::same(R_CONTROL))
        .stroke(egui::Stroke::NONE);

    ui.add(tombol).clicked()
}

/// Tombol bulat bergaya kaca, dipakai untuk kontrol pemutar.
#[allow(dead_code)]
pub fn circle_button(
    ui: &mut egui::Ui,
    glyph: &str,
    diameter: f32,
    utama: bool,
    theme: &Theme,
) -> bool {
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(diameter, diameter), egui::Sense::click());
    let hover = resp.hovered();
    let tekan = resp.is_pointer_button_down_on();

    // Press scaling 0.90x kenyal ala Mochy / Apple
    let d = if tekan {
        diameter * 0.90
    } else if hover {
        diameter * 1.05
    } else {
        diameter
    };

    let fill = if utama {
        if hover {
            Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 220)
        } else {
            Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 160)
        }
    } else if hover {
        Color32::from_white_alpha(45)
    } else {
        Color32::from_white_alpha(24)
    };

    let center = rect.center();

    // Pendar aksen halus saat hover tombol
    if hover {
        ui.painter().circle_filled(center, diameter * 0.65, theme.accent_soft);
    }

    ui.painter().circle_filled(center, d / 2.0, fill);
    ui.painter().text(
        center,
        egui::Align2::CENTER_CENTER,
        glyph,
        egui::FontId::proportional(d * 0.38),
        theme.ink,
    );

    resp.clicked()
}

/// Tombol kontrol media bergaya Spotify dengan ikon vektor dan efek hover/tekan tactile.
#[allow(dead_code)]
pub fn spotify_media_button(
    ui: &mut egui::Ui,
    icon: Icon,
    diameter: f32,
    utama: bool,
    _theme: &Theme,
) -> bool {
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(diameter, diameter), egui::Sense::click());
    let hover = resp.hovered();
    let tekan = resp.is_pointer_button_down_on();

    let scale = if tekan {
        0.91
    } else if hover {
        1.06
    } else {
        1.0
    };
    let d = diameter * scale;
    let center = rect.center();

    if utama {
        // Tombol Play/Pause utama ala Spotify:
        // Lingkaran putih terang solid dengan ikon hitam pekat kontras tinggi di tengah
        let bg_color = if hover {
            Color32::WHITE
        } else {
            Color32::from_rgb(245, 245, 245)
        };
        let icon_color = Color32::from_rgb(18, 18, 18);

        if hover {
            ui.painter().circle_filled(
                center,
                diameter * 0.70,
                Color32::from_white_alpha(35),
            );
        }

        ui.painter().circle_filled(center, d / 2.0, bg_color);
        let icon_size = (d * 0.44).round();
        let icon_rect = icons::square_rect(center, icon_size);
        icons::draw(ui.painter(), icon, icon_rect, icon_color, 2.0);
    } else {
        // Tombol sekunder (Prev, Next): ikon warna dim/abu-abu terang, menyala putih saat hover
        if hover {
            ui.painter().circle_filled(
                center,
                d / 2.0,
                Color32::from_white_alpha(30),
            );
        }

        let icon_color = if hover {
            Color32::WHITE
        } else {
            Color32::from_rgb(175, 175, 180)
        };
        let icon_size = (d * 0.48).round();
        let icon_rect = icons::square_rect(center, icon_size);
        icons::draw(ui.painter(), icon, icon_rect, icon_color, 1.8);
    }

    resp.clicked()
}

/// Tombol ikon polos (tanpa latar) untuk tombol tutup dan aksi header.
#[allow(dead_code)]
pub fn icon_hit(
    ui: &mut egui::Ui,
    icon: Icon,
    size: f32,
    theme: &Theme,
    aktif: bool,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());
    let hover = resp.hovered();

    if hover {
        ui.painter().rect_filled(
            rect,
            Rounding::same(R_CONTROL),
            Color32::from_white_alpha(26),
        );
    }

    let warna = if aktif || hover { theme.ink } else { theme.dim };
    let glyph = icons::square_rect(rect.center(), size * 0.62);
    icons::draw(ui.painter(), icon, glyph, warna, 1.5);

    resp.clicked()
}

/// Satu tab di bilah header: tombol ikon bersih berukuran 28×28 px tanpa tag teks keterangan.
#[allow(dead_code)]
pub fn tab_button(
    ui: &mut egui::Ui,
    icon: Icon,
    label: &str,
    aktif: bool,
    badge: u32,
    theme: &Theme,
) -> bool {
    let size = 28.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());
    let hover = resp.hovered();

    let bg = if aktif {
        theme.accent_soft
    } else if hover {
        Color32::from_white_alpha(26)
    } else {
        Color32::TRANSPARENT
    };
    if bg.a() > 0 {
        ui.painter().rect_filled(rect, Rounding::same(R_CONTROL), bg);
    }

    let warna = if aktif {
        theme.accent
    } else if hover {
        theme.ink
    } else {
        theme.dim
    };

    let glyph = icons::square_rect(rect.center(), 16.0);
    icons::draw(ui.painter(), icon, glyph, warna, 1.7);

    // Badge angka kecil di pojok kanan atas ikon jika ada
    if badge > 0 {
        let teks = badge.min(99).to_string();
        let font = egui::FontId::proportional(T_MICRO);
        let g = ui.painter().layout_no_wrap(teks, font, theme.on_accent);
        let bw = (g.size().x + 6.0).max(12.0);
        let c = egui::pos2(rect.right() - 3.0, rect.top() + 5.0);
        let pill = egui::Rect::from_center_size(c, egui::vec2(bw, 11.0));
        ui.painter()
            .rect_filled(pill, Rounding::same(R_PILL), theme.accent);
        ui.painter().galley(
            egui::pos2(
                pill.center().x - g.size().x / 2.0,
                pill.center().y - g.size().y / 2.0,
            ),
            g,
            theme.on_accent,
        );
    }

    let _ = label; // Tag teks keterangan dihilangkan dari tampilan menu

    resp.clicked()
}


/// Pil filter (Semua / Aktif / Selesai) dengan latar aksen saat aktif.
#[allow(dead_code)]
pub fn pill(ui: &mut egui::Ui, label: &str, aktif: bool, theme: &Theme) -> bool {
    let font = egui::FontId::proportional(T_BODY);
    let g = ui.painter().layout_no_wrap(label.to_owned(), font, theme.ink);
    let w = g.size().x + 22.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 22.0), egui::Sense::click());
    let hover = resp.hovered();

    if aktif {
        ui.painter()
            .rect_filled(rect, Rounding::same(R_PILL), theme.accent_soft);
        ui.painter().rect_stroke(
            rect,
            Rounding::same(R_PILL),
            egui::Stroke::new(1.0_f32, theme.accent),
        );
    } else {
        ui.painter().rect_stroke(
            rect,
            Rounding::same(R_PILL),
            egui::Stroke::new(
                1.0_f32,
                if hover {
                    Color32::from_white_alpha(46)
                } else {
                    theme.stroke
                },
            ),
        );
    }

    let warna = if aktif {
        theme.accent
    } else if hover {
        theme.ink
    } else {
        theme.dim
    };
    ui.painter().galley(
        egui::pos2(
            rect.center().x - g.size().x / 2.0,
            rect.center().y - g.size().y / 2.0,
        ),
        g,
        warna,
    );

    resp.clicked()
}

/// Tombol terisi aksen penuh, dipakai untuk aksi utama (mis. "Tambah").
#[allow(dead_code)]
pub fn solid_button(ui: &mut egui::Ui, label: &str, theme: &Theme) -> bool {
    let font = egui::FontId::proportional(T_BODY);
    let g = ui.painter().layout_no_wrap(label.to_owned(), font, theme.ink);
    let w = g.size().x + 26.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 30.0), egui::Sense::click());
    let hover = resp.hovered();

    ui.painter().rect_filled(
        rect,
        Rounding::same(R_CONTROL),
        if hover {
            theme.accent
        } else {
            theme.accent_soft
        },
    );
    ui.painter().rect_stroke(
        rect,
        Rounding::same(R_CONTROL),
        egui::Stroke::new(1.0_f32, theme.accent),
    );

    let warna = if hover { theme.on_accent } else { theme.accent };
    ui.painter().galley(
        egui::pos2(
            rect.center().x - g.size().x / 2.0,
            rect.center().y - g.size().y / 2.0,
        ),
        g,
        warna,
    );

    resp.clicked()
}

/// Tombol utama berikon (mis. "Mulai" di tab Fokus).
#[allow(dead_code)]
pub fn primary_button(ui: &mut egui::Ui, icon: Icon, label: &str, theme: &Theme) -> bool {
    let font = egui::FontId::proportional(T_BODY);
    let g = ui.painter().layout_no_wrap(label.to_owned(), font, theme.ink);
    let w = g.size().x + 56.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 38.0), egui::Sense::click());
    let hover = resp.hovered();

    let bg = if hover { theme.accent } else { theme.accent_soft };
    ui.painter().rect_filled(rect, Rounding::same(R_CARD), bg);
    ui.painter().rect_stroke(
        rect,
        Rounding::same(R_CARD),
        egui::Stroke::new(1.0_f32, theme.accent),
    );

    let warna = if hover { theme.on_accent } else { theme.accent };

    let glyph_rect = icons::square_rect(
        egui::pos2(rect.left() + 20.0, rect.center().y),
        14.0,
    );
    icons::draw(ui.painter(), icon, glyph_rect, warna, 1.8);

    ui.painter().galley(
        egui::pos2(
            rect.left() + 36.0,
            rect.center().y - g.size().y / 2.0,
        ),
        g,
        warna,
    );

    resp.clicked()
}

/// Tombol sekunder bergaris (mis. "Ulang" di tab Fokus).
#[allow(dead_code)]
pub fn ghost_button(ui: &mut egui::Ui, icon: Icon, label: &str, theme: &Theme) -> bool {
    let font = egui::FontId::proportional(T_BODY);
    let g = ui.painter().layout_no_wrap(label.to_owned(), font, theme.dim);
    let w = g.size().x + 52.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 38.0), egui::Sense::click());
    let hover = resp.hovered();

    ui.painter().rect_filled(
        rect,
        Rounding::same(R_CARD),
        if hover {
            Color32::from_white_alpha(30)
        } else {
            theme.surface
        },
    );

    let warna = if hover { theme.ink } else { theme.dim };
    let glyph_rect = icons::square_rect(
        egui::pos2(rect.left() + 18.0, rect.center().y),
        14.0,
    );
    icons::draw(ui.painter(), icon, glyph_rect, warna, 1.8);

    ui.painter().galley(
        egui::pos2(
            rect.left() + 34.0,
            rect.center().y - g.size().y / 2.0,
        ),
        g,
        warna,
    );

    resp.clicked()
}

/// Menggambar ikon garis ke `rect` dengan warna dan ketebalan tertentu.
pub fn glyph(painter: &egui::Painter, icon: Icon, rect: egui::Rect, warna: Color32, tebal: f32) {
    icons::draw(painter, icon, rect, warna, tebal);
}

/// Segmented control (`.seg` di prototipe): wadah abu-abu dengan segmen yang
/// dipilih berwarna terang.
///
/// Mengembalikan indeks segmen yang baru diklik, atau `None`.
pub fn segmented(
    ui: &mut egui::Ui,
    pilihan: &[(&str, bool)],
    theme: &Theme,
) -> Option<usize> {
    let font = egui::FontId::proportional(style::T_SUBTITLE);
    const PAD_X: f32 = 13.0;
    const H: f32 = 30.0;

    // Ukur dulu: lebar tiap segmen mengikuti teksnya.
    let lebar: Vec<f32> = pilihan
        .iter()
        .map(|(label, _)| {
            ui.painter()
                .layout_no_wrap((*label).to_owned(), font.clone(), theme.ink)
                .size()
                .x
                + PAD_X * 2.0
        })
        .collect();
    let total: f32 = lebar.iter().sum::<f32>() + 4.0;

    let (rect, _) = ui.allocate_exact_size(egui::vec2(total, H), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, Rounding::same(10.0), theme.surface);

    let mut hasil = None;
    let mut x = rect.left() + 2.0;
    for (i, (label, aktif)) in pilihan.iter().enumerate() {
        let w = lebar[i];
        let seg = egui::Rect::from_min_size(
            egui::pos2(x, rect.top() + 2.0),
            egui::vec2(w, H - 4.0),
        );
        let resp = ui.interact(seg, egui::Id::new("seg").with(i).with(label), egui::Sense::click());

        if *aktif {
            // `.seg button[aria-pressed=true]{background:rgba(255,255,255,.2)}`
            ui.painter()
                .rect_filled(seg, Rounding::same(8.0), Color32::from_white_alpha(51));
        } else if resp.hovered() {
            ui.painter().rect_filled(
                seg,
                Rounding::same(8.0),
                Color32::from_white_alpha(26),
            );
        }

        let g = ui.painter().layout_no_wrap(
            (*label).to_owned(),
            font.clone(),
            if *aktif { theme.ink } else { theme.dim },
        );
        ui.painter().galley(
            egui::pos2(
                seg.center().x - g.size().x / 2.0,
                seg.center().y - g.size().y / 2.0,
            ),
            g,
            if *aktif { theme.ink } else { theme.dim },
        );

        if resp.clicked() {
            hasil = Some(i);
        }
        x += w;
    }

    hasil
}

/// Tombol pil (`.pill` di prototipe): tinggi 36, radius penuh.
///
/// `utama = true` memberi latar aksen penuh — satu-satunya tombol berwarna
/// di halaman, jadi aksi utama selalu terbaca tanpa label tambahan.
pub fn pill_button(ui: &mut egui::Ui, label: &str, utama: bool, theme: &Theme) -> bool {
    let font = egui::FontId::proportional(style::T_BODY);
    let g = ui.painter().layout_no_wrap(label.to_owned(), font, theme.ink);
    let w = (g.size().x + 24.0).max(if utama { 72.0 } else { 54.0 });
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 28.0), egui::Sense::click());
    let hover = resp.hovered();
    let tekan = resp.is_pointer_button_down_on();

    let d = if tekan { 0.95 } else { 1.0 };
    let r = egui::Rect::from_center_size(
        rect.center(),
        egui::vec2(rect.width() * d, rect.height() * d),
    );

    if utama {
        ui.painter().rect_filled(
            r,
            Rounding::same(14.0),
            if hover { theme.rim } else { theme.accent },
        );
    } else if hover {
        ui.painter()
            .rect_filled(r, Rounding::same(14.0), theme.fill2);
    } else {
        ui.painter()
            .rect_filled(r, Rounding::same(14.0), theme.surface);
    }

    let warna = if utama { theme.on_accent } else { theme.ink };
    ui.painter().galley(
        egui::pos2(
            r.center().x - g.size().x / 2.0,
            r.center().y - g.size().y / 2.0,
        ),
        g,
        warna,
    );

    resp.clicked()
}

/// Tombol ikon bulat (`.ib` di prototipe): latar transparan, muncul saat
/// hover, mengecil 0.9× saat ditekan.
///
/// `aktif` memberi keadaan menyala — latar `accent-soft` dan ikon berwarna
/// aksen, sama seperti `.ib.on`.
pub fn icon_button(ui: &mut egui::Ui, icon: Icon, size: f32, aktif: bool, theme: &Theme) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());
    let hover = resp.hovered();
    let tekan = resp.is_pointer_button_down_on();

    // `.ib:active{transform:scale(.9)}`
    let d = if tekan { size * 0.90 } else { size };

    let bg = if aktif {
        theme.accent_soft
    } else if hover {
        theme.surface
    } else {
        Color32::TRANSPARENT
    };
    if bg.a() > 0 {
        ui.painter()
            .circle_filled(rect.center(), d / 2.0, bg);
    }

    let warna = if aktif {
        theme.accent
    } else if hover {
        theme.ink
    } else {
        theme.ink
    };
    let glyph_d = (d * 0.50).round();
    icons::draw(
        ui.painter(),
        icon,
        icons::square_rect(rect.center(), glyph_d),
        warna,
        1.7,
    );

    resp.clicked()
}

/// Tombol putar utama (`.ib.play`): lingkaran putih penuh dengan ikon hitam.
///
/// Ini satu-satunya elemen putih solid di panel; prototipe memakainya sebagai
/// titik fokus baris transport, jadi ukurannya lebih besar dari tombol lain.
pub fn play_button(ui: &mut egui::Ui, icon: Icon, size: f32, _theme: &Theme) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());
    let hover = resp.hovered();
    let tekan = resp.is_pointer_button_down_on();

    let d = if tekan {
        size * 0.94
    } else if hover {
        size * 1.06
    } else {
        size
    };

    ui.painter().circle_filled(rect.center(), d / 2.0, Color32::WHITE);
    icons::draw(
        ui.painter(),
        icon,
        icons::square_rect(rect.center(), (d * 0.44).round()),
        Color32::BLACK,
        1.8,
    );

    resp.clicked()
}

/// Headline untuk kartu surface besar — angka satu baris yang jadi alasan
/// halaman ini ada (stat Metrik sistem, sisa waktu Pomodoro, hitungan air).
#[allow(dead_code)]
pub fn card_headline(s: impl Into<String>, color: egui::Color32) -> egui::RichText {
    style::card_headline(s, color)
}

/// Tombol lebar penuh dengan ikon di kiri dan label di tengah.
#[allow(dead_code)]
pub fn wide_button(ui: &mut egui::Ui, icon: Icon, label: &str, theme: &Theme) -> bool {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 40.0), egui::Sense::click());
    let hover = resp.hovered();

    let bg = if hover {
        theme.accent
    } else {
        theme.accent_soft
    };
    ui.painter().rect_filled(rect, Rounding::same(R_CARD), bg);

    // Saat hover, tombol terisi penuh warna aksen — warnanya mengikuti
    // luminans aksen, jadi tetap terbaca walau cover lagunya gelap.
    let warna = if hover { theme.on_accent } else { theme.accent };
    let glyph_rect = icons::square_rect(egui::pos2(rect.left() + 20.0, rect.center().y), 15.0);
    icons::draw(ui.painter(), icon, glyph_rect, warna, 1.7);

    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(T_BODY),
        warna,
    );

    resp.clicked()
}

/// Menampilkan waktu mm:ss dengan animasi rolling digit mekanis untuk detik yang berganti (Odometer effect ala Mochy).
#[allow(dead_code)]
pub fn odometer_time(
    ui: &mut egui::Ui,
    time_s: f32,
    size: f32,
    color: egui::Color32,
) {
    let font_id = egui::FontId::monospace(size);
    let char_w = size * 0.60;
    let char_h = size * 1.25;

    let time_i = time_s.max(0.0) as i64;
    let mins = (time_i / 60).min(99);
    let secs = time_i % 60;
    let text = format!("{:02}:{:02}", mins, secs);

    // Hitung fase pergantian detik (150ms terakhir dari detik berjalan)
    let frac_sec = time_s.fract();
    let roll_t = if frac_sec > 0.85 {
        let t = (frac_sec - 0.85) / 0.15;
        // Cubic smooth-step
        t * t * (3.0 - 2.0 * t)
    } else {
        0.0
    };

    let total_w = char_w * 5.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(total_w, char_h), egui::Sense::hover());
    let painter = ui.painter().with_clip_rect(rect);

    for (i, c) in text.chars().enumerate() {
        let x = rect.left() + i as f32 * char_w + char_w / 2.0;
        let cy = rect.center().y;

        // Digit detik satuan (indeks 4) berputar rolling ke digit berikutnya
        if i == 4 && roll_t > 0.0 {
            let next_digit = ((secs + 1) % 10) as u8;
            let next_char = (b'0' + next_digit) as char;

            // Digit lama bergeser ke atas dan memudar
            let y_old = cy - roll_t * char_h;
            let alpha_old = ((1.0 - roll_t) * 255.0) as u8;
            painter.text(
                egui::pos2(x, y_old),
                egui::Align2::CENTER_CENTER,
                c.to_string(),
                font_id.clone(),
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha_old),
            );

            // Digit baru meluncur dari bawah dan menguat
            let y_new = cy + (1.0 - roll_t) * char_h;
            let alpha_new = (roll_t * 255.0) as u8;
            painter.text(
                egui::pos2(x, y_new),
                egui::Align2::CENTER_CENTER,
                next_char.to_string(),
                font_id.clone(),
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha_new),
            );
        } else {
            painter.text(
                egui::pos2(x, cy),
                egui::Align2::CENTER_CENTER,
                c.to_string(),
                font_id.clone(),
                color,
            );
        }
    }
}

/* ------------------------------------------------------- primitif halaman */

/// Kepala halaman notch: sabuk tipis berisi judul di kiri dan isi bebas di
/// kanan, dipisah garis rambut.
///
/// Semua halaman memakai ini supaya judul selalu berada di ketinggian yang
/// sama. Sebelumnya Music tidak punya judul, Tugas memakai huruf kapital
/// semua, dan Pomodoro memakai ukuran lain — tiga perlakuan untuk peran yang
/// sebenarnya satu. Pengguna tidak bisa membedakan "ini halaman lain" dari
/// "ini bagian lain", dan itu akar dari panel yang terasa acak.
///
/// Mengembalikan tinggi yang dipakai, supaya pemanggil bisa mengurangkan
/// sisa ruang untuk isi.
pub fn page_header(
    ui: &mut egui::Ui,
    judul: &str,
    theme: &Theme,
    kanan: impl FnOnce(&mut egui::Ui),
) -> f32 {
    const TINGGI: f32 = 20.0;

    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), TINGGI), egui::Sense::hover());

    ui.painter().text(
        egui::pos2(rect.left(), rect.center().y),
        egui::Align2::LEFT_CENTER,
        judul,
        egui::FontId::proportional(T_MICRO),
        theme.faint,
    );

    // Isi kanan memakai anak UI sendiri supaya bisa memuat chip/tombol tanpa
    // memaksa setiap halaman menghitung lebar sendiri.
    let kanan_w = rect.width() * 0.55;
    let mut kanan_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(egui::Rect::from_min_size(
                egui::pos2(rect.right() - kanan_w, rect.top()),
                egui::vec2(kanan_w, TINGGI),
            ))
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    kanan(&mut kanan_ui);

    ui.add_space(S1);
    ui.painter().line_segment(
        [egui::pos2(rect.left(), rect.bottom() + 2.0), egui::pos2(rect.right(), rect.bottom() + 2.0)],
        egui::Stroke::new(1.0_f32, theme.stroke),
    );
    ui.add_space(S2);

    TINGGI + S1 + S2 + 3.0
}

/// Kartu metrik (`.tile` di prototipe): label kecil, angka besar berwarna
/// aksen, dan sparkline area di dasar kartu.
///
/// Sparkline digambar di dalam kartu, bukan sebagai grafik terpisah — ia
/// berfungsi sebagai latar yang menunjukkan arah perubahan, bukan sebagai
/// grafik yang dibaca nilainya.
#[allow(clippy::too_many_arguments)]
pub fn spark_tile(
    ui: &mut egui::Ui,
    lebar: f32,
    tinggi: f32,
    label: &str,
    nilai: &str,
    satuan: Option<&str>,
    data: &[f32],
    min: f32,
    max: f32,
    warna: Color32,
    theme: &Theme,
) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(lebar, tinggi), egui::Sense::hover());
    let radius = Rounding::same(R_CARD);
    ui.painter().rect_filled(rect, radius, theme.surface);

    // Isi kartu dipotong ke bentuknya sendiri supaya sparkline tidak
    // menembus sudut membulat.
    let dalam = ui.painter().with_clip_rect(rect);
    let pad = 14.0;
    let kiri = rect.left() + pad;

    dalam.text(
        egui::pos2(kiri, rect.top() + 14.0),
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::proportional(style::T_CAPTION),
        theme.dim,
    );

    let nilai_galley = dalam.layout_no_wrap(
        nilai.to_owned(),
        egui::FontId::proportional(style::T_STAT),
        warna,
    );
    let nilai_y = rect.top() + 32.0;
    let nilai_w = nilai_galley.size().x;
    dalam.galley(egui::pos2(kiri, nilai_y), nilai_galley, warna);

    if let Some(s) = satuan {
        dalam.text(
            egui::pos2(kiri + nilai_w + 3.0, nilai_y + style::T_STAT - style::T_CAPTION - 2.0),
            egui::Align2::LEFT_TOP,
            s,
            egui::FontId::proportional(style::T_CAPTION),
            theme.dim,
        );
    }

    // ---- Sparkline area di dasar kartu ----
    if data.len() > 1 {
        let plot_h = (tinggi * 0.42).max(12.0);
        let plot = Rect::from_min_max(
            egui::pos2(rect.left(), rect.bottom() - plot_h),
            egui::pos2(rect.right(), rect.bottom()),
        );
        let span = (max - min).max(0.001);
        let step = plot.width() / (data.len() - 1) as f32;

        let titik: Vec<egui::Pos2> = data
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let t = ((v - min) / span).clamp(0.0, 1.0);
                egui::pos2(
                    plot.left() + i as f32 * step,
                    plot.bottom() - t * (plot.height() - 6.0) - 3.0,
                )
            })
            .collect();

        // Area: poligon tertutup ke dasar, dengan alpha rendah.
        let mut area = titik.clone();
        area.push(egui::pos2(plot.right(), plot.bottom()));
        area.push(egui::pos2(plot.left(), plot.bottom()));
        dalam.add(egui::Shape::convex_polygon(
            area,
            Color32::from_rgba_unmultiplied(warna.r(), warna.g(), warna.b(), 46),
            Stroke::NONE,
        ));

        dalam.add(egui::Shape::line(titik, Stroke::new(1.8_f32, warna)));
    }

    let _ = radius;
}

/// Satuan angka besar dengan label kecil di bawahnya.
///
/// Dipakai Metrik sistem, sisa waktu Pomodoro, dan hitungan air. Satu bentuk
/// untuk satu peran: "satu angka yang jadi alasan halaman ini ada".
#[allow(dead_code)]
pub fn stat_tile(
    ui: &mut egui::Ui,
    lebar: f32,
    label: &str,
    nilai: &str,
    warna: Color32,
    theme: &Theme,
) {
    const H: f32 = 46.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(lebar, H), egui::Sense::hover());

    ui.painter().rect_filled(rect, Rounding::same(R_CARD), theme.surface);

    ui.painter().text(
        egui::pos2(rect.left() + style::CARD_PAD, rect.top() + 9.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(T_MICRO),
        theme.faint,
    );
    ui.painter().text(
        egui::pos2(rect.left() + style::CARD_PAD, rect.bottom() - 13.0),
        egui::Align2::LEFT_CENTER,
        nilai,
        egui::FontId::proportional(style::T_TITLE),
        warna,
    );
}

/// Cincin progres. `frac` 0..1, `tebal` garis cincin.
///
/// Menggantikan busur yang digambar tangan di halaman air — di sana cincinnya
/// selalu penuh karena `circle_stroke` dipanggil dengan busur 360°, sehingga
/// "progres" hanya terlihat dari titik di tengah, bukan dari cincinnya.
pub fn progress_ring(
    ui: &mut egui::Ui,
    ukuran: f32,
    frac: f32,
    warna: Color32,
    theme: &Theme,
    tengah: impl FnOnce(&mut egui::Ui),
) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ukuran, ukuran), egui::Sense::hover());
    let center = rect.center();
    let r = ukuran / 2.0 - 2.5;
    let frac = frac.clamp(0.0, 1.0);

    ui.painter()
        .circle_stroke(center, r, egui::Stroke::new(2.5_f32, theme.surface));

    if frac > 0.001 {
        // Busur dari jam 12, searah jarum jam.
        let mulai = -std::f32::consts::FRAC_PI_2;
        let titik: Vec<egui::Pos2> = (0..=48)
            .map(|i| {
                let a = mulai + frac * std::f32::consts::TAU * (i as f32 / 48.0);
                egui::pos2(center.x + r * a.cos(), center.y + r * a.sin())
            })
            .collect();
        ui.painter()
            .add(egui::Shape::line(titik, egui::Stroke::new(2.5_f32, warna)));
    }

    let mut dalam = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(egui::Rect::from_center_size(center, egui::vec2(ukuran - 8.0, ukuran - 8.0)))
            .layout(egui::Layout::centered_and_justified(egui::Direction::TopDown)),
    );
    tengah(&mut dalam);
}

/// Teks satu baris yang memendek dengan elipsis di lebar yang diberikan.
///
/// `painter.text` tidak memotong apa pun — teks panjang menembus kartu dan
/// menimpa elemen di kanannya. Ini yang dulu terjadi pada judul lagu dan
/// nama tugas di halaman notch.
pub fn ellipsis(ui: &egui::Ui, teks: &str, lebar: f32) -> String {
    ellipsis_sized(ui, teks, lebar, T_BODY)
}

/// Sama seperti [`ellipsis`], tetapi untuk ukuran huruf tertentu.
///
/// Halaman lirik memakai 15 px, bukan `T_BODY` 14 px; memakai ukuran yang
/// salah membuat perkiraan lebar meleset dan elipsis muncul terlalu cepat
/// atau terlambat.
pub fn ellipsis_sized(ui: &egui::Ui, teks: &str, lebar: f32, size: f32) -> String {
    let font = egui::FontId::proportional(size);
    let g = ui.painter().layout_no_wrap(teks.to_owned(), font.clone(), Color32::WHITE);
    if g.size().x <= lebar {
        return teks.to_owned();
    }

    let mut hasil = String::new();
    for c in teks.chars() {
        let coba = format!("{hasil}{c}");
        let w = ui
            .painter()
            .layout_no_wrap(coba.clone(), font.clone(), Color32::WHITE)
            .size()
            .x;
        if w > lebar - 10.0 {
            break;
        }
        hasil.push(c);
    }
    format!("{}…", hasil.trim_end())
}

/// Judul lagu / teks satu baris yang beranimasi marquee ping-pong jika melebihi lebar kontainer.
///
/// Jika `text_w <= width`, teks digambar statis di `rect.min`.
/// Jika `text_w > width`, teks bergeser mulus (smooth cosine ease) bolak-balik:
/// jeda 1.8s di awal (offset = 0.0), geser ~30 px/detik ke max_scroll, jeda 1.8s di akhir
/// (offset = max_scroll), lalu geser kembali ke 0.0.
/// Offset tidak pernah melebihi `max_scroll` agar ujung judul terlihat tuntas tanpa tenggelam.
pub fn marquee_title(
    ui: &mut egui::Ui,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    width: f32,
) -> egui::Response {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font, color);
    let text_w = galley.size().x;
    let line_h = galley.size().y;

    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, line_h), egui::Sense::hover());

    if text_w <= width {
        ui.painter().galley(rect.min, galley, color);
        return resp;
    }

    let max_scroll = text_w - width;
    let scroll_dur = (max_scroll / 30.0).max(0.5);
    const PAUSE: f32 = 1.8;
    let cycle_dur = (PAUSE + scroll_dur + PAUSE + scroll_dur) as f64;

    let time = ui.input(|i| i.time);
    let t = (time % cycle_dur) as f32;

    let offset = if t < PAUSE {
        0.0
    } else if t < PAUSE + scroll_dur {
        let progress = (t - PAUSE) / scroll_dur;
        let ease = (1.0 - (progress * std::f32::consts::PI).cos()) * 0.5;
        ease * max_scroll
    } else if t < PAUSE + scroll_dur + PAUSE {
        max_scroll
    } else {
        let progress = (t - (PAUSE + scroll_dur + PAUSE)) / scroll_dur;
        let ease = (1.0 - (progress * std::f32::consts::PI).cos()) * 0.5;
        (1.0 - ease) * max_scroll
    };

    let offset = offset.clamp(0.0, max_scroll);

    ui.ctx().request_repaint();

    let prev_clip = ui.painter().clip_rect();
    let clip = rect.intersect(prev_clip);
    let painter = ui.painter().with_clip_rect(clip);
    painter.galley(egui::pos2(rect.left() - offset, rect.top()), galley, color);

    resp
}

/// Berkas yang ditampung sementara di File Drop Shelf.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ShelfFile {
    pub name: String,
    pub path: std::path::PathBuf,
    pub size_bytes: u64,
}

/// Menampilkan rak berkas sementara (File Drop Shelf).
pub fn render_drop_shelf(
    ui: &mut egui::Ui,
    files: &mut Vec<ShelfFile>,
    theme: &Theme,
    accent: Color32,
) {
    if files.is_empty() {
        return;
    }

    ui.add_space(4.0);
    egui::Frame::none()
        .fill(Color32::from_rgba_premultiplied(20, 20, 24, 220))
        .stroke(Stroke::new(1.0, Color32::from_white_alpha(30)))
        .rounding(Rounding::same(8.0))
        .inner_margin(egui::Margin::symmetric(10.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("📁 Drop Shelf")
                        .size(11.0)
                        .strong()
                        .color(accent),
                );
                ui.label(
                    egui::RichText::new(format!("({} berkas)", files.len()))
                        .size(10.0)
                        .color(theme.dim),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Bersihkan")
                                    .size(10.0)
                                    .color(theme.dim),
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        files.clear();
                    }
                });
            });

            ui.add_space(4.0);
            let mut to_remove = None;
            for (idx, f) in files.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("📄 {}", f.name))
                            .size(11.0)
                            .color(theme.ink),
                    );
                    let size_str = noktah_core::format::bytes(f.size_bytes);
                    ui.label(
                        egui::RichText::new(size_str)
                            .size(10.0)
                            .color(theme.dim),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("✕")
                                        .size(10.0)
                                        .color(Color32::from_rgb(239, 68, 68)),
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            to_remove = Some(idx);
                        }
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("📋 Salin").size(10.0).color(accent),
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            ui.output_mut(|o| o.copied_text = f.path.to_string_lossy().to_string());
                        }
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("📂 Buka")
                                        .size(10.0)
                                        .color(theme.dim),
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            let _ = std::process::Command::new("explorer.exe")
                                .arg(format!("/select,{}", f.path.display()))
                                .spawn();
                        }
                    });
                });
            }
            if let Some(idx) = to_remove {
                files.remove(idx);
            }
        });
}
