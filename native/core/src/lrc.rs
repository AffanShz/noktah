//! Parser LRC (lirik tersinkron).
//!
//! Satu baris bisa punya beberapa timestamp, dan pecahannya bisa 2 atau 3
//! digit: `[00:12.34]`, `[00:12.340]`, `[0:12]`.

#[derive(Clone, Debug, PartialEq)]
pub struct LrcLine {
    /// detik
    pub time: f64,
    pub text: String,
}

/// Membaca `mm:ss`, `mm:ss.ff`, atau `mm:ss:fff` menjadi detik.
fn parse_stamp(inner: &str) -> Option<f64> {
    let mut parts = inner.split([':', '.']);
    let menit: f64 = parts.next()?.trim().parse().ok()?;
    let detik: f64 = parts.next()?.trim().parse().ok()?;
    if menit < 0.0 || detik < 0.0 || detik >= 60.0 {
        return None;
    }

    let pecahan = match parts.next() {
        Some(f) => {
            let digits: String = f.trim().chars().take(3).collect();
            if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            // "5" = 0.5 detik, "05" = 0.05, "500" = 0.5
            let padded = format!("{digits:0<3}");
            padded.parse::<f64>().ok()? / 1000.0
        }
        None => 0.0,
    };

    Some(menit * 60.0 + detik + pecahan)
}

/// Membuang semua kelompok `[...]` yang tersisa di dalam teks.
fn strip_brackets(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut depth = 0usize;
    for c in input.chars() {
        match c {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

pub fn parse_lrc(raw: &str) -> Vec<LrcLine> {
    let mut out: Vec<LrcLine> = Vec::new();

    for line in raw.lines() {
        let mut rest = line.trim_start();
        let mut stamps: Vec<f64> = Vec::new();

        // Hanya timestamp di awal baris yang dihitung sebagai waktu.
        loop {
            if !rest.starts_with('[') {
                break;
            }
            let Some(end) = rest.find(']') else { break };
            let inner = &rest[1..end];
            match parse_stamp(inner) {
                Some(t) => stamps.push(t),
                None => break,
            }
            rest = &rest[end + 1..];
        }

        if stamps.is_empty() {
            continue;
        }

        let text = strip_brackets(rest).trim().to_string();
        if text.is_empty() {
            continue;
        }

        for time in stamps {
            out.push(LrcLine {
                time,
                text: text.clone(),
            });
        }
    }

    out.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// Indeks baris aktif; `None` kalau belum masuk baris pertama atau lirik polos (tanpa timestamp).
pub fn active_line(lines: &[LrcLine], position_ms: i64) -> Option<usize> {
    if lines.is_empty() || lines.first().map_or(false, |l| l.time < 0.0) {
        return None;
    }
    let now = position_ms as f64 / 1000.0;
    if now < 0.0 {
        return None;
    }
    let mut low = 0usize;
    let mut high = lines.len() - 1;
    let mut answer = None;

    while low <= high {
        let mid = (low + high) / 2;
        if lines[mid].time >= 0.0 && lines[mid].time <= now {
            answer = Some(mid);
            low = mid + 1;
        } else {
            if mid == 0 {
                break;
            }
            high = mid - 1;
        }
    }

    answer
}

/// Posisi yang harus dituju kalau satu baris lirik diklik.
///
/// Baris dianggap aktif saat `posisi + offset >= waktu baris`, jadi untuk
/// mendarat tepat di baris itu posisinya harus `waktu - offset`.
pub fn seek_target_ms(line_time_sec: f64, offset_ms: i64) -> i64 {
    (line_time_sec * 1000.0).round() as i64 - offset_ms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn membaca_timestamp_dua_digit() {
        let lines = parse_lrc("[00:12.34]Kau datang lagi");
        assert_eq!(lines.len(), 1);
        assert!((lines[0].time - 12.34).abs() < 0.001);
        assert_eq!(lines[0].text, "Kau datang lagi");
    }

    #[test]
    fn membaca_timestamp_tiga_digit() {
        let lines = parse_lrc("[01:05.500]Halo");
        assert!((lines[0].time - 65.5).abs() < 0.001);
    }

    #[test]
    fn satu_baris_beberapa_timestamp() {
        let lines = parse_lrc("[00:10.00][00:20.00]Ulang");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].text, "Ulang");
        assert!((lines[1].time - 20.0).abs() < 0.001);
    }

    #[test]
    fn mengabaikan_baris_tanpa_timestamp() {
        let lines = parse_lrc("judul lagu\n[00:01.00]isi");
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn mengabaikan_baris_kosong_setelah_timestamp() {
        let lines = parse_lrc("[00:01.00]   \n[00:02.00]ada");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "ada");
    }

    #[test]
    fn hasil_diurutkan() {
        let lines = parse_lrc("[00:20.00]b\n[00:10.00]a");
        assert_eq!(lines[0].text, "a");
        assert_eq!(lines[1].text, "b");
    }

    #[test]
    fn tag_inline_dibuang_dari_teks() {
        let lines = parse_lrc("[00:01.00]Halo [Chorus] dunia");
        assert_eq!(lines[0].text, "Halo  dunia");
    }

    #[test]
    fn baris_aktif_sebelum_lirik_pertama() {
        let lines = parse_lrc("[00:10.00]a\n[00:20.00]b");
        assert_eq!(active_line(&lines, 0), None);
        assert_eq!(active_line(&lines, 9_999), None);
    }

    #[test]
    fn baris_aktif_tepat_di_batas() {
        let lines = parse_lrc("[00:10.00]a\n[00:20.00]b");
        assert_eq!(active_line(&lines, 10_000), Some(0));
        assert_eq!(active_line(&lines, 19_999), Some(0));
        assert_eq!(active_line(&lines, 20_000), Some(1));
        assert_eq!(active_line(&lines, 999_999), Some(1));
    }

    #[test]
    fn baris_aktif_daftar_kosong() {
        assert_eq!(active_line(&[], 1000), None);
    }

    #[test]
    fn baris_aktif_lirik_polos_tanpa_timestamp_menghasilkan_none() {
        let plain = vec![
            LrcLine { time: -1.0, text: "Di sore hari yang lucu".into() },
            LrcLine { time: -1.0, text: "Kita".into() },
        ];
        assert_eq!(active_line(&plain, 0), None);
        assert_eq!(active_line(&plain, 76_000), None);
        assert_eq!(active_line(&plain, 276_000), None);
    }

    #[test]
    fn seek_mengurangi_offset() {
        assert_eq!(seek_target_ms(12.0, 0), 12_000);
        assert_eq!(seek_target_ms(12.0, 500), 11_500);
        assert_eq!(seek_target_ms(12.0, -500), 12_500);
    }

    #[test]
    fn timestamp_rusak_dilewati() {
        let lines = parse_lrc("[xx:yy]bukan\n[00:05.00]asli");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "asli");
    }
}
