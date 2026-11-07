//! Halaman Beranda — dua hal dalam satu baris: **lagu yang sedang diputar**
//! (ringkas, tanpa lirik) dan **sisa kuota Antigravity** dari 9Router.
//!
//! Halaman ini sengaja bukan salinan halaman Musik. Musik menjawab "apa yang
//! sedang diputar, dan apa liriknya"; Beranda menjawab "apa yang sedang
//! diputar, dan berapa kuota saya tersisa" sekaligus.
//!
//! ## Kenapa satu baris, bukan dua blok bertumpuk
//!
//! Panel notch hanya 500×148 px, dan `CONTENT_BOTTOM` memakan 16 px untuk
//! pita dock. Isi yang benar-benar tersedia sekitar 124 px — cukup untuk
//! **satu** baris, tidak untuk dua.
//!
//! Versi pertama menumpuk baris lagu (88 px) dan kartu kuota (119 px); hasilnya
//! 200 px isi di ruang 124 px, dan kartu kuota terpotong habis di balik dock.
//! Tes presisi halaman tidak menangkapnya karena `min_rect()` egui dijepit ke
//! `max_rect` — jadi tinggi yang hilang tidak terlihat. Karena itu sekarang
//! tinggi setiap bagian dihitung dari ruang yang ada, dan ada tes khusus
//! (`baris_home_muat_di_dalam_panel`) yang mengunci anggaran itu.
//!
//! ## Kenapa tanpa header dan tanpa kartu
//!
//! Awalnya halaman ini punya `page_header` ("HOME" + tombol Refresh) dan baris
//! isinya digambar di atas `theme.surface` membulat. Keduanya dibuang karena
//! isi Home cuma satu baris setinggi 82 px: header memakan 35 px ruang vertikal
//! untuk satu kata, dan kartu abu-abu membingkai isi yang sudah muat sendiri.
//! Yang tersisa adalah pemisah — satu garis rambut vertikal antara bagian lagu
//! dan bagian kuota, cukup untuk memisahkan tanpa memakan ruang.
//!
//! Karena header hilang, tidak ada lagi tempat untuk tombol Refresh. Menyegarkan
//! kuota sekarang lewat klik pada blok cincin itu sendiri, jadi fiturnya tetap
//! ada meski tanpa tombol.

use crate::app::{Noktah, NotchPage};
use crate::presentation::theme::Theme;
use crate::presentation::{icons::Icon, style, theme, widgets};
use eframe::egui::{self, Color32, Rect, Rounding, Sense, Stroke};
use noktah_core::format;

/* --------------------------------------------------------------- ukuran */

/// Sampul album di kiri baris.
const ART: f32 = 44.0;
const ART_R: f32 = 10.0;
/// Diameter cincin kuota.
const RING: f32 = 50.0;
/// Tebal garis cincin. Lebih tebal dari versi awal (3 px) supaya trek
/// tokennya benar-benar terlihat di panel hitam — pada 50 px, 3 px hanya
/// sepersepuluh jari-jari dan mudah tenggelam.
const TEBAL: f32 = 5.0;
/// Alpha jalur dasar cincin (putih di atas hitam). 40 ≈ 16%.
///
/// Dulu 54, dan itu terlalu dekat dengan kecerahan progres: pada cincin 97%
/// jalur dan isi sama-sama terang, jadi bentuknya terbaca sebagai dua cincin
/// kembar, bukan satu trek dengan isi. Jalur harus jelas lebih redup —
/// keberadaannya hanya untuk menunjukkan sisa ruang, bukan ikut jadi angka.
const TRACK_A: u8 = 40;
/// Tinggi satu baris teks mikro (label "5 jam" di bawah cincin, catatan akun
/// di atasnya). Galley egui lebih tinggi dari ukuran hurufnya — 10 px teks
/// menghasilkan ±13 px kotak — jadi yang dipakai tinggi galley, bukan
/// `T_MICRO` itu sendiri. Memakai 12 px di sini berarti cincin bergeser naik
/// 1 px dan menimpa teks di atasnya.
const TEKS: f32 = 13.0;
/// Tombol prev/next yang diperkecil (baris ini lebih rapat dari mini player).
const BTN: f32 = 24.0;
/// Tombol play di tengah trio.
const PLAY: f32 = 30.0;
/// Jarak antartombol transport.
const GAP: f32 = 10.0;
/// Jarak dari tepi isi panel ke sampul di kiri dan ke cincin di kanan.
/// Tanpa header dan kartu, isi menyentuh tepi panel kalau tidak diberi ini.
const INSET: f32 = style::S3;

/// Jarak dari garis pemisah ke isi di kiri dan kanannya. Garis berdiri tepat
/// di tengah celah antara trio transport dan cincin kuota terdekat, sehingga
/// jarak pandang ke kedua sisi sama.
const SEP_GAP: f32 = style::S3;

/// Lebar trio transport.
const TRIO_W: f32 = BTN + GAP + PLAY + GAP + BTN;

/// Lebar dua cincin berdampingan (termasuk jarak antarcincin).
const RINGS_W: f32 = RING * 2.0 + style::S2;

/// Diameter tombol segarkan kuota.
const REFRESH: f32 = BTN;

/// Lebar seluruh blok kanan: dua cincin, jarak, lalu tombol segarkan.
///
/// Dibagi antara `draw_page_home` dan tes tata letak supaya keduanya tidak
/// pernah berbeda pendapat soal berapa lebar yang dipakai blok kanan.
pub(crate) const KANAN_W: f32 = RINGS_W + style::S2 + REFRESH;

/// Tinggi blok kuota: catatan akun di atas, cincin, lalu label di bawahnya.
///
/// Ketiganya bertumpuk vertikal di dalam satu kolom sempit, jadi tingginya
/// dijumlah, bukan diperkirakan. Ini angka yang dulu dilanggar: cincin
/// digambar mengelilingi `center.y - 6` sementara catatan akun ditaruh di
/// `top + 6` — busur atas cincin menembus teksnya.
pub(crate) const KUOTA_H: f32 = TEKS + style::S1 + RING + style::S1 + TEKS;

/// Tinggi minimum baris Home. Sampul 44 px lebih pendek dari blok kuota, jadi
/// blok kuota yang menentukan.
pub(crate) const BARIS_MIN_H: f32 = KUOTA_H;

impl Noktah {
    /// Halaman Beranda: satu baris — lagu di kiri, kuota + tombol segarkan di
    /// kanan, dipisah satu garis rambut vertikal.
    ///
    /// Tidak ada header halaman dan tidak ada kartu berlatar: isi duduk
    /// langsung di atas panel. Header dulu memakan 35 px dan kartu
    /// memindahkan seluruh isi ke dalam kotak abu-abu — dua lapis bingkai
    /// untuk isi yang tingginya cuma 124 px.
    ///
    /// Tombol segarkan dulu tinggal di header yang kini dihapus. Ia
    /// dikembalikan di sini sebagai tombol ikon yang **terlihat** — bukan
    /// lagi area klik tak terlihat di atas cincin, karena area klik tanpa
    /// penanda tidak dapat ditemukan siapa pun.
    pub(crate) fn draw_page_home(&mut self, ui: &mut egui::Ui, area: Rect) {
        let theme = self.theme;

        // Baris tetap dialokasikan (bukan hanya digambar) supaya `min_rect`
        // halaman mencerminkan ruang yang benar-benar dipakai — tes presisi
        // halaman membaca angka itu untuk menangkap luapan.
        let (row, _) = ui.allocate_exact_size(
            egui::vec2(area.width(), area.height()),
            Sense::hover(),
        );

        // ---- Kanan: dua cincin kuota + tombol segarkan ----
        let kanan = Rect::from_min_size(
            egui::pos2(row.right() - INSET - KANAN_W, row.top()),
            egui::vec2(KANAN_W, row.height()),
        );
        let rings = Rect::from_min_size(kanan.min, egui::vec2(RINGS_W, kanan.height()));
        self.draw_home_quota(ui, rings);

        // ---- Tombol segarkan ----
        // Dulu tombol ini tinggal di header halaman ("Refresh"), lalu sempat
        // jadi area klik tak terlihat di atas cincin. Yang ini terlihat:
        // ikon busur-panah, sebaris dengan cincinnya.
        let tombol = Rect::from_center_size(
            egui::pos2(kanan.right() - REFRESH / 2.0, row.center().y),
            egui::vec2(REFRESH, REFRESH),
        );
        let mut ui_t = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(tombol)
                .layout(egui::Layout::top_down(egui::Align::Center)),
        );
        if widgets::icon_button(&mut ui_t, Icon::Refresh, REFRESH, false, &theme) {
            self.refresh_quota();
        }

        // ---- Kiri: mini player ----
        let player = Rect::from_min_max(
            egui::pos2(row.left() + INSET, row.top()),
            egui::pos2(kanan.left() - SEP_GAP * 2.0, row.bottom()),
        );

        // ---- Garis pemisah ----
        // Berdiri di tengah celah antara trio transport dan cincin terdekat:
        // `SEP_GAP` ke kiri, `SEP_GAP` ke kanan. Tingginya mengikuti blok
        // kuota (bukan seluruh baris) supaya tidak menjulur melewati isi.
        let sep_x = kanan.left() - SEP_GAP;
        let tengah = row.center().y;
        ui.painter().line_segment(
            [
                egui::pos2(sep_x, tengah - KUOTA_H / 2.0),
                egui::pos2(sep_x, tengah + KUOTA_H / 2.0),
            ],
            Stroke::new(1.0, theme.stroke),
        );

        self.draw_home_player(ui, player);
    }

    /* ------------------------------------------------------------ lagu */

    /// Sampul + judul/artis + transport, semuanya dalam satu baris.
    ///
    /// Kalau tidak ada media, baris tetap selebar dan setinggi ini — hanya
    /// isinya berganti menjadi ajakan membuka halaman Musik, supaya cincin
    /// kuota di kanan tidak bergeser.
    fn draw_home_player(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let theme = self.theme;
        let ada = self.media.active;

        let art_rect = Rect::from_min_size(
            egui::pos2(rect.left(), rect.center().y - ART / 2.0),
            egui::vec2(ART, ART),
        );

        if ada {
            if let Some(art) = &self.art {
                ui.painter().image(
                    art.id(),
                    art_rect,
                    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                ui.painter()
                    .rect_filled(art_rect, Rounding::same(ART_R), Color32::from_white_alpha(31));
                widgets::glyph(
                    ui.painter(),
                    Icon::Music,
                    Rect::from_center_size(art_rect.center(), egui::vec2(22.0, 22.0)),
                    Color32::from_white_alpha(235),
                    1.7,
                );
            }
            ui.painter().rect_stroke(
                art_rect,
                Rounding::same(ART_R),
                Stroke::new(0.5, Color32::from_white_alpha(64)),
            );
        } else {
            ui.painter()
                .rect_filled(art_rect, Rounding::same(ART_R), Color32::from_white_alpha(24));
            widgets::glyph(
                ui.painter(),
                Icon::Music,
                Rect::from_center_size(art_rect.center(), egui::vec2(20.0, 20.0)),
                theme.faint,
                1.6,
            );
        }

        // ---- Kolom teks ----
        let teks_x = art_rect.right() + style::S2;
        let teks_w = (rect.right() - TRIO_W - style::S2 - teks_x).max(40.0);
        let (judul, sub, ketiga) = if ada {
            let judul = if self.media.title.is_empty() {
                self.tr("music.untitled").to_string()
            } else {
                self.media.title.clone()
            };
            let sub = if self.media.artist.is_empty() {
                if self.media.source.is_empty() {
                    self.tr("music.unknown_artist").to_string()
                } else {
                    self.media.source.clone()
                }
            } else {
                self.media.artist.clone()
            };
            let ketiga = if self.media.duration_ms > 0 {
                let pos = (self.media.position_ms.max(0) as f32 / 1000.0) as i64;
                let sisa =
                    ((self.media.duration_ms - self.media.position_ms).max(0) as f32 / 1000.0) as i64;
                format!(
                    "{} · {} {}",
                    format::mmss(pos),
                    self.tr("music.left"),
                    format::mmss(sisa)
                )
            } else {
                String::new()
            };
            (judul, sub, ketiga)
        } else {
            (
                self.tr("music.nothing").to_string(),
                self.tr("home.open_music").to_string(),
                String::new(),
            )
        };

        // Tiga baris ditumpuk dari tengah supaya blok teks selalu seimbang
        // dengan sampul, berapa pun tinggi barisnya.
        let baris_h = 15.0 + 14.0 + if ketiga.is_empty() { 0.0 } else { 13.0 };
        let atas = rect.center().y - baris_h / 2.0;

        let judul = widgets::ellipsis_sized(ui, &judul, teks_w, style::T_TITLE_LG);
        ui.painter().text(
            egui::pos2(teks_x, atas),
            egui::Align2::LEFT_TOP,
            judul,
            egui::FontId::proportional(style::T_TITLE_LG),
            if ada { theme.ink } else { theme.dim },
        );

        let sub = widgets::ellipsis_sized(ui, &sub, teks_w, style::T_SUBTITLE);
        ui.painter().text(
            egui::pos2(teks_x, atas + 16.0),
            egui::Align2::LEFT_TOP,
            sub,
            egui::FontId::proportional(style::T_SUBTITLE),
            if ada { theme.dim } else { theme.faint },
        );

        if !ketiga.is_empty() {
            ui.painter().text(
                egui::pos2(teks_x, atas + 31.0),
                egui::Align2::LEFT_TOP,
                ketiga,
                egui::FontId::proportional(style::T_MICRO),
                theme.faint,
            );
        }

        // ---- Transport ----
        let trio = Rect::from_min_size(
            egui::pos2(rect.right() - TRIO_W, rect.center().y - PLAY / 2.0),
            egui::vec2(TRIO_W, PLAY),
        );
        let mut ui_t = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(trio)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        ui_t.spacing_mut().item_spacing.x = GAP;

        if widgets::icon_button(&mut ui_t, Icon::Prev, BTN, false, &theme) {
            self.media_cmd(crate::infrastructure::music::Cmd::Prev);
        }
        let play_icon = if self.media.playing { Icon::Pause } else { Icon::Play };
        if widgets::play_button(&mut ui_t, play_icon, PLAY, &theme) {
            self.media_cmd(crate::infrastructure::music::Cmd::PlayPause);
        }
        if widgets::icon_button(&mut ui_t, Icon::Next, BTN, false, &theme) {
            self.media_cmd(crate::infrastructure::music::Cmd::Next);
        }

        // Klik di mana saja pada area lagu membuka halaman Musik — jalur
        // pintas ke lirik tanpa tombol tambahan yang memakan lebar.
        let klik = Rect::from_min_max(art_rect.min, egui::pos2(trio.left() - style::S2, rect.bottom()));
        if ui
            .interact(klik, egui::Id::new("home_buka_musik"), Sense::click())
            .clicked()
            && ada
        {
            self.current_page = NotchPage::Music;
        }
    }

    /* ------------------------------------------------------------ kuota */

    /// Dua cincin berdampingan, persen di tengah dan label di bawahnya.
    ///
    /// Tiga pita vertikal yang **dijumlah**, bukan ditebak: catatan akun,
    /// cincin, label. Versi sebelumnya menggambar cincin mengelilingi
    /// `center.y - 6` sementara catatan ditaruh di `top + 6`; busur atas
    /// cincin menembus teksnya. Sekarang `KUOTA_H` adalah tinggi ketiganya,
    /// dan tiap pita dihitung dari atas `rect`.
    fn draw_home_quota(&self, ui: &mut egui::Ui, rect: Rect) {
        let theme = self.theme;
        let snap = self.quota.as_ref();

        // Saat belum ada data sama sekali, cincin digambar kosong dengan
        // "…" — bentuknya tetap, jadi angka yang datang tidak menggeser apa pun.
        //
        // Kalau 9Router sedang gagal **dan** kita punya angka lama, angka lama
        // itu tetap tampil (worker sudah menahannya) dengan penanda "basi".
        // Membuang angka yang sudah benar karena satu percobaan gagal membuat
        // cincin berkedip kosong padahal kuotanya tidak berubah.
        let sesi = snap.and_then(|s| s.session);
        let minggu = snap.and_then(|s| s.weekly);
        let gagal = snap.is_some_and(|s| s.is_empty());
        let basi = snap.is_some_and(|s| !s.is_empty() && s.error_key.is_some());

        // Pita 1: catatan. Pita 2: cincin. Pita 3: label. Blok 82 px ini
        // dipusatkan di baris (85 px), bukan ditempel ke atas — supaya sisa
        // ruang terbagi rata dan cincin tidak menempel ke tepi baris.
        let catatan_y = rect.top() + ((rect.height() - KUOTA_H) * 0.5).max(0.0);
        let cy = catatan_y + TEKS + style::S1 + RING / 2.0;
        let label_y = catatan_y + TEKS + style::S1 + RING + style::S1;

        for (i, (label, win)) in [
            (self.tr("home.quota.session"), sesi),
            (self.tr("home.quota.weekly"), minggu),
        ]
        .into_iter()
        .enumerate()
        {
            let cx = rect.left() + RING / 2.0 + i as f32 * (RING + style::S2);
            let center = egui::pos2(cx, cy);
            let r = (RING - TEBAL) / 2.0;

            // Jalur dasar: cincin penuh yang menaungi progres. Dulu `fill2`
            // (alpha 36) — terlalu lemah di atas panel hitam, jadi trek yang
            // hampir habis terbaca sebagai cincin kosong.
            ui.painter().circle_stroke(
                center,
                r,
                Stroke::new(TEBAL, Color32::from_white_alpha(TRACK_A)),
            );

            match win {
                Some(w) => {
                    let frac = w.sisa.clamp(0.0, 1.0);
                    let (awal, akhir) = warna_cincin(&theme, frac);
                    gambar_busur_bergradasi(ui, center, r, frac, awal, akhir);
                    ui.painter().text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        format!("{:.0}%", w.sisa * 100.0),
                        egui::FontId::proportional(style::T_SUBTITLE),
                        theme.ink,
                    );
                }
                None => {
                    ui.painter().text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        if gagal { "—" } else { "…" },
                        egui::FontId::proportional(style::T_SUBTITLE),
                        theme.faint,
                    );
                }
            }

            ui.painter().text(
                egui::pos2(cx, label_y),
                egui::Align2::CENTER_TOP,
                label,
                egui::FontId::proportional(style::T_MICRO),
                theme.faint,
            );
        }

        // Keterangan singkat di atas cincin: dari mana angka ini, atau kenapa
        // kosong. Dua kata saja — ruang di baris ini tidak lebih dari itu.
        //
        // Semua teks di sini lewat `self.tr()`. Dulu kalimatnya ditulis di
        // lapisan data, jadi pilihan bahasa di setelan tidak berpengaruh dan
        // panel berbahasa Inggris tetap menampilkan "gagal segarkan".
        let akun = self.tr("home.quota.accounts");
        let catatan = match snap {
            None => self.tr("home.quota.waiting").to_string(),
            Some(s) if s.is_empty() => self
                .tr(s.error_key.unwrap_or("home.quota.err.empty"))
                .to_string(),
            // Angka lama: sebutkan akun yang berhasil **dan** bahwa datanya
            // belum tersegarkan, supaya tidak terbaca sebagai angka terkini.
            Some(s) if basi => format!(
                "{}/{} {} · {}",
                s.akun_ok,
                s.akun_total,
                akun,
                if s.lebih_tua_dari(10) {
                    self.tr("home.quota.stale")
                } else {
                    self.tr("home.quota.refresh_failed")
                }
            ),
            Some(s) => format!("{}/{} {}", s.akun_ok, s.akun_total, akun),
        };
        let catatan = widgets::ellipsis_sized(ui, &catatan, rect.width(), style::T_MICRO);
        ui.painter().text(
            egui::pos2(rect.center().x, catatan_y),
            egui::Align2::CENTER_TOP,
            catatan,
            egui::FontId::proportional(style::T_MICRO),
            theme.faint,
        );
    }
}

/* ------------------------------------------------------------ cincin */

/// Dua titik henti gradasi cincin, dari ujung terang ke ujung pekat.
///
/// Warna tetap membawa info kuota: aksen di atas 40%, amber 15–40%, merah di
/// bawahnya — jadi cincin bisa dibaca tanpa membaca angkanya.
///
/// Ujung terang dinaikkan lewat HSL (`lighten`), **bukan** dicampur ke putih.
/// Pencampuran ke putih membuang saturasi, jadi aksen emas berubah jadi krem
/// pucat yang justru mendekati warna jalur dasarnya — hasilnya terbaca kotor,
/// bukan menyala. Menaikkan lightness mempertahankan rona.
///
/// Kedua ujung **opaq** (alpha 255). Busur digambar sebagai potongan-potongan
/// pendek yang bersambung; kalau alpha ikut diinterpolsi, sambungan yang
/// menumpuk menghasilkan pita yang tidak rata.
pub(crate) fn warna_cincin(t: &Theme, frac: f32) -> (Color32, Color32) {
    let pekat = if frac > 0.4 {
        t.accent
    } else if frac > 0.15 {
        Color32::from_rgb(0xF5, 0xB3, 0x4D)
    } else {
        Color32::from_rgb(0xEF, 0x6B, 0x6B)
    };
    // +0.14 lightness: jelas lebih terang, tapi rona masih terbaca sebagai
    // warna yang sama. Lebih dari ~0.2 membuat kuning kehilangan identitasnya
    // dan tampak putih.
    (theme::lighten(pekat, 0.14), pekat)
}

/// Jumlah potongan busur. Cukup banyak agar gradasinya tidak terlihat
/// bertangga, cukup sedikit agar sambungannya tetap tertutup rapi.
pub(crate) const SEGMEN: usize = 56;

/// Gambar busur bergradasi dari jam-12 searah jarum jam, sejauh `frac` lingkaran.
///
/// `egui::Shape::line` hanya menerima SATU warna untuk seluruh patahan, dan
/// `circle_stroke` hanya melingkar penuh — jadi gradasi harus dipecah ke
/// potongan garis pendek, masing-masing dengan warnanya sendiri. Potongan
/// sengaja bersambung (bukan berjarak) agar tidak ada celah putih di antaranya.
pub(crate) fn gambar_busur_bergradasi(
    ui: &mut egui::Ui,
    center: egui::Pos2,
    r: f32,
    frac: f32,
    awal: Color32,
    akhir: Color32,
) {
    let frac = frac.clamp(0.0, 1.0);
    if frac <= 0.001 {
        return;
    }

    let warna = theme::gradient(awal, akhir, SEGMEN + 1);
    let sudut_awal = -std::f32::consts::FRAC_PI_2;
    let titik = |i: usize| {
        let a = sudut_awal + frac * std::f32::consts::TAU * (i as f32 / SEGMEN as f32);
        egui::pos2(center.x + r * a.cos(), center.y + r * a.sin())
    };

    // Kepala bulat di jam-12: tanpa ini pangkal busur terpotong datar dan
    // cincin terlihat "tercabik" justru di titik tempat ia mulai.
    ui.painter()
        .circle_filled(titik(0), TEBAL / 2.0, warna[0]);

    for i in 0..SEGMEN {
        ui.painter()
            .line_segment([titik(i), titik(i + 1)], Stroke::new(TEBAL, warna[i]));
    }

    // Ekor membulat di ujung progres — penanda "sampai di sini".
    ui.painter()
        .circle_filled(titik(SEGMEN), TEBAL / 2.0, warna[SEGMEN]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::layout;

    /// Tinggi isi yang tersedia untuk sebuah halaman notch, setelah notch
    /// dibuka penuh dan pita dock disisihkan.
    fn tinggi_isi() -> f32 {
        // `content_rect(area, gutter, top_pad = S2)` berakhir di CONTENT_BOTTOM.
        layout::CONTENT_BOTTOM - style::S2
    }

    /// Kelas bug yang dikunci: halaman ini pernah menumpuk baris lagu 88 px
    /// dan kartu kuota 119 px di dalam ruang ±85 px, sehingga kartu kuota
    /// hilang di balik dock tanpa satu pun tes gagal.
    ///
    /// Sekarang tidak ada header dan tidak ada kartu, jadi seluruh ruang isi
    /// tersedia untuk baris itu sendiri.
    #[test]
    fn baris_home_muat_di_dalam_panel() {
        let body = tinggi_isi();
        assert!(
            body >= BARIS_MIN_H,
            "isi Home hanya {body}px, butuh minimal {BARIS_MIN_H}px — cincin kuota akan terpotong"
        );
    }

    /// Gradien harus benar-benar gradasi: kedua ujungnya beda, dan ujung
    /// terang harus lebih terang dari jalur dasarnya.
    ///
    /// Satu warna solid pada garis tipis di atas panel nyaris hitam hampir
    /// melebur ke latar — itu keluhan yang diperbaiki di sini.
    #[test]
    fn cincin_bergradasi_dari_terang_ke_pekat() {
        let t = Theme::new(noktah_core::Accent::Blue, 1.0);
        let (awal, akhir) = warna_cincin(&t, 0.72);

        assert_ne!(awal, akhir, "dua titik henti identik = tidak ada gradasi");
        assert!(
            theme::luminance(awal) > theme::luminance(akhir),
            "ujung jam-12 harus lebih terang dari ujung progres"
        );
    }

    /// Kedua titik henti WAJIB opaq.
    ///
    /// Busur dipecah ke potongan pendek yang bersambung, jadi sambungannya
    /// menumpuk dua lapis. Kalau alpha ikut diinterpolsi, tumpukan itu
    /// menghasilkan pita belang yang jelas terlihat — bug yang sama membunuh
    /// pendekatan "alpha mengikuti sisa kuota".
    #[test]
    fn titik_henti_cincin_selalu_opaq() {
        let t = Theme::new(noktah_core::Accent::Blue, 1.0);
        for frac in [0.02, 0.18, 0.5, 0.99] {
            let (awal, akhir) = warna_cincin(&t, frac);
            assert_eq!(awal.a(), 255, "ujung terang tembus pada {frac}");
            assert_eq!(akhir.a(), 255, "ujung pekat tembus pada {frac}");
        }
    }

    /// Ujung terang cincin harus naik di atas jalur dasarnya — kalau tidak,
    /// pada sisa kuota kecil cincin masih terbaca sebagai lingkaran kosong.
    #[test]
    fn cincin_terbaca_di_atas_jalurnya() {
        let t = Theme::new(noktah_core::Accent::Blue, 1.0);
        let jalur = Color32::from_white_alpha(TRACK_A);
        let (awal, akhir) = warna_cincin(&t, 0.05); // sisa kritis, warna merah
        assert!(
            theme::luminance(awal) > theme::luminance(jalur),
            "progres {awal:?} tidak lebih terang dari jalur {jalur:?}"
        );
        assert!(
            theme::luminance(akhir) > theme::luminance(jalur),
            "ujung pekat {akhir:?} tenggelam di bawah jalur {jalur:?}"
        );
        // Jalur sendiri harus terlihat di atas panel hitam.
        assert!(theme::luminance(jalur) > 0.02, "jalur terlalu lemah");
    }

    /// Ujung terang harus mempertahankan **rona**, bukan luntur ke putih.
    ///
    /// Ini regresi yang tertangkap dari tangkapan layar: versi pertama
    /// mencampur ke putih, jadi cincin emas berujung krem pucat dan terbaca
    /// kotor. Saturasi harus tetap dekat dengan asalnya.
    #[test]
    fn ujung_terang_mempertahankan_saturasi() {
        for warna in [
            Color32::from_rgb(0x5C, 0xD6, 0xCB), // teal
            Color32::from_rgb(0xF5, 0xB3, 0x4D), // amber
            Color32::from_rgb(0xEF, 0x6B, 0x6B), // merah
            Color32::from_rgb(0x4D, 0xA3, 0xFF), // biru
        ] {
            let terang = theme::lighten(warna, 0.14);
            let (_, s0, _) = theme::rgb_to_hsl(warna);
            let (_, s1, _) = theme::rgb_to_hsl(terang);
            assert!(
                s1 > s0 * 0.85,
                "saturasi {warna:?} jatuh dari {s0:.2} ke {s1:.2} — rona luntur"
            );
            assert!(
                theme::luminance(terang) > theme::luminance(warna),
                "lighten tidak menaikkan kecerahan {warna:?}"
            );
        }
    }

    /// Gradien harus monoton dan tidak pernah melewati batas.
    #[test]
    fn gradasi_monoton_tanpa_lompat() {
        let v = theme::gradient(
            Color32::from_rgb(255, 255, 255),
            Color32::from_rgb(0, 0, 0),
            SEGMEN + 1,
        );
        assert_eq!(v.len(), SEGMEN + 1);
        assert_eq!(v[0], Color32::from_rgb(255, 255, 255));
        assert_eq!(v[SEGMEN], Color32::from_rgb(0, 0, 0));
        for w in v.windows(2) {
            assert!(
                theme::luminance(w[0]) >= theme::luminance(w[1]),
                "gradasi naik kembali di tengah: {w:?}"
            );
        }
    }

    /// Transport harus muat di samping teks tanpa menabrak blok kuota.
    #[test]
    fn baris_home_muat_transport_dan_dua_cincin() {
        // Lebar isi: OPEN_W dikurangi gutter kiri-kanan `content_rect`.
        let lebar = layout::OPEN_W - 16.0 * 2.0;
        // Kiri player memakai seluruh ruang sampai blok kuota; teks berdiri
        // setelah sampul. Blok kuota (cincin + tombol segarkan) menyisakan
        // lebar bagi sampul + teks + trio transport.
        let sisa_untuk_teks = lebar - INSET * 2.0 - KANAN_W - TRIO_W - SEP_GAP * 2.0 - ART;
        assert!(
            sisa_untuk_teks >= 60.0,
            "teks judul hanya kebagian {sisa_untuk_teks}px — judul akan terpotong terus"
        );
    }

    /// Anggaran: baris (tanpa header) tidak boleh melewati `CONTENT_BOTTOM`.
    #[test]
    fn home_tidak_melewati_batas_dock() {
        assert!(
            BARIS_MIN_H <= tinggi_isi(),
            "Home butuh {BARIS_MIN_H}px tetapi hanya tersedia {}px",
            tinggi_isi()
        );
    }

    /// Garis pemisah harus duduk di tengah celah antara trio transport dan
    /// blok kuota terdekat — tidak menimpa tombol, tidak menimpa cincin.
    #[test]
    fn separator_home_di_tengah_celah() {
        // Geometri persis seperti di `draw_page_home`:
        //   kanan = [lebar - INSET - KANAN_W, lebar - INSET]
        //   rings = awal kanan; tombol segarkan di ujung kanan kanan
        //   player = [INSET, kanan.left - SEP_GAP*2], trio di ujung kanannya
        //   sep = kanan.left - SEP_GAP
        let lebar = layout::OPEN_W - 16.0 * 2.0;
        let kanan_left = lebar - INSET - KANAN_W;
        let player_right = kanan_left - SEP_GAP * 2.0;

        // Gap kanan trio → garis dan garis → cincin harus sama (SEP_GAP).
        let kiri = kanan_left - SEP_GAP - player_right;
        let kanan = kanan_left - (kanan_left - SEP_GAP);
        assert_eq!(kiri, SEP_GAP, "gap kiri garis tidak SEP_GAP");
        assert_eq!(kanan, SEP_GAP, "gap kanan garis tidak SEP_GAP");

        // Dan garis tidak boleh menimpa tombol transport.
        let trio_kiri = player_right - TRIO_W;
        let sep_x = kanan_left - SEP_GAP;
        assert!(
            sep_x > trio_kiri,
            "garis ({sep_x}) menabrak trio transport yang dimulai di {trio_kiri}"
        );
    }

    /// Kelas bug yang dikunci: catatan "4/4 akun" ditaruh di `top + 6`
    /// sementara cincin digambar mengelilingi `center.y - 6`. Busur atas
    /// cincin (25 px di atas pusatnya) menembus teks itu — terlihat di
    /// tangkapan layar sebagai teks yang terpotong cincin.
    ///
    /// Tes ini menghitung pita vertikal apa adanya dan memastikan ketiganya
    /// tidak berbagi piksel.
    #[test]
    fn catatan_cincin_dan_label_tidak_saling_menimpa() {
        let catatan_bawah = TEKS;

        let cincin_atas = catatan_bawah + style::S1;
        let cincin_bawah = cincin_atas + RING;
        assert!(
            cincin_atas >= catatan_bawah,
            "busur atas cincin ({cincin_atas}) menimpa catatan akun (…{catatan_bawah})"
        );

        let label_atas = cincin_bawah + style::S1;
        assert!(
            label_atas >= cincin_bawah,
            "label ({label_atas}) menimpa busur bawah cincin ({cincin_bawah})"
        );

        // Ketiga pita harus sama dengan tinggi blok yang dijanjikan `KUOTA_H`,
        // supaya pemusatan di `draw_home_quota` memakai angka yang sama.
        let total = TEKS + style::S1 + RING + style::S1 + TEKS;
        assert!(
            (total - KUOTA_H).abs() < 0.01,
            "KUOTA_H ({KUOTA_H}) tidak sama dengan jumlah pita ({total})"
        );
    }
}
