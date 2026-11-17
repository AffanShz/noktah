//! Format teks: jam, tanggal Indonesia, durasi, ukuran, sapaan.

use chrono::{Datelike, Local, Timelike, Weekday};

pub fn pad(n: u32) -> String {
    format!("{n:02}")
}

pub fn clock_text(hour: u32, minute: u32) -> String {
    format!("{}:{}", pad(hour), pad(minute))
}

/// Persentase perjalanan hari lokal (0..=100) dari jam dan menit saat ini.
pub fn day_progress_pct(hour: u32, minute: u32) -> u8 {
    let mins = (hour.min(23) * 60 + minute.min(59)) as f32;
    ((mins / 1440.0) * 100.0).round().clamp(0.0, 100.0) as u8
}

/// `mm:ss`, dipakai untuk posisi dan durasi lagu.
pub fn mmss(total_seconds: i64) -> String {
    let s = total_seconds.max(0);
    format!("{}:{}", pad((s / 60) as u32), pad((s % 60) as u32))
}

/// `3h 4j`, `2j 15m`, atau `45m`.
pub fn uptime_text(seconds: u64) -> String {
    let d = seconds / 86_400;
    let h = (seconds % 86_400) / 3_600;
    let m = (seconds % 3_600) / 60;
    if d > 0 {
        format!("{d}h {h}j")
    } else if h > 0 {
        format!("{h}j {m}m")
    } else {
        format!("{m}m")
    }
}

/// Ukuran byte dengan satu desimal, kecuali angka besar dan byte mentah.
pub fn bytes(n: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    if n == 0 {
        return "0 B".into();
    }
    let mut value = n as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if value >= 100.0 || unit == 0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

pub fn greeting(hour: u32, name: &str) -> String {
    let who = name.trim();
    let dasar = if hour < 4 {
        "Masih bangun"
    } else if hour < 11 {
        "Selamat pagi"
    } else if hour < 15 {
        "Selamat siang"
    } else if hour < 19 {
        "Selamat sore"
    } else {
        "Selamat malam"
    };

    if who.is_empty() {
        if hour < 4 {
            "Masih bangun?".into()
        } else {
            dasar.into()
        }
    } else if hour < 4 {
        format!("{dasar}, {who}?")
    } else {
        format!("{dasar}, {who}")
    }
}

pub fn hari_id(d: Weekday) -> &'static str {
    match d {
        Weekday::Mon => "Senin",
        Weekday::Tue => "Selasa",
        Weekday::Wed => "Rabu",
        Weekday::Thu => "Kamis",
        Weekday::Fri => "Jumat",
        Weekday::Sat => "Sabtu",
        Weekday::Sun => "Minggu",
    }
}

pub fn hari_singkat_id(d: Weekday) -> &'static str {
    match d {
        Weekday::Mon => "Sen",
        Weekday::Tue => "Sel",
        Weekday::Wed => "Rab",
        Weekday::Thu => "Kam",
        Weekday::Fri => "Jum",
        Weekday::Sat => "Sab",
        Weekday::Sun => "Min",
    }
}

pub fn bulan_id(m: u32) -> &'static str {
    const NAMA: [&str; 12] = [
        "Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September",
        "Oktober", "November", "Desember",
    ];
    NAMA[(m.saturating_sub(1) as usize).min(11)]
}

/// `Senin, 27 Oktober 2026`.
pub fn long_date(d: chrono::DateTime<Local>) -> String {
    format!(
        "{}, {} {} {}",
        hari_id(d.weekday()),
        d.day(),
        bulan_id(d.month()),
        d.year()
    )
}

/// `Sen 27 Okt`.
pub fn short_date(d: chrono::DateTime<Local>) -> String {
    format!(
        "{} {} {}",
        hari_singkat_id(d.weekday()),
        d.day(),
        &bulan_id(d.month())[..3.min(bulan_id(d.month()).len())]
    )
}

/// `14:05` untuk waktu hari ini, `Besok 14:05` untuk besok.
pub fn due_label(due_ms: i64, now_ms: i64) -> String {
    use chrono::TimeZone;
    let Some(dt) = Local.timestamp_millis_opt(due_ms).single() else {
        return String::new();
    };
    let jam = clock_text(dt.hour(), dt.minute());

    let selisih_hari = (due_ms - now_ms).div_euclid(86_400_000);
    if selisih_hari == 0 {
        jam
    } else if selisih_hari == 1 {
        format!("Besok {jam}")
    } else {
        format!("{} {}", short_date(dt), jam)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jam_dua_digit() {
        assert_eq!(clock_text(9, 5), "09:05");
        assert_eq!(clock_text(23, 59), "23:59");
    }

    #[test]
    fn mmss_tidak_pernah_negatif() {
        assert_eq!(mmss(-5), "00:00");
        assert_eq!(mmss(0), "00:00");
        assert_eq!(mmss(65), "01:05");
        assert_eq!(mmss(158), "02:38");
    }

    #[test]
    fn uptime_menyesuaikan_satuan() {
        assert_eq!(uptime_text(45), "0m");
        assert_eq!(uptime_text(3_600), "1j 0m");
        assert_eq!(uptime_text(90_000), "1h 1j");
    }

    #[test]
    fn bytes_memakai_satu_desimal_kecuali_besar() {
        assert_eq!(bytes(0), "0 B");
        assert_eq!(bytes(512), "512 B");
        assert_eq!(bytes(1536), "1.5 KB");
        assert_eq!(bytes(1_073_741_824), "1.0 GB");
        assert_eq!(bytes(512 * 1024 * 1024), "512 MB");
    }

    #[test]
    fn sapaan_mengikuti_jam() {
        assert_eq!(greeting(2, ""), "Masih bangun?");
        assert_eq!(greeting(2, "Budi"), "Masih bangun, Budi?");
        assert_eq!(greeting(8, ""), "Selamat pagi");
        assert_eq!(greeting(8, "Budi"), "Selamat pagi, Budi");
        assert_eq!(greeting(13, ""), "Selamat siang");
        assert_eq!(greeting(17, ""), "Selamat sore");
        assert_eq!(greeting(21, ""), "Selamat malam");
    }

    #[test]
    fn nama_dengan_spasi_diabaikan() {
        assert_eq!(greeting(9, "   "), "Selamat pagi");
    }

    #[test]
    fn nama_hari_dan_bulan_indonesia() {
        assert_eq!(hari_id(Weekday::Mon), "Senin");
        assert_eq!(hari_id(Weekday::Sun), "Minggu");
        assert_eq!(bulan_id(1), "Januari");
        assert_eq!(bulan_id(12), "Desember");
        assert_eq!(bulan_id(99), "Desember");
    }

    #[test]
    fn tenggat_hari_ini_hanya_jam() {
        let now = 1_700_000_000_000i64;
        let label = due_label(now + 3_600_000, now);
        assert_eq!(label.len(), 5);
        assert!(label.contains(':'));
    }

    #[test]
    fn persentase_hari_akurat() {
        assert_eq!(day_progress_pct(0, 0), 0);
        assert_eq!(day_progress_pct(12, 0), 50);
        assert_eq!(day_progress_pct(23, 59), 100);
    }
}
