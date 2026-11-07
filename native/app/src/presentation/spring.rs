//! Simulasi spring teredam untuk animasi antarmuka cair (Apple Dynamic Island).
//!
//! Menggunakan representasi `duration + bounce` ala SwiftUI dan motion design Apple:
//! - `duration`: satu periode osilasi penuh dalam detik
//! - `bounce`: tingkat pantulan (0.0 = kritis teredam, 0.22 = pantulan halus natural)
//!
//! Integrasi dilakukan dengan Euler semi-implisit pada langkah tetap 1/240 s
//! agar stabil dan identik di layar 60, 120, maupun 144 Hz.

use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct SpringSpec {
    pub duration: f32,
    pub bounce: f32,
}

impl SpringSpec {
    pub const fn new(duration: f32, bounce: f32) -> Self {
        Self { duration, bounce }
    }

    /// `ω₀ = 2π / duration` dan `ζ = 1 − bounce`, lalu diterjemahkan ke bentuk
    /// yang dipakai solver: `k = ω₀²`, `c = 2ζω₀`.
    pub fn stiffness(self) -> f32 {
        let w = std::f32::consts::TAU / self.duration.max(0.05);
        w * w
    }

    pub fn damping(self) -> f32 {
        let w = std::f32::consts::TAU / self.duration.max(0.05);
        let zeta = (1.0 - self.bounce).clamp(0.02, 1.0);
        2.0 * zeta * w
    }
}

/// Morph panel. Lebar bergerak lebih dulu, tinggi menyusul sekitar 90 ms
/// kemudian — itu yang membuat bentuknya terasa melar seperti Dynamic Island,
/// bukan kotak yang membesar serentak. Saat menutup urutannya dibalik:
/// tingginya jatuh lebih dulu supaya terasa "amblas", bukan mengecil pelan.
pub const MORPH_OPEN_W: SpringSpec = SpringSpec::new(0.34, 0.22);
pub const MORPH_OPEN_H: SpringSpec = SpringSpec::new(0.42, 0.18);
pub const MORPH_CLOSE_W: SpringSpec = SpringSpec::new(0.26, 0.10);
pub const MORPH_CLOSE_H: SpringSpec = SpringSpec::new(0.22, 0.06);

/// Lebar baris minimized saat isinya berubah (lagu mulai atau berhenti).
pub const MORPH_PILL: SpringSpec = SpringSpec::new(0.34, 0.22);

/// Menyusut ke garis tipis saat lagu berganti. Cepat dan hampir tanpa
/// pantulan — ini gerakan "menyingkir", bukan gerakan yang perlu dinikmati.
pub const MORPH_MINIMAL_IN: SpringSpec = SpringSpec::new(0.26, 0.06);

/// Membesar kembali ke baris minimized setelah lirik lagu baru siap.
pub const MORPH_MINIMAL_OUT: SpringSpec = SpringSpec::new(0.38, 0.24);

/// Lama garis tipis ditahan paling sedikit. Tanpa ini, lagu dengan lirik
/// cache langsung membuat shrink-expand terlihat seperti kedip.
pub const TRANSISI_HOLD: Duration = Duration::from_millis(260);

/// Dock membuka dari strip menjadi deretan ikon. Cukup cepat agar terasa
/// responsif saat kursor mendekat, dengan pantulan kecil supaya ikonnya
/// terlihat "muncul" alih-alih tumbuh lurus.
pub const MORPH_DOCK: SpringSpec = SpringSpec::new(0.30, 0.18);

/// Buka-tutup kolom lirik di halaman Musik: durasi 0.30s dengan pantulan sangat halus (0.12)
/// agar transisi dua kolom terasa organik ala Dynamic Island.
pub const MORPH_LYRICS: SpringSpec = SpringSpec::new(0.30, 0.12);

/// Posisi baris aktif lirik (indeks baris, bukan fraksi) saat baris berganti.
///
/// 0.34 s dipertahankan supaya rasa waktunya sama dengan transisi 350 ms yang
/// lama; yang berubah hanya besaran yang digerakkan. Spring membawa kecepatan,
/// jadi pergantian baris cepat beruntun melanjutkan gerak alih-alih restart
/// dari nol — itu yang membuatnya terasa satu tarikan napas, bukan lompatan.
pub const MORPH_LYRIC_LINE: SpringSpec = SpringSpec::new(0.34, 0.10);

/// Sneak peek: saat lagu berganti di luar panel, pill melebar sesaat
/// menampilkan judul + artis baru, lalu menyusut kembali. Pola ini yang
/// paling sering dipuji di aplikasi notch lain (Boring Notch, OpenNook).
pub const SNEAK_PEEK: Duration = Duration::from_millis(2200);

/// Batas menunggu lirik sebelum tetap membuka kembali tanpa lirik.
/// LRCLIB yang lambat pernah membuat garis tipis bertahan 1,15 detik — terlalu
/// lama untuk sesuatu yang seharusnya terasa seperti satu tarikan napas.
pub const TRANSISI_TIMEOUT: Duration = Duration::from_millis(900);

/// Spring teredam, setara `duration + bounce` milik Apple.
///
/// Dipakai alih-alih easing berbasis waktu karena spring menyimpan kecepatan:
/// kalau kursor masuk lalu keluar sebelum animasi selesai, gerakan berbalik
/// dari kecepatan saat itu — bukan melompat ke awal.
#[derive(Clone, Copy, Debug)]
pub struct Spring {
    pub pos: f32,
    vel: f32,
    pub target: f32,
    stiffness: f32,
    damping: f32,
    /// Sisa waktu yang belum diintegrasikan. Gerak dihitung pada langkah tetap
    /// 1/240 s, jadi hasilnya identik di 60, 120, maupun 144 Hz.
    leftover: f32,
}

impl Spring {
    pub fn with(value: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            pos: value,
            vel: 0.0,
            target: value,
            stiffness,
            damping,
            leftover: 0.0,
        }
    }

    pub fn from_spec(value: f32, spec: SpringSpec) -> Self {
        Self::with(value, spec.stiffness(), spec.damping())
    }

    /// Mengganti parameter tanpa menyentuh kecepatan: gerak yang sedang
    /// berjalan tetap membawa momentumnya saat arah berubah di tengah jalan.
    pub fn set_spec(&mut self, spec: SpringSpec) {
        self.stiffness = spec.stiffness();
        self.damping = spec.damping();
    }

    /// Euler semi-implisit pada langkah tetap 1/240 s.
    ///
    /// Langkah tetap membuat osilasi tidak pernah meledak saat frame drop, dan
    /// membuat geraknya bisa direproduksi: 4 sub-langkah per frame di 60 Hz,
    /// 2 di 120 Hz, dengan sisa waktu yang dibawa ke frame berikutnya.
    pub fn step(&mut self, dt: f32) {
        const FIXED: f32 = 1.0 / 240.0;
        const MAX_STEPS: u32 = 24;

        self.leftover = (self.leftover + dt.clamp(0.0, 0.05)).min(FIXED * MAX_STEPS as f32);

        let mut n = 0;
        while self.leftover >= FIXED && n < MAX_STEPS {
            let accel = -self.stiffness * (self.pos - self.target) - self.damping * self.vel;
            self.vel += accel * FIXED;
            self.pos += self.vel * FIXED;
            self.leftover -= FIXED;
            n += 1;
        }

        // Snap: tanpa ini sisa 0,3 px membuat panel terus meminta frame baru
        // dan animasinya tidak pernah benar-benar berhenti.
        if self.settled() {
            self.pos = self.target;
            self.vel = 0.0;
            self.leftover = 0.0;
        }
    }

    /// Langsung ke tujuan tanpa animasi. Kecepatan ikut dinolkan: kalau tidak,
    /// `settled()` tetap false dan UI meminta frame baru selamanya.
    pub fn snap(&mut self) {
        self.pos = self.target;
        self.vel = 0.0;
        self.leftover = 0.0;
    }

    /// Ambang diam menyesuaikan jarak tempuh.
    ///
    /// Ambang tetap 0,0008 masuk akal untuk nilai 0..1 (morph panel), tapi
    /// untuk pill yang bergerak 193 → 489 px artinya "berhenti dalam
    /// seperseribu piksel" — simulasinya butuh 925 ms, dua kali lipat dari
    /// yang dirancang. Ambang relatif membuat keduanya selesai serentak.
    pub fn settled(&self) -> bool {
        let ambang_pos = (self.target.abs() * 0.0015).clamp(0.0008, 0.4);
        (self.pos - self.target).abs() < ambang_pos && self.vel.abs() < ambang_pos * 10.0
    }
}
