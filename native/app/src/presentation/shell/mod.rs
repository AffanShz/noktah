//! Submodul kerangka luar (shell) Noktah.
//!
//! Bertanggung jawab atas:
//! - Siklus frame egui dan sinkronisasi jendela (`frame`)
//! - Penggambaran shell notch, kontainer panel, dan jendela setelan (`render`)
//! - Polling hasil worker SMTC, AI, LRCLIB, cuaca (`sync`)
//! - Persistensi data setelan dan pengingat (`persistence`)

pub mod frame;
pub mod persistence;
pub mod render;
pub mod sync;
