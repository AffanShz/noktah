//! Baca/tulis data ke disk.
//!
//! Semua file ditulis lewat file sementara lalu di-rename, supaya setelan
//! tidak pernah tersisa separuh jadi kalau aplikasi dimatikan di tengah
//! penulisan.

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::PathBuf;

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
    let dir = data_dir();
    for nama in ["settings.json", "tasks.json", "chat.json", "focus.json"] {
        let _ = std::fs::remove_file(dir.join(nama));
    }
}
