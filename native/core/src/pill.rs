//! Lebar pill dihitung dari segmen yang aktif, bukan diukur dari teks.
//!
//! Judul lagu dan baris lirik berganti-ganti; kalau lebarnya diukur dari isi,
//! pill akan "bernafas" setiap kali lirik berganti. Angka tetap membuat
//! bentuknya stabil.

pub const SEG_CLOCK: f32 = 46.0;
pub const SEG_WEATHER: f32 = 50.0;
pub const SEG_TASKS: f32 = 34.0;
pub const SEG_EQ: f32 = 16.0;
pub const SEG_TITLE: f32 = 116.0;
pub const SEG_LYRIC: f32 = 180.0;

const PADDING: f32 = 30.0;
const DOT: f32 = 6.0;
const GAP: f32 = 9.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PillShow {
    pub clock: bool,
    pub weather: bool,
    pub title: bool,
    pub lyrics: bool,
}

impl Default for PillShow {
    fn default() -> Self {
        Self {
            clock: true,
            weather: true,
            title: true,
            lyrics: true,
        }
    }
}

/// Lebar pill dalam piksel logis.
pub fn compute_pill(playing: bool, show: PillShow) -> f32 {
    let mut segments: Vec<f32> = Vec::new();

    if playing {
        segments.push(SEG_EQ);
        if show.title {
            segments.push(SEG_TITLE);
        }
        if show.lyrics {
            segments.push(SEG_LYRIC);
        }
        if show.clock {
            segments.push(SEG_CLOCK);
        }
        if show.weather {
            segments.push(SEG_WEATHER);
        }
    } else {
        if show.clock {
            segments.push(SEG_CLOCK);
        }
        if show.weather {
            segments.push(SEG_WEATHER);
        }
        segments.push(SEG_TASKS);
    }

    let total: f32 = segments.iter().sum();
    (PADDING + DOT + GAP * segments.len() as f32 + total).round()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tanpa_lagu_hanya_jam_cuaca_tugas() {
        let w = compute_pill(false, PillShow::default());
        // 30 + 6 + 9*3 + 46 + 50 + 34
        assert_eq!(w, 193.0);
    }

    #[test]
    fn tanpa_lagu_dan_tanpa_jam_lebih_sempit() {
        let show = PillShow {
            clock: false,
            ..Default::default()
        };
        assert_eq!(compute_pill(false, show), 193.0 - 46.0 - 9.0);
    }

    #[test]
    fn sedang_putar_lagu_menampilkan_semua() {
        let w = compute_pill(true, PillShow::default());
        // 30 + 6 + 9*5 + 16 + 116 + 180 + 46 + 50
        assert_eq!(w, 489.0);
    }

    #[test]
    fn mematikan_lirik_mempersempit_pill() {
        let show = PillShow {
            lyrics: false,
            ..Default::default()
        };
        assert_eq!(compute_pill(true, show), 489.0 - 180.0 - 9.0);
    }

    #[test]
    fn mematikan_semua_saat_putar_tersisa_equalizer() {
        let show = PillShow {
            clock: false,
            weather: false,
            title: false,
            lyrics: false,
        };
        assert_eq!(compute_pill(true, show), 30.0 + 6.0 + 9.0 + 16.0);
    }
}
