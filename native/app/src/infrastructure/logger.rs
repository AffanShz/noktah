//! Pencatat log diagnostik lokal berputar (`noktah.log`).
//!
//! Menyimpan maksimal 256 KB sebelum dirotasi ke `noktah.old.log` agar ukuran
//! folder data tetap terkendali. Tidak pernah menulis API key atau isi chat.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const MAX_LOG_BYTES: u64 = 256 * 1024;

static LOG_LOCK: Mutex<()> = Mutex::new(());

fn logs_dir() -> PathBuf {
    crate::infrastructure::store::data_dir().join("logs")
}

/// Menulis satu baris log ke `logs_dir/noktah.log` dengan rotasi otomatis.
pub(crate) fn write_entry_in(logs_dir: &Path, level: &str, scope: &str, msg: &str) {
    let _guard = LOG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _ = std::fs::create_dir_all(logs_dir);

    let log_path = logs_dir.join("noktah.log");
    let old_path = logs_dir.join("noktah.old.log");

    if let Ok(meta) = std::fs::metadata(&log_path) {
        if meta.len() >= MAX_LOG_BYTES {
            // Di Windows rename gagal jika target sudah ada, jadi hapus backup lama dulu.
            let _ = std::fs::remove_file(&old_path);
            let _ = std::fs::rename(&log_path, &old_path);
        }
    }

    let clean_msg: String = msg
        .chars()
        .map(|c| if c == '\r' || c == '\n' { ' ' } else { c })
        .collect();
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!("[{timestamp}] [{level}] [{scope}] {clean_msg}\n");

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = file.write_all(line.as_bytes());
    }
}

/// Membaca paling banyak `max_lines` baris terakhir dari `noktah.log`
/// beserta bagian `[crash.log]` bila berkas crash ada dan tidak kosong.
pub(crate) fn tail_lines_in(logs_dir: &Path, max_lines: usize) -> String {
    let _guard = LOG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut all_lines: Vec<String> = Vec::new();

    if let Ok(main_log) = std::fs::read_to_string(logs_dir.join("noktah.log")) {
        all_lines.extend(main_log.lines().map(ToOwned::to_owned));
    }

    if let Ok(crash_log) = std::fs::read_to_string(logs_dir.join("crash.log")) {
        if !crash_log.trim().is_empty() {
            if !all_lines.is_empty() {
                all_lines.push(String::new());
            }
            all_lines.push("[crash.log]".to_string());
            all_lines.extend(crash_log.lines().map(ToOwned::to_owned));
        }
    }

    if max_lines == 0 || all_lines.is_empty() {
        return String::new();
    }

    let start = all_lines.len().saturating_sub(max_lines);
    all_lines[start..].join("\n")
}

/// Menghapus seluruh berkas log diagnostik dan crash.
pub(crate) fn clear_in(logs_dir: &Path) {
    let _guard = LOG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    for name in ["noktah.log", "noktah.old.log", "crash.log"] {
        let _ = std::fs::remove_file(logs_dir.join(name));
    }
}

pub fn info(scope: &str, msg: &str) {
    if cfg!(test) {
        return;
    }
    write_entry_in(&logs_dir(), "INFO", scope, msg);
}

pub fn warn(scope: &str, msg: &str) {
    if cfg!(test) {
        return;
    }
    write_entry_in(&logs_dir(), "WARN", scope, msg);
}

pub fn error(scope: &str, msg: &str) {
    if cfg!(test) {
        return;
    }
    write_entry_in(&logs_dir(), "ERROR", scope, msg);
}

pub fn tail_lines(max_lines: usize) -> String {
    tail_lines_in(&logs_dir(), max_lines)
}

pub fn clear() {
    clear_in(&logs_dir());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_logs_dir(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!(
            "noktah_logger_test_{name}_{}_{nanos}",
            std::process::id()
        ))
    }

    #[test]
    fn write_tail_dan_clear_berfungsi() {
        let dir = temp_logs_dir("basic");
        write_entry_in(&dir, "INFO", "app", "baris satu\nbaris dua");
        write_entry_in(&dir, "WARN", "lyrics", "gagal ambil");
        std::fs::write(dir.join("crash.log"), "PANIC contoh\n").unwrap();

        let tail = tail_lines_in(&dir, 10);
        assert!(tail.contains("[INFO] [app] baris satu baris dua"));
        assert!(tail.contains("[WARN] [lyrics] gagal ambil"));
        assert!(tail.contains("[crash.log]"));
        assert!(tail.contains("PANIC contoh"));

        let limited = tail_lines_in(&dir, 2);
        assert_eq!(limited.lines().count(), 2);

        clear_in(&dir);
        assert!(!dir.join("noktah.log").exists());
        assert!(!dir.join("crash.log").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rotasi_log_menimpa_backup_lama_tanpa_gagal() {
        let dir = temp_logs_dir("rotate");
        std::fs::create_dir_all(&dir).unwrap();
        let log_path = dir.join("noktah.log");
        let old_path = dir.join("noktah.old.log");

        // Simulasikan log penuh dan backup lama sudah ada.
        std::fs::write(&old_path, b"backup lama").unwrap();
        std::fs::write(&log_path, vec![b'a'; MAX_LOG_BYTES as usize]).unwrap();

        write_entry_in(&dir, "INFO", "rotasi", "entri setelah rotasi 1");
        assert!(old_path.exists());
        assert_eq!(std::fs::metadata(&old_path).unwrap().len(), MAX_LOG_BYTES);
        assert!(std::fs::read_to_string(&log_path)
            .unwrap()
            .contains("entri setelah rotasi 1"));

        // Rotasi kedua saat noktah.old.log sudah terisi.
        std::fs::write(&log_path, vec![b'b'; MAX_LOG_BYTES as usize + 10]).unwrap();
        write_entry_in(&dir, "WARN", "rotasi", "entri setelah rotasi 2");
        assert_eq!(
            std::fs::metadata(&old_path).unwrap().len(),
            MAX_LOG_BYTES + 10
        );
        assert!(std::fs::read_to_string(&log_path)
            .unwrap()
            .contains("entri setelah rotasi 2"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
