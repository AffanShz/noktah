//! Pemeriksa pembaruan aplikasi Noktah dari GitHub Releases API.
//!
//! Dijalankan secara asinkron di thread terpisah agar tidak memblokir antarmuka UI.
//! Jika ada rilis baru, status diperbarui dengan informasi versi, catatan rilis (changelog),
//! dan URL download.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

/// Informasi rilis yang dikembalikan GitHub Releases API.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub html_url: String,
    pub published_at: Option<String>,
}

/// Status pengecekan pembaruan.
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
    },
    Error(String),
}

/// Handle komunikasi antara thread UI egui dan thread background updater.
pub struct UpdaterHandle {
    pub rx: Receiver<UpdateStatus>,
    pub tx: Sender<()>,
}

/// Memulai background thread untuk memeriksa pembaruan saat diminta.
pub fn spawn(ctx: egui::Context) -> UpdaterHandle {
    let (tx_out, rx_out) = channel();
    let (tx_in, rx_in) = channel::<()>();

    let _ = std::thread::Builder::new()
        .name("noktah-updater".into())
        .stack_size(256 * 1024)
        .spawn(move || {
            while let Ok(()) = rx_in.recv() {
                let _ = tx_out.send(UpdateStatus::Checking);
                ctx.request_repaint();

                let res = check_github();
                let _ = tx_out.send(res);
                ctx.request_repaint();
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
        Err(e) => {
            let msg = format!("Gagal menghubungi GitHub: {e}");
            crate::infrastructure::logger::warn("updater", &msg);
            return UpdateStatus::Error(msg);
        }
    };

    let release: ReleaseInfo = match resp.into_json() {
        Ok(rel) => rel,
        Err(e) => {
            let msg = format!("Gagal mengurai respons: {e}");
            crate::infrastructure::logger::warn("updater", &msg);
            return UpdateStatus::Error(msg);
        }
    };

    let current = env!("CARGO_PKG_VERSION");
    let remote_ver = release.tag_name.trim_start_matches('v').trim();

    if is_newer_version(remote_ver, current) {
        let tag = release.tag_name;
        crate::infrastructure::logger::info("updater", &format!("Update available: {tag}"));
        let name = release.name.unwrap_or_else(|| format!("Rilis {tag}"));
        UpdateStatus::UpdateAvailable {
            version: tag,
            name,
            body: release.body.unwrap_or_default(),
            url: release.html_url,
        }
    } else {
        UpdateStatus::LatestVersion(format!("v{current}"))
    }
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
}
