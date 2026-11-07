//! Isi halaman notch.
//!
//! Berkas ini dulunya 2025 baris berisi sepuluh method. Isinya sekarang
//! pecah ke `presentation/pages/` — satu halaman, satu berkas. Yang tersisa
//! di sini hanya re-export, supaya pemanggil lama tidak perlu diubah pada
//! langkah yang sama dengan pemecahan ini.

pub use crate::presentation::pages::*;
