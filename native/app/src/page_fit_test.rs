//! Uji bahwa isi setiap halaman notch benar-benar muat di dalam panel.
//!
//! Kelas bug yang ditangkap: halaman menggambar melebihi tinggi yang tersedia,
//! sehingga kontrol paling bawah — tombol transport Musik, tombol Mulai di
//! Focus — terpotong dan terlihat "hilang". Itu bukan bug gambar: `egui`
//! memotong di `clip_rect`, jadi tombolnya memang tidak pernah tampil.
//!
//! Angka di sini sengaja terpisah dari kode halaman. Kalau seseorang menambah
//! baris di halaman tanpa menyesuaikan tinggi panel, test ini gagal sebelum
//! bug-nya sampai ke layar.

#[cfg(test)]
mod tests {
    use crate::presentation::{layout, style};

    /// Tinggi ruang yang benar-benar tersedia untuk isi halaman.
    ///
    /// Diturunkan dari `draw_notch_pages`: gutter atas `S2` dan dock dalam
    /// keadaan **terbuka** di dasar panel. Kasus terburuk yang dipakai —
    /// saat dock menganggur, isi halaman justru dapat lebih banyak ruang.
    fn body_height() -> f32 {
        layout::CONTENT_BOTTOM - style::S2 - header_height()
    }

    /// Tinggi yang dipakai `widgets::page_header`: TINGGI 20 + S1 + S2 + 3.
    fn header_height() -> f32 {
        20.0 + style::S1 + style::S2 + 3.0
    }

    fn available_for(page: &str) -> f32 {
        if page == "Music" {
            layout::CONTENT_BOTTOM - style::S2
        } else {
            body_height()
        }
    }

    /// Kebutuhan vertikal minimum tiap halaman, mengikuti angka kompak baru.
    fn page_needs() -> [(&'static str, f32); 5] {
        let art = 44.0; // `.art` sampul halaman Musik
        let scrubber = 12.0;
        let transport = 36.0; // `.ib.play` tombol terbesar
        let ring = 76.0; // `.ring` dan cincin Focus
        let input = 32.0; // `.field`
        let tile = 64.0; // kartu sistem (tiga tile sejajar)

        [
            ("Music", art + style::S1 + scrubber + style::S1 + transport),
            ("Tasks", input + style::S1 + 30.0),
            ("Focus", ring),
            ("System", tile),
            ("Ask", input),
        ]
    }

    #[test]
    fn setiap_halaman_muat_di_dalam_panel() {
        let mut terlalu_tinggi: Vec<String> = Vec::new();

        for (nama, butuh) in page_needs() {
            let limit = available_for(nama);
            if butuh > limit {
                terlalu_tinggi.push(format!(
                    "{nama}: butuh {butuh:.0}px, tersedia {limit:.0}px (lebih {:.0}px)",
                    butuh - limit
                ));
            }
        }

        assert!(
            terlalu_tinggi.is_empty(),
            "isi halaman terpotong di dalam panel:\n  {}",
            terlalu_tinggi.join("\n  ")
        );
    }

    #[test]
    fn panel_terbuka_memuat_island() {
        // Bentuk terbuka yang dipakai halaman tidak boleh lebih tinggi atau
        // lebih lebar dari panel — kalau iya, tepinya terpotong.
        assert!(
            layout::ISLAND_H <= layout::OPEN_H,
            "ISLAND_H ({}) melebihi OPEN_H ({})",
            layout::ISLAND_H,
            layout::OPEN_H
        );
        assert!(
            layout::ISLAND_W <= layout::OPEN_W,
            "ISLAND_W ({}) melebihi OPEN_W ({})",
            layout::ISLAND_W,
            layout::OPEN_W
        );
    }

    #[test]
    fn ruang_isi_halaman_tidak_bergantung_pada_dock() {
        // Kebalikan dari dugaan awal: isi halaman justru TIDAK boleh mendapat
        // ruang ekstra saat dock menganggur. Kalau ia memanjang, kursor yang
        // menunjuk tombol di dasar panel akan membuka dock — yang menggeser
        // tombolnya — yang menutup dock lagi. Ruang tetap adalah yang membuat
        // umpan balik itu mustahil.
        let area = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(0.0, 0.0),
            eframe::egui::vec2(layout::OPEN_W, layout::OPEN_H),
        );
        let isi = layout::content_rect(area, 16.0, style::S2);

        // Tidak ada parameter keadaan dock di `content_rect` — itulah jaminannya.
        assert_eq!(
            isi.bottom(),
            layout::dock_band(area).top(),
            "isi halaman harus berakhir tepat di atas pita dock"
        );
        assert_eq!(
            isi.bottom(),
            area.bottom() - layout::DOCK_STRIP_H,
            "isi halaman harus berjarak DOCK_STRIP_H dari dasar panel idle"
        );
        assert!(
            layout::DOCK_STRIP_H < layout::DOCK_H,
            "strip harus lebih tipis dari pita dock"
        );
    }

    #[test]
    fn semua_halaman_muat_saat_dock_terbuka() {
        // Kasus terburuk: dock terbuka penuh. Kalau semua halaman muat di
        // sini, mereka juga muat saat dock menganggur.
        for (nama, butuh) in page_needs() {
            let limit = available_for(nama);
            assert!(
                butuh <= limit,
                "{nama} terpotong saat dock terbuka: butuh {butuh:.0}px, tersedia {limit:.0}px"
            );
        }
    }

    #[test]
    fn notch_tetap_menempel_di_tepi_atas() {
        // Noktah adalah notch, bukan dynamic island: sudut atasnya harus
        // menyentuh tepi layar supaya menyatu dengan bezel.
        let layar = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(0.0, 0.0),
            eframe::egui::vec2(1920.0, 1080.0),
        );
        let rect = layout::notch_rect(
            0.0,
            0.0,
            layout::PILL_W,
            0.0,
            layar,
            layout::OPEN_W,
            layout::OPEN_H,
        );
        assert_eq!(rect.top(), 0.0, "notch harus menempel di tepi atas layar");

        let terbuka = layout::notch_rect(
            1.0,
            1.0,
            layout::PILL_W,
            0.0,
            layar,
            layout::OPEN_W,
            layout::OPEN_H,
        );
        assert_eq!(
            terbuka.top(),
            0.0,
            "panel terbuka juga harus tetap menempel di tepi atas"
        );
    }

    #[test]
    fn pill_idle_tidak_menyisakan_celah_kosong() {
        use noktah_core::Settings;
        let s = Settings::default();
        let w = s.mini_width(false, false);

        // Jam (13.5px, ~40px) + equalizer (~22px) + padding kiri/kanan 18px
        // masing-masing. Lebih lebar dari 180 berarti ada ruang mati.
        assert!(
            (120.0..=180.0).contains(&w),
            "lebar pill idle {w}px di luar rentang wajar 120..=180"
        );
    }

    /// Isi halaman dan pita dock tidak boleh berbagi ruang — di tinggi mana pun.
    ///
    /// Kelas bug yang ditangkap: dulu tinggi isi mengikuti pegas dock
    /// (`area.bottom() - dock_h`), sedangkan kapsul selalu duduk di pita
    /// 58 px yang tetap. Saat dock menganggur isi menjulur 44 px ke dalam
    /// pita itu, sehingga kapsul menggambar di atas baris transport — dan
    /// kursor yang mengarah ke tombol itu justru membuka dock, yang
    /// menggeser tombolnya naik, yang menutup dock lagi. Berayun tanpa henti.
    #[test]
    fn isi_halaman_tidak_pernah_masuk_pita_dock() {
        let area = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(0.0, 0.0),
            eframe::egui::vec2(layout::OPEN_W, layout::OPEN_H),
        );
        let isi = layout::content_rect(area, 16.0, style::S2);
        let pita = layout::dock_band(area);

        assert!(
            isi.bottom() <= pita.top(),
            "isi halaman berakhir {}px, pita dock mulai {}px — tumpang tindih {}px",
            isi.bottom(),
            pita.top(),
            isi.bottom() - pita.top()
        );
    }

    #[test]
    fn shell_luar_dock_tumbuh_ke_bawah_tanpa_menggeser_panel_dasar() {
        let base = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(10.0, 0.0),
            eframe::egui::vec2(layout::OPEN_W, layout::OPEN_H),
        );
        let content = layout::content_rect(base, 16.0, style::S2);
        let compact_band = layout::dock_band(base);

        for (open, expected_h) in [
            (0.0, layout::OPEN_H),
            (0.5, layout::OPEN_H + layout::DOCK_MENU_H * 0.5),
            (1.0, layout::OUTER_OPEN_H),
        ] {
            let outer = layout::outer_shell_rect(base, open);
            let menu = layout::dock_menu_band(base, open);
            let hover = layout::dock_hover_zone(base, open);

            assert_eq!(outer.top(), base.top());
            assert_eq!(outer.left(), base.left());
            assert_eq!(outer.width(), base.width());
            assert_eq!(outer.height(), expected_h);
            assert_eq!(content.bottom(), compact_band.top());
            assert!(outer.contains_rect(base));
            assert!(outer.contains_rect(menu));
            assert!(hover.contains_rect(compact_band));
            assert!(hover.contains_rect(menu));
        }

        assert_eq!(layout::WINDOW_H, layout::OUTER_OPEN_H + 70.0);
        assert!(layout::WINDOW_H >= layout::OUTER_OPEN_H);
    }

    #[test]
    fn kursor_dari_pita_kompak_ke_menu_tetap_di_zona_hover() {
        let base = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(0.0, 0.0),
            eframe::egui::vec2(layout::OPEN_W, layout::OPEN_H),
        );
        let hover = layout::dock_hover_zone(base, 1.0);
        let compact = layout::dock_band(base);
        let menu = layout::dock_menu_band(base, 1.0);

        assert!(hover.contains(compact.center()));
        assert!(hover.contains(menu.center()));
        assert_eq!(compact.bottom(), menu.top());
    }

    /// Kapsul latar dock harus selalu memuat seluruh baris tab.
    ///
    /// Kelas bug yang ditangkap: lebar kapsul dulu dikalikan pegas
    /// (`dock_group_w(n) * dock_open`) sedangkan tab tetap ditata pada lebar penuh.
    /// Di tengah animasi kapsul hanya selebar 54 px setengah — ikon terjauh
    /// menonjol 51 px keluar, dan tepi kapsul memotongnya. Gejalanya di layar:
    /// tab Focus dan Ask terlihat terpotong oleh latar navbar.
    #[test]
    fn kapsul_dock_memuat_seluruh_tab() {
        let tab_row = layout::DOCK_TAB_W * crate::app::NotchPage::COUNT as f32;
        let group_w = layout::dock_group_w(crate::app::NotchPage::COUNT);

        // Baris tab tidak boleh lebih lebar dari kapsulnya, berapa pun nilai
        // pegas — karena lebar sekarang tetap, cukup diperiksa sekali.
        assert!(
            group_w >= tab_row,
            "kapsul {group_w}px lebih sempit dari baris tab {tab_row}px — ikon akan menonjol"
        );

        // Tepi kapsul harus memuat tab paling kanan ditambah padding.
        let padding = (group_w - tab_row) / 2.0;
        assert!(
            padding > 0.0,
            "tidak ada padding kiri/kanan antara tab dan tepi kapsul"
        );
    }

    /// Zona pemicu dock harus menutup rapat dari ujung isi halaman sampai
    /// dasar panel — tidak ada celah mati, dan tidak ada yang tumpang tindih.
    ///
    /// Kelas bug yang ditangkap: pemicunya pernah dipersempit ke strip 14 px
    /// di dasar, sehingga kursor yang berhenti di tengah pita dock tidak
    /// membuka apa pun. Dock tampak "macet" setengah terbuka.
    #[test]
    fn zona_pemicu_dock_menutup_rapat() {
        let area = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(0.0, 0.0),
            eframe::egui::vec2(layout::OPEN_W, layout::OPEN_H),
        );
        let isi = layout::content_rect(area, 16.0, style::S2);
        let pemicu = layout::dock_band(area);

        assert_eq!(
            pemicu.top(),
            isi.bottom(),
            "harus tidak ada celah antara ujung isi halaman dan zona pemicu dock"
        );
        assert_eq!(
            pemicu.bottom(),
            area.bottom(),
            "zona pemicu dock harus sampai dasar panel"
        );
        // Strip tetap ada sebagai isyarat visual saat dock masih menganggur.
        assert!(
            pemicu.contains_rect(layout::strip_band(area)),
            "strip penanda harus berada di dalam zona pemicu"
        );
    }
}
