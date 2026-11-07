//! Pencocokan lagu dengan hasil pencarian lirik.
//!
//! Pencarian teks bebas sering mengembalikan versi yang salah: remaster,
//! rekaman live, atau lagu lain dengan judul mirip. Karena itu hasil diberi
//! skor dari judul, artis, dan durasi — bukan diambil yang pertama.

#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub title: String,
    pub artist: String,
    /// detik; 0 berarti tidak diketahui
    pub duration: i64,
}

/// Kata yang menandakan versi berbeda dari rekaman aslinya.
const SUFFIX_NOISE: [&str; 8] = [
    "remaster",
    "remastered",
    "version",
    "live",
    "acoustic",
    "edit",
    "mono",
    "stereo",
];

/// Kata kunci noise umum dalam judul video/audio SMTC untuk dibersihkan saat request API lirik.
const NOISE_KEYWORDS: [&str; 14] = [
    "official",
    "video",
    "audio",
    "lyrics",
    "lyric",
    "visualizer",
    "hd",
    "4k",
    "remaster",
    "remastered",
    "mv",
    "m/v",
    "feat",
    "ft.",
];

/// Membersihkan embel-embel judul SMTC/YouTube untuk pencarian lirik di API.
///
/// Berbeda dengan `normalize` yang membuat string jadi huruf kecil dan tanpa tanda baca,
/// fungsi ini mempertahankan kapitalisasi asli dan tanda baca nama lagu, tapi membuang
/// noise kurung/suffix video/audio seperti "(Official Music Video)", "[MV]", "(feat. X)".
pub fn clean_search_title(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // 1. Buang suffix pipa: "Judul | Official Music Video"
    let mut s = if let Some(idx) = trimmed.find('|') {
        let after = trimmed[idx + 1..].to_lowercase();
        if NOISE_KEYWORDS.iter().any(|k| after.contains(k)) {
            trimmed[..idx].trim()
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    // 2. Buang suffix strip: "Judul - Official Video" / "Judul - Lyric Video"
    if let Some(idx) = s.rfind(" - ") {
        let after = s[idx + 3..].to_lowercase();
        if NOISE_KEYWORDS.iter().any(|k| after.contains(k)) {
            s = s[..idx].trim();
        }
    }

    // 3. Buang kurung yang mengandung noise: (Official Video), [MV], (feat. X), dll.
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        let close_char = match c {
            '(' => Some(')'),
            '[' => Some(']'),
            '【' => Some('】'),
            _ => None,
        };

        if let Some(close) = close_char {
            let mut inside = String::new();
            let mut found_close = false;
            for inner in chars.by_ref() {
                if inner == close {
                    found_close = true;
                    break;
                }
                inside.push(inner);
            }

            let lower = inside.to_lowercase();
            let is_noise = NOISE_KEYWORDS.iter().any(|k| lower.contains(k));
            if !is_noise {
                out.push(c);
                out.push_str(&inside);
                if found_close {
                    out.push(close);
                }
            }
        } else {
            out.push(c);
        }
    }

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Huruf kecil, tanpa tanda baca, tanpa keterangan versi.
pub fn normalize(input: &str) -> String {
    let lower = input.to_lowercase();

    // Buang isi kurung: "(feat. X)", "[Remastered 2011]".
    let mut tanpa_kurung = String::with_capacity(lower.len());
    let mut depth = 0usize;
    for c in lower.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => tanpa_kurung.push(c),
            _ => {}
        }
    }

    // Buang embel-embel setelah tanda hubung: "Lagu - Remastered 2011".
    let mut bersih = String::new();
    for (i, bagian) in tanpa_kurung.split(" - ").enumerate() {
        if i == 0 {
            bersih.push_str(bagian);
        } else if !SUFFIX_NOISE.iter().any(|n| bagian.contains(n)) {
            bersih.push(' ');
            bersih.push_str(bagian);
        }
    }

    bersih
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Kemiripan dua teks, 0.0–1.0, berbasis irisan kata.
pub fn similarity(a: &str, b: &str) -> f32 {
    let ka = normalize(a);
    let kb = normalize(b);
    if ka.is_empty() || kb.is_empty() {
        return 0.0;
    }
    if ka == kb {
        return 1.0;
    }

    let set_a: std::collections::HashSet<&str> = ka.split_whitespace().collect();
    let set_b: std::collections::HashSet<&str> = kb.split_whitespace().collect();

    let irisan = set_a.intersection(&set_b).count() as f32;
    let gabungan = set_a.union(&set_b).count() as f32;
    if gabungan == 0.0 {
        0.0
    } else {
        irisan / gabungan
    }
}

/// Skor durasi: 1.0 kalau selisih ≤ 2 detik, turun linear sampai 0 di 10 detik.
fn duration_score(want: i64, got: i64) -> f32 {
    if want <= 0 || got <= 0 {
        // Tidak ada data: jangan menghukum, tapi juga jangan memuji.
        return 0.5;
    }
    let diff = (want - got).abs() as f32;
    if diff <= 2.0 {
        1.0
    } else if diff >= 10.0 {
        0.0
    } else {
        1.0 - (diff - 2.0) / 8.0
    }
}

/// Skor gabungan. Judul paling menentukan, artis menengah, durasi sebagai
/// penentu akhir ketika dua kandidat berjudul sama.
pub fn score(c: &Candidate, want_title: &str, want_artist: &str, want_duration: i64) -> f32 {
    let judul = similarity(&c.title, want_title);
    let durasi = duration_score(want_duration, c.duration);

    // Kalau artis tidak diketahui, bobotnya dialihkan ke judul — bukan diberi
    // nilai tengah, karena itu akan menghukum lagu yang sebenarnya cocok.
    if want_artist.trim().is_empty() {
        return judul * 0.90 + durasi * 0.10;
    }

    let artis = similarity(&c.artist, want_artist);
    judul * 0.62 + artis * 0.28 + durasi * 0.10
}

/// Kandidat terbaik di atas ambang, beserta skornya.
pub fn best(
    candidates: &[Candidate],
    want_title: &str,
    want_artist: &str,
    want_duration: i64,
    threshold: f32,
) -> Option<(usize, f32)> {
    let mut hasil: Option<(usize, f32)> = None;
    for (i, c) in candidates.iter().enumerate() {
        let s = score(c, want_title, want_artist, want_duration);
        if s >= threshold && hasil.map_or(true, |(_, best_score)| s > best_score) {
            hasil = Some((i, s));
        }
    }
    hasil
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_membuang_kurung_dan_tanda_baca() {
        assert_eq!(normalize("Blinding Lights (Remastered)"), "blinding lights");
        assert_eq!(normalize("Hati-Hati di Jalan"), "hati hati di jalan");
        assert_eq!(normalize("As It Was!"), "as it was");
    }

    #[test]
    fn normalize_membuang_embel_versi_setelah_hubung() {
        assert_eq!(
            normalize("Blinding Lights - Remastered 2011"),
            "blinding lights"
        );
        // Bagian yang bukan penanda versi tetap dipertahankan.
        assert_eq!(normalize("Runtuh - Live at Jakarta"), "runtuh");
    }

    #[test]
    fn normalize_menghapus_feat() {
        assert_eq!(normalize("Lagu (feat. Orang)"), "lagu");
    }

    #[test]
    fn kemiripan_identik_satu() {
        assert_eq!(similarity("Runtuh", "runtuh"), 1.0);
    }

    #[test]
    fn kemiripan_sebagian() {
        let s = similarity("Runtuh", "Runtuh (Remastered)");
        assert!((s - 1.0).abs() < 0.001);
    }

    #[test]
    fn kemiripan_berbeda_nol() {
        assert_eq!(similarity("Runtuh", "Blinding Lights"), 0.0);
    }

    #[test]
    fn kemiripan_kosong_nol() {
        assert_eq!(similarity("", "apa saja"), 0.0);
        assert_eq!(similarity("", ""), 0.0);
    }

    #[test]
    fn judul_cocok_menang_atas_judul_mirip() {
        let kandidat = vec![
            Candidate {
                title: "Runtuh".into(),
                artist: "Feby Putri".into(),
                duration: 250,
            },
            Candidate {
                title: "Runtuh Hati".into(),
                artist: "Orang Lain".into(),
                duration: 200,
            },
        ];
        let (idx, skor) = best(&kandidat, "Runtuh", "Feby Putri", 250, 0.5).unwrap();
        assert_eq!(idx, 0);
        assert!(skor > 0.9);
    }

    #[test]
    fn durasi_memutuskan_saat_judul_dan_artis_sama() {
        let kandidat = vec![
            Candidate {
                title: "Lagu".into(),
                artist: "Penyanyi".into(),
                duration: 120,
            },
            Candidate {
                title: "Lagu".into(),
                artist: "Penyanyi".into(),
                duration: 305,
            },
        ];
        let (idx, _) = best(&kandidat, "Lagu", "Penyanyi", 300, 0.5).unwrap();
        assert_eq!(idx, 1);
    }

    #[test]
    fn kandidat_jauh_ditolak_ambang() {
        let kandidat = vec![Candidate {
            title: "Lagu Lain".into(),
            artist: "Orang Lain".into(),
            duration: 100,
        }];
        assert!(best(&kandidat, "Runtuh", "Feby Putri", 250, 0.55).is_none());
    }

    #[test]
    fn tanpa_durasi_tidak_menghukum() {
        let c = Candidate {
            title: "Lagu".into(),
            artist: "Penyanyi".into(),
            duration: 0,
        };
        let s = score(&c, "Lagu", "Penyanyi", 0);
        assert!(s > 0.9);
    }

    #[test]
    fn artis_kosong_tidak_menjatuhkan_skor() {
        let c = Candidate {
            title: "Lagu".into(),
            artist: String::new(),
            duration: 200,
        };
        let s = score(&c, "Lagu", "", 200);
        assert!(s > 0.9);
    }

    #[test]
    fn clean_search_title_membersihkan_noise_smtc() {
        assert_eq!(
            clean_search_title("Blinding Lights (Official Music Video)"),
            "Blinding Lights"
        );
        assert_eq!(
            clean_search_title("Shape of You [Official Video]"),
            "Shape of You"
        );
        assert_eq!(
            clean_search_title("Starboy (feat. Daft Punk)"),
            "Starboy"
        );
        assert_eq!(
            clean_search_title("夜に駆ける 【MV】"),
            "夜に駆ける"
        );
        assert_eq!(
            clean_search_title("Lagu Keren - Lyric Video"),
            "Lagu Keren"
        );
        assert_eq!(
            clean_search_title("Track Title | Official Audio"),
            "Track Title"
        );
        // Mempertahankan kurung judul lagu asli yang bukan noise
        assert_eq!(
            clean_search_title("Requiem (Part 1)"),
            "Requiem (Part 1)"
        );
    }
}
