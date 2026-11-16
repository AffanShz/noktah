//! Port: kontrak yang dibutuhkan lapisan aplikasi dari dunia luar.
//!
//! Tidak ada satu pun tipe egui, `std::fs`, atau `windows` di file ini —
//! hanya trait. Implementasinya hidup di `crate::infrastructure`, dan versi
//! palsunya di `crate::infrastructure::fake`. Itulah yang membuat use case
//! bisa diuji tanpa disk, registry, atau jendela.

use noktah_core::model::{ChatMessage, MediaState, PomodoroState, Todo};

/* --------------------------------------------------------------- penyimpanan */

/// Satu berkas JSON di direktori data. Nama berkas adalah kuncinya.
///
/// Dibuat trait, bukan fungsi bebas, supaya test bisa memakai penyimpanan
/// di memori dan memeriksa apa yang ditulis tanpa menyentuh disk pengguna.
pub trait Store {
    fn load<T>(&self, name: &str) -> T
    where
        T: serde::de::DeserializeOwned + Default;

    fn save<T>(&self, name: &str, value: &T)
    where
        T: serde::Serialize;

    /// Hapus seluruh data pengguna. Dipakai "Reset" di setelan.
    fn wipe(&self);

    /// Direktori data, untuk modul yang menyimpan berkas sendiri (cache lirik).
    fn data_dir(&self) -> std::path::PathBuf;
}

/// Kunci API yang disimpan terenkripsi (DPAPI di Windows).
///
/// Terpisah dari [`Store`] karena tingkat bahayanya berbeda: ini rahasia,
/// bukan sekadar preferensi. Memisahkannya membuat permukaan "tulis rahasia"
/// kecil dan mudah diaudit.
pub trait SecretStore {
    fn load(&self) -> String;
    fn save(&self, value: &str) -> Result<(), String>;
}

/* ------------------------------------------------------------------- jaringan */

/// Kebutuhan HTTP lapisan aplikasi, dinyatakan dalam istilah domain.
///
/// Pemanggil tidak perlu tahu ini ureq, reqwest, atau mock. Mengembalikan
/// `None`/`Err` yang sudah berbentuk teks siap tampil, karena use case tidak
/// punya urusan menerjemahkan kode status HTTP.
pub trait Network {
    /// Cuaca sekarang untuk sebuah kota.
    fn weather(&self, city: &str) -> Option<noktah_core::model::WeatherNow>;

    /// Lirik untuk satu lagu.
    fn lyrics(
        &self,
        artist: &str,
        title: &str,
        album: &str,
        duration_ms: i64,
    ) -> Option<LyricsDoc>;

    /// Kirim percakapan ke penyedia AI dan alirkan potongan jawaban lewat
    /// `on_delta`. `Ok(())` berarti selesai normal.
    fn chat(
        &self,
        req: ChatRequest,
        on_delta: &mut dyn FnMut(&str),
    ) -> Result<(), String>;
}

/// Lirik hasil pencarian, sudah dalam bentuk domain.
#[derive(Clone, Debug, Default)]
pub struct LyricsDoc {
    pub synced: Vec<noktah_core::lrc::LrcLine>,
    pub plain: String,
}

#[derive(Clone, Debug)]
pub struct ChatRequest {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    pub messages: Vec<(noktah_core::model::Role, String)>,
}

/* ---------------------------------------------------------------- platform OS */

/// Hal-hal yang hanya bisa dijawab sistem operasi.
///
/// Setiap method di sini punya implementasi no-op di luar Windows, sehingga
/// use case tetap bisa dikompilasi dan diuji di mesin apa pun.
pub trait SystemInfo {
    /// (sedang mengisi daya, persentase baterai)
    fn power(&self) -> (bool, u8);
    fn cpu_percent(&self) -> f32;
    fn ram(&self) -> (u64, u64);
    fn uptime_secs(&self) -> u64;
    fn cores(&self) -> usize;
}

/// Sisi efek samping dari preferensi pengguna.
pub trait SystemActions {
    fn set_autostart(&self, enabled: bool) -> Result<(), String>;
    fn autostart_enabled(&self) -> bool;
    fn toggle_volume_mute(&self);
    fn set_always_on_top(&self, on: bool);
}

/* --------------------------------------------------------- sumber media (SMTC) */

/// Keadaan pemutar musik yang sedang aktif di sistem.
pub trait MediaSource {
    /// Coba baca keadaan sekarang. `None` = tidak ada sesi media.
    fn snapshot(&self) -> Option<MediaSnapshot>;
}

#[derive(Clone, Debug, Default)]
pub struct MediaSnapshot {
    pub state: MediaState,
    /// Sampul album mentah (JPEG/PNG). Aplikasi yang memutuskan ukuran tekstur.
    pub art: Option<Vec<u8>>,
    /// Identitas lagu, supaya pemanggil tahu kapan sampul perlu dimuat ulang.
    pub track_id: String,
}

/* ----------------------------------------------------------------- waktu */

/// Sumber waktu, dibungkus supaya test bisa mengendalikannya.
pub trait Clock {
    fn now_ms(&self) -> i64;
    fn now_local_hour(&self) -> u32;
}

/* ---------------------------------------------------------------- use case */

/// Semua port yang dibutuhkan satu use case, dikumpulkan di satu tempat.
///
/// Ini memaksa setiap use case menyatakan kebutuhannya di tanda tangan —
/// kalau sebuah use case tiba-tiba butuh jaringan, itu terlihat di sini,
/// bukan tersembunyi di dalam pemanggilan langsung.
pub trait Deps {
    type Store: Store;
    type Secret: SecretStore;
    type Net: Network;
    type Sys: SystemInfo;
    type Actions: SystemActions;
    type Media: MediaSource;
    type Clock: Clock;

    fn store(&self) -> &Self::Store;
    fn secret(&self) -> &Self::Secret;
    fn net(&self) -> &Self::Net;
    fn sys(&self) -> &Self::Sys;
    fn actions(&self) -> &Self::Actions;
    fn media(&self) -> &Self::Media;
    fn clock(&self) -> &Self::Clock;
}

/* ------------------------------------------------------------ tipe data tersimpan */

/// Nama berkas data. Dikumpulkan di sini supaya tidak ada string ajaib
/// tersebar di seluruh kode — satu salah ketik di sini akan diam-diam
/// membuat berkas kedua.
pub mod files {
    pub const SETTINGS: &str = "settings.json";
    pub const TASKS: &str = "tasks.json";
    pub const CHAT: &str = "chat.json";
    pub const POMODORO: &str = "pomodoro.json";
}

/// Bentuk data yang disimpan, dikumpulkan supaya adapter penyimpanan tidak
/// perlu tahu tipe konkret satu per satu.
#[derive(Default, Clone, Debug)]
pub struct Persisted {
    pub settings: noktah_core::Settings,
    pub todos: Vec<Todo>,
    pub chat: Vec<ChatMessage>,
    pub pomodoro: PomodoroState,
}
