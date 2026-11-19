//! Parser dan model data kalender lokal (.ics / RFC 5545).
//!
//! Dirancang fail-safe dan murni Rust tanpa ketergantungan GUI atau library eksternal,
//! agar dapat mengimpor agenda lokal dari berkas kalender Outlook, Google Calendar export,
//! Thunderbird, maupun Apple Calendar tanpa telemetri atau OAuth cloud.

use serde::{Deserialize, Serialize};

/// Agenda atau rapat dari berkas kalender .ics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub uid: String,
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
    /// Waktu mulai dalam Unix epoch seconds (UTC).
    pub start_timestamp: i64,
    /// Waktu selesai dalam Unix epoch seconds (UTC), jika tersedia.
    pub end_timestamp: Option<i64>,
    /// Apakah acara ini berlaku sepanjang hari (all-day).
    pub all_day: bool,
}

impl CalendarEvent {
    /// Apakah acara sedang berlangsung pada detik `now_sec`.
    pub fn is_ongoing(&self, now_sec: i64) -> bool {
        if let Some(end) = self.end_timestamp {
            now_sec >= self.start_timestamp && now_sec < end
        } else {
            // Jika tidak ada end, asumsikan durasi 30 menit
            now_sec >= self.start_timestamp && now_sec < self.start_timestamp + 1800
        }
    }

    /// Apakah acara masih di masa depan terhadap detik `now_sec`.
    pub fn is_future(&self, now_sec: i64) -> bool {
        self.start_timestamp > now_sec
    }

    /// Format teks ringkas status waktu (mis. "15 mnt lagi", "Sedang berlangsung", "Besok 09:00").
    pub fn relative_time_label(&self, now_sec: i64) -> String {
        if self.is_ongoing(now_sec) {
            return "Sedang berlangsung".into();
        }

        let selisih = self.start_timestamp - now_sec;
        if selisih < 0 {
            return "Selesai".into();
        }

        let menit = selisih / 60;
        let jam = menit / 60;

        if menit < 60 {
            format!("{} mnt lagi", menit.max(1))
        } else if jam < 24 {
            let sisa_menit = menit % 60;
            if sisa_menit > 0 {
                format!("{}j {}m lagi", jam, sisa_menit)
            } else {
                format!("{} jam lagi", jam)
            }
        } else {
            let hari = jam / 24;
            format!("{} hari lagi", hari)
        }
    }
}

/// Menyatukan baris-baris terlipat (RFC 5545 unfolding).
/// Baris lanjutan yang diawali spasi (' ') atau tab ('\t') digabungkan ke baris sebelumnya.
fn unfold_lines(raw: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for line in raw.lines() {
        let trimmed_cr = line.trim_end_matches('\r');
        if (trimmed_cr.starts_with(' ') || trimmed_cr.starts_with('\t')) && !lines.is_empty() {
            if let Some(last) = lines.last_mut() {
                last.push_str(&trimmed_cr[1..]);
            }
        } else if !trimmed_cr.is_empty() {
            lines.push(trimmed_cr.to_string());
        }
    }
    lines
}

/// Parse tanggal-waktu iCalendar string (mis. "20261119T140000Z", "20261119T140000", "20261119")
/// ke Unix epoch seconds. Fail-safe tanpa panic.
pub fn parse_ics_datetime(val: &str) -> Option<(i64, bool)> {
    let clean = val.trim();
    // Format tanggal saja (all-day): YYYYMMDD (8 karakter angka)
    if clean.len() == 8 && clean.chars().all(|c| c.is_ascii_digit()) {
        let year: i32 = clean[0..4].parse().ok()?;
        let month: u32 = clean[4..6].parse().ok()?;
        let day: u32 = clean[6..8].parse().ok()?;

        let nd = chrono::NaiveDate::from_ymd_opt(year, month, day)?;
        let ndt = nd.and_hms_opt(0, 0, 0)?;
        let dt = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(ndt, chrono::Utc);
        return Some((dt.timestamp(), true));
    }

    // Format tanggal-waktu: YYYYMMDDTHHMMSS atau YYYYMMDDTHHMMSSZ
    let s = clean.trim_end_matches('Z');
    if s.len() == 15 && s.contains('T') {
        let parts: Vec<&str> = s.split('T').collect();
        if parts.len() == 2 && parts[0].len() == 8 && parts[1].len() == 6 {
            let y: i32 = parts[0][0..4].parse().ok()?;
            let m: u32 = parts[0][4..6].parse().ok()?;
            let d: u32 = parts[0][6..8].parse().ok()?;

            let h: u32 = parts[1][0..2].parse().ok()?;
            let min: u32 = parts[1][2..4].parse().ok()?;
            let sec: u32 = parts[1][4..6].parse().ok()?;

            let nd = chrono::NaiveDate::from_ymd_opt(y, m, d)?;
            let ndt = nd.and_hms_opt(h, min, sec)?;
            let dt = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(ndt, chrono::Utc);
            return Some((dt.timestamp(), false));
        }
    }

    None
}

/// Menghapus escape character standar .ics (\n, \N, \,, \;)
fn unescape_ics_text(s: &str) -> String {
    s.replace("\\n", "\n")
        .replace("\\N", "\n")
        .replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\\\", "\\")
}

/// Parse isi berkas kalender .ics menjadi daftar `CalendarEvent`.
pub fn parse_ics(content: &str) -> Vec<CalendarEvent> {
    let lines = unfold_lines(content);
    let mut events = Vec::new();

    let mut in_event = false;
    let mut uid = String::new();
    let mut summary = String::new();
    let mut description: Option<String> = None;
    let mut location: Option<String> = None;
    let mut start_ts: Option<i64> = None;
    let mut end_ts: Option<i64> = None;
    let mut all_day = false;
    let mut status_cancelled = false;

    for line in lines {
        let line_trimmed = line.trim();
        if line_trimmed.eq_ignore_ascii_case("BEGIN:VEVENT") {
            in_event = true;
            uid.clear();
            summary.clear();
            description = None;
            location = None;
            start_ts = None;
            end_ts = None;
            all_day = false;
            status_cancelled = false;
            continue;
        }

        if line_trimmed.eq_ignore_ascii_case("END:VEVENT") {
            if in_event && !status_cancelled && start_ts.is_some() {
                events.push(CalendarEvent {
                    uid: if uid.is_empty() {
                        format!("evt-{}", events.len())
                    } else {
                        uid.clone()
                    },
                    summary: if summary.is_empty() {
                        "Acara Tanpa Judul".into()
                    } else {
                        summary.clone()
                    },
                    description: description.clone(),
                    location: location.clone(),
                    start_timestamp: start_ts.unwrap(),
                    end_timestamp: end_ts,
                    all_day,
                });
            }
            in_event = false;
            continue;
        }

        if !in_event {
            continue;
        }

        // Pisahkan key dan value: misal "DTSTART;VALUE=DATE:20261119" -> key_part="DTSTART;VALUE=DATE", val="20261119"
        let Some((key_part, val)) = line_trimmed.split_once(':') else {
            continue;
        };

        let key_name = key_part.split(';').next().unwrap_or("").trim();

        if key_name.eq_ignore_ascii_case("UID") {
            uid = val.trim().to_string();
        } else if key_name.eq_ignore_ascii_case("SUMMARY") {
            summary = unescape_ics_text(val.trim());
        } else if key_name.eq_ignore_ascii_case("DESCRIPTION") {
            description = Some(unescape_ics_text(val.trim()));
        } else if key_name.eq_ignore_ascii_case("LOCATION") {
            location = Some(unescape_ics_text(val.trim()));
        } else if key_name.eq_ignore_ascii_case("STATUS") {
            if val.trim().eq_ignore_ascii_case("CANCELLED") {
                status_cancelled = true;
            }
        } else if key_name.eq_ignore_ascii_case("DTSTART") {
            if let Some((ts, is_ad)) = parse_ics_datetime(val) {
                start_ts = Some(ts);
                if is_ad {
                    all_day = true;
                }
            }
        } else if key_name.eq_ignore_ascii_case("DTEND") {
            if let Some((ts, _)) = parse_ics_datetime(val) {
                end_ts = Some(ts);
            }
        }
    }

    // Urutkan berdasarkan waktu mulai (kronologis)
    events.sort_by_key(|e| e.start_timestamp);
    events
}

/// Mengambil agenda aktif dan agenda terdekat (maksimal `limit` item).
pub fn upcoming_events<'a>(
    events: &'a [CalendarEvent],
    now_sec: i64,
    limit: usize,
) -> Vec<&'a CalendarEvent> {
    events
        .iter()
        .filter(|e| {
            // Tampilkan jika sedang berlangsung atau dimulai dalam 48 jam ke depan
            e.is_ongoing(now_sec) || (e.is_future(now_sec) && e.start_timestamp - now_sec <= 172800)
        })
        .take(limit)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_ICS: &str = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Noktah Test//ID
BEGIN:VEVENT
UID:evt-001@noktah
SUMMARY:Rapat Koordinasi v1.4.0
DESCRIPTION:Membahas arsitektur multi-monitor\ndan fitur drop shelf.
LOCATION:Ruang Rapat Utama
DTSTART:20261119T080000Z
DTEND:20261119T090000Z
STATUS:CONFIRMED
END:VEVENT
BEGIN:VEVENT
UID:evt-cancelled@noktah
SUMMARY:Sesi Dibatalkan
DTSTART:20261119T100000Z
STATUS:CANCELLED
END:VEVENT
BEGIN:VEVENT
UID:evt-002@noktah
SUMMARY:All-Day Release Day
DTSTART;VALUE=DATE:20261120
END:VEVENT
END:VCALENDAR"#;

    #[test]
    fn test_parse_ics_sample() {
        let events = parse_ics(SAMPLE_ICS);
        assert_eq!(events.len(), 2, "Acara yang dibatalkan harus dilewati");

        let first = &events[0];
        assert_eq!(first.uid, "evt-001@noktah");
        assert_eq!(first.summary, "Rapat Koordinasi v1.4.0");
        assert_eq!(first.location.as_deref(), Some("Ruang Rapat Utama"));
        assert!(first.description.as_ref().unwrap().contains('\n'));
        assert!(!first.all_day);

        let second = &events[1];
        assert_eq!(second.summary, "All-Day Release Day");
        assert!(second.all_day);
    }

    #[test]
    fn test_relative_time_label() {
        let evt = CalendarEvent {
            uid: "1".into(),
            summary: "Sprint".into(),
            description: None,
            location: None,
            start_timestamp: 1000,
            end_timestamp: Some(2000),
            all_day: false,
        };

        assert_eq!(evt.relative_time_label(1500), "Sedang berlangsung");
        assert_eq!(evt.relative_time_label(400), "10 mnt lagi");
        assert_eq!(evt.relative_time_label(2500), "Selesai");
    }

    #[test]
    fn test_unfold_long_lines() {
        let folded = "SUMMARY:Ini adalah judul yang sang\r\n at panjang sekali dan ter\r\n lipat";
        let unfolded = unfold_lines(folded);
        assert_eq!(unfolded.len(), 1);
        assert_eq!(
            unfolded[0],
            "SUMMARY:Ini adalah judul yang sangat panjang sekali dan terlipat"
        );
    }
}
