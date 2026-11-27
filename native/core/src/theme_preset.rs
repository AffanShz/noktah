//! Tema Preset kurasi dan serialisasi JSON untuk Noktah.
//!
//! Logika murni: tidak ada dependensi ke `egui` atau `windows` di sini.
//! Mengatur palet warna, opasitas, kilau tepi (specular rim), dan efek kaca.

use crate::settings::{default_true, Accent, GlassEffect, Settings};
use serde::{Deserialize, Serialize};

/// Preset tema tampilan lengkap yang dapat diekspor dan diimpor.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct ThemePreset {
    pub schema_version: u32,
    pub name: String,
    pub accent: Accent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_accent_hex: Option<String>,
    #[serde(default = "default_true")]
    pub dynamic_accent: bool,
    pub glass_effect: GlassEffect,
    pub opacity: f32,
    pub glass_tint_rgb: [u8; 3],
    pub glass_specular: f32,
    pub glass_blur_radius: u8,
    pub glass_saturation: f32,
    #[serde(default = "default_true")]
    pub squircle: bool,
    #[serde(default)]
    pub ears: bool,
}

impl Default for ThemePreset {
    fn default() -> Self {
        Self {
            schema_version: 1,
            name: "Custom".to_string(),
            accent: Accent::Teal,
            custom_accent_hex: None,
            dynamic_accent: true,
            glass_effect: GlassEffect::Light,
            opacity: 0.94,
            glass_tint_rgb: [0, 0, 0],
            glass_specular: 0.0,
            glass_blur_radius: 3,
            glass_saturation: 1.35,
            squircle: true,
            ears: false,
        }
    }
}

/// Mengurai string kode warna heksadesimal 6 digit `#RRGGBB` atau `RRGGBB`.
pub fn parse_hex_rgb(raw: &str) -> Option<[u8; 3]> {
    let s = raw.trim();
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some([r, g, b])
}

/// Memformat nilai RGB `[u8; 3]` menjadi format `#RRGGBB` huruf kapital.
pub fn format_hex_rgb(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

impl ThemePreset {
    /// Lima preset bawaan hasil kurasi desain.
    pub fn built_in_presets() -> Vec<ThemePreset> {
        vec![
            ThemePreset {
                schema_version: 1,
                name: "Obsidian".to_string(),
                accent: Accent::Teal,
                custom_accent_hex: None,
                dynamic_accent: true,
                glass_effect: GlassEffect::Light,
                opacity: 0.94,
                glass_tint_rgb: [0, 0, 0],
                glass_specular: 0.0,
                glass_blur_radius: 3,
                glass_saturation: 1.35,
                squircle: true,
                ears: false,
            },
            ThemePreset {
                schema_version: 1,
                name: "Liquid Glass".to_string(),
                accent: Accent::Blue,
                custom_accent_hex: None,
                dynamic_accent: true,
                glass_effect: GlassEffect::Full,
                opacity: 0.72,
                glass_tint_rgb: [18, 24, 36],
                glass_specular: 0.65,
                glass_blur_radius: 4,
                glass_saturation: 1.55,
                squircle: true,
                ears: true,
            },
            ThemePreset {
                schema_version: 1,
                name: "Midnight Frost".to_string(),
                accent: Accent::Blue,
                custom_accent_hex: None,
                dynamic_accent: true,
                glass_effect: GlassEffect::Full,
                opacity: 0.80,
                glass_tint_rgb: [12, 18, 32],
                glass_specular: 0.45,
                glass_blur_radius: 3,
                glass_saturation: 1.35,
                squircle: true,
                ears: false,
            },
            ThemePreset {
                schema_version: 1,
                name: "Aurora".to_string(),
                accent: Accent::Pink,
                custom_accent_hex: None,
                dynamic_accent: true,
                glass_effect: GlassEffect::Full,
                opacity: 0.76,
                glass_tint_rgb: [24, 14, 34],
                glass_specular: 0.55,
                glass_blur_radius: 4,
                glass_saturation: 1.60,
                squircle: true,
                ears: true,
            },
            ThemePreset {
                schema_version: 1,
                name: "Emerald Slate".to_string(),
                accent: Accent::Green,
                custom_accent_hex: None,
                dynamic_accent: true,
                glass_effect: GlassEffect::Light,
                opacity: 0.88,
                glass_tint_rgb: [10, 24, 20],
                glass_specular: 0.30,
                glass_blur_radius: 2,
                glass_saturation: 1.25,
                squircle: true,
                ears: false,
            },
        ]
    }

    /// Menghasilkan identitas slug huruf kecil untuk preset (misal "liquid-glass").
    pub fn id_slug(&self) -> String {
        match self.name.to_lowercase().as_str() {
            "obsidian" => "obsidian".into(),
            "liquid glass" | "liquid-glass" => "liquid-glass".into(),
            "midnight frost" | "midnight-frost" => "midnight-frost".into(),
            "aurora" => "aurora".into(),
            "emerald slate" | "emerald-slate" => "emerald-slate".into(),
            _ => {
                let slug: String = self
                    .name
                    .to_lowercase()
                    .chars()
                    .map(|c| if c.is_alphanumeric() { c } else { '-' })
                    .collect();
                let trimmed = slug.trim_matches('-');
                if trimmed.is_empty() {
                    "custom".into()
                } else {
                    trimmed.to_string()
                }
            }
        }
    }

    /// Membuat instance `ThemePreset` dari setelan saat ini.
    pub fn from_settings(name: &str, s: &Settings) -> Self {
        let mut preset = Self {
            schema_version: 1,
            name: if name.trim().is_empty() {
                "Custom".to_string()
            } else {
                name.trim().to_string()
            },
            accent: s.accent,
            custom_accent_hex: s.custom_accent_hex.clone(),
            dynamic_accent: s.dynamic_accent,
            glass_effect: s.glass_effect,
            opacity: s.opacity,
            glass_tint_rgb: s.glass_tint_rgb,
            glass_specular: s.glass_specular,
            glass_blur_radius: s.glass_blur_radius,
            glass_saturation: s.glass_saturation,
            squircle: s.squircle,
            ears: s.ears,
        };
        preset.sanitize();
        preset
    }

    /// Menerapkan preset ke setelan `Settings`, men-sanitize dan menyimpan slug preset.
    pub fn apply_to(&self, s: &mut Settings) {
        let mut clean = self.clone();
        clean.sanitize();
        let slug = clean.id_slug();

        s.accent = clean.accent;
        s.custom_accent_hex = clean.custom_accent_hex;
        s.dynamic_accent = clean.dynamic_accent;
        s.glass_effect = clean.glass_effect;
        s.opacity = clean.opacity;
        s.glass_tint_rgb = clean.glass_tint_rgb;
        s.glass_specular = clean.glass_specular;
        s.glass_blur_radius = clean.glass_blur_radius;
        s.glass_saturation = clean.glass_saturation;
        s.squircle = clean.squircle;
        s.ears = clean.ears;
        s.theme_preset = slug;
        s.sanitize();
    }

    /// Memastikan seluruh nilai parameter berada dalam rentang aman.
    pub fn sanitize(&mut self) {
        if self.schema_version == 0 {
            self.schema_version = 1;
        }

        let trimmed_name = self.name.trim();
        if trimmed_name.is_empty() {
            self.name = "Custom".to_string();
        } else if trimmed_name.chars().count() > 40 {
            self.name = trimmed_name.chars().take(40).collect();
        } else {
            self.name = trimmed_name.to_string();
        }

        self.opacity = if self.opacity.is_finite() {
            self.opacity.clamp(0.5, 1.0)
        } else {
            0.94
        };

        self.glass_specular = if self.glass_specular.is_finite() {
            self.glass_specular.clamp(0.0, 1.0)
        } else {
            0.0
        };

        self.glass_blur_radius = self.glass_blur_radius.clamp(1, 5);

        self.glass_saturation = if self.glass_saturation.is_finite() {
            self.glass_saturation.clamp(0.8, 2.0)
        } else {
            1.35
        };

        if let Some(raw) = &self.custom_accent_hex {
            if let Some(rgb) = parse_hex_rgb(raw) {
                self.custom_accent_hex = Some(format_hex_rgb(rgb));
            } else {
                self.custom_accent_hex = None;
            }
        }
    }

    /// Mengonversi preset ke string JSON pretty.
    pub fn to_json_pretty(&self) -> Result<String, String> {
        let mut clean = self.clone();
        clean.sanitize();
        serde_json::to_string_pretty(&clean).map_err(|e| e.to_string())
    }

    /// Membaca dan menvalidasi preset dari string JSON (maksimal 32 KB).
    pub fn from_json_str(raw: &str) -> Result<ThemePreset, String> {
        if raw.len() > 32 * 1024 {
            return Err("Preset payload exceeds 32 KB limit".into());
        }
        let val: serde_json::Value =
            serde_json::from_str(raw).map_err(|e| format!("Invalid JSON: {e}"))?;
        let obj = val
            .as_object()
            .ok_or_else(|| "Preset JSON must be an object".to_string())?;

        const RECOGNIZED_KEYS: &[&str] = &[
            "schema_version",
            "accent",
            "custom_accent_hex",
            "glass_effect",
            "opacity",
            "glass_tint_rgb",
            "glass_specular",
            "glass_blur_radius",
            "glass_saturation",
        ];

        let has_recognized_key = obj.keys().any(|k| RECOGNIZED_KEYS.contains(&k.as_str()));
        if !has_recognized_key {
            return Err("Preset JSON does not contain any recognized theme preset keys".into());
        }

        let mut preset: ThemePreset =
            serde_json::from_value(val).map_err(|e| format!("Invalid preset structure: {e}"))?;
        preset.sanitize();
        Ok(preset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_format_hex_rgb() {
        assert_eq!(parse_hex_rgb("#4DA3FF"), Some([0x4D, 0xA3, 0xFF]));
        assert_eq!(parse_hex_rgb("4da3ff"), Some([0x4D, 0xA3, 0xFF]));
        assert_eq!(parse_hex_rgb("  #FF6B9D  "), Some([0xFF, 0x6B, 0x9D]));
        assert_eq!(format_hex_rgb([0x4D, 0xA3, 0xFF]), "#4DA3FF");

        assert_eq!(parse_hex_rgb(""), None);
        assert_eq!(parse_hex_rgb("#12345"), None);
        assert_eq!(parse_hex_rgb("#1234567"), None);
        assert_eq!(parse_hex_rgb("#GGGGGG"), None);
        assert_eq!(parse_hex_rgb("#+1+2+3"), None);
        assert_eq!(parse_hex_rgb("+#1+2+3"), None);
        assert_eq!(parse_hex_rgb("+1+2+3"), None);
    }

    #[test]
    fn test_built_in_presets_count_and_slugs() {
        let presets = ThemePreset::built_in_presets();
        assert_eq!(presets.len(), 5);

        let slugs: Vec<String> = presets.iter().map(|p| p.id_slug()).collect();
        assert_eq!(
            slugs,
            vec![
                "obsidian",
                "liquid-glass",
                "midnight-frost",
                "aurora",
                "emerald-slate"
            ]
        );
    }

    #[test]
    fn test_sanitize_clamps_and_normalizes() {
        let mut p = ThemePreset {
            schema_version: 0,
            name: "   ".into(),
            accent: Accent::Blue,
            custom_accent_hex: Some("#4da3ff".into()),
            dynamic_accent: true,
            glass_effect: GlassEffect::Full,
            opacity: 0.1, // out of range
            glass_tint_rgb: [10, 20, 30],
            glass_specular: 2.5, // out of range
            glass_blur_radius: 10, // out of range
            glass_saturation: 5.0, // out of range
            squircle: true,
            ears: true,
        };
        p.sanitize();

        assert_eq!(p.schema_version, 1);
        assert_eq!(p.name, "Custom");
        assert_eq!(p.custom_accent_hex, Some("#4DA3FF".into()));
        assert_eq!(p.opacity, 0.5);
        assert_eq!(p.glass_specular, 1.0);
        assert_eq!(p.glass_blur_radius, 5);
        assert_eq!(p.glass_saturation, 2.0);
    }

    #[test]
    fn test_json_roundtrip_and_size_guard() {
        let original = ThemePreset::built_in_presets()[1].clone();
        let json = original.to_json_pretty().unwrap();

        let parsed = ThemePreset::from_json_str(&json).unwrap();
        assert_eq!(original, parsed);

        let huge = " ".repeat(33 * 1024);
        assert!(ThemePreset::from_json_str(&huge).is_err());

        // Reject empty objects or arbitrary non-preset JSON
        assert!(ThemePreset::from_json_str("{}").is_err());
        assert!(ThemePreset::from_json_str(r#"{"hello": "world"}"#).is_err());
        assert!(ThemePreset::from_json_str(r#"{"foo": 123, "bar": false}"#).is_err());
    }

    #[test]
    fn test_apply_to_settings() {
        let preset = ThemePreset::built_in_presets()[1].clone(); // liquid-glass
        let mut settings = Settings::default();

        preset.apply_to(&mut settings);

        assert_eq!(settings.theme_preset, "liquid-glass");
        assert_eq!(settings.opacity, 0.72);
        assert_eq!(settings.glass_specular, 0.65);
        assert_eq!(settings.glass_blur_radius, 4);
        assert_eq!(settings.glass_saturation, 1.55);
        assert_eq!(settings.glass_tint_rgb, [18, 24, 36]);
        assert!(settings.ears);
    }
}
