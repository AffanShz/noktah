//! Lapisan aplikasi: use case dan kontrak.
//!
//! Aturan tingkat aplikasi hidup di sini — bukan di UI, bukan di adapter.
//! Setiap modul di bawah adalah satu use case yang bisa diuji tanpa jendela,
//! disk, atau jaringan, karena semua efek sampingnya lewat trait di [`ports`].

pub mod ports;
pub mod tasks;

pub use ports::{files, Clock, Deps, Network, SecretStore, Store, SystemActions, SystemInfo};
