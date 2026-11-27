//! Font yang dipakai antarmuka.
//!
//! Skynight untuk tipografi judul dan tampilan artistik modern, M PLUS Rounded 1c
//! untuk teks Jepang/CJK dengan gaya rounded yang serasi dengan Skynight,
//! Plus Jakarta Sans untuk teks isi dan fallback, JetBrains Mono untuk angka dan kode.
//!
//! Untuk bahasa Korea (Hangul) dan glif CJK yang tidak dicakup oleh M PLUS Rounded,
//! sistem memuat font bawaan Windows (Malgun Gothic `malgun.ttf` dan Microsoft YaHei `msyh.ttc`)
//! secara dinamis sebagai fallback di rantai font definitions.
//! Seluruh font esensial tetap dibundel secara statis di dalam binary.

use eframe::egui;

pub const FAMILY_SKYNIGHT: &str = "skynight";
pub const FAMILY_SEMIBOLD: &str = "jakarta-semibold";
pub const FAMILY_BOLD: &str = "jakarta-bold";
#[allow(dead_code)]
pub const FAMILY_JAPANESE: &str = "mplus-rounded";

pub fn install(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "skynight".into(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/Skynight.ttf")),
    );
    fonts.font_data.insert(
        "mplus-rounded".into(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/MPLUSRounded1c-Medium.ttf")),
    );
    fonts.font_data.insert(
        "jakarta".into(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/PlusJakartaSans-Regular.ttf")),
    );
    fonts.font_data.insert(
        "jakarta-semibold".into(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/PlusJakartaSans-SemiBold.ttf")),
    );
    fonts.font_data.insert(
        "jakarta-bold".into(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/PlusJakartaSans-Bold.ttf")),
    );
    fonts.font_data.insert(
        "jetbrains".into(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf")),
    );

    // Muat font CJK fallback dari sistem Windows (Korea: Malgun Gothic, Tionghoa: Microsoft YaHei)
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
    let font_dir = std::path::Path::new(&windir).join("Fonts");

    let mut fallback_cjk: Vec<String> = Vec::new();

    // 1. Korea (Hangul) - Malgun Gothic mencakup seluruh 11.172 suku kata Hangul + Jamo
    let korean_candidates = ["malgun.ttf", "gulim.ttc", "batang.ttc"];
    for cand in korean_candidates {
        let p = font_dir.join(cand);
        if p.exists() {
            if let Ok(bytes) = std::fs::read(&p) {
                fonts.font_data.insert("korean-fallback".into(), egui::FontData::from_owned(bytes));
                fallback_cjk.push("korean-fallback".into());
                break;
            }
        }
    }

    // 2. Tionghoa (Simplified/Traditional) - Microsoft YaHei / SimSun
    let chinese_candidates = ["msyh.ttc", "simsun.ttc"];
    for cand in chinese_candidates {
        let p = font_dir.join(cand);
        if p.exists() {
            if let Ok(bytes) = std::fs::read(&p) {
                fonts.font_data.insert("chinese-fallback".into(), egui::FontData::from_owned(bytes));
                fallback_cjk.push("chinese-fallback".into());
                break;
            }
        }
    }

    // Tipografi: Skynight sebagai font proporsional utama dengan fallback ke
    // M PLUS Rounded (Jepang/Kanji), lalu font sistem CJK (Korea/Tionghoa), lalu Jakarta.
    let mut prop = vec!["skynight".to_string(), "mplus-rounded".to_string()];
    prop.extend(fallback_cjk.iter().cloned());
    prop.push("jakarta".to_string());

    fonts.families.insert(
        egui::FontFamily::Proportional,
        prop.clone(),
    );

    let mut mono = vec!["jetbrains".to_string(), "mplus-rounded".to_string()];
    mono.extend(fallback_cjk.iter().cloned());
    fonts.families.insert(
        egui::FontFamily::Monospace,
        mono,
    );

    // Bobot lebih tebal butuh keluarga sendiri; egui tidak punya sumbu berat.
    fonts.families.insert(
        egui::FontFamily::Name(FAMILY_SKYNIGHT.into()),
        prop,
    );

    let mut semibold = vec![
        "jakarta-semibold".to_string(),
        "mplus-rounded".to_string(),
        "skynight".to_string(),
    ];
    semibold.extend(fallback_cjk.iter().cloned());
    semibold.push("jakarta".to_string());
    fonts.families.insert(
        egui::FontFamily::Name(FAMILY_SEMIBOLD.into()),
        semibold,
    );

    let mut bold = vec![
        "jakarta-bold".to_string(),
        "mplus-rounded".to_string(),
        "skynight".to_string(),
    ];
    bold.extend(fallback_cjk.iter().cloned());
    bold.push("jakarta".to_string());
    fonts.families.insert(
        egui::FontFamily::Name(FAMILY_BOLD.into()),
        bold,
    );

    ctx.set_fonts(fonts);
}

/// Teks Skynight untuk judul dan tampilan.
#[allow(dead_code)]
pub fn skynight(text: impl Into<String>, size: f32) -> egui::RichText {
    egui::RichText::new(text)
        .size(size)
        .family(egui::FontFamily::Name(FAMILY_SKYNIGHT.into()))
}

/// Teks semi-tebal.
#[allow(dead_code)]
pub fn semibold(text: impl Into<String>, size: f32) -> egui::RichText {
    egui::RichText::new(text)
        .size(size)
        .family(egui::FontFamily::Name(FAMILY_SEMIBOLD.into()))
}

/// Teks tebal.
#[allow(dead_code)]
pub fn bold(text: impl Into<String>, size: f32) -> egui::RichText {
    egui::RichText::new(text)
        .size(size)
        .family(egui::FontFamily::Name(FAMILY_BOLD.into()))
}

/// Angka monospace, untuk jam dan hitungan waktu.
#[allow(dead_code)]
pub fn mono(text: impl Into<String>, size: f32) -> egui::RichText {
    egui::RichText::new(text)
        .size(size)
        .family(egui::FontFamily::Monospace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fonts_install_supports_korean_and_cjk() {
        let ctx = egui::Context::default();
        install(&ctx);

        let _ = ctx.run(Default::default(), |ctx| {
            // Uji lirik Korea (Hangul) persis seperti lagu Sour Grapes oleh LE SSERAFIM
            let has_hangul = ctx.fonts(|f| {
                f.has_glyphs(&egui::FontId::proportional(14.0), "낭만적인 fairy-tale")
                    && f.has_glyphs(&egui::FontId::proportional(14.0), "사다리를 오를 때")
                    && f.has_glyphs(&egui::FontId::proportional(14.0), "두 발아래 난 아찔해")
            });
            assert!(has_hangul, "Font definitions must support Korean Hangul lyrics!");

            // Uji teks Jepang (Hiragana/Katakana/Kanji)
            let has_japanese = ctx.fonts(|f| {
                f.has_glyphs(&egui::FontId::proportional(14.0), "夜に駆ける YOASOBI")
            });
            assert!(has_japanese, "Font definitions must support Japanese lyrics!");

            // Uji bobot semibold dan bold
            let has_korean_bold = ctx.fonts(|f| {
                f.has_glyphs(
                    &egui::FontId::new(14.0, egui::FontFamily::Name(FAMILY_BOLD.into())),
                    "르세라핌 LE SSERAFIM",
                )
            });
            assert!(has_korean_bold, "Bold font family must support Korean characters!");
        });
    }
}
