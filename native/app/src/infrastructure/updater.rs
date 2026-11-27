//! Pemeriksa dan pemasang pembaruan aplikasi Noktah dari GitHub Releases API.
//!
//! Dijalankan secara asinkron di thread terpisah agar tidak memblokir antarmuka UI.
//! Mendukung pengunduhan paket rilis `.zip`, verifikasi hash SHA-256 terhadap
//! `BACA-DULU.txt`, serta penggantian biner dan restart otomatis.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

const MAX_RELEASE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

/// Informasi rilis yang dikembalikan GitHub Releases API.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub html_url: String,
    #[allow(dead_code)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub assets: Vec<ReleaseAsset>,
}

/// Perintah yang dikirim UI ke background thread updater.
#[derive(Clone, Debug)]
pub enum UpdaterCmd {
    Check,
    DownloadAndStage { version: String, zip_url: String },
}

/// Kegagalan terstruktur pada pemeriksa/pemasang pembaruan (netral bahasa).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateError {
    GitHubUnreachable(String),
    HttpStatus { status: u16 },
    InvalidReleaseJson(String),
    UntrustedDownloadUrl,
    AssetMissing(String),
    StagingDirFailed(String),
    DownloadFailed(String),
    ArchiveReadFailed(String),
    ArchiveTooLarge { max_mb: usize },
    ArchiveWriteFailed(String),
    ExtractFailed { code: Option<i32> },
    ExtractSpawnFailed(String),
    InvalidBinary { file: String },
    BinaryReadFailed { file: String, detail: String },
    Sha256Mismatch { file: String },
    InstallError(String),
    HookUpdateFailed(String),
}

impl UpdateError {
    /// Ringkasan teknis netral untuk pencatatan log internal.
    pub(crate) fn log_detail(&self) -> String {
        match self {
            Self::GitHubUnreachable(e) => format!("GitHub unreachable: {e}"),
            Self::HttpStatus { status } => format!("HTTP status {status}"),
            Self::InvalidReleaseJson(e) => format!("Invalid release JSON: {e}"),
            Self::UntrustedDownloadUrl => "Untrusted download URL".into(),
            Self::AssetMissing(file) => format!("Missing release asset: {file}"),
            Self::StagingDirFailed(e) => format!("Staging dir failed: {e}"),
            Self::DownloadFailed(e) => format!("Download failed: {e}"),
            Self::ArchiveReadFailed(e) => format!("Archive read failed: {e}"),
            Self::ArchiveTooLarge { max_mb } => format!("Archive exceeds {max_mb} MB limit"),
            Self::ArchiveWriteFailed(e) => format!("Archive write failed: {e}"),
            Self::ExtractFailed { code } => format!("Extract exited with {code:?}"),
            Self::ExtractSpawnFailed(e) => format!("Extract launch failed: {e}"),
            Self::InvalidBinary { file } => format!("Invalid extracted binary: {file}"),
            Self::BinaryReadFailed { file, detail } => {
                format!("Failed reading {file} for verification: {detail}")
            }
            Self::Sha256Mismatch { file } => format!("SHA-256 mismatch for {file}"),
            Self::InstallError(e) => format!("Install error: {e}"),
            Self::HookUpdateFailed(e) => format!("Hook update error: {e}"),
        }
    }
}

/// Status pengecekan dan pemasangan pembaruan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    LatestVersion(String),
    UpdateAvailable {
        version: String,
        name: String,
        body: String,
        url: String,
        zip_url: Option<String>,
    },
    Downloading {
        version: String,
    },
    ReadyToRestart {
        version: String,
        staged_exe: PathBuf,
        staged_hook: Option<PathBuf>,
        sha256_verified: bool,
    },
    Error(UpdateError),
}

/// Handle komunikasi antara thread UI egui dan thread background updater.
pub struct UpdaterHandle {
    pub rx: Receiver<UpdateStatus>,
    pub tx: Sender<UpdaterCmd>,
}

impl UpdaterHandle {
    #[cfg(test)]
    pub fn dummy() -> Self {
        let (_, rx) = channel();
        let (tx, _) = channel();
        Self { rx, tx }
    }
}

/// Memulai background thread untuk memeriksa dan mengunduh pembaruan saat diminta.
pub fn spawn(ctx: egui::Context) -> UpdaterHandle {
    let (tx_out, rx_out) = channel();
    let (tx_in, rx_in) = channel::<UpdaterCmd>();

    let _ = std::thread::Builder::new()
        .name("noktah-updater".into())
        .stack_size(256 * 1024)
        .spawn(move || {
            while let Ok(cmd) = rx_in.recv() {
                match cmd {
                    UpdaterCmd::Check => {
                        let _ = tx_out.send(UpdateStatus::Checking);
                        ctx.request_repaint();

                        let res = check_github();
                        let _ = tx_out.send(res);
                        ctx.request_repaint();
                    }
                    UpdaterCmd::DownloadAndStage { version, zip_url } => {
                        let _ = tx_out.send(UpdateStatus::Downloading {
                            version: version.clone(),
                        });
                        ctx.request_repaint();

                        let res = download_and_stage(&version, &zip_url);
                        let _ = tx_out.send(res);
                        ctx.request_repaint();
                    }
                }
            }
        });

    UpdaterHandle {
        rx: rx_out,
        tx: tx_in,
    }
}

/// Memanggil GitHub API untuk memeriksa rilis terbaru.
pub fn check_github() -> UpdateStatus {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(6))
        .user_agent("Noktah-App")
        .build();

    let resp = match agent
        .get("https://api.github.com/repos/AffanShz/noktah/releases/latest")
        .set("Accept", "application/vnd.github.v3+json")
        .call()
    {
        Ok(r) => r,
        Err(ureq::Error::Status(status, _)) => {
            let err = UpdateError::HttpStatus { status };
            crate::infrastructure::logger::warn("updater", &err.log_detail());
            return UpdateStatus::Error(err);
        }
        Err(ureq::Error::Transport(t)) => {
            let err = UpdateError::GitHubUnreachable(t.to_string());
            crate::infrastructure::logger::warn("updater", &err.log_detail());
            return UpdateStatus::Error(err);
        }
    };

    let release: ReleaseInfo = match resp.into_json() {
        Ok(rel) => rel,
        Err(e) => {
            let err = UpdateError::InvalidReleaseJson(e.to_string());
            crate::infrastructure::logger::warn("updater", &err.log_detail());
            return UpdateStatus::Error(err);
        }
    };

    let current = env!("CARGO_PKG_VERSION");
    let remote_ver = release.tag_name.trim_start_matches('v').trim();

    if is_newer_version(remote_ver, current) {
        let tag = release.tag_name;
        crate::infrastructure::logger::info("updater", &format!("Update available: {tag}"));
        let name = release.name.unwrap_or_else(|| format!("Rilis {tag}"));
        let zip_url = release
            .assets
            .iter()
            .find(|a| a.name.ends_with("-win64.zip"))
            .or_else(|| release.assets.iter().find(|a| a.name.ends_with(".zip")))
            .map(|a| a.browser_download_url.clone());
        UpdateStatus::UpdateAvailable {
            version: tag,
            name,
            body: release.body.unwrap_or_default(),
            url: release.html_url,
            zip_url,
        }
    } else {
        UpdateStatus::LatestVersion(format!("v{current}"))
    }
}

/// Mengekstrak hash SHA-256 `(noktah-native.exe, noktah-hook.exe)` dari berkas `BACA-DULU.txt`.
pub fn parse_baca_dulu_hashes(text: &str) -> (Option<String>, Option<String>) {
    let mut found = Vec::new();
    for line in text.lines() {
        if !line.to_ascii_lowercase().contains("sha256") {
            continue;
        }
        for tok in line.split(|c: char| c.is_whitespace() || c == ':') {
            if tok.len() == 64 && tok.bytes().all(|b| b.is_ascii_hexdigit()) {
                found.push(tok.to_ascii_lowercase());
                break;
            }
        }
    }
    (found.first().cloned(), found.get(1).cloned())
}

pub(crate) fn verify_file_sha256(path: &Path, expected_hex: &str) -> Result<(), UpdateError> {
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    let bytes = std::fs::read(path).map_err(|e| UpdateError::BinaryReadFailed {
        file: file_name.clone(),
        detail: e.to_string(),
    })?;
    let actual = crate::infrastructure::quota::sha256::hex(
        &crate::infrastructure::quota::sha256::digest(&bytes),
    );
    if actual.eq_ignore_ascii_case(expected_hex) {
        Ok(())
    } else {
        Err(UpdateError::Sha256Mismatch { file: file_name })
    }
}

/// Memeriksa berkas hasil ekstraksi di `extracted_dir`, memverifikasi SHA-256
/// terhadap `BACA-DULU.txt` bila tersedia, dan menentukan status `ReadyToRestart`.
pub(crate) fn stage_extracted_files(
    version: &str,
    stage_dir: &Path,
    extracted_dir: &Path,
) -> UpdateStatus {
    let staged_exe = extracted_dir.join("noktah-native.exe");
    if !staged_exe.is_file() {
        let _ = std::fs::remove_dir_all(stage_dir);
        return UpdateStatus::Error(UpdateError::AssetMissing("noktah-native.exe".into()));
    }
    let exe_len = std::fs::metadata(&staged_exe).map(|m| m.len()).unwrap_or(0);
    if exe_len <= 1_000_000 {
        let _ = std::fs::remove_dir_all(stage_dir);
        return UpdateStatus::Error(UpdateError::InvalidBinary {
            file: "noktah-native.exe".into(),
        });
    }

    let hook_candidate = extracted_dir.join("noktah-hook.exe");
    let staged_hook = if hook_candidate.is_file() {
        Some(hook_candidate)
    } else {
        None
    };

    let mut sha256_verified = false;
    let baca_path = extracted_dir.join("BACA-DULU.txt");
    if baca_path.is_file() {
        if let Ok(baca) = std::fs::read_to_string(&baca_path) {
            let (expected_exe, expected_hook) = parse_baca_dulu_hashes(&baca);
            if let Some(exp) = expected_exe.as_deref() {
                if let Err(err) = verify_file_sha256(&staged_exe, exp) {
                    let _ = std::fs::remove_dir_all(stage_dir);
                    crate::infrastructure::logger::error("updater", &err.log_detail());
                    return UpdateStatus::Error(err);
                }
            }
            if let (Some(hook), Some(exp)) = (staged_hook.as_ref(), expected_hook.as_deref()) {
                if let Err(err) = verify_file_sha256(hook, exp) {
                    let _ = std::fs::remove_dir_all(stage_dir);
                    crate::infrastructure::logger::error("updater", &err.log_detail());
                    return UpdateStatus::Error(err);
                }
            }
            let exe_verified = expected_exe.is_some();
            let hook_verified = staged_hook.is_none() || expected_hook.is_some();
            sha256_verified = exe_verified && hook_verified;
        }
    }

    crate::infrastructure::logger::info(
        "updater",
        &format!("Staged update {version} (sha256_verified={sha256_verified})"),
    );
    UpdateStatus::ReadyToRestart {
        version: version.to_string(),
        staged_exe,
        staged_hook,
        sha256_verified,
    }
}

/// Mengunduh arsip rilis, mengekstrak, dan memverifikasi SHA-256 di folder staging.
pub fn download_and_stage(version: &str, zip_url: &str) -> UpdateStatus {
    const ALLOWED_PREFIX: &str = "https://github.com/AffanShz/noktah/releases/download/";
    if !zip_url.starts_with(ALLOWED_PREFIX) {
        return UpdateStatus::Error(UpdateError::UntrustedDownloadUrl);
    }

    let safe_ver: String = version
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect();
    let safe_ver = if safe_ver.is_empty() {
        "latest".to_string()
    } else {
        safe_ver
    };

    let stage_dir = crate::infrastructure::store::data_dir()
        .join("updates")
        .join(&safe_ver);
    let extracted_dir = stage_dir.join("extracted");
    let _ = std::fs::remove_dir_all(&stage_dir);
    if let Err(e) = std::fs::create_dir_all(&extracted_dir) {
        return UpdateStatus::Error(UpdateError::StagingDirFailed(e.to_string()));
    }

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(60))
        .user_agent("Noktah-App")
        .build();

    let resp = match agent.get(zip_url).call() {
        Ok(r) => r,
        Err(ureq::Error::Status(status, _)) => {
            let _ = std::fs::remove_dir_all(&stage_dir);
            let err = UpdateError::HttpStatus { status };
            crate::infrastructure::logger::warn("updater", &err.log_detail());
            return UpdateStatus::Error(err);
        }
        Err(ureq::Error::Transport(t)) => {
            let _ = std::fs::remove_dir_all(&stage_dir);
            let err = UpdateError::DownloadFailed(t.to_string());
            crate::infrastructure::logger::warn("updater", &err.log_detail());
            return UpdateStatus::Error(err);
        }
    };

    let mut bytes = Vec::new();
    let mut reader = resp.into_reader().take((MAX_RELEASE_BYTES + 1) as u64);
    if let Err(e) = reader.read_to_end(&mut bytes) {
        let _ = std::fs::remove_dir_all(&stage_dir);
        return UpdateStatus::Error(UpdateError::ArchiveReadFailed(e.to_string()));
    }
    if bytes.len() > MAX_RELEASE_BYTES {
        let _ = std::fs::remove_dir_all(&stage_dir);
        return UpdateStatus::Error(UpdateError::ArchiveTooLarge {
            max_mb: MAX_RELEASE_BYTES / (1024 * 1024),
        });
    }

    let zip_path = stage_dir.join("update.zip");
    if let Err(e) = std::fs::write(&zip_path, &bytes) {
        let _ = std::fs::remove_dir_all(&stage_dir);
        return UpdateStatus::Error(UpdateError::ArchiveWriteFailed(e.to_string()));
    }

    let mut cmd = std::process::Command::new("tar");
    cmd.arg("-xf")
        .arg(&zip_path)
        .arg("-C")
        .arg(&extracted_dir);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    match cmd.status() {
        Ok(st) if st.success() => {}
        Ok(st) => {
            let _ = std::fs::remove_dir_all(&stage_dir);
            return UpdateStatus::Error(UpdateError::ExtractFailed { code: st.code() });
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&stage_dir);
            return UpdateStatus::Error(UpdateError::ExtractSpawnFailed(e.to_string()));
        }
    }

    stage_extracted_files(version, &stage_dir, &extracted_dir)
}

fn ps_quote(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}

/// Menjalankan skrip pengganti biner setelah proses saat ini keluar lalu membuka versi baru.
pub fn apply_staged_and_restart(
    staged_exe: &Path,
    staged_hook: Option<&Path>,
) -> Result<(), UpdateError> {
    if !staged_exe.is_file() {
        return Err(UpdateError::AssetMissing("noktah-native.exe".into()));
    }
    if let Some(hook_src) = staged_hook {
        if !hook_src.is_file() {
            return Err(UpdateError::HookUpdateFailed("noktah-hook.exe".into()));
        }
    }
    let current_exe =
        std::env::current_exe().map_err(|e| UpdateError::InstallError(e.to_string()))?;
    let pid = std::process::id();

    let hook_block = if let Some(hook_src) = staged_hook {
        let dir = current_exe
            .parent()
            .ok_or_else(|| UpdateError::HookUpdateFailed("missing target directory".into()))?;
        let hook_dst = dir.join("noktah-hook.exe");
        format!(
            "if (Test-Path '{}') {{\n    Copy-Item -LiteralPath '{}' -Destination '{}' -Force\n}}\n",
            ps_quote(&hook_dst),
            ps_quote(hook_src),
            ps_quote(&hook_dst),
        )
    } else {
        String::new()
    };

    let script_content = format!(
        "Wait-Process -Id {pid} -Timeout 10 -ErrorAction SilentlyContinue\n\
         Copy-Item -LiteralPath '{src}' -Destination '{dst}' -Force\n\
         {hook_block}\
         Start-Process -FilePath '{dst}'\n",
        src = ps_quote(staged_exe),
        dst = ps_quote(&current_exe),
    );

    let updates_dir = crate::infrastructure::store::data_dir().join("updates");
    let _ = std::fs::create_dir_all(&updates_dir);
    let script_path = updates_dir.join("apply-update.ps1");
    std::fs::write(&script_path, script_content)
        .map_err(|e| UpdateError::InstallError(e.to_string()))?;

    let mut cmd = std::process::Command::new("powershell.exe");
    cmd.args([
        "-NoProfile",
        "-WindowStyle",
        "Hidden",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
    ])
    .arg(&script_path);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    cmd.spawn()
        .map_err(|e| UpdateError::InstallError(e.to_string()))?;

    crate::infrastructure::logger::info("updater", "Applying staged update and restarting");
    std::process::exit(0);
}

/// Bandingkan semver sederhana x.y.z
pub fn is_newer_version(remote: &str, current: &str) -> bool {
    let parse_parts = |s: &str| -> Vec<u32> {
        s.split(|c: char| c == '.' || c == '-')
            .filter_map(|p| p.parse::<u32>().ok())
            .collect()
    };
    let r_parts = parse_parts(remote);
    let c_parts = parse_parts(current);

    for (r, c) in r_parts.iter().zip(c_parts.iter()) {
        if r > c {
            return true;
        } else if r < c {
            return false;
        }
    }
    r_parts.len() > c_parts.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version_standard() {
        assert!(is_newer_version("1.2.0", "1.1.0"));
        assert!(is_newer_version("2.0.0", "1.9.9"));
        assert!(is_newer_version("1.1.1", "1.1.0"));
        assert!(!is_newer_version("1.1.0", "1.1.0"));
        assert!(!is_newer_version("1.0.9", "1.1.0"));
        assert!(!is_newer_version("0.9.5", "1.1.0"));
    }

    #[test]
    fn test_is_newer_version_extra_components() {
        assert!(is_newer_version("1.1.0.1", "1.1.0"));
        assert!(!is_newer_version("1.1.0", "1.1.0.1"));
    }

    #[test]
    fn test_parse_baca_dulu_hashes_dan_verifikasi() {
        let sample_ci = "\
File     : noktah-native.exe
SHA256   : ab607eb7ee09bbe570e571e9da3b8341f31c01574b187c5ee27b81f0a445571f
Pendamping: noktah-hook.exe
SHA256   : b5ac75adc9a167eb3df20ff574d514a28da97109e0c6c39fb017f74cf2c656b3
";
        let (exe_h, hook_h) = parse_baca_dulu_hashes(sample_ci);
        assert_eq!(
            exe_h.as_deref(),
            Some("ab607eb7ee09bbe570e571e9da3b8341f31c01574b187c5ee27b81f0a445571f")
        );
        assert_eq!(
            hook_h.as_deref(),
            Some("b5ac75adc9a167eb3df20ff574d514a28da97109e0c6c39fb017f74cf2c656b3")
        );

        // Uji verifikasi hash nyata pada berkas sementara
        let tmp = std::env::temp_dir().join(format!("noktah_sha_test_{}.bin", std::process::id()));
        std::fs::write(&tmp, b"hello noktah").unwrap();
        let expected = crate::infrastructure::quota::sha256::hex(
            &crate::infrastructure::quota::sha256::digest(b"hello noktah"),
        );
        assert!(verify_file_sha256(&tmp, &expected).is_ok());
        assert_eq!(
            verify_file_sha256(&tmp, &"0".repeat(64)),
            Err(UpdateError::Sha256Mismatch {
                file: format!("noktah_sha_test_{}.bin", std::process::id())
            })
        );
        let _ = std::fs::remove_file(&tmp);
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        crate::infrastructure::quota::sha256::hex(&crate::infrastructure::quota::sha256::digest(
            bytes,
        ))
    }

    fn temp_stage_dir(label: &str) -> (PathBuf, PathBuf) {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let stage_dir = std::env::temp_dir().join(format!(
            "noktah_updater_stage_{label}_{}_{nanos}",
            std::process::id()
        ));
        let extracted_dir = stage_dir.join("extracted");
        std::fs::create_dir_all(&extracted_dir).unwrap();
        (stage_dir, extracted_dir)
    }

    #[test]
    fn test_stage_extracted_files_sha256_verified_true_and_false_and_mismatch() {
        let exe_bytes = vec![0x5Au8; 1_000_001];
        let hook_bytes = b"hook-binary-payload".to_vec();
        let exe_hash = sha256_hex(&exe_bytes);
        let hook_hash = sha256_hex(&hook_bytes);

        // 1. Both exe and hook present & hashes match -> sha256_verified == true
        let (stage1, ext1) = temp_stage_dir("verified_both");
        std::fs::write(ext1.join("noktah-native.exe"), &exe_bytes).unwrap();
        std::fs::write(ext1.join("noktah-hook.exe"), &hook_bytes).unwrap();
        std::fs::write(
            ext1.join("BACA-DULU.txt"),
            format!("File : noktah-native.exe\nSHA256 : {exe_hash}\nPendamping: noktah-hook.exe\nSHA256 : {hook_hash}\n"),
        )
        .unwrap();
        let res1 = stage_extracted_files("v1.6.1", &stage1, &ext1);
        assert!(matches!(
            res1,
            UpdateStatus::ReadyToRestart {
                sha256_verified: true,
                staged_hook: Some(_),
                ..
            }
        ));
        let _ = std::fs::remove_dir_all(&stage1);

        // 2. BACA-DULU.txt missing -> ReadyToRestart { sha256_verified: false }
        let (stage2, ext2) = temp_stage_dir("missing_baca");
        std::fs::write(ext2.join("noktah-native.exe"), &exe_bytes).unwrap();
        let res2 = stage_extracted_files("v1.6.1", &stage2, &ext2);
        assert!(matches!(
            res2,
            UpdateStatus::ReadyToRestart {
                sha256_verified: false,
                staged_hook: None,
                ..
            }
        ));
        let _ = std::fs::remove_dir_all(&stage2);

        // 3. Hook binary present but BACA-DULU.txt has only exe hash -> sha256_verified == false
        let (stage3, ext3) = temp_stage_dir("missing_hook_hash");
        std::fs::write(ext3.join("noktah-native.exe"), &exe_bytes).unwrap();
        std::fs::write(ext3.join("noktah-hook.exe"), &hook_bytes).unwrap();
        std::fs::write(
            ext3.join("BACA-DULU.txt"),
            format!("File : noktah-native.exe\nSHA256 : {exe_hash}\n"),
        )
        .unwrap();
        let res3 = stage_extracted_files("v1.6.1", &stage3, &ext3);
        assert!(matches!(
            res3,
            UpdateStatus::ReadyToRestart {
                sha256_verified: false,
                staged_hook: Some(_),
                ..
            }
        ));
        let _ = std::fs::remove_dir_all(&stage3);

        // 4. Exe hash mismatch -> UpdateStatus::Error(Sha256Mismatch) and stage_dir cleaned up
        let (stage4, ext4) = temp_stage_dir("mismatch_exe");
        std::fs::write(ext4.join("noktah-native.exe"), &exe_bytes).unwrap();
        std::fs::write(
            ext4.join("BACA-DULU.txt"),
            format!("File : noktah-native.exe\nSHA256 : {}\n", "0".repeat(64)),
        )
        .unwrap();
        let res4 = stage_extracted_files("v1.6.1", &stage4, &ext4);
        assert_eq!(
            res4,
            UpdateStatus::Error(UpdateError::Sha256Mismatch {
                file: "noktah-native.exe".into()
            })
        );
        assert!(!stage4.exists(), "stage_dir must be removed on checksum mismatch");

        // 5. Hook hash mismatch -> UpdateStatus::Error(Sha256Mismatch) and stage_dir cleaned up
        let (stage5, ext5) = temp_stage_dir("mismatch_hook");
        std::fs::write(ext5.join("noktah-native.exe"), &exe_bytes).unwrap();
        std::fs::write(ext5.join("noktah-hook.exe"), &hook_bytes).unwrap();
        std::fs::write(
            ext5.join("BACA-DULU.txt"),
            format!(
                "File : noktah-native.exe\nSHA256 : {exe_hash}\nPendamping: noktah-hook.exe\nSHA256 : {}\n",
                "f".repeat(64)
            ),
        )
        .unwrap();
        let res5 = stage_extracted_files("v1.6.1", &stage5, &ext5);
        assert_eq!(
            res5,
            UpdateStatus::Error(UpdateError::Sha256Mismatch {
                file: "noktah-hook.exe".into()
            })
        );
        assert!(!stage5.exists(), "stage_dir must be removed on hook checksum mismatch");
    }
}
