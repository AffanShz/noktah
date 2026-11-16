//! Kuota & pemakaian token Antigravity dari 9Router.
//!
//! Ada dua angka yang dibutuhkan halaman Beranda, dan keduanya hidup di
//! tempat yang berbeda — itu bukan pilihan kita, itu bentuk data 9Router:
//!
//! 1. **Sisa kuota** (`gemini_session`, `gemini_weekly`, `claude_gpt_session`,
//!    `claude_gpt_weekly`) hanya ada di respons HTTP
//!    `GET /api/usage/{connectionId}`. 9Router tidak menyimpannya ke SQLite,
//!    jadi tidak ada cara membacanya saat dashboard/server mati.
//! 2. **Token terpakai** ada di `usageHistory` di dalam
//!    `%APPDATA%\9router\db\data.sqlite`.
//!
//! Yang ditampilkan halaman Beranda adalah (1): sisa kuota dari akun
//! Antigravity yang benar-benar dipakai, untuk jendela 5 jam dan mingguan.
//!
//! ## Token CLI
//!
//! Middleware 9Router menerima header `x-9r-cli-token` yang diturunkan dari
//! tiga berkas di folder datanya:
//!
//! ```text
//! sha256(machine-id + "9r-cli-auth" + auth/cli-secret)[..16]
//! ```
//!
//! Rahasia itu dibaca dari disk setiap kali, tidak pernah ditulis ke setelan
//! Noktah dan tidak pernah masuk log.

use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

/// Satu jendela kuota. `sisa` 0..1 (1 = penuh).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Window {
    pub sisa: f32,
    /// Kapan jendela ini pulih, epoch ms. 0 = tidak diketahui.
    pub reset_ms: i64,
}

/// Sisa kuota Antigravity untuk akun yang benar-benar dipakai.
///
/// Satu angka per jendela — dan angka itu **selalu salah satu angka yang
/// tampil di dashboard 9Router**, tidak pernah hasil hitungan sendiri.
///
/// Versi sebelumnya merata-ratakan dua bucket (`gemini_weekly` dan
/// `claude_gpt_weekly`) menjadi satu persen. Itu mengarang angka yang tidak
/// ada di mana pun: pada satu akun dengan Gemini 67% dan Claude 100%, cincin
/// menampilkan 84% — angka yang tidak bisa dicocokkan pengguna dengan bar
/// mana pun di 9Router. Yang dipakai sekarang bucket **terendah**, karena
/// itulah yang menentukan kapan kuota benar-benar habis.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QuotaSnapshot {
    /// Sisa 5 jam: bucket terendah dari `gemini_session` / `claude_gpt_session`.
    pub session: Option<Window>,
    /// Sisa mingguan: bucket terendah dari `gemini_weekly` / `claude_gpt_weekly`.
    pub weekly: Option<Window>,
    /// Berapa akun yang berhasil dibaca.
    pub akun_ok: usize,
    /// Berapa akun aktif yang dicoba.
    pub akun_total: usize,
    /// Sebab kegagalan, sebagai **kunci terjemahan** — bukan kalimat jadi.
    ///
    /// Lapisan data tidak tahu bahasa antarmuka, dan tidak seharusnya tahu.
    /// Dulu ia mengirim kalimat Indonesia yang lalu tampil di antarmuka
    /// berbahasa Inggris. Sekarang halaman Home yang menerjemahkan.
    pub error_key: Option<&'static str>,
    /// Kapan pembacaan ini selesai, epoch ms. Dipakai UI untuk membedakan
    /// angka baru dari angka lama yang ditahan saat 9Router sedang gagal.
    pub at_ms: i64,
}

impl QuotaSnapshot {
    /// Tidak ada satu pun bucket yang terbaca.
    pub fn is_empty(&self) -> bool {
        self.session.is_none() && self.weekly.is_none()
    }

    /// Sudah lebih tua dari `menit` menit — angka lama tidak boleh dipakai
    /// selamanya tanpa penanda.
    pub fn lebih_tua_dari(&self, menit: i64) -> bool {
        let now = chrono::Local::now().timestamp_millis();
        now.saturating_sub(self.at_ms) > menit * 60_000
    }
}

/* ------------------------------------------------------------ folder data */

/// Folder data 9Router. Mengikuti konvensi yang dipakai CLI-nya sendiri:
/// `%APPDATA%\9router` di Windows.
pub fn data_dir() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    let dir = PathBuf::from(appdata).join("9router");
    if dir.is_dir() {
        Some(dir)
    } else {
        None
    }
}

/// Apakah 9Router terpasang di mesin ini sama sekali.
pub fn terdeteksi() -> bool {
    data_dir().map(|d| d.join("machine-id").is_file()).unwrap_or(false)
}

/* ------------------------------------------------------------ token CLI */

/// SHA-256 minimal — cukup untuk token CLI 9Router, tanpa menambah dependensi
/// kriptografi baru ke aplikasi 8 MB.
///
/// Implementasi langsung dari FIPS 180-4. Hanya dipakai untuk menurunkan token
/// lokal dari berkas rahasia yang sudah ada di disk, bukan untuk menyimpan
/// sandi.
mod sha256 {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    const H0: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    /// Hash `data` dan kembalikan 32 byte mentah.
    pub fn digest(data: &[u8]) -> [u8; 32] {
        let mut h = H0;

        // Padding: 0x80, lalu nol, lalu panjang 64-bit big-endian.
        let bit_len = (data.len() as u64).wrapping_mul(8);
        let mut msg = Vec::with_capacity(data.len() + 72);
        msg.extend_from_slice(data);
        msg.push(0x80);
        while msg.len() % 64 != 56 {
            msg.push(0);
        }
        msg.extend_from_slice(&bit_len.to_be_bytes());

        for block in msg.chunks_exact(64) {
            let mut w = [0u32; 64];
            for (i, word) in w.iter_mut().take(16).enumerate() {
                let b = &block[i * 4..i * 4 + 4];
                *word = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
            }
            for i in 16..64 {
                let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
                let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
                w[i] = w[i - 16]
                    .wrapping_add(s0)
                    .wrapping_add(w[i - 7])
                    .wrapping_add(s1);
            }

            let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
                (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);

            for i in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ ((!e) & g);
                let t1 = hh
                    .wrapping_add(s1)
                    .wrapping_add(ch)
                    .wrapping_add(K[i])
                    .wrapping_add(w[i]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let t2 = s0.wrapping_add(maj);

                hh = g;
                g = f;
                f = e;
                e = d.wrapping_add(t1);
                d = c;
                c = b;
                b = a;
                a = t1.wrapping_add(t2);
            }

            for (s, v) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
                *s = s.wrapping_add(v);
            }
        }

        let mut out = [0u8; 32];
        for (i, v) in h.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&v.to_be_bytes());
        }
        out
    }

    pub fn hex(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            s.push(HEX[(b >> 4) as usize] as char);
            s.push(HEX[(b & 0x0f) as usize] as char);
        }
        s
    }
}

/// Token CLI 9Router, persis seperti yang diturunkan middleware-nya.
pub fn cli_token(dir: &std::path::Path) -> Option<String> {
    let mid = std::fs::read_to_string(dir.join("machine-id")).ok()?;
    let sec = std::fs::read_to_string(dir.join("auth").join("cli-secret")).ok()?;
    let mid = mid.trim();
    let sec = sec.trim();
    if mid.is_empty() || sec.is_empty() {
        return None;
    }
    // Middleware memakai createHash("sha256").update(machineId + salt + secret).
    let hash = sha256::digest(format!("{mid}9r-cli-auth{sec}").as_bytes());
    Some(sha256::hex(&hash)[..16].to_string())
}

/* ------------------------------------------------------------ akun aktif */

/// Akun Antigravity yang aktif, dibaca langsung dari SQLite 9Router.
///
/// Kolom `data` sengaja tidak dibaca sama sekali: di dalamnya ada access token
/// dan refresh token. Yang dibutuhkan hanya `id`, `provider`, dan `isActive`.
///
/// Urutannya mengikuti `priority` — urutan yang sama dipakai 9Router untuk
/// memilih akun saat merutekan permintaan, jadi akun pertama di sini adalah
/// akun yang paling mungkin benar-benar dipakai.
fn akun_aktif(dir: &std::path::Path) -> Result<Vec<String>, &'static str> {
    let db = dir.join("db").join("data.sqlite");
    let conn = rusqlite::Connection::open_with_flags(
        &db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| "home.quota.err.db")?;

    let mut stmt = conn
        .prepare(
            "SELECT id FROM providerConnections \
             WHERE provider = 'antigravity' AND isActive = 1 \
             ORDER BY priority",
        )
        .map_err(|_| "home.quota.err.db")?;

    let ids = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|_| "home.quota.err.db")?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    Ok(ids)
}

/* ------------------------------------------------------------- respons */

#[derive(Deserialize)]
struct QuotaEntry {
    #[serde(default)]
    used: f64,
    #[serde(default)]
    total: f64,
    #[serde(default)]
    #[serde(rename = "remainingPercentage")]
    remaining_percentage: Option<f64>,
    #[serde(default)]
    #[serde(rename = "resetAt")]
    reset_at: Option<String>,
}

#[derive(Deserialize)]
struct UsageResponse {
    #[serde(default)]
    quotas: std::collections::HashMap<String, QuotaEntry>,
    #[serde(default)]
    message: Option<String>,
}

fn reset_to_ms(s: Option<&String>) -> i64 {
    s.and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
        .map(|d| d.timestamp_millis())
        .unwrap_or(0)
}

impl QuotaEntry {
    /// Sisa sebagai 0..1. `remainingPercentage` diutamakan karena 9Router
    /// sudah menghitungnya; kalau tidak ada, turunkan dari used/total.
    fn sisa(&self) -> Option<f32> {
        if let Some(pct) = self.remaining_percentage {
            if pct.is_finite() {
                return Some((pct / 100.0).clamp(0.0, 1.0) as f32);
            }
        }
        if self.total > 0.0 {
            let used = self.used.clamp(0.0, self.total);
            return Some(((self.total - used) / self.total) as f32);
        }
        None
    }
}

/// Ambil kuota dari API lokal 9Router untuk **satu** akun.
///
/// Mengembalikan bucket 5-jam dan mingguan yang **terendah**, dengan syarat:
/// bucket yang persis `0%` diabaikan kalau masih ada bucket lain yang belum
/// reset. Ini meniru logika dashboard 9Router itu sendiri (berkas
/// `dashboard/quota/page.js`): di sana bucket yang habis (`remainingPercentage
/// === 0`) tidak ditampilkan selama `resetAt`-nya belum berubah, karena "habis
/// di 0%" bisa berarti dua hal yang sangat berbeda — kuota benar-benar kosong
/// sekarang, atau kuota baru saja reset dan belum tercatat. Mengabaikan bucket
/// yang tepat 0 dengan `resetAt` lama membuat angka tidak mengerikan setiap
/// kali sebuah bucket reset.
#[derive(Clone, Debug, Default, PartialEq)]
struct SatuAkun {
    sesi: Option<Window>,
    minggu: Option<Window>,
}

fn pilih_terendah(kumpulan: Vec<Window>) -> Option<Window> {
    if kumpulan.is_empty() {
        return None;
    }
    // Reset lama = yang bisa dianggap "benar-benar habis" (bukan baru reset).
    let reset_tertua = kumpulan
        .iter()
        .filter(|w| w.reset_ms > 0)
        .map(|w| w.reset_ms)
        .min()
        .unwrap_or(0);
    let reset_ubah = |w: &Window| w.reset_ms == reset_tertua;

    kumpulan
        .iter()
        .filter(|w| !(w.sisa <= 0.001 && reset_ubah(w)))
        .min_by(|a, b| a.sisa.total_cmp(&b.sisa))
        .cloned()
}

fn akun_dari_respons(resp: &UsageResponse) -> SatuAkun {
    let mut sesi: Vec<Window> = Vec::new();
    let mut minggu: Vec<Window> = Vec::new();

    // Kunci bucket yang dipakai 9Router untuk ringkasan Antigravity:
    //   gemini_session / gemini_weekly       → model Gemini
    //   claude_gpt_session / claude_gpt_weekly → model Claude & GPT
    for key in ["gemini_session", "claude_gpt_session"] {
        if let Some(e) = resp.quotas.get(key) {
            if let Some(sisa) = e.sisa() {
                sesi.push(Window {
                    sisa,
                    reset_ms: reset_to_ms(e.reset_at.as_ref()),
                });
            }
        }
    }
    for key in ["gemini_weekly", "claude_gpt_weekly"] {
        if let Some(e) = resp.quotas.get(key) {
            if let Some(sisa) = e.sisa() {
                minggu.push(Window {
                    sisa,
                    reset_ms: reset_to_ms(e.reset_at.as_ref()),
                });
            }
        }
    }

    SatuAkun {
        sesi: pilih_terendah(sesi),
        minggu: pilih_terendah(minggu),
    }
}

/// Ambil satu connection dari API lokal 9Router.
fn fetch_connection(
    agent: &ureq::Agent,
    base: &str,
    token: &str,
    id: &str,
) -> Result<UsageResponse, &'static str> {
    let url = format!("{base}/api/usage/{id}");
    agent
        .get(&url)
        .set("x-9r-cli-token", token)
        .call()
        .map_err(|_| "home.quota.err.server")?
        .into_json::<UsageResponse>()
        .map_err(|_| "home.quota.err.server")
}

/// Baca kuota Antigravity dari 9Router yang berjalan di mesin ini.
///
/// Mengembalikan `QuotaSnapshot` walau sebagian akun gagal — satu akun
/// bermasalah tidak boleh mengosongkan seluruh kartu.
pub fn fetch() -> QuotaSnapshot {
    fetch_partial()
}

/* ----------------------------------------------------- file status lokal */

#[derive(Deserialize)]
struct LocalQuotaWindow {
    remaining_pct: f64,
    #[serde(default)]
    reset_ms: Option<i64>,
    #[serde(default)]
    reset_at: Option<String>,
}

impl LocalQuotaWindow {
    fn to_window(&self) -> Option<Window> {
        if !self.remaining_pct.is_finite() {
            return None;
        }
        let sisa = (self.remaining_pct / 100.0).clamp(0.0, 1.0) as f32;
        let reset_ms = self
            .reset_ms
            .filter(|&ms| ms > 0)
            .unwrap_or_else(|| reset_to_ms(self.reset_at.as_ref()));
        Some(Window { sisa, reset_ms })
    }
}

#[derive(Deserialize)]
struct LocalQuotaFile {
    #[serde(default)]
    session: Option<LocalQuotaWindow>,
    #[serde(default)]
    weekly: Option<LocalQuotaWindow>,
    #[serde(default)]
    updated_at_ms: Option<i64>,
}

fn local_quota_paths() -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(2);
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join("agy-quota.json"));
        }
    }
    paths.push(crate::infrastructure::store::data_dir().join("agy-quota.json"));
    paths
}

fn parse_local_quota_json(raw: &str, fallback_at_ms: i64) -> Option<QuotaSnapshot> {
    let parsed: LocalQuotaFile = serde_json::from_str(raw).ok()?;
    let session = parsed.session.as_ref().and_then(LocalQuotaWindow::to_window);
    let weekly = parsed.weekly.as_ref().and_then(LocalQuotaWindow::to_window);
    if session.is_none() && weekly.is_none() {
        return None;
    }
    let at_ms = parsed
        .updated_at_ms
        .filter(|&ms| ms > 0)
        .unwrap_or(fallback_at_ms);
    Some(QuotaSnapshot {
        session,
        weekly,
        akun_ok: 1,
        akun_total: 1,
        error_key: None,
        at_ms,
    })
}

fn read_local_quota_file() -> Option<QuotaSnapshot> {
    for path in local_quota_paths() {
        let Ok(raw) = std::fs::read_to_string(&path) else {
            continue;
        };
        let mtime_ms = std::fs::metadata(&path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or_else(|| chrono::Local::now().timestamp_millis());
        if let Some(snap) = parse_local_quota_json(&raw, mtime_ms) {
            return Some(snap);
        }
    }
    None
}

/// Satu pembacaan penuh. Setiap jalur keluar mengisi `at_ms` supaya UI bisa
/// menilai kesegaran angka.
fn fetch_partial() -> QuotaSnapshot {
    if let Some(local) = read_local_quota_file() {
        return local;
    }

    let Some(dir) = data_dir() else {
        return QuotaSnapshot {
            error_key: Some("home.quota.err.none"),
            at_ms: chrono::Local::now().timestamp_millis(),
            ..Default::default()
        };
    };
    let Some(token) = cli_token(&dir) else {
        return QuotaSnapshot {
            error_key: Some("home.quota.err.token"),
            at_ms: chrono::Local::now().timestamp_millis(),
            ..Default::default()
        };
    };

    // Port bawaan 9Router. Kalau server tidak menyala, `call()` gagal cepat
    // dan snapshot kosong — halaman menampilkan sebabnya, bukan angka palsu.
    let base = "http://127.0.0.1:20128";
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(20))
        .build();

    let akun = match akun_aktif(&dir) {
        Ok(a) if !a.is_empty() => a,
        Ok(_) => {
            return QuotaSnapshot {
                error_key: Some("home.quota.err.no_account"),
                at_ms: chrono::Local::now().timestamp_millis(),
                ..Default::default()
            }
        }
        Err(key) => {
            return QuotaSnapshot {
                error_key: Some(key),
                at_ms: chrono::Local::now().timestamp_millis(),
                ..Default::default()
            }
        }
    };

    let total = akun.len();

    // Akun yang **dipakai**, bukan rata-rata semua akun aktif.
    //
    // Versi sebelumnya merata-ratakan kuota seluruh akun aktif. Dengan satu
    // akun aktif hasilnya kebetulan sama, tetapi begitu ada dua, cincin
    // menampilkan angka yang tidak dimiliki akun mana pun — pengguna tidak bisa
    // mencocokkannya dengan kartu mana pun di dashboard 9Router. Satu akun,
    // satu angka.
    //
    // Yang dipakai adalah akun pertama yang berhasil dibaca menurut urutan
    // `priority` 9Router (ORDER BY priority di `akun_aktif`), yang berarti
    // akun yang paling dulu dipakai router saat merutekan permintaan.
    let mut hasil: Option<SatuAkun> = None;
    let mut akun_terpilih = 0usize;
    let mut last_err: Option<&'static str> = None;
    let mut ok = 0usize;

    for id in &akun {
        let resp = match fetch_connection(&agent, base, &token, id) {
            Ok(r) => r,
            Err(e) => {
                last_err = Some(e);
                continue;
            }
        };

        let satu = akun_dari_respons(&resp);

        if satu.sesi.is_none() && satu.minggu.is_none() {
            if resp.message.is_some() {
                // Pesan 9Router panjangnya bisa satu kalimat penuh
                // ("Antigravity error fetch(...)"). Baris Home hanya selebar
                // dua cincin, jadi yang disimpan cukup sebabnya.
                last_err = Some("home.quota.err.upstream");
            }
            continue;
        }

        ok += 1;
        if hasil.is_none() {
            hasil = Some(satu);
            akun_terpilih = ok;
        }
        // Akun pertama yang berhasil sudah cukup — sisanya tidak perlu
        // diambil, jadi loop berhenti di sini.
        break;
    }

    QuotaSnapshot {
        session: hasil.as_ref().and_then(|h| h.sesi),
        weekly: hasil.as_ref().and_then(|h| h.minggu),
        akun_ok: if hasil.is_some() { akun_terpilih } else { 0 },
        akun_total: total,
        error_key: if hasil.is_none() {
            Some(last_err.unwrap_or("home.quota.err.empty"))
        } else {
            None
        },
        at_ms: chrono::Local::now().timestamp_millis(),
    }
}

/* ------------------------------------------------------------- worker */

/// Sisi UI dari worker kuota.
pub struct QuotaHandle {
    pub rx: std::sync::mpsc::Receiver<QuotaSnapshot>,
    pub tx: std::sync::mpsc::Sender<()>,
}

/// Jarak antar-penyegaran. Kuota 5 jam tidak berubah per detik, dan tiap
/// pembacaan memakan ~3 detik HTTP ke Google lewat 9Router.
const REFRESH: Duration = Duration::from_secs(180);

/// Thread pembaca kuota.
///
/// Mengikuti pola worker cuaca: satu thread, kanal masuk untuk meminta
/// penyegaran, kanal keluar untuk hasil. Kalau 9Router mati, worker tetap
/// hidup dan mencoba lagi di putaran berikutnya.
pub fn spawn(ctx: eframe::egui::Context) -> QuotaHandle {
    let (tx_out, rx_out) = std::sync::mpsc::channel();
    let (tx_req, rx_req) = std::sync::mpsc::channel::<()>();

    let _ = std::thread::Builder::new()
        .name("noktah-quota".into())
        .stack_size(256 * 1024)
        .spawn(move || {
            let mut terakhir: Option<std::time::Instant> = None;
            let mut dipaksa = true;

            // Angka terakhir yang berhasil, dipakai sbg cadangan ketika 9Router
            // sedang gagal sesaat. Antigravity upstream kadang membalas
            // "error fetch" selama beberapa detik; membuang angka yang sudah
            // benar karena itu membuat cincin berkedip kosong tanpa alasan.
            let mut terakhir_ok: Option<QuotaSnapshot> = None;

            loop {
                if rx_req.try_recv().is_ok() {
                    dipaksa = true;
                }

                let perlu = dipaksa
                    || terakhir.map_or(true, |t: std::time::Instant| t.elapsed() > REFRESH);

                if perlu {
                    dipaksa = false;
                    terakhir = Some(std::time::Instant::now());
                    let snap = fetch_partial();

                    if snap.is_empty() {
                        // Gagal. Kalau ada angka sebelumnya, kirim angka itu
                        // **beserta** pesan gagalnya: UI akan menandainya
                        // sebagai angka lama, bukan membuangnya. Membuang angka
                        // yang sudah benar karena satu percobaan gagal membuat
                        // cincin berkedip kosong padahal kuotanya tidak berubah.
                        let lama = terakhir_ok.clone().map(|prev| QuotaSnapshot {
                            session: prev.session,
                            weekly: prev.weekly,
                            akun_ok: prev.akun_ok,
                            akun_total: snap.akun_total.max(prev.akun_total),
                            // sebab dari percobaan ini yang menandai "basi"
                            error_key: snap.error_key,
                            at_ms: prev.at_ms,
                        });
                        let _ = tx_out.send(lama.unwrap_or(snap));
                    } else {
                        terakhir_ok = Some(snap.clone());
                        let _ = tx_out.send(snap);
                    }
                    ctx.request_repaint();
                }

                std::thread::sleep(Duration::from_secs(2));
            }
        });

    QuotaHandle { rx: rx_out, tx: tx_req }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_variants() {
        // Vektor uji FIPS 180-4 yang paling sering dipakai.
        assert_eq!(
            sha256::hex(&sha256::digest(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256::hex(&sha256::digest(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        // 56 byte: satu blok penuh tepat sebelum padding panjang.
        assert_eq!(
            sha256::hex(&sha256::digest(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq")),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn sisa_dari_persentase() {
        let e = QuotaEntry {
            used: 900.0,
            total: 1000.0,
            remaining_percentage: Some(10.0),
            reset_at: None,
        };
        assert!((e.sisa().unwrap() - 0.10).abs() < 1e-6);
    }

    #[test]
    fn sisa_diturunkan_dari_used_total() {
        let e = QuotaEntry {
            used: 250.0,
            total: 1000.0,
            remaining_percentage: None,
            reset_at: None,
        };
        assert!((e.sisa().unwrap() - 0.75).abs() < 1e-6);
    }

    #[test]
    fn sisa_menolak_total_nol() {
        let e = QuotaEntry {
            used: 0.0,
            total: 0.0,
            remaining_percentage: None,
            reset_at: None,
        };
        assert!(e.sisa().is_none());
    }

    #[test]
    fn reset_rfc3339_ke_ms() {
        let s = "2026-11-07T06:25:57.000Z".to_string();
        assert_eq!(reset_to_ms(Some(&s)), 1_794_032_757_000);
        assert_eq!(reset_to_ms(None), 0);
    }

    /// Kelas bug yang dikunci: cincin menampilkan 84% (rata-rata 67% + 100%)
    /// padahal tidak ada angka itu di dashboard 9Router. Angka harus selalu
    /// salah satu angka yang benar-benar tampil di dashboard.
    #[test]
    fn bucket_terendah_selalu_angka_yang_ada() {
        // Satu akun, Gemini 67% dan Claude 100%.
        let resp = UsageResponse {
            quotas: [
                (
                    "gemini_weekly".into(),
                    QuotaEntry {
                        used: 33.0,
                        total: 100.0,
                        remaining_percentage: Some(67.0),
                        reset_at: Some("2026-11-08T06:25:57.000Z".into()),
                    },
                ),
                (
                    "claude_gpt_weekly".into(),
                    QuotaEntry {
                        used: 0.0,
                        total: 100.0,
                        remaining_percentage: Some(100.0),
                        reset_at: Some("2026-11-08T06:25:57.000Z".into()),
                    },
                ),
            ]
            .into(),
            message: None,
        };

        let a = akun_dari_respons(&resp);
        // Harus 67%, BUKAN 84% — angka yang dapat dicocokkan dengan dashboard.
        let sisa = a.minggu.unwrap().sisa;
        assert!((sisa - 0.67).abs() < 1e-6, "mingguan = {sisa}, harus 0.67");
        assert!(a.sesi.is_none());
    }

    /// Bucket yang tepat 0% dengan `resetAt` lama diabaikan — dashboard 9Router
    /// tidak menampilkannya (angka 0 bisa berarti "baru saja reset").
    #[test]
    fn bucket_nol_dengan_reset_lama_diabaikan() {
        // Gemini habis (0%) dengan resetAt lama; Claude masih 100% resetAt baru.
        let resp = UsageResponse {
            quotas: [
                (
                    "gemini_weekly".into(),
                    QuotaEntry {
                        used: 100.0,
                        total: 100.0,
                        remaining_percentage: Some(0.0),
                        reset_at: Some("2026-11-01T06:25:57.000Z".into()),
                    },
                ),
                (
                    "claude_gpt_weekly".into(),
                    QuotaEntry {
                        used: 0.0,
                        total: 100.0,
                        remaining_percentage: Some(100.0),
                        reset_at: Some("2026-11-08T06:25:57.000Z".into()),
                    },
                ),
            ]
            .into(),
            message: None,
        };

        let a = akun_dari_respons(&resp);
        assert!((a.minggu.unwrap().sisa - 1.0).abs() < 1e-6);
    }

    /// Dua bucket sama-sama di bawah 100%: yang lebih rendah menang.
    #[test]
    fn bucket_terendah_dari_dua() {
        let resp = UsageResponse {
            quotas: [
                (
                    "gemini_weekly".into(),
                    QuotaEntry {
                        used: 50.0,
                        total: 100.0,
                        remaining_percentage: Some(50.0),
                        reset_at: None,
                    },
                ),
                (
                    "claude_gpt_weekly".into(),
                    QuotaEntry {
                        used: 20.0,
                        total: 100.0,
                        remaining_percentage: Some(80.0),
                        reset_at: None,
                    },
                ),
            ]
            .into(),
            message: None,
        };

        let a = akun_dari_respons(&resp);
        assert!((a.minggu.unwrap().sisa - 0.50).abs() < 1e-6);
    }

    #[test]
    fn parse_local_quota_json_reads_windows_and_timestamps() {
        let raw = r#"{
            "session": { "remaining_pct": 85.0, "reset_ms": 1763300000000 },
            "weekly": { "remaining_pct": 62.5, "reset_at": "2026-11-20T00:00:00Z" },
            "updated_at_ms": 1763290000000
        }"#;
        let snap = parse_local_quota_json(raw, 1234).expect("valid local quota json");
        assert!((snap.session.unwrap().sisa - 0.85).abs() < 1e-6);
        assert_eq!(snap.session.unwrap().reset_ms, 1763300000000);
        assert!((snap.weekly.unwrap().sisa - 0.625).abs() < 1e-6);
        assert!(snap.weekly.unwrap().reset_ms > 0);
        assert_eq!(snap.at_ms, 1763290000000);
        assert_eq!(snap.akun_ok, 1);
        assert_eq!(snap.akun_total, 1);
    }

    #[test]
    fn parse_local_quota_json_ignores_empty_payload() {
        assert!(parse_local_quota_json("{}", 1234).is_none());
        assert!(parse_local_quota_json("not json", 1234).is_none());
    }
}

