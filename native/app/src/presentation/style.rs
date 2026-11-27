//! Skala desain: satu sumber untuk spasi, ukuran huruf, dan radius.
//!
//! Sebelum modul ini ada, `tabs.rs` memakai 15 ukuran huruf, 8 nilai spasi,
//! dan 9 radius berbeda tanpa pola. Itu pelanggaran prinsip Apple "nothing is
//! random — every spacing, timing, and alignment value is a deliberate choice
//! you can defend". Angka yang berbeda 0,5 px tidak terlihat sebagai keputusan;
//! yang terlihat hanya hasilnya: tidak rata.
//!
//! Aturan yang dipakai:
//! - **Spasi kelipatan 4.** Empat langkah saja. Kalau butuh nilai lain,
//!   kemungkinan hierarkinya belum jelas — bukan skalanya yang kurang.
//! - **Empat ukuran huruf.** Display, judul, badan, keterangan. Hierarki
//!   dibangun dari bobot dan warna, bukan dari menambah ukuran baru.
//! - **Radius konsentris.** Sudut dalam = sudut luar − padding. Ini yang
//!   membuat kartu bersarang terlihat "dipahat", bukan ditempel.
//!
//! Catatan jujur: egui tidak punya letter-spacing, jadi aturan tracking
//! Apple (rapatkan huruf besar, renggangkan huruf kecil) belum bisa diterapkan.
//! Yang bisa dilakukan — memakai bobot dan warna untuk hierarki — sudah.

use eframe::egui::{self, Color32, FontId};

/* --------------------------------------------------------------- spasi */

/// Jarak antarelemen dalam satu kelompok (label dengan nilainya).
pub const S1: f32 = 4.0;
/// Jarak antar-baris teks yang berkerabat.
pub const S2: f32 = 8.0;
/// Jarak antar-kelompok di dalam satu bagian.
pub const S3: f32 = 12.0;
/// Jarak antar-bagian besar. Juga padding tepi panel.
pub const S4: f32 = 24.0;

/// Padding tepi panel. Satu angka untuk kiri dan kanan — dulu kiri 24 dan
/// kanan 18, dan itu yang membuat isi panel terlihat miring.
#[allow(dead_code)]
pub const PAD: f32 = S4;

/// Padding di dalam kartu. Radius dalam = radius luar − nilai ini.
#[allow(dead_code)]
pub const CARD_PAD: f32 = S3;

/* --------------------------------------------------------------- radius */

// Radius mengikuti prototipe `noktah-redesign.html`:
//   `.tile` 20 · `.field` 14 · `.art` 16 · `.tab` 17 · `.tabs` 22 ·
//   `.pill` 18 · `.sw` 14 · `.group` 14.
/// Kartu permukaan besar (`.tile` 20 px).
pub const R_CARD: f32 = 20.0;
/// Grup dan kartu setelan (`.group` 14 px).
pub const R_GROUP: f32 = 14.0;
/// Sampul album (`.art` 16 px).
#[allow(dead_code)]
pub const R_ART: f32 = 16.0;
/// Kontrol: chip, tab, kotak input (`.field` 14 px).
pub const R_CONTROL: f32 = 14.0;
/// Elemen kecil: penanda, badge (`.chip` 8 px).
pub const R_SMALL: f32 = 8.0;
/// Pil dan saklar: sepenuhnya membulat.
pub const R_PILL: f32 = 999.0;

/// Radius dalam untuk elemen yang bersarang di dalam `outer` dengan padding
/// `pad`. Menjaga lengkung tetap sepusat, seperti sudut bertingkat di iOS.
#[allow(dead_code)]
pub fn concentric(outer: f32, pad: f32) -> f32 {
    (outer - pad).max(0.0)
}

/* ----------------------------------------------------------------- tipe */

/// Angka raksasa: suhu cuaca (`.temp` 36 px di prototipe kompak).
#[allow(dead_code)]
pub const T_DISPLAY_LG: f32 = 36.0;
/// Jam besar dan angka utama panel.
#[allow(dead_code)]
pub const T_DISPLAY: f32 = 26.0;
/// Angka metrik kartu (`.tv` 24 px).
pub const T_STAT: f32 = 24.0;
/// Judul lagu dan kepala kartu (`.title` 15 px).
pub const T_TITLE_LG: f32 = 15.0;
/// Judul bagian dan angka penting.
pub const T_TITLE: f32 = 13.5;
/// Subjudul: nama artis, keterangan (`.artist` 12 px).
pub const T_SUBTITLE: f32 = 12.0;
/// Teks badan: label setelan, isi percakapan, judul tugas.
pub const T_BODY: f32 = 12.5;
/// Keterangan: nama kota, satuan, status sekunder.
pub const T_CAPTION: f32 = 11.0;
/// Teks mikro: badge, footer. Hanya untuk satu-dua kata.
pub const T_MICRO: f32 = 10.0;

/// Huruf badan.
pub fn body() -> FontId {
    FontId::proportional(T_BODY)
}

/// Huruf keterangan.
pub fn caption() -> FontId {
    FontId::proportional(T_CAPTION)
}

/// Huruf mikro untuk metadata kecil (timer, label status).
#[allow(dead_code)]
pub fn micro() -> FontId {
    FontId::proportional(T_MICRO)
}

/// Teks badan dengan warna tertentu.
pub fn text(s: impl Into<String>, color: Color32) -> egui::RichText {
    egui::RichText::new(s).size(T_BODY).color(color)
}

/// Keterangan dengan warna tertentu.
pub fn cap(s: impl Into<String>, color: Color32) -> egui::RichText {
    egui::RichText::new(s).size(T_CAPTION).color(color)
}

/// Mikro teks dengan warna tertentu.
#[allow(dead_code)]
pub fn micro_text(s: impl Into<String>, color: Color32) -> egui::RichText {
    egui::RichText::new(s).size(T_MICRO).color(color)
}

/// Judul lagu / judul kartu: 17 px semi-tebal dengan tracking rapat.
///
/// Prototipe memakai `.title{font-size:17px;font-weight:600;letter-spacing:-.02em}`.
/// egui tidak punya letter-spacing, jadi hierarkinya dibawa oleh ukuran dan
/// bobot — itu bagian yang masih bisa diterapkan.
pub fn title(s: impl Into<String>, color: Color32) -> egui::RichText {
    egui::RichText::new(s).size(T_TITLE_LG).strong().color(color)
}

/// Subjudul 13 px: nama artis, keterangan di bawah judul.
pub fn subtitle(s: impl Into<String>, color: Color32) -> egui::RichText {
    egui::RichText::new(s).size(T_SUBTITLE).color(color)
}

/// Angka tabular untuk label waktu. Monospace supaya digit tidak bergeser
/// lebar saat detik berubah (prototipe: `font-variant-numeric:tabular-nums`).
pub fn micro_mono(s: impl Into<String>, color: Color32) -> egui::RichText {
    egui::RichText::new(s).size(T_MICRO).monospace().color(color)
}

/// Angka besar untuk metrik dan hitungan: 28 px, tebal, tabular.
#[allow(dead_code)]
pub fn stat_value(s: impl Into<String>, color: Color32) -> egui::RichText {
    egui::RichText::new(s).size(T_STAT).strong().color(color)
}

/// Angka raksasa untuk suhu cuaca (`.temp` 42 px).
#[allow(dead_code)]
pub fn display_value(s: impl Into<String>, color: Color32) -> egui::RichText {
    egui::RichText::new(s).size(T_DISPLAY_LG).strong().color(color)
}

/// Label kolom kartu (`.tl` 12 px, label2).
#[allow(dead_code)]
pub fn tile_label(s: impl Into<String>, color: Color32) -> egui::RichText {
    egui::RichText::new(s).size(T_CAPTION).color(color)
}

/// Judul bagian: tebal, sedikit lebih besar dari badan.
///
/// Dulu memakai HURUF KAPITAL SEMUA dengan ukuran 9,5. Kapital semua
/// memperlambat membaca dan membuat setiap judul terlihat sama pentingnya
/// dengan isinya. Sekarang: huruf biasa, lebih tebal, warna redup.
#[allow(dead_code)]
pub fn heading(s: impl Into<String>, color: Color32) -> egui::RichText {
    egui::RichText::new(s).size(T_BODY).strong().color(color)
}

/// Title untuk header halaman notch — huruf biasa, tidak kapital, lebih
/// ringan dibanding heading bagian setelan agar tidak bersaing dengan judul.
#[allow(dead_code)]
pub fn page_title(s: impl Into<String>, color: Color32) -> egui::RichText {
    egui::RichText::new(s).size(T_TITLE).strong().color(color)
}

/// Headline untuk kartu dan surface utama — dipakai di halaman Water, Focus,
/// System sebagai "satu angka yang jadi alasan halaman ini ada".
#[allow(dead_code)]
pub fn card_headline(s: impl Into<String>, color: Color32) -> egui::RichText {
    egui::RichText::new(s).size(T_DISPLAY).strong().color(color)
}

/* --------------------------------------------------------------- helpers */

/// Align Y ke baseline rata tengah (slot_height - item_height) / 2.0
#[allow(dead_code)]
pub fn center_baseline(slot_height: f32, item_height: f32) -> f32 {
    (slot_height - item_height) * 0.5
}
