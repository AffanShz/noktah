//! Klien lirik LRCLIB.
//!
//! Tiga tahap: coba `get` dengan durasi (paling akurat), lalu `get` tanpa
//! durasi, lalu `search` dan pilih kandidat dengan skor tertinggi. Hasilnya
//! bisa tersinkron (LRC) atau teks polos.

use noktah_core::lyrics::{self, Candidate};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

/// Satu hasil lirik yang disimpan ke disk.
#[derive(serde::Serialize, serde::Deserialize)]
struct Cached {
    lines: Vec<(f64, String)>,
    synced: bool,
    #[serde(default)]
    provider: String,
    at: i64,
}

/// Kunci cache: judul + artis + durasi, di-hash supaya aman jadi nama file.
fn cache_key(req: &Request) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in format!("{}|{}|{}", req.title, req.artist, req.duration).bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:016x}")
}

fn cache_load(req: &Request) -> Option<LyricsResult> {
    let path = crate::infrastructure::store::data_dir().join("lyrics").join(format!("{}.json", cache_key(req)));
    let raw = std::fs::read_to_string(path).ok()?;
    let cached: Cached = serde_json::from_str(&raw).ok()?;

    // Cache kedaluwarsa: lirik tersinkron 30 hari, teks polos 1 hari (agar jika ada yang upload lirik sync di hulu segera terbarui).
    let umur = chrono::Local::now().timestamp_millis() - cached.at;
    let max_age_ms = if cached.synced {
        30 * 24 * 3600 * 1000
    } else {
        24 * 3600 * 1000
    };
    if umur > max_age_ms {
        return None;
    }

    Some(LyricsResult {
        track_id: req.track_id.clone(),
        lines: cached
            .lines
            .into_iter()
            .map(|(time, text)| noktah_core::lrc::LrcLine { time, text })
            .collect(),
        synced: cached.synced,
        provider: if cached.provider.is_empty() {
            "Cached".into()
        } else {
            cached.provider
        },
    })
}

fn cache_save(req: &Request, result: &LyricsResult) {
    let dir = crate::infrastructure::store::data_dir().join("lyrics");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let cached = Cached {
        lines: result.lines.iter().map(|l| (l.time, l.text.clone())).collect(),
        synced: result.synced,
        provider: result.provider.clone(),
        at: chrono::Local::now().timestamp_millis(),
    };
    if let Ok(raw) = serde_json::to_string(&cached) {
        let _ = std::fs::write(dir.join(format!("{}.json", cache_key(req))), raw);
    }
}

/// GET dengan percobaan ulang untuk 429 dan 5xx.
fn get_json(agent: &ureq::Agent, url: &str) -> Option<serde_json::Value> {
    for attempt in 0..3 {
        match agent.get(url).call() {
            Ok(resp) => return resp.into_json().ok(),
            Err(ureq::Error::Status(code, _)) if code == 429 || code >= 500 => {
                std::thread::sleep(Duration::from_millis(400 * (attempt + 1)));
            }
            Err(_) => return None,
        }
    }
    None
}

pub struct LyricsResult {
    pub track_id: String,
    pub lines: Vec<noktah_core::lrc::LrcLine>,
    pub synced: bool,
    pub provider: String,
}

pub struct LyricsHandle {
    pub rx: Receiver<LyricsResult>,
    pub tx: std::sync::mpsc::Sender<Request>,
}

impl LyricsHandle {
    #[cfg(test)]
    pub fn dummy() -> Self {
        let (_, rx) = channel();
        let (tx, _) = channel();
        Self { rx, tx }
    }
}

pub struct Request {
    pub track_id: String,
    pub title: String,
    pub artist: String,
    pub duration: i64,
    pub album: String,
}

pub fn spawn(ctx: egui::Context) -> LyricsHandle {
    let (tx_result, rx_result) = channel();
    let (tx_req, rx_req) = channel::<Request>();

    let _ = std::thread::Builder::new()
        .name("noktah-lyrics".into())
        .stack_size(256 * 1024)
        .spawn(move || {
            while let Ok(req) = rx_req.recv() {
                // Cache dulu: lirik yang sama tidak perlu diunduh dua kali.
                if let Some(hit) = cache_load(&req) {
                    let _ = tx_result.send(hit);
                    ctx.request_repaint();
                    continue;
                }

                if let Some(result) = fetch(&req) {
                    cache_save(&req, &result);
                    let _ = tx_result.send(result);
                    ctx.request_repaint();
                }
            }
        });

    LyricsHandle {
        rx: rx_result,
        tx: tx_req,
    }
}

fn fetch(req: &Request) -> Option<LyricsResult> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(8))
        .user_agent("Noktah/1.0 (https://github.com/AffanShz/noktah)")
        .build();

    let clean_title = noktah_core::lyrics::clean_search_title(&req.title);
    let title_to_use = if clean_title.is_empty() {
        req.title.as_str()
    } else {
        clean_title.as_str()
    };

    let mut fallback_plain: Option<LyricsResult> = None;

    // 1. Coba LRCLIB (Direct Get & Search) - prioritaskan lirik tersinkronisasi
    if let Some(res) = fetch_lrclib(&agent, req, title_to_use) {
        if res.synced {
            return Some(res);
        }
        fallback_plain = Some(res);
    }

    // 2. Fallback ke KuGou Mobile API
    if let Some(res) = fetch_kugou(&agent, req, title_to_use) {
        if res.synced {
            return Some(res);
        }
        if fallback_plain.is_none() {
            fallback_plain = Some(res);
        }
    }

    // 3. Fallback ke NetEase Cloud Music
    if let Some(res) = fetch_netease(&agent, req, title_to_use) {
        if res.synced {
            return Some(res);
        }
        if fallback_plain.is_none() {
            fallback_plain = Some(res);
        }
    }

    // Kembalikan lirik polos jika tidak ada satu pun provider yang memiliki lirik tersinkron
    fallback_plain
}

fn fetch_lrclib(agent: &ureq::Agent, req: &Request, clean_title: &str) -> Option<LyricsResult> {
    let artist = urlencode(&req.artist);
    let title = urlencode(clean_title);
    let album = urlencode(&req.album);

    let mut plain_fallback: Option<LyricsResult> = None;

    // 1a. Coba get langsung dengan durasi (utamakan yang synced):
    if req.duration > 0 {
        let url = format!(
            "https://lrclib.net/api/get?artist_name={artist}&track_name={title}&duration={}",
            req.duration
        );
        if let Some((lines, synced)) = parse_response(agent, &url) {
            if synced {
                return Some(LyricsResult {
                    track_id: req.track_id.clone(),
                    lines,
                    synced: true,
                    provider: "LRCLIB".into(),
                });
            } else if plain_fallback.is_none() {
                plain_fallback = Some(LyricsResult {
                    track_id: req.track_id.clone(),
                    lines,
                    synced: false,
                    provider: "LRCLIB".into(),
                });
            }
        }

        // Jika judul asli berbeda dari judul bersih, coba juga judul aslinya
        if clean_title != req.title {
            let raw_title = urlencode(&req.title);
            let url_raw = format!(
                "https://lrclib.net/api/get?artist_name={artist}&track_name={raw_title}&duration={}",
                req.duration
            );
            if let Some((lines, synced)) = parse_response(agent, &url_raw) {
                if synced {
                    return Some(LyricsResult {
                        track_id: req.track_id.clone(),
                        lines,
                        synced: true,
                        provider: "LRCLIB".into(),
                    });
                } else if plain_fallback.is_none() {
                    plain_fallback = Some(LyricsResult {
                        track_id: req.track_id.clone(),
                        lines,
                        synced: false,
                        provider: "LRCLIB".into(),
                    });
                }
            }
        }
    }

    // 1b. Tanpa durasi — beberapa unggahan tidak mencantumkannya.
    let url_no_dur = format!("https://lrclib.net/api/get?artist_name={artist}&track_name={title}");
    if let Some((lines, synced)) = parse_response(agent, &url_no_dur) {
        if synced {
            return Some(LyricsResult {
                track_id: req.track_id.clone(),
                lines,
                synced: true,
                provider: "LRCLIB".into(),
            });
        } else if plain_fallback.is_none() {
            plain_fallback = Some(LyricsResult {
                track_id: req.track_id.clone(),
                lines,
                synced: false,
                provider: "LRCLIB".into(),
            });
        }
    }

    // 1c. Pencarian kandidat: coba dengan album, tanpa album, dan query umum `q`
    let mut search_urls = Vec::new();
    if !req.album.is_empty() {
        search_urls.push(format!(
            "https://lrclib.net/api/search?track_name={title}&artist_name={artist}&album_name={album}"
        ));
    }
    search_urls.push(format!(
        "https://lrclib.net/api/search?track_name={title}&artist_name={artist}"
    ));
    let q_query = if req.artist.trim().is_empty() {
        clean_title.to_string()
    } else {
        format!("{} {}", clean_title, req.artist)
    };
    search_urls.push(format!(
        "https://lrclib.net/api/search?q={}",
        urlencode(&q_query)
    ));

    for search_url in search_urls {
        let Some(json) = get_json(agent, &search_url) else { continue };
        let Some(items) = json.as_array() else { continue };
        if items.is_empty() {
            continue;
        }

        let mut kandidat: Vec<Candidate> = Vec::new();
        let mut teks: Vec<(Option<String>, Option<String>)> = Vec::new();

        for item in items {
            let judul = item["trackName"].as_str().unwrap_or_default().to_string();
            let artis = item["artistName"].as_str().unwrap_or_default().to_string();
            let durasi = item["duration"].as_f64().unwrap_or(0.0) as i64;
            kandidat.push(Candidate {
                title: judul,
                artist: artis,
                duration: durasi,
            });
            teks.push((
                item["syncedLyrics"].as_str().map(str::to_string),
                item["plainLyrics"].as_str().map(str::to_string),
            ));
        }

        // Prioritas tertinggi: cari kandidat yang punya synced lyrics
        let synced_indices: Vec<usize> = teks
            .iter()
            .enumerate()
            .filter_map(|(i, (synced, _))| if synced.is_some() { Some(i) } else { None })
            .collect();

        if !synced_indices.is_empty() {
            let synced_candidates: Vec<Candidate> = synced_indices
                .iter()
                .map(|&i| kandidat[i].clone())
                .collect();
            if let Some((sub_idx, _)) = lyrics::best(&synced_candidates, clean_title, &req.artist, req.duration, 0.40) {
                let orig_idx = synced_indices[sub_idx];
                if let Some(raw) = &teks[orig_idx].0 {
                    let lines = noktah_core::lrc::parse_lrc(raw);
                    if !lines.is_empty() {
                        return Some(LyricsResult {
                            track_id: req.track_id.clone(),
                            lines,
                            synced: true,
                            provider: "LRCLIB".into(),
                        });
                    }
                }
            }
        }

        // Simpan plain fallback jika belum ada
        if plain_fallback.is_none() {
            if let Some((idx, _)) = lyrics::best(&kandidat, clean_title, &req.artist, req.duration, 0.45) {
                if let Some(raw) = &teks[idx].1 {
                    let lines: Vec<noktah_core::lrc::LrcLine> = raw
                        .lines()
                        .filter(|l| !l.trim().is_empty())
                        .map(|l| noktah_core::lrc::LrcLine {
                            time: -1.0,
                            text: l.trim().to_string(),
                        })
                        .collect();
                    if !lines.is_empty() {
                        plain_fallback = Some(LyricsResult {
                            track_id: req.track_id.clone(),
                            lines,
                            synced: false,
                            provider: "LRCLIB".into(),
                        });
                    }
                }
            }
        }
    }

    plain_fallback
}

#[allow(dead_code)]
fn fetch_betterlyrics(agent: &ureq::Agent, req: &Request, clean_title: &str) -> Option<LyricsResult> {
    let title = urlencode(clean_title);
    let artist = urlencode(&req.artist);
    let mut url = format!("https://lyrics-api.boidu.dev/getLyrics?s={title}&a={artist}");
    if req.duration > 0 {
        url.push_str(&format!("&d={}", req.duration));
    }
    if !req.album.is_empty() {
        url.push_str(&format!("&al={}", urlencode(&req.album)));
    }

    let json: serde_json::Value = get_json(agent, &url)?;
    let ttml = json["ttml"].as_str()?;
    let lines = parse_ttml(ttml);
    if !lines.is_empty() {
        Some(LyricsResult {
            track_id: req.track_id.clone(),
            lines,
            synced: true,
            provider: "BetterLyrics".into(),
        })
    } else {
        None
    }
}

fn fetch_netease(agent: &ureq::Agent, req: &Request, clean_title: &str) -> Option<LyricsResult> {
    let query = if req.artist.trim().is_empty() {
        clean_title.to_string()
    } else {
        format!("{} {}", clean_title, req.artist)
    };
    let url = format!(
        "https://music.163.com/api/search/get?s={}&type=1&limit=5",
        urlencode(&query)
    );
    let json: serde_json::Value = get_json(agent, &url)?;
    let songs = json.get("result")?.get("songs")?.as_array()?;
    if songs.is_empty() {
        return None;
    }

    let mut kandidat = Vec::new();
    let mut ids = Vec::new();

    for song in songs {
        let id = song.get("id")?.as_i64()?;
        let judul = song.get("name")?.as_str().unwrap_or_default().to_string();
        let durasi_ms = song.get("duration")?.as_i64().unwrap_or(0);
        let durasi = durasi_ms / 1000;
        let mut artis_list = Vec::new();
        if let Some(artists) = song.get("artists").and_then(|a| a.as_array()) {
            for a in artists {
                if let Some(nama) = a.get("name").and_then(|n| n.as_str()) {
                    artis_list.push(nama);
                }
            }
        }
        let artis = artis_list.join(", ");
        kandidat.push(Candidate {
            title: judul,
            artist: artis,
            duration: durasi,
        });
        ids.push(id);
    }

    let (best_idx, _) = lyrics::best(&kandidat, clean_title, &req.artist, req.duration, 0.45)?;
    let song_id = ids.get(best_idx)?;

    let lyric_url = format!("https://music.163.com/api/song/lyric?id={song_id}&lv=1&kv=1&tv=1");
    let lyric_json: serde_json::Value = get_json(agent, &lyric_url)?;
    let lrc_raw = lyric_json.get("lrc")?.get("lyric")?.as_str()?;

    let lines = noktah_core::lrc::parse_lrc(lrc_raw);
    if !lines.is_empty() {
        Some(LyricsResult {
            track_id: req.track_id.clone(),
            lines,
            synced: true,
            provider: "NetEase".into(),
        })
    } else {
        let plain_lines: Vec<noktah_core::lrc::LrcLine> = lrc_raw
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| noktah_core::lrc::LrcLine {
                time: -1.0,
                text: l.trim().to_string(),
            })
            .collect();
        if !plain_lines.is_empty() {
            Some(LyricsResult {
                track_id: req.track_id.clone(),
                lines: plain_lines,
                synced: false,
                provider: "NetEase".into(),
            })
        } else {
            None
        }
    }
}

fn fetch_kugou(agent: &ureq::Agent, req: &Request, clean_title: &str) -> Option<LyricsResult> {
    let query = if req.artist.trim().is_empty() {
        clean_title.to_string()
    } else {
        format!("{} - {}", clean_title, req.artist)
    };
    let url = format!(
        "http://mobilecdn.kugou.com/api/v3/search/song?format=json&keyword={}&page=1&pagesize=5",
        urlencode(&query)
    );
    let json: serde_json::Value = get_json(agent, &url)?;
    let info = json.get("data")?.get("info")?.as_array()?;
    if info.is_empty() {
        return None;
    }

    let mut kandidat = Vec::new();
    let mut hashes = Vec::new();

    for song in info {
        let hash = song.get("hash")?.as_str()?.to_string();
        let judul = song.get("songname")?.as_str().unwrap_or_default().to_string();
        let artis = song.get("singername")?.as_str().unwrap_or_default().to_string();
        let durasi = song.get("duration")?.as_i64().unwrap_or(0);

        kandidat.push(Candidate {
            title: judul,
            artist: artis,
            duration: durasi,
        });
        hashes.push(hash);
    }

    let (best_idx, _) = lyrics::best(&kandidat, clean_title, &req.artist, req.duration, 0.45)?;
    let song_hash = hashes.get(best_idx)?;

    let search_lyr_url = format!(
        "http://lyrics.kugou.com/search?ver=1&man=yes&client=pc&hash={song_hash}"
    );
    let lyr_json: serde_json::Value = get_json(agent, &search_lyr_url)?;
    let candidate = lyr_json.get("candidates")?.as_array()?.first()?;
    let cand_id = candidate.get("id")?.as_i64()?;
    let cand_key = candidate.get("accesskey")?.as_str()?;

    let download_url = format!(
        "http://lyrics.kugou.com/download?ver=1&client=pc&id={cand_id}&accesskey={cand_key}&fmt=lrc&charset=utf8"
    );
    let down_json: serde_json::Value = get_json(agent, &download_url)?;
    let b64_content = down_json.get("content")?.as_str()?;
    let decoded = decode_base64(b64_content)?;
    let clean_decoded = decoded.trim_start_matches('\u{feff}');

    let lines = noktah_core::lrc::parse_lrc(clean_decoded);
    if !lines.is_empty() {
        Some(LyricsResult {
            track_id: req.track_id.clone(),
            lines,
            synced: true,
            provider: "KuGou".into(),
        })
    } else {
        None
    }
}

fn decode_base64(input: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;

    for &b in input.as_bytes() {
        let val = match b {
            b'A'..=b'Z' => (b - b'A') as u32,
            b'a'..=b'z' => (b - b'a' + 26) as u32,
            b'0'..=b'9' => (b - b'0' + 52) as u32,
            b'+' => 62,
            b'/' => 63,
            b'=' | b'\r' | b'\n' | b' ' => continue,
            _ => return None,
        };
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push((buf >> bits) as u8);
        }
    }
    String::from_utf8(bytes).ok()
}

#[allow(dead_code)]
fn parse_ttml(raw: &str) -> Vec<noktah_core::lrc::LrcLine> {
    let mut lines = Vec::new();
    let mut pos = 0;
    while let Some(start) = raw[pos..].find("<p ") {
        let tag_start = pos + start;
        let Some(tag_end) = raw[tag_start..].find('>') else { break };
        let tag_header = &raw[tag_start..tag_start + tag_end];
        let Some(p_close) = raw[tag_start + tag_end..].find("</p>") else { break };
        let body = &raw[tag_start + tag_end + 1..tag_start + tag_end + p_close];
        pos = tag_start + tag_end + p_close + 4;

        let time_sec = if let Some(b_idx) = tag_header.find("begin=\"") {
            let after = &tag_header[b_idx + 7..];
            let end_quote = after.find('"').unwrap_or(after.len());
            parse_ttml_time(&after[..end_quote])
        } else if let Some(b_idx) = tag_header.find("begin='") {
            let after = &tag_header[b_idx + 7..];
            let end_quote = after.find('\'').unwrap_or(after.len());
            parse_ttml_time(&after[..end_quote])
        } else {
            continue;
        };

        let mut clean = String::with_capacity(body.len());
        let mut in_tag = false;
        for c in body.chars() {
            if c == '<' {
                in_tag = true;
            } else if c == '>' {
                in_tag = false;
            } else if !in_tag {
                clean.push(c);
            }
        }
        let clean = clean.trim().to_string();
        if !clean.is_empty() {
            lines.push(noktah_core::lrc::LrcLine {
                time: time_sec,
                text: clean,
            });
        }
    }
    lines
}

#[allow(dead_code)]
fn parse_ttml_time(s: &str) -> f64 {
    let s = s.trim();
    if s.contains(':') {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() == 2 {
            let m: f64 = parts[0].parse().unwrap_or(0.0);
            let sec: f64 = parts[1].parse().unwrap_or(0.0);
            m * 60.0 + sec
        } else if parts.len() == 3 {
            let h: f64 = parts[0].parse().unwrap_or(0.0);
            let m: f64 = parts[1].parse().unwrap_or(0.0);
            let sec: f64 = parts[2].parse().unwrap_or(0.0);
            h * 3600.0 + m * 60.0 + sec
        } else {
            0.0
        }
    } else {
        s.parse::<f64>().unwrap_or(0.0)
    }
}

fn parse_response(
    agent: &ureq::Agent,
    url: &str,
) -> Option<(Vec<noktah_core::lrc::LrcLine>, bool)> {
    let json: serde_json::Value = get_json(agent, url)?;

    if let Some(raw) = json["syncedLyrics"].as_str() {
        let lines = noktah_core::lrc::parse_lrc(raw);
        if !lines.is_empty() {
            return Some((lines, true));
        }
    }

    if let Some(raw) = json["plainLyrics"].as_str() {
        let lines: Vec<noktah_core::lrc::LrcLine> = raw
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| noktah_core::lrc::LrcLine {
                time: -1.0,
                text: l.trim().to_string(),
            })
            .collect();
        if !lines.is_empty() {
            return Some((lines, false));
        }
    }

    None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_base64() {
        assert_eq!(decode_base64("SGVsbG8gV29ybGQ=").as_deref(), Some("Hello World"));
        assert_eq!(decode_base64("WzAwOjAxLjAwXUxpcmlr").as_deref(), Some("[00:01.00]Lirik"));
        assert_eq!(decode_base64(""), Some("".to_string()));
    }

    #[test]
    fn test_parse_ttml_time() {
        assert!((parse_ttml_time("01:23.45") - 83.45).abs() < 0.001);
        assert!((parse_ttml_time("10.5") - 10.5).abs() < 0.001);
        assert!((parse_ttml_time("01:02:03") - 3723.0).abs() < 0.001);
    }
}
