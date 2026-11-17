//! Lapisan infrastruktur: adaptor nyata untuk setiap port.
//!
//! Setiap modul di sini mengimplementasikan salah satu trait di
//! `crate::application::ports`. Ini satu-satunya tempat yang boleh menyentuh
//! disk, registry Windows, soket, atau API SMTC — supaya use case bisa diuji
//! dengan fake tanpa menyentuh apa pun dari itu.

pub mod ai;
pub mod fake;
pub mod audio;
pub mod blur;
pub mod glass;
pub mod ipc;
pub mod logger;
pub mod lyrics_client;
pub mod music;
pub mod os;
pub mod platform;
pub mod quota;
pub mod secret;
pub mod store;
pub mod sys;
pub mod updater;
pub mod weather;
