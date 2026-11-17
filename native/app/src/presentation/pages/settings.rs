//! Isi jendela Setelan dengan tabbed UI .
//!
//! Setiap tab punya fokus: General (start behavior, language), Display (window chrome),
//! Appearance (colors, glass effect), Music (lyrics, pill), Tasks (todo reminders),
//! Pomodoro (focus settings), AI (provider, model), About (version).

use crate::app::{Noktah, SettingsTab};
use crate::presentation::{style, theme::Theme, widgets};
use eframe::egui::{self, Color32};
use noktah_core::{i18n, model::PomodoroMode};

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
            widgets::card_divider(ui);
            widgets::row_desc(
                ui,
                i18n::t(lang, "settings.default_page"),
                i18n::t(lang, "settings.default_page_desc"),
                &theme,
                |ui| {
                    egui::ComboBox::from_id_salt("default_page")
                        .selected_text(self.settings.default_page.label(lang))
                        .show_ui(ui, |ui| {
                            for page in noktah_core::DefaultPage::ALL {
                                if ui
                                    .selectable_label(self.settings.default_page == page, page.label(lang))
                                    .clicked()
                                {
                                    self.settings.default_page = page;
                                    self.touch_settings();
                                }
                            }
                        });
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

        widgets::section(ui, "Navigation Menu", &theme);
        widgets::card_group(ui, |ui| {
            widgets::row_desc(
                ui,
                "Home",
                "Tampilkan halaman Dashboard di menu navigasi",
                &theme,
                |ui| {
                    if widgets::toggle(ui, &mut self.settings.nav_home, &theme) {
                        self.touch_settings();
                    }
                },
            );
            widgets::card_divider(ui);
            widgets::row_desc(
                ui,
                "Music",
                "Tampilkan pemutar musik & lirik di menu navigasi",
                &theme,
                |ui| {
                    if widgets::toggle(ui, &mut self.settings.nav_music, &theme) {
                        self.touch_settings();
                    }
                },
            );
            widgets::card_divider(ui);
            widgets::row_desc(
                ui,
                "Focus (Pomodoro)",
                "Tampilkan timer fokus Pomodoro di menu navigasi",
                &theme,
                |ui| {
                    if widgets::toggle(ui, &mut self.settings.nav_pomodoro, &theme) {
                        self.touch_settings();
                    }
                },
            );
            widgets::card_divider(ui);
            widgets::row_desc(
                ui,
                "Tasks",
                "Tampilkan daftar tugas to-do di menu navigasi",
                &theme,
                |ui| {
                    if widgets::toggle(ui, &mut self.settings.nav_tasks, &theme) {
                        self.touch_settings();
                    }
                },
            );
            widgets::card_divider(ui);
            widgets::row_desc(
                ui,
                "System",
                "Tampilkan monitor sistem CPU & RAM di menu navigasi",
                &theme,
                |ui| {
                    if widgets::toggle(ui, &mut self.settings.nav_system, &theme) {
                        self.touch_settings();
                    }
                },
            );
            widgets::card_divider(ui);
            widgets::row_desc(
                ui,
                "Ask (AI)",
                "Tampilkan obrolan AI di menu navigasi",
                &theme,
                |ui| {
                    if widgets::toggle(ui, &mut self.settings.nav_ask, &theme) {
                        self.touch_settings();
                    }
                },
            );
            widgets::card_divider(ui);
            widgets::row_desc(
                ui,
                "Agent",
                "Tampilkan agen Claude Code di menu navigasi",
                &theme,
                |ui| {
                    if widgets::toggle(ui, &mut self.settings.nav_agent, &theme) {
                        self.touch_settings();
                    }
                },
            );
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

        widgets::row_desc(
            ui,
            i18n::t(lang, "settings.mini_idle_on_pause"),
            i18n::t(lang, "settings.mini_idle_on_pause_desc"),
            theme,
            |ui| {
                if widgets::toggle(ui, &mut self.settings.mini_idle_on_pause, theme) {
                    self.touch_settings();
                }
            },
        );

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
    fn draw_pomodoro_tab(&mut self, ui: &mut egui::Ui, lang: noktah_core::settings::Lang, theme: Theme) {
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
                        if !self.pomodoro.running && self.pomodoro.mode == PomodoroMode::Focus {
                            self.pomodoro.remaining_secs = self.settings.focus_minutes * 60;
                            self.save_pomodoro();
                        }
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
                    if !self.pomodoro.running {
                        if self.pomodoro.mode == PomodoroMode::ShortBreak {
                            self.pomodoro.remaining_secs = self.settings.break_minutes * 60;
                            self.save_pomodoro();
                        } else if self.pomodoro.mode == PomodoroMode::LongBreak {
                            self.pomodoro.remaining_secs =
                                (self.settings.break_minutes * 3).max(15) * 60;
                            self.save_pomodoro();
                        }
                    }
                    self.touch_settings();
                }
                ui.label(style::cap(
                    format!("{} min", self.settings.break_minutes),
                    theme.faint,
                ));
            });
            widgets::card_divider(ui);
            widgets::row_desc(
                ui,
                i18n::t(lang, "settings.focus_dnd"),
                i18n::t(lang, "settings.focus_dnd_desc"),
                &theme,
                |ui| {
                    if widgets::toggle(ui, &mut self.settings.focus_dnd, &theme) {
                        self.touch_settings();
                    }
                },
            );
        });

        widgets::section(ui, "Activity", &theme);
        widgets::card_group(ui, |ui| {
            let summary = if lang == noktah_core::Lang::Id {
                format!(
                    "{} sesi selesai hari ini · {} sesi dalam 7 hari terakhir",
                    self.pomodoro.sessions_today,
                    self.pomodoro.weekly_total()
                )
            } else {
                format!(
                    "{} sessions completed today · {} in last 7 days",
                    self.pomodoro.sessions_today,
                    self.pomodoro.weekly_total()
                )
            };
            ui.label(style::cap(summary, theme.ink));
        });
    }

    /// Tab 6: AI — provider, model, API key & Mascot Hooks
    fn draw_ai_tab(&mut self, ui: &mut egui::Ui, lang: noktah_core::settings::Lang, theme: Theme) {
        let is_id = lang == noktah_core::settings::Lang::Id;

        widgets::section(
            ui,
            if is_id {
                "Konfigurasi Model AI"
            } else {
                "AI Model Configuration"
            },
            &theme,
        );
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

        widgets::section(
            ui,
            if is_id {
                "Integrasi Maskot & Agent Hooks (Noki)"
            } else {
                "Mascot & Agent Hooks (Noki)"
            },
            &theme,
        );
        widgets::card_group(ui, |ui| {
            ui.label(style::text(
                if is_id {
                    "Sinkronkan Aktivitas AI Agent ke Maskot"
                } else {
                    "Sync AI Agent Activity with Mascot"
                },
                theme.ink,
            ));
            ui.label(style::cap(
                if is_id {
                    "Noki akan otomatis berekspresi saat Claude Code, Antigravity, atau skrip CLI bekerja, berpikir, selesai, atau error."
                } else {
                    "Noki automatically reacts when Claude Code, Antigravity, or CLI scripts work, think, finish, or error."
                },
                theme.faint,
            ));

            widgets::card_divider(ui);

            widgets::row_desc(
                ui,
                if is_id {
                    "Prompt Hook Claude Code"
                } else {
                    "Claude Code Hook Prompt"
                },
                if is_id {
                    "Prompt instruksi otomatis untuk dipaste ke Claude Code"
                } else {
                    "Ready-to-use prompt for Claude Code chat"
                },
                &theme,
                |ui| {
                    if widgets::chip_button(
                        ui,
                        if is_id {
                            "Salin Prompt"
                        } else {
                            "Copy Prompt"
                        },
                        false,
                        &theme,
                    ) {
                        ui.output_mut(|o| o.copied_text = PROMPT_CLAUDE_CODE_HOOKS.to_string());
                        self.banner = Some((
                            if is_id {
                                "Prompt hook Claude Code disalin ke clipboard!".into()
                            } else {
                                "Claude Code hook prompt copied!".into()
                            },
                            std::time::Instant::now(),
                        ));
                    }
                },
            );

            widgets::card_divider(ui);

            widgets::row_desc(
                ui,
                if is_id {
                    "Konfigurasi JSON Hooks"
                } else {
                    "Hooks JSON Configuration"
                },
                if is_id {
                    "Snippet JSON untuk settings.json Claude Code"
                } else {
                    "Raw JSON snippet for settings.json"
                },
                &theme,
                |ui| {
                    if widgets::chip_button(
                        ui,
                        if is_id {
                            "Salin JSON"
                        } else {
                            "Copy JSON"
                        },
                        false,
                        &theme,
                    ) {
                        ui.output_mut(|o| o.copied_text = JSON_CLAUDE_CODE_HOOKS.to_string());
                        self.banner = Some((
                            if is_id {
                                "Konfigurasi JSON hooks disalin ke clipboard!".into()
                            } else {
                                "Hooks JSON snippet copied!".into()
                            },
                            std::time::Instant::now(),
                        ));
                    }
                },
            );

            widgets::card_divider(ui);

            widgets::row_desc(
                ui,
                if is_id {
                    "Antigravity (hooks.json)"
                } else {
                    "Antigravity (hooks.json)"
                },
                if is_id {
                    "Hubungkan agy IDE/CLI ke maskot & refresh agy-quota.json"
                } else {
                    "Connect agy IDE/CLI to mascot & refresh agy-quota.json"
                },
                &theme,
                |ui| {
                    if widgets::chip_button(
                        ui,
                        if is_id {
                            "Salin hooks.json"
                        } else {
                            "Copy hooks.json"
                        },
                        false,
                        &theme,
                    ) {
                        ui.output_mut(|o| o.copied_text = JSON_ANTIGRAVITY_HOOKS.to_string());
                        self.banner = Some((
                            if is_id {
                                "Konfigurasi hooks.json Antigravity disalin!".into()
                            } else {
                                "Antigravity hooks.json copied!".into()
                            },
                            std::time::Instant::now(),
                        ));
                    }
                },
            );

            widgets::card_divider(ui);

            widgets::row_desc(
                ui,
                if is_id {
                    "Panduan CLI Universal"
                } else {
                    "Universal CLI Guide"
                },
                if is_id {
                    "Perintah noktah-hook untuk skrip terminal / CI"
                } else {
                    "noktah-hook commands for terminal scripts"
                },
                &theme,
                |ui| {
                    if widgets::chip_button(
                        ui,
                        if is_id {
                            "Salin Panduan"
                        } else {
                            "Copy Guide"
                        },
                        false,
                        &theme,
                    ) {
                        ui.output_mut(|o| o.copied_text = GUIDE_CLI_HOOKS.to_string());
                        self.banner = Some((
                            if is_id {
                                "Panduan CLI universal disalin ke clipboard!".into()
                            } else {
                                "Universal CLI guide copied!".into()
                            },
                            std::time::Instant::now(),
                        ));
                    }
                },
            );

            widgets::card_divider(ui);

            ui.vertical(|ui| {
                ui.label(style::cap(
                    if is_id {
                        "💡 Langkah Cepat Menghubungkan Noki:"
                    } else {
                        "💡 Quick Setup Steps:"
                    },
                    theme.ink,
                ));
                ui.label(style::cap(
                    if is_id {
                        "1. Pastikan biner `noktah-hook.exe` ada di PATH sistem."
                    } else {
                        "1. Ensure `noktah-hook.exe` is added to system PATH."
                    },
                    theme.faint,
                ));
                ui.label(style::cap(
                    if is_id {
                        "2. Claude Code: Salin Prompt lalu tempel ke chat Claude untuk instalasi hook otomatis."
                    } else {
                        "2. Claude Code: Copy Prompt and paste into Claude chat for automated hook setup."
                    },
                    theme.faint,
                ));
                ui.label(style::cap(
                    if is_id {
                        "3. CLI / Terminal: Panggil `noktah-hook working \"tugas\"` dan `noktah-hook done`."
                    } else {
                        "3. CLI / Terminal: Call `noktah-hook working \"task\"` and `noktah-hook done`."
                    },
                    theme.faint,
                ));
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

        widgets::section(ui, i18n::t(lang, "settings.updates"), &theme);
        widgets::card_group(ui, |ui| {
            use crate::infrastructure::updater::{UpdateStatus, UpdaterCmd};

            ui.horizontal(|ui| {
                let busy = matches!(
                    self.update_status,
                    UpdateStatus::Checking | UpdateStatus::Downloading { .. }
                );
                let btn_label = if matches!(self.update_status, UpdateStatus::Checking) {
                    i18n::t(lang, "settings.checking_update")
                } else {
                    i18n::t(lang, "settings.check_update")
                };
                if widgets::chip_button(ui, btn_label, busy, &theme) && !busy {
                    self.update_status = UpdateStatus::Checking;
                    let _ = self.updater.tx.send(UpdaterCmd::Check);
                }
            });

            ui.add_space(6.0);

            let mut pending_cmd: Option<UpdaterCmd> = None;
            let mut apply_staged: Option<(std::path::PathBuf, Option<std::path::PathBuf>)> = None;

            match &self.update_status {
                UpdateStatus::Idle => {
                    ui.label(style::cap("Klik tombol di atas untuk memeriksa rilis terbaru di GitHub.", theme.dim));
                }
                UpdateStatus::Checking => {
                    ui.label(style::cap("Menghubungi GitHub Releases…", theme.accent));
                }
                UpdateStatus::LatestVersion(ver) => {
                    ui.label(
                        egui::RichText::new(format!("✓ Noktah sudah menggunakan versi terbaru ({ver})"))
                            .size(style::T_CAPTION)
                            .color(Color32::from_rgb(34, 197, 94)),
                    );
                }
                UpdateStatus::UpdateAvailable { version, name, body, url, zip_url } => {
                    ui.label(
                        egui::RichText::new(format!("★ Pembaruan Tersedia: {name} ({version})"))
                            .size(style::T_BODY)
                            .strong()
                            .color(theme.accent),
                    );

                    if !body.trim().is_empty() {
                        ui.add_space(4.0);
                        let snippet: String = body.lines().take(6).collect::<Vec<_>>().join("\n");
                        ui.label(style::cap(snippet, theme.dim));
                    }

                    ui.add_space(6.0);
                    let url_clone = url.clone();
                    let ver_clone = version.clone();
                    let zip_clone = zip_url.clone();
                    ui.horizontal_wrapped(|ui| {
                        if let Some(z_url) = zip_clone {
                            if widgets::chip_button(ui, i18n::t(lang, "settings.self_update"), true, &theme) {
                                pending_cmd = Some(UpdaterCmd::DownloadAndStage {
                                    version: ver_clone,
                                    zip_url: z_url,
                                });
                            }
                        }
                        if widgets::chip_button(ui, "Buka Halaman Rilis GitHub ↗", false, &theme) {
                            ui.ctx().open_url(egui::OpenUrl::new_tab(&url_clone));
                            #[cfg(windows)]
                            {
                                let _ = std::process::Command::new("cmd")
                                    .args(["/C", "start", "", &url_clone])
                                    .spawn();
                            }
                        }
                    });
                }
                UpdateStatus::Downloading { version } => {
                    ui.label(
                        egui::RichText::new(format!(
                            "{} ({version})",
                            i18n::t(lang, "settings.downloading_update")
                        ))
                        .size(style::T_CAPTION)
                        .color(theme.accent),
                    );
                }
                UpdateStatus::ReadyToRestart { version, staged_exe, staged_hook } => {
                    ui.label(
                        egui::RichText::new(format!("✓ Paket {version} terverifikasi SHA-256 dan siap dipasang."))
                            .size(style::T_CAPTION)
                            .color(Color32::from_rgb(34, 197, 94)),
                    );
                    ui.add_space(6.0);
                    if widgets::chip_button(ui, i18n::t(lang, "settings.restart_to_apply"), true, &theme) {
                        apply_staged = Some((staged_exe.clone(), staged_hook.clone()));
                    }
                }
                UpdateStatus::Error(err) => {
                    ui.label(
                        egui::RichText::new(format!("Gagal: {err}"))
                            .size(style::T_CAPTION)
                            .color(Color32::from_rgb(239, 68, 68)),
                    );
                }
            }

            if let Some(cmd) = pending_cmd {
                if let UpdaterCmd::DownloadAndStage { ref version, .. } = cmd {
                    self.update_status = UpdateStatus::Downloading {
                        version: version.clone(),
                    };
                }
                let _ = self.updater.tx.send(cmd);
            }

            if let Some((exe, hook)) = apply_staged {
                if self.save_at.is_some() {
                    crate::infrastructure::store::save("settings.json", &self.settings);
                    self.save_at = None;
                }
                if self.notes_save_at.is_some() {
                    crate::infrastructure::store::save("notes.json", &self.scratchpad);
                    self.notes_save_at = None;
                }
                match crate::infrastructure::updater::apply_staged_and_restart(&exe, hook.as_deref()) {
                    Ok(()) => {
                        std::process::exit(0);
                    }
                    Err(err) => {
                        self.update_status = UpdateStatus::Error(err);
                    }
                }
            }
        });

        widgets::section(ui, i18n::t(lang, "settings.data"), &theme);
        widgets::card_group(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if widgets::chip_button(ui, i18n::t(lang, "settings.export"), false, &theme) {
                    crate::infrastructure::store::save("settings-export.json", &self.settings);
                    self.banner = Some((
                        "Setelan diekspor ke settings-export.json".into(),
                        std::time::Instant::now(),
                    ));
                }
                if widgets::chip_button(ui, i18n::t(lang, "settings.import"), false, &theme) {
                    let mut baru: noktah_core::Settings =
                        crate::infrastructure::store::load("settings-export.json");
                    baru.sanitize();
                    self.settings = baru;
                    self.touch_settings();
                    self.banner = Some(("Setelan diimpor".into(), std::time::Instant::now()));
                }
                let confirming = self
                    .confirm_reset_at
                    .map(|t| t.elapsed() < std::time::Duration::from_secs(5))
                    .unwrap_or(false);
                if !confirming {
                    self.confirm_reset_at = None;
                }
                let reset_label = if confirming {
                    i18n::t(lang, "settings.reset_confirm")
                } else {
                    i18n::t(lang, "settings.reset")
                };
                if widgets::chip_button(ui, reset_label, confirming, &theme) {
                    if !confirming {
                        self.confirm_reset_at = Some(std::time::Instant::now());
                    } else {
                        crate::infrastructure::store::wipe();
                        self.settings = noktah_core::Settings::default();
                        self.todos.clear();
                        self.scratchpad = noktah_core::model::Scratchpad::default();
                        self.notes_save_at = None;
                        self.chat.clear();
                        self.pomodoro = noktah_core::model::PomodoroState::default();
                        self.api_key.clear();
                        self.api_key_input.clear();
                        self.lyrics.clear();
                        self.lyrics_for.clear();
                        self.confirm_reset_at = None;
                        self.touch_settings();
                        self.banner = Some((
                            i18n::t(lang, "settings.reset_done").into(),
                            std::time::Instant::now(),
                        ));
                    }
                }
            });
        });

        widgets::section(ui, i18n::t(lang, "settings.diagnostics"), &theme);
        widgets::card_group(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if widgets::chip_button(ui, i18n::t(lang, "settings.open_data_dir"), false, &theme)
                {
                    let dir = crate::infrastructure::store::data_dir();
                    let _ = std::fs::create_dir_all(&dir);
                    #[cfg(windows)]
                    {
                        let _ = std::process::Command::new("explorer").arg(&dir).spawn();
                    }
                }

                let toggle_label = if self.show_diag_logs {
                    i18n::t(lang, "settings.hide_logs")
                } else {
                    i18n::t(lang, "settings.view_logs")
                };
                if widgets::chip_button(ui, toggle_label, self.show_diag_logs, &theme) {
                    self.show_diag_logs = !self.show_diag_logs;
                    if self.show_diag_logs {
                        self.diag_logs_cache = crate::infrastructure::logger::tail_lines(200);
                    }
                }

                if self.show_diag_logs
                    && widgets::chip_button(ui, i18n::t(lang, "common.refresh"), false, &theme)
                {
                    self.diag_logs_cache = crate::infrastructure::logger::tail_lines(200);
                }

                if widgets::chip_button(ui, i18n::t(lang, "settings.copy_logs"), false, &theme) {
                    let logs = if self.diag_logs_cache.is_empty() {
                        crate::infrastructure::logger::tail_lines(200)
                    } else {
                        self.diag_logs_cache.clone()
                    };
                    ui.output_mut(|o| o.copied_text = logs);
                    self.banner = Some((
                        i18n::t(lang, "settings.logs_copied").into(),
                        std::time::Instant::now(),
                    ));
                }

                if widgets::chip_button(ui, i18n::t(lang, "settings.clear_logs"), false, &theme) {
                    crate::infrastructure::logger::clear();
                    self.diag_logs_cache.clear();
                    self.banner = Some((
                        i18n::t(lang, "settings.logs_cleared").into(),
                        std::time::Instant::now(),
                    ));
                }
            });

            if self.show_diag_logs {
                ui.add_space(6.0);
                egui::ScrollArea::vertical()
                    .id_salt("diag_logs_scroll")
                    .max_height(150.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let text = if self.diag_logs_cache.trim().is_empty() {
                            i18n::t(lang, "settings.logs_empty")
                        } else {
                            self.diag_logs_cache.as_str()
                        };
                        ui.label(
                            egui::RichText::new(text)
                                .font(egui::FontId::monospace(10.5))
                                .color(theme.dim),
                        );
                    });
            }
        });
    }
}

const PROMPT_CLAUDE_CODE_HOOKS: &str = r#"Pasang hook Noktah ke konfigurasi Claude Code agar maskot Noki dapat merespons aktivitas agent secara real-time.

Tambahkan hooks berikut ke konfigurasi Claude Code (settings.json):
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "noktah-hook.exe SessionStart"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "noktah-hook.exe UserPromptSubmit"
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "noktah-hook.exe PreToolUse"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "noktah-hook.exe PostToolUse"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "noktah-hook.exe Stop"
          }
        ]
      }
    ],
    "PermissionRequest": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "noktah-hook.exe PermissionRequest"
          }
        ]
      }
    ]
  }
}

Catatan: Pastikan `noktah-hook.exe` sudah tersedia di PATH lingkungan sistem Anda."#;

const JSON_CLAUDE_CODE_HOOKS: &str = r#"{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "noktah-hook.exe SessionStart"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "noktah-hook.exe UserPromptSubmit"
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "noktah-hook.exe PreToolUse"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "noktah-hook.exe PostToolUse"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "noktah-hook.exe Stop"
          }
        ]
      }
    ],
    "PermissionRequest": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "noktah-hook.exe PermissionRequest"
          }
        ]
      }
    ]
  }
}"#;

const GUIDE_CLI_HOOKS: &str = r#"# Panduan Penggunaan noktah-hook (Universal CLI)

Pastikan aplikasi Noktah sedang berjalan.

1. Mengirim status saat mulai bekerja / menjalankan perintah:
   noktah-hook working "Menjalankan cargo test"
   noktah-hook -s working -m "Membangun proyek"

2. Mengirim status saat tugas selesai sukses:
   noktah-hook done "Semua tes lulus"
   noktah-hook -s done -m "Build sukses"

3. Mengirim status saat terjadi kesalahan / kegagalan:
   noktah-hook error "Kompilasi gagal pada modul X"
   noktah-hook -s error -m "Test failed"

4. Mengembalikan maskot ke status menganggur (idle):
   noktah-hook idle
   noktah-hook -s idle

5. Mengirim status persetujuan / konfirmasi:
   noktah-hook -s approval -m "Menunggu konfirmasi pengguna"

6. Mode stdin pipe (membaca output perintah lain):
   cargo check | noktah-hook -s working -m "Memeriksa kode"
"#;

const JSON_ANTIGRAVITY_HOOKS: &str = r#"{
  "noktah-mascot": {
    "SessionStart": [
      { "type": "command", "command": "noktah-hook.exe SessionStart" }
    ],
    "PreInvocation": [
      { "type": "command", "command": "noktah-hook.exe PreInvocation" }
    ],
    "PostInvocation": [
      { "type": "command", "command": "noktah-hook.exe PostInvocation" }
    ],
    "PreToolUse": [
      {
        "matcher": "*",
        "hooks": [
          { "type": "command", "command": "noktah-hook.exe PreToolUse" }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "*",
        "hooks": [
          { "type": "command", "command": "noktah-hook.exe PostToolUse" }
        ]
      }
    ],
    "Stop": [
      { "type": "command", "command": "noktah-hook.exe Stop" }
    ]
  }
}"#;
