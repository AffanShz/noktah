//! Lapisan presentasi: cara menggambar, bukan apa yang digambar.
//!
//! Isinya pengetahuan tentang piksel — token skala, warna, ikon, tipografi,
//! dan komponen. Tidak ada satu pun aturan bisnis di sini: kalau sebuah
//! keputusan bisa diuji tanpa membuka jendela, tempatnya di `noktah_core`.
//!
//! Aturan ketergantungan: lapisan ini boleh tahu tentang `application` dan
//! `domain`; tidak sebaliknya.

pub mod fonts;
pub mod icons;
pub mod layout;
pub mod markdown;
pub mod mascot;
pub mod mini;
pub mod pages;
pub mod shell;
pub mod spring;
pub mod style;
pub mod theme;
pub mod widgets;
