//! Cuaca dari Open-Meteo (tanpa kunci API).
//!
//! Diambil di thread sendiri dan disegarkan tiap 10 menit. Kalau jaringan
//! gagal, data lama tetap dipakai — lebih baik menampilkan angka kemarin
//! daripada mengosongkan layar.

use noktah_core::model::{ForecastDay, ForecastHour, WeatherNow};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

pub struct WeatherHandle {
    pub rx: Receiver<WeatherNow>,
    pub tx: Sender<String>,
}

impl WeatherHandle {
    #[cfg(test)]
    pub fn dummy() -> Self {
        let (_, rx) = channel();
        let (tx, _) = channel();
        Self { rx, tx }
    }
}

pub fn spawn(ctx: egui::Context) -> WeatherHandle {
    let (tx_out, rx_out) = channel();
    let (tx_req, rx_req) = channel::<String>();

    let _ = std::thread::Builder::new()
        .name("noktah-weather".into())
        .stack_size(256 * 1024)
        .spawn(move || {
            let mut terakhir: Option<std::time::Instant> = None;
            let mut kota = String::new();

            loop {
                // Permintaan baru, atau penyegaran berkala.
                if let Ok(baru) = rx_req.try_recv() {
                    if baru != kota {
                        kota = baru;
                        terakhir = None;
                    }
                }

                let perlu = terakhir.map_or(true, |t| t.elapsed() > Duration::from_secs(600));
                if perlu && !kota.trim().is_empty() {
                    terakhir = Some(std::time::Instant::now());
                    if let Some(cuaca) = fetch(&kota) {
                        let _ = tx_out.send(cuaca);
                        ctx.request_repaint();
                    }
                }

                std::thread::sleep(Duration::from_secs(5));
            }
        });

    WeatherHandle {
        rx: rx_out,
        tx: tx_req,
    }
}

fn fetch(kota: &str) -> Option<WeatherNow> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(8))
        .user_agent("Noktah/0.2")
        .build();

    let cari = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=id&format=json",
        urlencode(kota)
    );
    let geo: serde_json::Value = agent.get(&cari).call().ok()?.into_json().ok()?;
    let tempat = geo["results"].get(0)?;

    let lat = tempat["latitude"].as_f64()?;
    let lon = tempat["longitude"].as_f64()?;
    let nama = tempat["name"].as_str().unwrap_or(kota).to_string();

    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}\
         &current=temperature_2m,apparent_temperature,relative_humidity_2m,wind_speed_10m,weather_code,is_day\
         &hourly=temperature_2m,weather_code&forecast_hours=8\
         &daily=weather_code,temperature_2m_max,temperature_2m_min&forecast_days=5\
         &timezone=auto"
    );
    let data: serde_json::Value = agent.get(&url).call().ok()?.into_json().ok()?;
    let cur = &data["current"];

    // Strip per jam: mulai dari jam berjalan, ambil 6 titik.
    let mut hours = Vec::new();
    if let (Some(times), Some(temps), Some(codes)) = (
        data["hourly"]["time"].as_array(),
        data["hourly"]["temperature_2m"].as_array(),
        data["hourly"]["weather_code"].as_array(),
    ) {
        let now_hour = chrono::Local::now().format("%Y-%m-%dT%H:00").to_string();
        let start = times
            .iter()
            .position(|t| t.as_str().map(|s| s == now_hour).unwrap_or(false))
            .unwrap_or(0);

        for i in start..times.len().min(start + 6) {
            let label = if i == start {
                "Kini".to_string()
            } else {
                times[i]
                    .as_str()
                    .and_then(|s| s.get(11..13))
                    .unwrap_or("--")
                    .to_string()
            };
            hours.push(ForecastHour {
                label,
                code: codes.get(i).and_then(|c| c.as_u64()).unwrap_or(0) as u8,
                temp_c: temps.get(i).and_then(|t| t.as_f64()).unwrap_or(0.0) as f32,
                is_now: i == start,
            });
        }
    }

    // Prakiraan harian.
    let mut days = Vec::new();
    if let (Some(dates), Some(codes), Some(maxs), Some(mins)) = (
        data["daily"]["time"].as_array(),
        data["daily"]["weather_code"].as_array(),
        data["daily"]["temperature_2m_max"].as_array(),
        data["daily"]["temperature_2m_min"].as_array(),
    ) {
        for i in 0..dates.len().min(5) {
            let label = dates[i]
                .as_str()
                .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                .map(|d| {
                    use chrono::Datelike;
                    noktah_core::format::hari_singkat_id(d.weekday()).to_string()
                })
                .unwrap_or_else(|| "--".to_string());
            days.push(ForecastDay {
                label,
                code: codes.get(i).and_then(|c| c.as_u64()).unwrap_or(0) as u8,
                max_c: maxs.get(i).and_then(|t| t.as_f64()).unwrap_or(0.0) as f32,
                min_c: mins.get(i).and_then(|t| t.as_f64()).unwrap_or(0.0) as f32,
            });
        }
    }

    Some(WeatherNow {
        place: nama,
        temp_c: cur["temperature_2m"].as_f64().unwrap_or(0.0) as f32,
        feels_c: cur["apparent_temperature"].as_f64().unwrap_or(0.0) as f32,
        humidity: cur["relative_humidity_2m"].as_u64().unwrap_or(0) as u8,
        wind_kmh: cur["wind_speed_10m"].as_f64().unwrap_or(0.0) as f32,
        code: cur["weather_code"].as_u64().unwrap_or(0) as u8,
        is_day: cur["is_day"].as_u64().unwrap_or(1) == 1,
        fetched_at: chrono::Local::now().timestamp_millis(),
        days,
        hours,
    })
}

/// Label singkat untuk kode cuaca WMO.
pub fn label(code: u8) -> &'static str {
    match code {
        0 => "Cerah",
        1 | 2 => "Berawan sebagian",
        3 => "Mendung",
        45 | 48 => "Berkabut",
        51 | 53 | 55 | 56 | 57 => "Gerimis",
        61 | 63 | 65 | 66 | 67 => "Hujan",
        71 | 73 | 75 | 77 => "Salju",
        80 | 81 | 82 => "Hujan lokal",
        85 | 86 => "Hujan salju",
        95 | 96 | 99 => "Badai",
        _ => "Cuaca",
    }
}


/// Versi ikon garis dari `icon`, untuk kartu cuaca dan strip per jam.
pub fn icon_for(code: u8, is_day: bool) -> crate::presentation::icons::Icon {
    use crate::presentation::icons::Icon;
    match code {
        0 => {
            if is_day {
                Icon::Sun
            } else {
                Icon::Moon
            }
        }
        1 | 2 => Icon::PartlyCloudy,
        3 => Icon::Cloud,
        45 | 48 => Icon::Fog,
        51..=57 => Icon::Rain,
        61..=67 => Icon::Rain,
        71..=77 | 85 | 86 => Icon::Snow,
        80..=82 => Icon::Rain,
        95..=99 => Icon::Storm,
        _ => Icon::Cloud,
    }
}

fn urlencode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
