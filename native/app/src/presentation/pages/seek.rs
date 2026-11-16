//! Permintaan lompat posisi pemutar.
//!
//! Dipindah dari `tabs.rs` tanpa perubahan perilaku. Satu halaman, satu berkas.

use crate::app::Noktah;

impl Noktah {
    /// Placeholder sampai lapisan SMTC tersedia.
    pub fn request_seek(&mut self, target_ms: i64) {
    self.media_cmd(crate::infrastructure::music::Cmd::Seek(target_ms));
    }
}
