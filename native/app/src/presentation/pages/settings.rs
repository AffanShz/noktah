//! Isi jendela Setelan dengan tabbed UI .
//!
//! Setiap tab punya fokus: General (start behavior, language), Display (window chrome),
//! Appearance (colors, glass effect), Music (lyrics, pill), Tasks (todo reminders),
//! Pomodoro (focus settings), AI (provider, model), About (version).

use crate::app::{Noktah, NotchPage, SettingsTab};
use crate::presentation::{icons::Icon, style, theme::Theme, widgets};
use eframe::egui::{self, Color32, Id, Rect, Rounding, Sense, Stroke};
use noktah_core::{format, i18n, model::PomodoroMode, model::Todo};

impl Noktah {
    pub(crate) fn draw_settings_tab_content(&mut self, ui: &mut egui::Ui) {
        let theme = self.theme;
        let lang = self.lang;

        match self.settings_tab {
            SettingsTab::General => self.draw_general_tab(ui, lang, theme),
            SettingsTab::Display => self.draw_display_tab(ui, lang, theme),
            SettingsTab::Appearance => self.draw_appearance_tab(ui, lang, theme),
            SettingsTab::Music => self.draw_music_tab(ui, lang, theme),
            SettingsTab::Todo => self.draw_todo_tab(ui, lang, theme),
            SettingsTab::Pomodoro => self.draw_pomodoro_tab(ui, lang, theme),
            SettingsTab::Ai => self.draw_ai_tab(ui, lang, theme),
            SettingsTab::About => self.draw_about_tab(ui, lang, theme),
        }
    }

    /// Tab 0: General — start behavior, language
    fn draw_general_tab(&mut self, ui: &mut egui::Ui, lang: noktah_core::settings::Lang, theme: Theme) {
        widgets::section(ui, "Startup", &theme);
        widgets::card_group(ui, |ui| {
            widgets::row_desc(
                ui,
                i18n::t(lang, "settings.start_with_windows"),
                i18n::t(lang, "settings.start_with_windows_desc"),
                &theme,
                |ui| {
                    if widgets::toggle(ui, &mut self.settings.start_with_windows, &theme) {
                        let _ = crate::infrastructure::os::set_autostart(self.settings.start_with_windows);
                        self.touch_settings();
                    }
                },
            );
        });

        widgets::section(ui, i18n::t(lang, "settings.language"), &theme);
        widgets::card_group(ui, |ui| {
            ui.horizontal(|ui| {
                if widgets::chip_button(
                    ui,
                    "Indonesia",
                    self.settings.language == noktah_core::Lang::Id,
                    &theme,
                ) {
                    self.settings.language = noktah_core::Lang::Id;
                    self.lang = noktah_core::Lang::Id;
                    self.touch_settings();
                }
                if widgets::chip_button(
                    ui,
                    "English",
                    self.settings.language == noktah_core::Lang::En,
                    &theme,
                ) {
                    self.settings.language = noktah_core::Lang::En;
                    self.lang = noktah_core::Lang::En;
                    self.touch_settings();
                }
            });
        });
    }

    /// Tab 1: Display — window chrome, notch shape, animations
    fn draw_display_tab(&mut self, ui: &mut egui::Ui, lang: noktah_core::settings::Lang, theme: Theme) {
        widgets::section(ui, "Window", &theme);
        widgets::card_group(ui, |ui| {
            widgets::row_desc(
                ui,
                i18n::t(lang, "settings.always_on_top"),
                i18n::t(lang, "settings.always_on_top_desc"),
                &theme,
                |ui| {
                    if widgets::toggle(ui, &mut self.settings.always_on_top, &theme) {
                        crate::infrastructure::os::set_always_on_top(self.settings.always_on_top);
                        self.touch_settings();
                    }
                },
            );
            widgets::card_divider(ui);
            widgets::row_desc(
                ui,
                i18n::t(lang, "settings.start_expanded"),
                i18n::t(lang, "settings.start_expanded_desc"),
                &theme,
                |ui| {
                    if widgets::toggle(ui, &mut self.settings.start_expanded, &theme) {
                        self.touch_settings();
                    }
                },
            );
        });

        widgets::section(ui, "Notch & Motion", &theme);
        widgets::card_group(ui, |ui| {
            widgets::row_desc(
                ui,
                i18n::t(lang, "settings.minimal_notch"),
                "Ciutkan notch jadi garis tipis saat panel tertutup",
                &theme,
                |ui| {
                    if widgets::toggle(ui, &mut self.settings.minimal_notch, &theme) {
                        self.touch_settings();
                    }
                },
            );
            widgets::card_divider(ui);
            widgets::row(ui, i18n::t(lang, "settings.squircle"), &theme, |ui| {
                if widgets::toggle(ui, &mut self.settings.squircle, &theme) {
                    self.touch_settings();
                }
            });
            widgets::card_divider(ui);
            widgets::row(ui, i18n::t(lang, "settings.animation"), &theme, |ui| {
                if widgets::toggle(ui, &mut self.settings.force_animation, &theme) {
                    self.reduce_motion =
                        !crate::infrastructure::os::animations_enabled() && !self.settings.force_animation;
                    self.touch_settings();
                }
            });
        });
    }

    /// Tab 2: Appearance — accent color, glass effect, opacity
    fn draw_appearance_tab(&mut self, ui: &mut egui::Ui, lang: noktah_core::settings::Lang, theme: Theme) {
        widgets::section(ui, "Theme & Accent", &theme);
        widgets::card_group(ui, |ui| {
            widgets::row_desc(
                ui,
                i18n::t(lang, "settings.accent"),
                i18n::t(lang, "settings.accent_desc"),
                &theme,
                |ui| {
                    for accent in noktah_core::Accent::ALL {
                        let dipilih = self.settings.accent == accent;
                        let [r, g, b] = accent.rgb();
                        if widgets::swatch(ui, Color32::from_rgb(r, g, b), dipilih, &theme) {
                            self.settings.accent = accent;
                            self.touch_settings();
                        }
                    }
                },
            );
            widgets::card_divider(ui);
            widgets::row_desc(
                ui,
                i18n::t(lang, "settings.dynamic_accent"),
                "Aksen dinamis mengikuti warna album musik",
                &theme,
                |ui| {
                    if widgets::toggle(ui, &mut self.settings.dynamic_accent, &theme) {
                        self.touch_settings();
                    }
                },
            );
            widgets::card_divider(ui);
            widgets::row_desc(
                ui,
                i18n::t(lang, "settings.glass"),
                &format!("{:.0}%", self.settings.opacity * 100.0),
                &theme,
                |ui| {
                    ui.spacing_mut().slider_width = 150.0;
                    if ui
                        .add(
                            egui::Slider::new(&mut self.settings.opacity, 0.5..=1.0)
                                .show_value(false)
                                .trailing_fill(true),
                        )
                        .changed()
                    {
                        self.touch_settings();
                    }
                },
            );
            widgets::card_divider(ui);
            widgets::row(ui, i18n::t(lang, "settings.ears"), &theme, |ui| {
                if widgets::toggle(ui, &mut self.settings.ears, &theme) {
                    self.touch_settings();
                }
            });
        });

        widgets::section(ui, i18n::t(lang, "settings.glass_effect"), &theme);
        widgets::card_group(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for efek in noktah_core::settings::GlassEffect::ALL {
                    let dipilih = self.settings.glass_effect == efek;
                    if widgets::chip_button(ui, efek.label(), dipilih, &theme) {
                        self.settings.glass_effect = efek;
                        self.touch_settings();
                    }
                }
            });
        });
    }

    /// Tab 3: Music — lyrics, pill content
    fn draw_music_tab(&mut self, ui: &mut egui::Ui, lang: noktah_core::settings::Lang, theme: Theme) {
        widgets::section(ui, i18n::t(lang, "settings.lyrics"), &theme);
        widgets::card_group(ui, |ui| {
            widgets::row_desc(
                ui,
                i18n::t(lang, "settings.lyrics_enabled"),
                i18n::t(lang, "settings.lyrics_enabled_desc"),
                &theme,
                |ui| {
                    if widgets::toggle(ui, &mut self.settings.lyrics_enabled, &theme) {
                        self.touch_settings();
                    }
                },
            );
            widgets::card_divider(ui);
            widgets::row_desc(
                ui,
                i18n::t(lang, "settings.lyric_offset"),
                &format!("{} ms", self.settings.lyric_offset_ms),
                &theme,
                |ui| {
                    ui.spacing_mut().slider_width = 150.0;
                    if ui
                        .add(
                            egui::Slider::new(&mut self.settings.lyric_offset_ms, -3000..=3000)
                                .show_value(false)
                                .trailing_fill(true),
                        )
                        .changed()
                    {
                        self.touch_settings();
                    }
                },
            );
        });

        // Konten baris compact diatur sekali di sini lewat `draw_mini_row_settings`.
        // Sebelumnya ada section "Pill" terpisah yang mengatur item yang sama
        // (jam, cuaca, judul, lirik) tapi saklarnya tidak tersambung ke render —
        // duplikat yang membingungkan, jadi dihapus.
        ui.add_space(style::S2);
        self.draw_mini_row_settings(ui, lang, &theme);
    }

    /// Menu Pengaturan Baris Compact / Minimized Notch (pilihan elemen yang tampil saat notch tertutup)
    fn draw_mini_row_settings(&mut self, ui: &mut egui::Ui, lang: noktah_core::settings::Lang, theme: &Theme) {
        widgets::section(ui, i18n::t(lang, "settings.mini_row"), theme);
        ui.label(
            egui::RichText::new("Pilih komponen yang tampil di baris compact saat notch tertutup:")
                .size(style::T_CAPTION)
                .color(theme.dim),
        );
        ui.add_space(style::S1);

        // Preset Cepat Siap Pakai
        ui.horizontal_wrapped(|ui| {
            if widgets::chip_button(ui, "Mode Lirik", false, theme) {
                self.settings.apply_preset(noktah_core::MiniPreset::ModeLirik);
                self.touch_settings();
            }
            if widgets::chip_button(ui, "Fokus Lirik", false, theme) {
                self.settings.apply_preset(noktah_core::MiniPreset::FokusLirik);
                self.touch_settings();
            }
            if widgets::chip_button(ui, "Standar", false, theme) {
                self.settings.apply_preset(noktah_core::MiniPreset::Standar);
                self.touch_settings();
            }
            if widgets::chip_button(ui, "Lengkap", false, theme) {
                self.settings.apply_preset(noktah_core::MiniPreset::Lengkap);
                self.touch_settings();
            }
        });
        ui.add_space(style::S2);

        // Toggle Bebas Setiap Elemen
        widgets::row_desc(
            ui,
            i18n::t(lang, "settings.mini_lyrics"),
            "Tampilkan lirik lagu berjalan satu baris",
            theme,
            |ui| {
                if widgets::toggle(ui, &mut self.settings.mini_lyrics, theme) {
                    self.touch_settings();
                }
            },
        );
        widgets::row_desc(
            ui,
            i18n::t(lang, "settings.mini_wave"),
            "Bilah equalizer audio bergerak",
            theme,
            |ui| {
                if widgets::toggle(ui, &mut self.settings.mini_wave, theme) {
                    self.touch_settings();
                }
            },
        );
        widgets::row_desc(
            ui,
            i18n::t(lang, "settings.mini_agent"),
            "Avatar Maskot Noki & status aktivitas Claude Code",
            theme,
            |ui| {
                if widgets::toggle(ui, &mut self.settings.mini_agent, theme) {
                    self.touch_settings();
                }
            },
        );
        widgets::row_desc(
            ui,
            i18n::t(lang, "settings.mini_art"),
            "Gambar sampul album melingkar",
            theme,
            |ui| {
                if widgets::toggle(ui, &mut self.settings.mini_art, theme) {
                    self.touch_settings();
                }
            },
        );
        widgets::row_desc(
            ui,
            i18n::t(lang, "settings.mini_title"),
            "Nama lagu dan artis musik",
            theme,
            |ui| {
                if widgets::toggle(ui, &mut self.settings.mini_title, theme) {
                    self.touch_settings();
                }
            },
        );
        widgets::row_desc(
            ui,
            i18n::t(lang, "settings.mini_progress"),
            "Pendar garis progres lagu di dasar pill",
            theme,
            |ui| {
                if widgets::toggle(ui, &mut self.settings.mini_progress, theme) {
                    self.touch_settings();
                }
            },
        );
        widgets::row_desc(
            ui,
            i18n::t(lang, "settings.mini_controls"),
            "Tombol putar, jeda, dan lagu berikutnya",
            theme,
            |ui| {
                if widgets::toggle(ui, &mut self.settings.mini_controls, theme) {
                    self.touch_settings();
                }
            },
        );
        widgets::row_desc(
            ui,
            i18n::t(lang, "settings.mini_clock"),
            "Penunjuk jam digital di pill",
            theme,
            |ui| {
                if widgets::toggle(ui, &mut self.settings.mini_clock, theme) {
                    self.touch_settings();
                }
            },
        );
        widgets::row_desc(
            ui,
            i18n::t(lang, "settings.mini_cpu"),
            i18n::t(lang, "settings.mini_monitor_desc"),
            theme,
            |ui| {
                if widgets::toggle(ui, &mut self.settings.mini_cpu, theme) {
                    self.touch_settings();
                }
            },
        );
        widgets::row_desc(
            ui,
            i18n::t(lang, "settings.mini_ram"),
            i18n::t(lang, "settings.mini_monitor_desc"),
            theme,
            |ui| {
                if widgets::toggle(ui, &mut self.settings.mini_ram, theme) {
                    self.touch_settings();
                }
            },
        );
    }

    /// Tab 4: Todo / Tasks — todo settings (placeholder for future reminders)
    fn draw_todo_tab(&mut self, ui: &mut egui::Ui, lang: noktah_core::settings::Lang, theme: Theme) {
        widgets::section(ui, "Tasks & Reminders", &theme);
        widgets::card_group(ui, |ui| {
            ui.label(
                egui::RichText::new("Task reminders coming soon")
                    .size(style::T_BODY)
                    .color(theme.dim),
            );
        });

        widgets::section(ui, "Profile & Location", &theme);
        widgets::card_group(ui, |ui| {
            widgets::row(ui, i18n::t(lang, "settings.city"), &theme, |ui| {
                let mut teks = self.settings.city.clone();
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut teks)
                            .desired_width(170.0)
                            .hint_text("Jakarta"),
                    )
                    .changed()
                {
                    self.settings.city = teks;
                    self.touch_settings();
                }
            });
            widgets::card_divider(ui);
            widgets::row(ui, i18n::t(lang, "settings.name"), &theme, |ui| {
                let mut teks = self.settings.user_name.clone();
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut teks)
                            .desired_width(170.0)
                            .hint_text("Kamu"),
                    )
                    .changed()
                {
                    self.settings.user_name = teks;
                    self.touch_settings();
                }
            });
        });
    }

    /// Tab 5: Pomodoro — focus timer settings
    fn draw_pomodoro_tab(&mut self, ui: &mut egui::Ui, _lang: noktah_core::settings::Lang, theme: Theme) {
        widgets::section(ui, "Focus Session", &theme);
        widgets::card_group(ui, |ui| {
            widgets::row(
                ui,
                "Focus length",
                &theme,
                |ui| {
                    let mut menit = self.settings.focus_minutes as f32;
                    ui.spacing_mut().slider_width = 150.0;
                    if ui
                        .add(
                            egui::Slider::new(&mut menit, 5.0..=90.0)
                                .step_by(5.0)
                                .show_value(false),
                        )
                        .changed()
                    {
                        self.settings.focus_minutes = menit as u32;
                        self.pomodoro.set_mode(PomodoroMode::Focus);
                        self.touch_settings();
                    }
                    ui.label(style::cap(
                        format!("{} min", self.settings.focus_minutes),
                        theme.faint,
                    ));
                },
            );
            widgets::card_divider(ui);
            widgets::row(ui, "Break length", &theme, |ui| {
                let mut menit = self.settings.break_minutes as f32;
                ui.spacing_mut().slider_width = 150.0;
                if ui
                    .add(
                        egui::Slider::new(&mut menit, 1.0..=30.0)
                            .step_by(1.0)
                            .show_value(false),
                    )
                    .changed()
                {
                    self.settings.break_minutes = menit as u32;
                    self.touch_settings();
                }
                ui.label(style::cap(
                    format!("{} min", self.settings.break_minutes),
                    theme.faint,
                ));
            });
        });

        widgets::section(ui, "Activity", &theme);
        widgets::card_group(ui, |ui| {
            ui.label(style::cap(
                format!("{} sessions completed today", self.pomodoro.sessions_today),
                theme.ink,
            ));
        });
    }

    /// Tab 6: AI — provider, model, API key
    fn draw_ai_tab(&mut self, ui: &mut egui::Ui, lang: noktah_core::settings::Lang, theme: Theme) {
        widgets::section(ui, "AI Model Configuration", &theme);
        widgets::card_group(ui, |ui| {
            widgets::row(ui, i18n::t(lang, "settings.provider"), &theme, |ui| {
                egui::ComboBox::from_id_salt("penyedia")
                    .selected_text(self.settings.provider.label())
                    .show_ui(ui, |ui| {
                        for p in noktah_core::Provider::ALL {
                            if ui
                                .selectable_label(self.settings.provider == p, p.label())
                                .clicked()
                            {
                                self.settings.switch_provider(p);
                                self.touch_settings();
                            }
                        }
                    });
            });
            widgets::card_divider(ui);
            widgets::row(ui, i18n::t(lang, "settings.model"), &theme, |ui| {
                if ui
                    .add(egui::TextEdit::singleline(&mut self.settings.model).desired_width(190.0))
                    .changed()
                {
                    self.touch_settings();
                }
            });
        });
    }

    /// Tab 7: About — version, credits
    fn draw_about_tab(&mut self, ui: &mut egui::Ui, lang: noktah_core::settings::Lang, theme: Theme) {
        widgets::section(ui, "Noktah", &theme);
        widgets::card_group(ui, |ui| {
            ui.label(
                egui::RichText::new(format!("Noktah v{}", env!("CARGO_PKG_VERSION")))
                    .size(style::T_TITLE)
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.add_space(4.0);
            ui.label(style::text("Native Windows Dynamic Island & Notch", theme.dim));
            ui.add_space(8.0);
            ui.label(style::cap("© 2026 AffanShz. MIT License.", theme.faint));
        });

        widgets::section(ui, i18n::t(lang, "settings.data"), &theme);
        widgets::card_group(ui, |ui| {
            ui.horizontal(|ui| {
                if widgets::chip_button(ui, "Ekspor", false, &theme) {
                    crate::infrastructure::store::save("settings-export.json", &self.settings);
                    self.banner = Some((
                        "Setelan diekspor ke settings-export.json".into(),
                        std::time::Instant::now(),
                    ));
                }
                if widgets::chip_button(ui, "Impor", false, &theme) {
                    let mut baru: noktah_core::Settings =
                        crate::infrastructure::store::load("settings-export.json");
                    baru.sanitize();
                    self.settings = baru;
                    self.touch_settings();
                    self.banner = Some(("Setelan diimpor".into(), std::time::Instant::now()));
                }
                if widgets::chip_button(ui, i18n::t(lang, "settings.reset"), false, &theme) {
                    crate::infrastructure::store::wipe();
                    self.settings = noktah_core::Settings::default();
                    self.todos.clear();
                    self.chat.clear();
                    self.touch_settings();
                }
            });
        });
    }

    // Legacy flat settings panel (kept for reference)
    pub(crate) fn tab_settings(&mut self, ui: &mut egui::Ui) {
        // This method is now a no-op since we use tabbed UI
        // Kept for backwards compatibility if needed
        self.draw_settings_tab_content(ui);
    }
}
