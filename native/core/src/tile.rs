//! Model dan penyimpanan status tile kustom dari noktah-hook.
//!
//! Logika murni: tidak ada dependensi ke UI (`egui`) atau OS (`windows`).

use serde::{Deserialize, Serialize};

/// Warna atau nuansa visual sebuah tile status.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TileTone {
    Neutral,
    Accent,
    Success,
    Warning,
    Error,
}

impl Default for TileTone {
    fn default() -> Self {
        TileTone::Neutral
    }
}

impl TileTone {
    /// Parsing nilai tone dari string bebas case-insensitive.
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "accent" | "info" | "blue" => TileTone::Accent,
            "success" | "ok" | "green" => TileTone::Success,
            "warning" | "warn" | "amber" | "yellow" => TileTone::Warning,
            "error" | "err" | "fail" | "danger" | "red" => TileTone::Error,
            _ => TileTone::Neutral,
        }
    }
}

/// Potong string secara aman pada batas karakter UTF-8.
pub fn truncate_chars(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

/// Satu kartu metrik / status kustom.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct CustomTile {
    pub id: String,
    pub title: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub tone: TileTone,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl_secs: Option<u64>,
    #[serde(default)]
    pub updated_at_ms: i64,
    #[serde(default)]
    pub pin_mini: bool,
}

impl Default for CustomTile {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            value: String::new(),
            subtitle: None,
            tone: TileTone::default(),
            progress: None,
            ttl_secs: None,
            updated_at_ms: 0,
            pin_mini: false,
        }
    }
}

impl CustomTile {
    /// Normalisasi batas karakter, nilai default id/title, pembersihan progress & TTL.
    pub fn normalized(mut self, now_ms: i64) -> Self {
        let id_trimmed = self.id.trim();
        self.id = if id_trimmed.is_empty() {
            "tile".to_string()
        } else {
            truncate_chars(id_trimmed, 32)
        };

        let title_trimmed = self.title.trim();
        self.title = if title_trimmed.is_empty() {
            truncate_chars(&self.id, 32)
        } else {
            truncate_chars(title_trimmed, 32)
        };

        self.value = truncate_chars(self.value.trim(), 48);

        self.subtitle = self
            .subtitle
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| truncate_chars(s, 64));

        self.progress = self
            .progress
            .filter(|p| p.is_finite())
            .map(|p| p.clamp(0.0, 1.0));

        self.ttl_secs = self.ttl_secs.map(|t| t.min(86_400));

        self.updated_at_ms = if self.updated_at_ms > 0 {
            self.updated_at_ms
        } else {
            now_ms
        };

        self
    }

    /// Kapan tile kedaluwarsa dalam milidetik epoch (jika memiliki TTL).
    pub fn expires_at_ms(&self) -> Option<i64> {
        self.ttl_secs
            .map(|ttl| self.updated_at_ms.saturating_add((ttl as i64).saturating_mul(1000)))
    }

    /// Apakah tile telah melewati batas TTL pada waktu `now_ms`.
    pub fn is_expired(&self, now_ms: i64) -> bool {
        self.expires_at_ms().map_or(false, |exp| now_ms >= exp)
    }
}

/// Penyimpanan in-memory daftar tile status kustom.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TileStore {
    items: Vec<CustomTile>,
}

impl TileStore {
    pub const MAX_TILES: usize = 6;

    /// Tambah atau perbarui tile.
    ///
    /// Menghapus tile kedaluwarsa terlebih dahulu.
    /// Mengembalikan false jika tile tidak memiliki isi (value, subtitle, dan progress semuanya kosong).
    /// Jika tile baru dan kapasitas penuh, mengeluarkan item paling lama (berdasarkan `updated_at_ms`).
    pub fn upsert(&mut self, tile: CustomTile, now_ms: i64) -> bool {
        self.prune_expired(now_ms);
        let tile = tile.normalized(now_ms);

        if tile.value.is_empty() && tile.subtitle.is_none() && tile.progress.is_none() {
            return false;
        }

        if let Some(pos) = self.items.iter().position(|t| t.id == tile.id) {
            self.items[pos] = tile;
            return true;
        }

        if self.items.len() >= Self::MAX_TILES {
            if let Some((idx, _)) = self
                .items
                .iter()
                .enumerate()
                .min_by_key(|(_, item)| item.updated_at_ms)
            {
                self.items.remove(idx);
            }
        }

        self.items.push(tile);
        true
    }

    /// Hapus tile berdasarkan ID. Mengembalikan true jika ditemukan dan dihapus.
    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|t| t.id != id);
        self.items.len() < before
    }

    /// Bersihkan semua tile.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Bersihkan tile yang sudah kedaluwarsa. Mengembalikan true jika ada yang dihapus.
    pub fn prune_expired(&mut self, now_ms: i64) -> bool {
        let before = self.items.len();
        self.items.retain(|t| !t.is_expired(now_ms));
        self.items.len() < before
    }

    /// Berapa milidetik lagi sampai tile tercepat berikutnya kedaluwarsa.
    pub fn next_expiry_in_ms(&self, now_ms: i64) -> Option<u64> {
        self.items
            .iter()
            .filter_map(|t| {
                t.expires_at_ms().map(|exp| {
                    if exp > now_ms {
                        (exp - now_ms) as u64
                    } else {
                        0
                    }
                })
            })
            .min()
    }

    /// Mengambil tile yang disematkan ke mini pill (paling baru di-update dengan pin_mini = true).
    pub fn pinned_mini(&self) -> Option<&CustomTile> {
        self.items
            .iter()
            .filter(|t| t.pin_mini)
            .max_by_key(|t| t.updated_at_ms)
    }

    pub fn items(&self) -> &[CustomTile] {
        &self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_tone_parsing() {
        assert_eq!(TileTone::parse("accent"), TileTone::Accent);
        assert_eq!(TileTone::parse("Info"), TileTone::Accent);
        assert_eq!(TileTone::parse("BLUE"), TileTone::Accent);
        assert_eq!(TileTone::parse("success"), TileTone::Success);
        assert_eq!(TileTone::parse("ok"), TileTone::Success);
        assert_eq!(TileTone::parse("GREEN"), TileTone::Success);
        assert_eq!(TileTone::parse("warning"), TileTone::Warning);
        assert_eq!(TileTone::parse("warn"), TileTone::Warning);
        assert_eq!(TileTone::parse("Amber"), TileTone::Warning);
        assert_eq!(TileTone::parse("yellow"), TileTone::Warning);
        assert_eq!(TileTone::parse("error"), TileTone::Error);
        assert_eq!(TileTone::parse("FAIL"), TileTone::Error);
        assert_eq!(TileTone::parse("danger"), TileTone::Error);
        assert_eq!(TileTone::parse("red"), TileTone::Error);
        assert_eq!(TileTone::parse("unknown"), TileTone::Neutral);
        assert_eq!(TileTone::parse(""), TileTone::Neutral);
    }

    #[test]
    fn utf8_truncate_chars() {
        let ascii = "1234567890";
        assert_eq!(truncate_chars(ascii, 5), "12345");

        let cjk = "こんにちは世界";
        assert_eq!(truncate_chars(cjk, 4), "こんにち");

        let mixed = "Build 🚀 #42 Selesai";
        assert_eq!(truncate_chars(mixed, 7), "Build 🚀");
        assert_eq!(truncate_chars(mixed, 8), "Build 🚀 ");
    }

    #[test]
    fn normalization_fallbacks_and_limits() {
        let t = CustomTile {
            id: "   ".into(),
            title: "".into(),
            value: "   85%   ".into(),
            subtitle: Some("  Active Worker  ".into()),
            tone: TileTone::Success,
            progress: Some(1.5),
            ttl_secs: Some(100_000),
            updated_at_ms: 0,
            pin_mini: true,
        };
        let norm = t.normalized(10_000);
        assert_eq!(norm.id, "tile");
        assert_eq!(norm.title, "tile");
        assert_eq!(norm.value, "85%");
        assert_eq!(norm.subtitle.as_deref(), Some("Active Worker"));
        assert_eq!(norm.progress, Some(1.0));
        assert_eq!(norm.ttl_secs, Some(86_400));
        assert_eq!(norm.updated_at_ms, 10_000);
        assert!(norm.pin_mini);
    }

    #[test]
    fn upsert_replaces_in_place_and_preserves_order() {
        let mut store = TileStore::default();
        let now = 1_000;

        let t1 = CustomTile {
            id: "cpu".into(),
            title: "CPU".into(),
            value: "10%".into(),
            ..Default::default()
        };
        let t2 = CustomTile {
            id: "ram".into(),
            title: "RAM".into(),
            value: "4GB".into(),
            ..Default::default()
        };
        assert!(store.upsert(t1, now));
        assert!(store.upsert(t2, now));
        assert_eq!(store.len(), 2);
        assert_eq!(store.items()[0].id, "cpu");
        assert_eq!(store.items()[1].id, "ram");

        // Update t1
        let t1_updated = CustomTile {
            id: "cpu".into(),
            title: "CPU".into(),
            value: "95%".into(),
            tone: TileTone::Error,
            ..Default::default()
        };
        assert!(store.upsert(t1_updated, now + 100));
        assert_eq!(store.len(), 2);
        assert_eq!(store.items()[0].id, "cpu");
        assert_eq!(store.items()[0].value, "95%");
        assert_eq!(store.items()[0].tone, TileTone::Error);
        assert_eq!(store.items()[1].id, "ram");
    }

    #[test]
    fn upsert_empty_content_returns_false() {
        let mut store = TileStore::default();
        let empty = CustomTile {
            id: "test".into(),
            title: "Test".into(),
            value: "".into(),
            subtitle: None,
            progress: None,
            ..Default::default()
        };
        assert!(!store.upsert(empty, 1000));
        assert!(store.is_empty());
    }

    #[test]
    fn max_cap_eviction_oldest_first() {
        let mut store = TileStore::default();
        for i in 1..=TileStore::MAX_TILES {
            let t = CustomTile {
                id: format!("t{i}"),
                title: format!("T{i}"),
                value: format!("{i}"),
                updated_at_ms: i as i64 * 100,
                ..Default::default()
            };
            assert!(store.upsert(t, i as i64 * 100));
        }
        assert_eq!(store.len(), TileStore::MAX_TILES);
        assert_eq!(store.items()[0].id, "t1");

        // Upsert 7th tile: t1 has oldest updated_at_ms (100) -> should be evicted
        let t7 = CustomTile {
            id: "t7".into(),
            title: "T7".into(),
            value: "7".into(),
            updated_at_ms: 700,
            ..Default::default()
        };
        assert!(store.upsert(t7, 700));
        assert_eq!(store.len(), TileStore::MAX_TILES);
        assert!(store.items().iter().all(|t| t.id != "t1"));
        assert_eq!(store.items().last().unwrap().id, "t7");
    }

    #[test]
    fn ttl_expiry_and_prune() {
        let mut store = TileStore::default();
        let now = 10_000;

        let expiring = CustomTile {
            id: "temp".into(),
            title: "Temp".into(),
            value: "val".into(),
            ttl_secs: Some(5), // expires at 15_000
            updated_at_ms: now,
            ..Default::default()
        };
        let permanent = CustomTile {
            id: "perm".into(),
            title: "Perm".into(),
            value: "val".into(),
            ttl_secs: None,
            updated_at_ms: now,
            ..Default::default()
        };

        store.upsert(expiring, now);
        store.upsert(permanent, now);
        assert_eq!(store.len(), 2);

        // Before expiry
        assert!(!store.prune_expired(14_999));
        assert_eq!(store.len(), 2);

        // At/after expiry
        assert!(store.prune_expired(15_000));
        assert_eq!(store.len(), 1);
        assert_eq!(store.items()[0].id, "perm");
    }

    #[test]
    fn next_expiry_in_ms() {
        let mut store = TileStore::default();
        let now = 10_000;

        assert_eq!(store.next_expiry_in_ms(now), None);

        let t1 = CustomTile {
            id: "t1".into(),
            value: "1".into(),
            ttl_secs: Some(10), // expires at 20_000
            updated_at_ms: now,
            ..Default::default()
        };
        let t2 = CustomTile {
            id: "t2".into(),
            value: "2".into(),
            ttl_secs: Some(25), // expires at 35_000
            updated_at_ms: now,
            ..Default::default()
        };
        store.upsert(t1, now);
        store.upsert(t2, now);

        assert_eq!(store.next_expiry_in_ms(now), Some(10_000));
        assert_eq!(store.next_expiry_in_ms(now + 3_000), Some(7_000));
        assert_eq!(store.next_expiry_in_ms(now + 12_000), Some(0)); // t1 already expired
    }

    #[test]
    fn pinned_mini_selection() {
        let mut store = TileStore::default();
        let now = 1_000;

        assert!(store.pinned_mini().is_none());

        let t1 = CustomTile {
            id: "t1".into(),
            value: "1".into(),
            pin_mini: true,
            updated_at_ms: now,
            ..Default::default()
        };
        let t2 = CustomTile {
            id: "t2".into(),
            value: "2".into(),
            pin_mini: false,
            updated_at_ms: now + 500,
            ..Default::default()
        };
        let t3 = CustomTile {
            id: "t3".into(),
            value: "3".into(),
            pin_mini: true,
            updated_at_ms: now + 300,
            ..Default::default()
        };

        store.upsert(t1, now);
        store.upsert(t2, now + 500);
        store.upsert(t3, now + 300);

        // t3 is pinned and has higher updated_at_ms than t1
        let pinned = store.pinned_mini().unwrap();
        assert_eq!(pinned.id, "t3");
    }

    #[test]
    fn serde_json_roundtrip() {
        let tile = CustomTile {
            id: "ci-status".into(),
            title: "CI Run".into(),
            value: "Passed".into(),
            subtitle: Some("main #142".into()),
            tone: TileTone::Success,
            progress: Some(0.85),
            ttl_secs: Some(3600),
            updated_at_ms: 123456789,
            pin_mini: true,
        };

        let json = serde_json::to_string(&tile).expect("serialize failed");
        let decoded: CustomTile = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(tile, decoded);
    }
}
