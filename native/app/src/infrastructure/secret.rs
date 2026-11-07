//! Penyimpanan kunci API.
//!
//! Di Windows kunci dienkripsi dengan DPAPI — hanya akun pengguna yang sama
//! di mesin yang sama yang bisa membukanya. Kunci tidak pernah masuk ke
//! `settings.json` dan tidak pernah ikut tersalin kalau folder data disalin
//! ke komputer lain.

use std::path::PathBuf;

fn secret_path() -> PathBuf {
    let dir = crate::infrastructure::store::data_dir();
    let _ = std::fs::create_dir_all(&dir);
    dir.join("secret.bin")
}

#[cfg(windows)]
pub fn save(value: &str) -> Result<(), String> {
    use windows::Win32::Foundation::{HLOCAL, LocalFree};
    use windows::Win32::Security::Cryptography::{
        CryptProtectData, CRYPT_INTEGER_BLOB,
    };

    if value.is_empty() {
        let _ = std::fs::remove_file(secret_path());
        return Ok(());
    }

    unsafe {
        let mut input = value.as_bytes().to_vec();
        let blob_in = CRYPT_INTEGER_BLOB {
            cbData: input.len() as u32,
            pbData: input.as_mut_ptr(),
        };
        let mut blob_out = CRYPT_INTEGER_BLOB::default();

        CryptProtectData(&blob_in, None, None, None, None, 0, &mut blob_out)
            .map_err(|e| e.to_string())?;

        let bytes = std::slice::from_raw_parts(blob_out.pbData, blob_out.cbData as usize).to_vec();
        let _ = LocalFree(HLOCAL(blob_out.pbData as *mut _));
        std::fs::write(secret_path(), bytes).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(windows)]
pub fn load() -> String {
    use windows::Win32::Foundation::{HLOCAL, LocalFree};
    use windows::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};

    let Ok(mut raw) = std::fs::read(secret_path()) else {
        return String::new();
    };
    if raw.is_empty() {
        return String::new();
    }

    unsafe {
        let blob_in = CRYPT_INTEGER_BLOB {
            cbData: raw.len() as u32,
            pbData: raw.as_mut_ptr(),
        };
        let mut blob_out = CRYPT_INTEGER_BLOB::default();

        if CryptUnprotectData(&blob_in, None, None, None, None, 0, &mut blob_out).is_err() {
            return String::new();
        }

        let bytes = std::slice::from_raw_parts(blob_out.pbData, blob_out.cbData as usize).to_vec();
        let _ = LocalFree(HLOCAL(blob_out.pbData as *mut _));
        String::from_utf8(bytes).unwrap_or_default()
    }
}

#[cfg(not(windows))]
pub fn save(value: &str) -> Result<(), String> {
    if value.is_empty() {
        let _ = std::fs::remove_file(secret_path());
        return Ok(());
    }
    std::fs::write(secret_path(), value.as_bytes()).map_err(|e| e.to_string())
}

#[cfg(not(windows))]
pub fn load() -> String {
    std::fs::read_to_string(secret_path()).unwrap_or_default()
}
