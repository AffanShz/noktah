//! Baca/tulis data ke disk.
//!
//! Semua file ditulis lewat file sementara lalu di-rename, supaya setelan
//! tidak pernah tersisa separuh jadi kalau aplikasi dimatikan di tengah
//! penulisan.

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::{Path, PathBuf};

pub fn data_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("Noktah");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".local/share/noktah");
    }
    PathBuf::from(".")
}

fn path_for(name: &str) -> PathBuf {
    let dir = data_dir();
    let _ = std::fs::create_dir_all(&dir);
    dir.join(name)
}

/// Membaca JSON dari path tertentu secara ketat tanpa nilai bawaan.
pub fn load_strict<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str::<T>(&raw).map_err(|e| e.to_string())
}

/// Membaca JSON; kalau file tidak ada atau rusak, kembalikan nilai bawaan.
/// File yang rusak disalin dulu ke `.rusak` supaya tidak hilang diam-diam.
pub fn load<T: DeserializeOwned + Default>(name: &str) -> T {
    let path = path_for(name);
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return T::default();
    };
    match serde_json::from_str::<T>(&raw) {
        Ok(value) => value,
        Err(_) => {
            let _ = std::fs::copy(&path, path.with_extension("rusak"));
            T::default()
        }
    }
}

pub fn save<T: Serialize>(name: &str, value: &T) {
    let path = path_for(name);
    let tmp = path.with_extension("tmp");
    let Ok(raw) = serde_json::to_string_pretty(value) else {
        return;
    };
    if std::fs::write(&tmp, raw).is_ok() {
        let _ = std::fs::rename(&tmp, &path);
    }
}

/// Menghapus semua data milik aplikasi (dipakai tombol "Hapus semua data").
pub fn wipe() {
    wipe_in(&data_dir());
}

pub(crate) fn wipe_in(dir: &std::path::Path) {
    for nama in [
        "settings.json",
        "tasks.json",
        "chat.json",
        "pomodoro.json",
        "notes.json",
        "water.json",
        "settings-export.json",
        "theme-preset.json",
        "secret.bin",
    ] {
        let _ = std::fs::remove_file(dir.join(nama));
    }
    let _ = std::fs::remove_dir_all(dir.join("lyrics"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wipe_menghapus_seluruh_data_dan_cache_lirik() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let tmp = std::env::temp_dir().join(format!(
            "noktah_store_wipe_{}_{nanos}",
            std::process::id()
        ));
        let lyrics_dir = tmp.join("lyrics");
        std::fs::create_dir_all(&lyrics_dir).unwrap();

        for nama in [
            "settings.json",
            "tasks.json",
            "chat.json",
            "pomodoro.json",
            "notes.json",
            "water.json",
            "settings-export.json",
            "theme-preset.json",
            "secret.bin",
        ] {
            std::fs::write(tmp.join(nama), b"{}").unwrap();
        }
        std::fs::write(lyrics_dir.join("song.json"), b"{}").unwrap();

        wipe_in(&tmp);

        for nama in [
            "settings.json",
            "tasks.json",
            "chat.json",
            "pomodoro.json",
            "notes.json",
            "water.json",
            "settings-export.json",
            "theme-preset.json",
            "secret.bin",
        ] {
            assert!(!tmp.join(nama).exists(), "{nama} harus terhapus");
        }
        assert!(!lyrics_dir.exists(), "folder cache lyrics harus terhapus");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn load_strict_menolak_berkas_hilang_dan_json_rusak() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let tmp = std::env::temp_dir().join(format!(
            "noktah_store_strict_{}_{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&tmp).unwrap();

        let missing = tmp.join("missing.json");
        assert!(load_strict::<noktah_core::Settings>(&missing).is_err());

        let broken = tmp.join("broken.json");
        std::fs::write(&broken, b"{ not valid json").unwrap();
        assert!(load_strict::<noktah_core::Settings>(&broken).is_err());
        assert!(
            !broken.with_extension("rusak").exists(),
            "load_strict tidak boleh membuat berkas sampingan .rusak"
        );

        let valid = tmp.join("valid.json");
        std::fs::write(&valid, br#"{"city":"Bandung","focus_minutes":40}"#).unwrap();
        let loaded = load_strict::<noktah_core::Settings>(&valid).unwrap();
        assert_eq!(loaded.city, "Bandung");
        assert_eq!(loaded.focus_minutes, 40);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
