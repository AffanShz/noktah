//! Satu halaman notch, satu berkas.
//!
//! Setiap modul membuka `impl Noktah` dan menambahkan satu method gambar.
//! Pemisahan ini murni organisasi — tidak ada state baru, tidak ada perilaku
//! yang berubah; `git diff` pada tiap berkas bisa ditelusuri ke `tabs.rs`.

pub mod agent;
pub mod ask;
pub mod focus;
pub mod home;
pub mod music;
pub mod notch;
pub mod now_playing;
pub mod seek;
pub mod settings;
pub mod system;
pub mod tasks;
pub mod water;
