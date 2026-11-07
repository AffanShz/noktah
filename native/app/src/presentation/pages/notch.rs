//! Kerangka notch: area isi dan dock navigasi.
//!
//! Dock mengikuti pola **grabber** seperti panel bawah macOS: saat menganggur
//! ia hanya tiga garis pendek di dasar panel, dan melebar menjadi deretan ikon
//! ketika kursor mendekat. Sebelumnya dock selalu terbuka — latar kapsulnya
//! menutupi baris terakhir isi halaman, dan chevron/setelan duduk di atas
//! konten alih-alih di ruangnya sendiri.
//!
//! Tombol tidak memakai latar saat aktif maupun hover; keadaan dibawa oleh
//! warna ikon saja.

use crate::app::{Noktah, NotchPage};
use crate::presentation::{icons::Icon, layout, style, widgets};
use eframe::egui::{self, Color32, Id, Rect, Rounding, Sense};

impl Noktah {
    /// Notch shell: konten memakai panel dasar, sedangkan menu dock memakai
    /// ekstensi shell yang dapat tumbuh di bawahnya.
    pub fn draw_notch_pages(&mut self, ui: &mut egui::Ui, base_rect: Rect, shell_rect: Rect) {
        let theme = self.theme;

        // Urutan tombol datang dari `NotchPage::SWITCHER` — sumber yang sama
        // yang dipakai `prev()`/`next()`, jadi dock dan chevron tidak pernah
        // berbeda pendapat soal urutan.
        let halaman: Vec<(NotchPage, Icon, &str)> = NotchPage::SWITCHER
            .iter()
            .map(|p| {
                let label = match p {
                    NotchPage::Music => "Music",
                    NotchPage::Pomodoro => "Focus",
                    NotchPage::Tasks => "Tasks",
                    NotchPage::System => "System",
                    NotchPage::Ask => "Ask",
                    NotchPage::NowPlaying => "Now Playing",
                    NotchPage::Water => "Water",
                    NotchPage::Home => "Home",
                    NotchPage::Agent => "Agent",
                };
                (*p, p.icon(), label)
            })
            .collect();

        // `dock_open` adalah spring 0..1 dari `frame.rs`.
        let dock_open = self.dock_open.pos.clamp(0.0, 1.0);

        // Ruang dock SELALU disediakan penuh, terlepas dari pegas.
        //
        // Dulu tinggi isi mengikuti `dock_h`, jadi saat dock menganggur isi
        // halaman memanjang 44 px lebih dalam daripada kapsul dock. Dua
        // akibatnya terlihat sekaligus: kapsul menimpa baris transport, dan
        // kursor yang berada di dasar panel membuka dock → isi menyempit →
        // tombol bergeser naik meninggalkan kursor → dock menutup → tombol
        // kembali ke bawah kursor → begitu terus tanpa henti.
        //
        // Dengan cadangan tetap, isi halaman tidak pernah bergerak karena
        // dock, jadi tidak ada umpan balik yang bisa berayun — dan kapsul
        // tidak mungkin menimpa isi karena keduanya tidak berbagi ruang.
        let content_rect = layout::content_rect(base_rect, 16.0, style::S2);

        let mut page_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );
        page_ui.set_clip_rect(content_rect);

        match self.current_page {
            NotchPage::Music => self.draw_page_music(&mut page_ui, content_rect),
            NotchPage::Tasks => self.draw_page_todo(&mut page_ui, content_rect),
            NotchPage::Pomodoro => self.draw_page_pomodoro(&mut page_ui, content_rect),
            NotchPage::System => self.draw_page_system(&mut page_ui, content_rect),
            NotchPage::NowPlaying => self.draw_page_now_playing(&mut page_ui, content_rect),
            NotchPage::Water => self.draw_page_water(&mut page_ui, content_rect),
            NotchPage::Home => self.draw_page_home(&mut page_ui, content_rect),
            NotchPage::Ask => self.draw_page_ai(&mut page_ui, content_rect),
            NotchPage::Agent => self.draw_page_agent(&mut page_ui, content_rect),
        }

        // ---- Dock ----
        // Hover/linger dimutasi di `frame.rs`, karena lifecycle harus melihat
        // geometri outer sebelum Area menu menerima event egui.
        let (mx, my) = self.hover.get();
        let cursor = egui::pos2(mx, my);
        let compact_band = layout::dock_band(base_rect);
        let menu_band = layout::dock_menu_band(base_rect, dock_open);

        // Strip penanda tiga garis hanya tampil saat dock menganggur.
        // Saat dock membuka, strip memudar halus digantikan oleh kapsul menu.
        let strip_alpha = (1.0 - dock_open * 2.5).clamp(0.0, 1.0);
        if strip_alpha > 0.01 {
            self.draw_dock_strip(ui, compact_band, compact_band.contains(cursor), strip_alpha);
        }

        // Tab hanya mulai menerima interaksi setelah menu cukup tinggi untuk
        // kelompok 30 px. Semua kontrol dock berada di ekstensi yang di-clip
        // oleh shell saat ini.
        if menu_band.height() < 28.0 {
            return;
        }

        let muted = dock_open < 0.6;
        let aktif_idx = halaman.iter().position(|(p, _, _)| *p == self.current_page);

        let dock_rect = Rect::from_min_max(
            egui::pos2(menu_band.left() + 10.0, menu_band.top()),
            egui::pos2(menu_band.right() - 10.0, menu_band.bottom()),
        )
        .intersect(shell_rect);

        // Kelompok ikon: satu tab 36 px, jumlahnya mengikuti SWITCHER.
        const TAB_W: f32 = layout::DOCK_TAB_W;
        let group_w = TAB_W * halaman.len() as f32;
        let group_rect = Rect::from_center_size(
            egui::pos2(dock_rect.center().x, dock_rect.center().y),
            egui::vec2(group_w, 30.0),
        );

        // Latar kapsul: baris tab ditambah padding kiri-kanan.
        let capsule = Rect::from_center_size(
            egui::pos2(dock_rect.center().x, dock_rect.center().y),
            egui::vec2(layout::dock_group_w(halaman.len()), 30.0),
        );
        ui.painter().rect_filled(
            capsule,
            Rounding::same(capsule.height() / 2.0),
            theme.surface.gamma_multiply(dock_open),
        );

        let left = group_rect.center().x - (TAB_W * halaman.len() as f32) / 2.0;

        for (idx, (page, icon, label)) in halaman.iter().enumerate() {
            let r = Rect::from_min_size(
                egui::pos2(left + idx as f32 * TAB_W, group_rect.top()),
                egui::vec2(TAB_W, group_rect.height()),
            );
            let resp = ui.interact(
                r,
                Id::new("dock_tab").with(idx),
                if muted {
                    Sense::hover()
                } else {
                    Sense::click()
                },
            );
            let hover = resp.hovered() && !muted;
            let aktif = Some(idx) == aktif_idx;

            // Tanpa latar. Hierarki warna: aktif = aksen, hover = ink,
            // sisanya dim. Semua dikalikan alpha supaya ikut memudar masuk.
            let warna = if aktif {
                theme.accent
            } else if hover {
                theme.ink
            } else {
                theme.dim
            };
            widgets::glyph(
                ui.painter(),
                *icon,
                Rect::from_center_size(r.center(), egui::vec2(16.0, 16.0)),
                warna.gamma_multiply(dock_open),
                1.7,
            );

            // Penanda posisi: garis 2 px di bawah ikon aktif. Menggantikan
            // latar tombol sebagai satu-satunya bentuk yang digambar.
            if aktif && dock_open > 0.5 {
                let bar = Rect::from_center_size(
                    egui::pos2(r.center().x, r.bottom() - 3.0),
                    egui::vec2(12.0, 2.0),
                );
                ui.painter().rect_filled(
                    bar,
                    Rounding::same(1.0),
                    theme.accent.gamma_multiply(dock_open),
                );
            }

            if hover && resp.hovered() {
                let g =
                    ui.painter()
                        .layout_no_wrap((*label).to_string(), style::caption(), theme.ink);
                let tip_size = egui::vec2(g.size().x + style::S2 * 2.0, g.size().y + 8.0);
                let tip = Rect::from_center_size(
                    egui::pos2(
                        r.center().x,
                        group_rect.top() - tip_size.y / 2.0 - 4.0,
                    ),
                    tip_size,
                );
                ui.painter().rect_filled(
                    tip,
                    Rounding::same(style::R_SMALL),
                    Color32::from_rgba_unmultiplied(22, 23, 28, 245),
                );
                ui.painter().galley(
                    egui::pos2(
                        tip.center().x - g.size().x / 2.0,
                        tip.center().y - g.size().y / 2.0,
                    ),
                    g.clone(),
                    theme.ink,
                );
            }

            if resp.clicked() {
                self.current_page = *page;
            }
        }

        // Chevron kiri di tepi, setelan + chevron kanan di tepi kanan.
        const NAV: f32 = 30.0;
        let chev_left = Rect::from_center_size(
            egui::pos2(dock_rect.left() + NAV / 2.0, group_rect.center().y),
            egui::vec2(NAV, NAV),
        );
        let chev_right = Rect::from_center_size(
            egui::pos2(dock_rect.right() - NAV / 2.0, group_rect.center().y),
            egui::vec2(NAV, NAV),
        );
        let gear = Rect::from_center_size(
            egui::pos2(
                dock_rect.right() - NAV * 1.5 - style::S2,
                group_rect.center().y,
            ),
            egui::vec2(NAV, NAV),
        );

        // Setelan membuka jendela terpisah — bukan halaman dalam siklus,
        // jadi ia duduk di luar kelompok ikon.
        let gear_resp = ui.interact(
            gear,
            Id::new("dock_gear"),
            if muted {
                Sense::hover()
            } else {
                Sense::click()
            },
        );
        widgets::glyph(
            ui.painter(),
            Icon::Settings,
            Rect::from_center_size(gear.center(), egui::vec2(18.0, 18.0)),
            (if gear_resp.hovered() && !muted {
                theme.ink
            } else {
                theme.dim
            })
            .gamma_multiply(dock_open),
            1.7,
        );
        if gear_resp.clicked() {
            self.show_settings = true;
        }

        for (rect, prev, icon) in [
            (chev_left, true, Icon::ChevronLeft),
            (chev_right, false, Icon::ChevronRight),
        ] {
            let resp = ui.allocate_rect(
                rect,
                if muted {
                    Sense::hover()
                } else {
                    Sense::click()
                },
            );
            widgets::glyph(
                ui.painter(),
                icon,
                Rect::from_center_size(rect.center(), egui::vec2(18.0, 18.0)),
                (if resp.hovered() && !muted {
                    theme.ink
                } else {
                    theme.dim
                })
                .gamma_multiply(dock_open),
                1.8,
            );
            if resp.clicked() {
                self.current_page = if prev {
                    self.current_page.prev()
                } else {
                    self.current_page.next()
                };
            }
        }
    }

    /// Strip penanda saat dock menganggur: tiga garis pendek di tengah dasar.
    fn draw_dock_strip(&self, ui: &mut egui::Ui, reveal: Rect, hover: bool, alpha: f32) {
        let theme = self.theme;
        let cy = reveal.center().y;
        let cx = reveal.center().x;

        // Warnanya naik saat kursor mendekat — isyarat bahwa ini bisa dibuka,
        // tanpa harus memunculkan kontrol lebih awal. Alpha memudarkannya saat menu membuka.
        let base_col = if hover { theme.ink } else { theme.faint };
        let warna = base_col.gamma_multiply(alpha);

        for i in 0..3 {
            let y = cy - 4.0 + i as f32 * 4.0;
            // Baris tengah paling panjang; tepinya menyempit 1,5 px.
            let w = 13.0 - (i as f32 - 1.0).abs() * 1.5;
            ui.painter().line_segment(
                [egui::pos2(cx - w / 2.0, y), egui::pos2(cx + w / 2.0, y)],
                egui::Stroke::new(1.5, warna),
            );
        }
    }
}
