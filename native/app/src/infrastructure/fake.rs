//! Implementasi palsu untuk setiap port.
//!
//! Hidup di `infrastructure` (bukan di modul test) supaya use case bisa
//! mengimpornya di `#[cfg(test)]` tanpa menarik `std::fs` atau Win32. Semua
//! state disimpan di memori dan bisa diklon, jadi satu test bisa memeriksa
//! apa yang ditulis dengan memuat ulang.

use crate::application::ports::{
    ChatRequest, Clock, LyricsDoc, MediaSnapshot, MediaSource,
    Network, SecretStore, Store, SystemActions, SystemInfo,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Penyimpanan JSON di memori. Klon berbagi isi yang sama, meniru satu
/// direktori data yang dilihat banyak pemegang.
#[derive(Clone, Default)]
pub struct FakeStore {
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl Store for FakeStore {
    fn load<T>(&self, name: &str) -> T
    where
        T: serde::de::DeserializeOwned + Default,
    {
        let map = self.data.lock().unwrap();
        match map.get(name) {
            Some(raw) => serde_json::from_str(raw).unwrap_or_default(),
            None => T::default(),
        }
    }

    fn save<T>(&self, name: &str, value: &T)
    where
        T: serde::Serialize,
    {
        let raw = serde_json::to_string(value).unwrap_or_default();
        self.data.lock().unwrap().insert(name.to_string(), raw);
    }

    fn wipe(&self) {
        self.data.lock().unwrap().clear();
    }

    fn data_dir(&self) -> std::path::PathBuf {
        std::path::PathBuf::from("/fake")
    }
}

impl FakeStore {
    /// Nama-nama berkas yang pernah ditulis. Dipakai test untuk memastikan
    /// use case benar-benar menyimpan, bukan hanya mengubah state di memori.
    pub fn written(&self) -> Vec<String> {
        let mut v: Vec<String> = self.data.lock().unwrap().keys().cloned().collect();
        v.sort();
        v
    }
}

#[derive(Clone, Default)]
pub struct FakeSecret {
    value: Arc<Mutex<String>>,
}

impl SecretStore for FakeSecret {
    fn load(&self) -> String {
        self.value.lock().unwrap().clone()
    }

    fn save(&self, value: &str) -> Result<(), String> {
        *self.value.lock().unwrap() = value.to_string();
        Ok(())
    }
}

/// Jaringan palsu dengan jawaban yang bisa diatur per test.
#[derive(Clone, Default)]
pub struct FakeNetwork {
    pub weather: Option<noktah_core::model::WeatherNow>,
    pub lyrics: Option<LyricsDoc>,
    /// Potongan jawaban AI yang akan dialirkan, berurutan.
    pub chat_chunks: Vec<String>,
    pub chat_error: Option<String>,
    /// Permintaan chat yang benar-benar diterima, untuk diperiksa.
    pub sent: Rc<RefCell<Vec<ChatRequest>>>,
}

impl Network for FakeNetwork {
    fn weather(&self, _city: &str) -> Option<noktah_core::model::WeatherNow> {
        self.weather.clone()
    }

    fn lyrics(
        &self,
        _artist: &str,
        _title: &str,
        _album: &str,
        _duration_ms: i64,
    ) -> Option<LyricsDoc> {
        self.lyrics.clone()
    }

    fn chat(
        &self,
        req: ChatRequest,
        on_delta: &mut dyn FnMut(&str),
    ) -> Result<(), String> {
        self.sent.borrow_mut().push(req);
        if let Some(err) = &self.chat_error {
            return Err(err.clone());
        }
        for chunk in &self.chat_chunks {
            on_delta(chunk);
        }
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct FakeSystem {
    pub charging: bool,
    pub battery: u8,
    pub cpu: f32,
    pub ram_used: u64,
    pub ram_total: u64,
    pub uptime: u64,
    pub cores: usize,
}

impl SystemInfo for FakeSystem {
    fn power(&self) -> (bool, u8) {
        (self.charging, self.battery)
    }
    fn cpu_percent(&self) -> f32 {
        self.cpu
    }
    fn ram(&self) -> (u64, u64) {
        (self.ram_used, self.ram_total)
    }
    fn uptime_secs(&self) -> u64 {
        self.uptime
    }
    fn cores(&self) -> usize {
        self.cores
    }
}

#[derive(Clone, Default)]
pub struct FakeActions {
    pub calls: Rc<RefCell<Vec<String>>>,
    pub autostart: Rc<RefCell<bool>>,
}

impl SystemActions for FakeActions {
    fn set_autostart(&self, enabled: bool) -> Result<(), String> {
        self.calls.borrow_mut().push(format!("autostart:{enabled}"));
        *self.autostart.borrow_mut() = enabled;
        Ok(())
    }
    fn autostart_enabled(&self) -> bool {
        *self.autostart.borrow()
    }
    fn toggle_volume_mute(&self) {
        self.calls.borrow_mut().push("mute".into());
    }
    fn set_always_on_top(&self, on: bool) {
        self.calls.borrow_mut().push(format!("ontop:{on}"));
    }
}

#[derive(Clone, Default)]
pub struct FakeMedia {
    pub snapshot: Option<MediaSnapshot>,
}

impl MediaSource for FakeMedia {
    fn snapshot(&self) -> Option<MediaSnapshot> {
        self.snapshot.clone()
    }
}

#[derive(Clone)]
pub struct FakeClock {
    pub ms: Arc<Mutex<i64>>,
}

impl Default for FakeClock {
    fn default() -> Self {
        Self {
            ms: Arc::new(Mutex::new(0)),
        }
    }
}

impl FakeClock {
    pub fn at(ms: i64) -> Self {
        Self {
            ms: Arc::new(Mutex::new(ms)),
        }
    }
    pub fn advance(&self, by: i64) {
        *self.ms.lock().unwrap() += by;
    }
}

impl Clock for FakeClock {
    fn now_ms(&self) -> i64 {
        *self.ms.lock().unwrap()
    }
    fn now_local_hour(&self) -> u32 {
        ((self.now_ms() / 3_600_000) % 24) as u32
    }
}
