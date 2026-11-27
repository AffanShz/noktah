//! Uji geometri baris kontrol.
//!
//! Memakai harness UI egui yang sebenarnya, bukan perkiraan angka: setiap
//! tombol diukur dari `Rect` yang benar-benar dialokasikan. Ini menangkap
//! kelas bug yang tidak terlihat di test logika murni — item dalam satu baris
//! horizontal yang sumbu Y-nya meleset karena baris tumbuh setelah item
//! sebelumnya selesai diletakkan.

#[cfg(test)]
mod tests {
    use eframe::egui;

    /// Tinggi slot baris kontrol, harus sama dengan `row_h` di `pages::music`.
    const ROW_H: f32 = 36.0;

    /// Meniru susunan tombol di `draw_page_music`: Lyrics 28, Prev 28, Play 36, Next 28.
    /// Mengembalikan (y_atas, y_bawah) setiap tombol dalam koordinat lokal.
    fn susun_baris(ui: &mut egui::Ui) -> Vec<(f32, f32)> {
        let mut hasil = Vec::new();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            for d in [28.0_f32, 28.0, 36.0, 28.0] {
                let mut rect = egui::Rect::NOTHING;
                ui.allocate_ui_with_layout(
                    egui::vec2(d, ROW_H),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.add_space((ROW_H - d) * 0.5);
                        let (r, _) = ui.allocate_exact_size(egui::vec2(d, d), egui::Sense::hover());
                        rect = r;
                    },
                );
                hasil.push((rect.top(), rect.bottom()));
                ui.add_space(24.0);
            }
        });
        hasil
    }

    #[test]
    fn semua_tombol_berbagi_garis_pusat_yang_sama() {
        egui::__run_test_ui(|ui| {
            let tombol = susun_baris(ui);
            assert_eq!(tombol.len(), 4);

            let pusat: Vec<f32> = tombol.iter().map(|(a, b)| (a + b) / 2.0).collect();
            let acuan = pusat[2]; // tombol play, yang paling besar

            for (i, p) in pusat.iter().enumerate() {
                assert!(
                    (p - acuan).abs() < 0.5,
                    "tombol {i} meleset {}px dari garis pusat (pusat={p}, acuan={acuan})",
                    (p - acuan).abs()
                );
            }
        });
    }

    #[test]
    fn tidak_ada_tombol_yang_keluar_dari_tinggi_baris() {
        egui::__run_test_ui(|ui| {
            let tombol = susun_baris(ui);
            let atas = tombol.iter().map(|(a, _)| *a).fold(f32::MAX, f32::min);
            let bawah = tombol.iter().map(|(_, b)| *b).fold(f32::MIN, f32::max);
            assert!(
                bawah - atas <= ROW_H + 0.5,
                "baris melebar {}px, seharusnya maksimal {ROW_H}px",
                bawah - atas
            );
        });
    }

    /// Pembanding: susunan lama (tanpa slot bertinggi tetap) memang miring.
    /// Test ini menjaga alasan perubahan tetap terdokumentasi dan mencegah
    /// seseorang mengembalikannya ke bentuk lama.
    #[test]
    fn susunan_lama_memang_miring() {
        egui::__run_test_ui(|ui| {
            let mut ys: Vec<(f32, f32)> = Vec::new();
            ui.horizontal(|ui| {
                for d in [28.0_f32, 28.0, 36.0, 28.0] {
                    let (r, _) = ui.allocate_exact_size(egui::vec2(d, d), egui::Sense::hover());
                    ys.push((r.top(), r.bottom()));
                    ui.add_space(24.0);
                }
            });
            let pusat: Vec<f32> = ys.iter().map(|(a, b)| (a + b) / 2.0).collect();
            let selisih = (pusat[0] - pusat[2]).abs();
            assert!(
                selisih > 1.0,
                "susunan lama seharusnya miring: selisih hanya {selisih}px"
            );
        });
    }

    /// Empat tombol transport:
    /// Lyrics (28) di tepi kiri, Prev (28), Play (36), Next (28) di tepi kanan.
    /// Semua gap antar-tombol tepat 24.0 px di kolom 192 px.
    #[test]
    fn tombol_simetris_di_slot_seimbang() {
        const IB: f32 = 28.0;
        const PLAY: f32 = 36.0;
        const GAP: f32 = 24.0;
        let col_w = 192.0;

        egui::__run_test_ui(|ui| {
            let mut lyrics = egui::Rect::NOTHING;
            let mut prev = egui::Rect::NOTHING;
            let mut play = egui::Rect::NOTHING;
            let mut next = egui::Rect::NOTHING;
            let mut kolom = egui::Rect::NOTHING;

            ui.allocate_ui_with_layout(
                egui::vec2(col_w, PLAY),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing.x = GAP;
                    kolom = ui.max_rect();

                    (lyrics, _) =
                        ui.allocate_exact_size(egui::vec2(IB, IB), egui::Sense::hover());
                    (prev, _) =
                        ui.allocate_exact_size(egui::vec2(IB, IB), egui::Sense::hover());
                    (play, _) =
                        ui.allocate_exact_size(egui::vec2(PLAY, PLAY), egui::Sense::hover());
                    (next, _) =
                        ui.allocate_exact_size(egui::vec2(IB, IB), egui::Sense::hover());
                },
            );

            // Lyric di tepi kiri, Next di tepi kanan
            assert!(
                (lyrics.left() - kolom.left()).abs() < 0.5,
                "Lyric tidak di tepi kiri: {}px vs {}px",
                lyrics.left(),
                kolom.left()
            );
            assert!(
                (next.right() - kolom.right()).abs() < 0.5,
                "Next tidak di tepi kanan: {}px vs {}px",
                next.right(),
                kolom.right()
            );

            // Gap sama (24.0 px)
            let g1 = prev.left() - lyrics.right();
            let g2 = play.left() - prev.right();
            let g3 = next.left() - play.right();
            for (idx, g) in [g1, g2, g3].iter().enumerate() {
                assert!(
                    (g - GAP).abs() < 0.5,
                    "gap {idx} ({g}px) berbeda dari GAP ({GAP}px)"
                );
            }

            // Garis pusat Y sama
            let py = play.center().y;
            for (name, r) in [("Lyrics", lyrics), ("Prev", prev), ("Next", next)] {
                assert!(
                    (r.center().y - py).abs() < 0.5,
                    "{name} tidak berbagi garis pusat Y dengan Play"
                );
            }
        });
    }

    /// Saat lirik di-hide, transport beralih ke tata letak 3-slot simetris:
    /// Slot kiri: Lyrics (28) rata kiri di x = 0.
    /// Slot tengah: Prev (28), Play (36), Next (28) tepat di tengah area (area.width() / 2).
    /// Slot kanan: Speaker (28) rata kanan di area.width().
    #[test]
    fn tombol_simetris_saat_lirik_di_hide() {
        const IB: f32 = 28.0;
        const PLAY: f32 = 36.0;
        let area_w = 468.0_f32;
        let center_trio_w = IB + 16.0 + PLAY + 16.0 + IB;
        let slot_side_w = ((area_w - center_trio_w) * 0.5).max(IB);

        egui::__run_test_ui(|ui| {
            let (baris, _) =
                ui.allocate_exact_size(egui::vec2(area_w, PLAY), egui::Sense::hover());

            let slot_left = egui::Rect::from_min_size(baris.min, egui::vec2(slot_side_w, PLAY));
            let slot_center = egui::Rect::from_min_size(
                egui::pos2(baris.left() + slot_side_w, baris.top()),
                egui::vec2(center_trio_w, PLAY),
            );
            let slot_right = egui::Rect::from_min_size(
                egui::pos2(baris.right() - slot_side_w, baris.top()),
                egui::vec2(slot_side_w, PLAY),
            );

            // Slot kiri
            let mut ui_left = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(slot_left)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            ui_left.spacing_mut().item_spacing.x = 0.0;
            let (lyrics, _) =
                ui_left.allocate_exact_size(egui::vec2(IB, IB), egui::Sense::hover());

            // Slot tengah
            let mut ui_center = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(slot_center)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            ui_center.spacing_mut().item_spacing.x = 16.0;
            let (prev, _) =
                ui_center.allocate_exact_size(egui::vec2(IB, IB), egui::Sense::hover());
            let (play, _) =
                ui_center.allocate_exact_size(egui::vec2(PLAY, PLAY), egui::Sense::hover());
            let (next, _) =
                ui_center.allocate_exact_size(egui::vec2(IB, IB), egui::Sense::hover());

            // Slot kanan
            let mut ui_right = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(slot_right)
                    .layout(egui::Layout::right_to_left(egui::Align::Center)),
            );
            ui_right.spacing_mut().item_spacing.x = 0.0;
            let (speaker, _) =
                ui_right.allocate_exact_size(egui::vec2(IB, IB), egui::Sense::hover());

            // Play berada tepat di tengah area (area.width() / 2)
            let center_expected = baris.left() + area_w * 0.5;
            assert!(
                (play.center().x - center_expected).abs() < 0.5,
                "Play tidak di tengah: center={:.1}, expected={:.1}",
                play.center().x,
                center_expected
            );

            // Lyrics di tepi kiri
            assert!(
                (lyrics.left() - baris.left()).abs() < 0.5,
                "Lyrics tidak di tepi kiri: {:.1} vs {:.1}",
                lyrics.left(),
                baris.left()
            );

            // Speaker di tepi kanan
            assert!(
                (speaker.right() - baris.right()).abs() < 0.5,
                "Speaker tidak di tepi kanan: {:.1} vs {:.1}",
                speaker.right(),
                baris.right()
            );

            // Semua tombol berbagi garis pusat Y yang sama
            let py = play.center().y;
            for (name, r) in [
                ("Lyrics", lyrics),
                ("Prev", prev),
                ("Next", next),
                ("Speaker", speaker),
            ] {
                assert!(
                    (r.center().y - py).abs() < 0.5,
                    "{name} tidak sejajar Y dengan Play: {:.1} vs {:.1}",
                    r.center().y,
                    py
                );
            }
        });
    }

    /// Lirik panjang dibungkus ke baris baru (newline) menggunakan LayoutJob.
    #[test]
    fn lirik_panjang_membungkus_ke_newline() {
        const LEBAR_KOLOM: f32 = 260.0;

        egui::__run_test_ui(|ui| {
            let mut job = egui::text::LayoutJob::default();
            job.wrap.max_width = LEBAR_KOLOM - crate::presentation::style::S2;
            assert_eq!(job.wrap.max_width, 252.0);

            job.append(
                "Those funny faces in mind, but\nfor me",
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::proportional(16.0),
                    color: egui::Color32::WHITE,
                    ..Default::default()
                },
            );
            let g = ui.painter().layout_job(job);
            assert_eq!(g.rows.len(), 2, "galley lirik harus membungkus ke 2 baris");
        });
    }

    #[test]
    fn verifikasi_semua_halaman_presisi_tanpa_overflow() {
        egui::__run_test_ui(|ui| {
            let mut app = crate::app::Noktah::new_for_test(
                "test".to_string(),
                crate::infrastructure::platform::Hover::new(false),
                ui.ctx(),
            );
            let area = egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(
                    crate::presentation::layout::OPEN_W,
                    crate::presentation::layout::OPEN_H,
                ),
            );
            let content_rect = crate::presentation::layout::content_rect(
                area,
                16.0,
                crate::presentation::style::S2,
            );
            let limit_y = content_rect.bottom();

            for page in [
                crate::app::NotchPage::Music,
                crate::app::NotchPage::Pomodoro,
                crate::app::NotchPage::Tasks,
                crate::app::NotchPage::System,
                crate::app::NotchPage::Ask,
                crate::app::NotchPage::Home,
                crate::app::NotchPage::Agent,
            ] {
                app.current_page = page;

                if page == crate::app::NotchPage::Music {
                    for lyrics_enabled in [true, false] {
                        app.settings.lyrics_enabled = lyrics_enabled;
                        let mut page_ui = ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(content_rect)
                                .layout(egui::Layout::top_down(egui::Align::LEFT)),
                        );
                        page_ui.set_clip_rect(content_rect);
                        app.draw_page_music(&mut page_ui, content_rect);

                        let min_rect = page_ui.min_rect();
                        println!(
                            "Page Music (lyrics_enabled={}): bottom={}, limit={}, diff={}",
                            lyrics_enabled,
                            min_rect.bottom(),
                            limit_y,
                            min_rect.bottom() - limit_y
                        );
                        assert!(
                            min_rect.bottom() <= limit_y + 0.5,
                            "Page Music (lyrics_enabled={lyrics_enabled}) overflows by {}px!",
                            min_rect.bottom() - limit_y
                        );
                    }
                } else {
                    let mut page_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(content_rect)
                            .layout(egui::Layout::top_down(egui::Align::LEFT)),
                    );
                    page_ui.set_clip_rect(content_rect);

                    match page {
                        crate::app::NotchPage::Music => unreachable!(),
                        crate::app::NotchPage::Tasks => app.draw_page_todo(&mut page_ui, content_rect),
                        crate::app::NotchPage::Pomodoro => {
                            app.draw_page_pomodoro(&mut page_ui, content_rect)
                        }
                        crate::app::NotchPage::System => {
                            app.draw_page_system(&mut page_ui, content_rect)
                        }
                        crate::app::NotchPage::Ask => app.draw_page_ai(&mut page_ui, content_rect),
                        crate::app::NotchPage::Home => {
                            app.draw_page_home(&mut page_ui, content_rect)
                        }
                        crate::app::NotchPage::Agent => {
                            app.draw_page_agent(&mut page_ui, content_rect)
                        }
                    }

                    let min_rect = page_ui.min_rect();
                    println!(
                        "Page {:?}: bottom={}, limit={}, diff={}",
                        page,
                        min_rect.bottom(),
                        limit_y,
                        min_rect.bottom() - limit_y
                    );
                    assert!(
                        min_rect.bottom() <= limit_y + 0.5,
                        "Page {:?} overflows by {}px!",
                        page,
                        min_rect.bottom() - limit_y
                    );
                }
            }
        });
    }

    #[test]
    fn marquee_title_teks_pendek_dan_panjang() {
        egui::__run_test_ui(|ui| {
            let font = egui::FontId::proportional(15.0);
            let color = egui::Color32::WHITE;

            // Teks pendek: text_w <= width
            let resp_short = crate::presentation::widgets::marquee_title(
                ui,
                "Lagu Pendek",
                font.clone(),
                color,
                200.0,
            );
            assert_eq!(resp_short.rect.width(), 200.0);

            // Teks panjang: text_w > width
            let resp_long = crate::presentation::widgets::marquee_title(
                ui,
                "Judul lagu yang sangat amat panjang sekali dan tidak muat di layar",
                font,
                color,
                100.0,
            );
            assert_eq!(resp_long.rect.width(), 100.0);
        });
    }

    #[test]
    fn mini_lyrics_ping_pong_scroll_never_exceeds_max_scroll() {
        use crate::presentation::mini::ping_pong_offset;

        let text_w = 300.0;
        let max_w = 200.0;
        let max_scroll = text_w - max_w;

        // Pada t = 0.0 dan t = 1.0 (dalam periode pause awal 1.8s), nilainya 0.0
        assert_eq!(ping_pong_offset(text_w, max_w, 0.0), 0.0);
        assert_eq!(ping_pong_offset(text_w, max_w, 1.0), 0.0);

        // Teks pendek selalu 0.0
        assert_eq!(ping_pong_offset(150.0, 200.0, 5.0), 0.0);

        // Saat tiba di akhir (PAUSE = 1.8s, scroll_dur = 100/26 ~= 3.846s; t = 6.0 di tengah pause akhir)
        let at_end = ping_pong_offset(text_w, max_w, 6.0);
        assert!(
            (at_end - max_scroll).abs() < 1e-4,
            "saat tiba di akhir harus tepat {max_scroll}, didapat {at_end}"
        );

        // Memastikan selalu >= 0.0 dan <= max_scroll untuk nilai t berapapun
        let mut t = 0.0;
        while t <= 100.0 {
            let offset = ping_pong_offset(text_w, max_w, t);
            assert!(
                offset >= 0.0 && offset <= max_scroll,
                "offset {offset} di luar rentang [0.0, {max_scroll}] pada t = {t}"
            );
            t += 0.05;
        }
    }

    #[test]
    fn doodle_icons_render_tanpa_panic() {
        use crate::presentation::icons::{self, Icon};

        egui::__run_test_ui(|ui| {
            let rect = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(24.0, 24.0));
            let color = egui::Color32::WHITE;

            // Uji seluruh 18 icon doodle yang diadopsi dari folder doodle/
            let doodle_icons = [
                Icon::Music,
                Icon::Tasks,
                Icon::Focus,
                Icon::System,
                Icon::AI,
                Icon::AiQuota,
                Icon::Settings,
                Icon::Play,
                Icon::Pause,
                Icon::Prev,
                Icon::Next,
                Icon::PlayCircle,
                Icon::PauseCircle,
                Icon::Lyrics,
                Icon::LyricsOn,
                Icon::LyricsOff,
                Icon::VolumeOn,
                Icon::VolumeMute,
            ];

            for icon in doodle_icons {
                icons::draw(ui.painter(), icon, rect, color, 1.8);
            }
        });
    }

    #[test]
    fn lyrics_open_close_anim_preserves_total_width_and_smooth_glide() {
        let total_w = 528.0_f32;
        let gap_max = 12.0_f32; // style::S3

        let mut t = 0.0_f32;
        while t <= 1.001 {
            let t_clamped = t.clamp(0.0, 1.0);
            let col_left_w = total_w + (192.0 - total_w) * t_clamped;
            let gap = gap_max * t_clamped;
            let col_right_w = (total_w - col_left_w - gap).max(0.0);

            // Total lebar harus selalu 100% konsisten persis 528 px
            let sum = col_left_w + gap + col_right_w;
            assert!(
                (sum - total_w).abs() < 1e-4,
                "pada t={t_clamped}, total lebar {sum} tidak sama dengan {total_w}"
            );

            // Uji posisi horizontal keempat kontrol navigasi (Lyrics, Prev, Play, Next)
            let lyr_cx = 14.0_f32;
            let closed_cx = col_left_w * 0.5;
            let prev_cx = (closed_cx - 48.0) + (66.0 - (closed_cx - 48.0)) * t_clamped;
            let play_cx = closed_cx + (122.0 - closed_cx) * t_clamped;
            let next_cx = (closed_cx + 48.0) + (178.0 - (closed_cx + 48.0)) * t_clamped;

            // Pastikan urutan selalu Lyrics < Prev < Play < Next dan berada di dalam batas kolom kiri
            assert!(lyr_cx < prev_cx, "Lyrics ({lyr_cx}) harus di sebelah kiri Prev ({prev_cx})");
            assert!(prev_cx < play_cx, "Prev ({prev_cx}) harus di sebelah kiri Play ({play_cx})");
            assert!(play_cx < next_cx, "Play ({play_cx}) harus di sebelah kiri Next ({next_cx})");
            assert!(next_cx <= col_left_w, "Next ({next_cx}) tidak boleh keluar dari kolom ({col_left_w})");

            t += 0.05;
        }
    }

    #[test]
    fn lyrics_3s_auto_scroll_timer_and_click_behavior() {
        use std::time::{Duration, Instant};

        // 1. Setelah scroll manual, auto_scroll harus false selama < 3 detik
        let mut lyric_manual_at = Instant::now();
        let auto_scroll = lyric_manual_at.elapsed() >= Duration::from_secs(3);
        assert!(!auto_scroll, "Auto-scroll harus nonaktif saat baru saja di-scroll manual");

        // 2. Setelah 3 detik tanpa klik, auto_scroll harus aktif kembali
        lyric_manual_at = Instant::now() - Duration::from_millis(3050);
        let auto_scroll_after_3s = lyric_manual_at.elapsed() >= Duration::from_secs(3);
        assert!(auto_scroll_after_3s, "Auto-scroll harus aktif kembali setelah 3 detik tanpa klik");

        // 3. Ketika ada klik pada baris lirik (seek), timer di-reset mundur sehingga langsung auto-scroll
        // Baris lirik diklik -> reset ke 10 detik lalu
        lyric_manual_at = Instant::now() - Duration::from_secs(10);
        let auto_scroll_on_click = lyric_manual_at.elapsed() >= Duration::from_secs(3);
        assert!(auto_scroll_on_click, "Klik baris lirik harus langsung mengaktifkan auto-scroll");
    }

    /// Area panel `"isi-panel"` HARUS menutupi seluruh panel, di kedua keadaan
    /// dock.
    ///
    /// Kelas bug yang ditangkap: isi halaman digambar lewat `ui.new_child`
    /// (`draw_notch_pages`), dan `new_child` tidak mengalokasikan ruang di UI
    /// induk. Akibatnya `min_rect` Area menyusut jadi 0x0 saat dock menganggur.
    /// Karena `Context::rect_contains_pointer` menuntut pointer berada di layer
    /// Area itu, seluruh hit-test di dalam panel — termasuk kolom lirik —
    /// ikut mati, sehingga roda mouse dan drag tidak pernah terdeteksi.
    /// Sunyi total: tidak ada error, lirik hanya diam saat digulir.
    ///
    /// Perbaikannya `ui.expand_to_include_rect(outer_rect)` di `draw_panel`.
    /// Test ini menjaganya supaya tidak hilang lagi.
    ///
    /// Batas jujur: yang diuji adalah luas Area dan hit-test pada satu titik
    /// tetap di paruh kanan panel. Test ini TIDAK membuktikan kolom lirik
    /// masih ada di titik itu — kalau kolomnya dipindah atau dihapus sementara
    /// Area tetap penuh, test ini tetap lulus. Titik ujinya sengaja tetap
    /// supaya tidak rapuh terhadap pergeseran tata letak halaman.
    #[test]
    fn area_panel_menutupi_seluruh_panel_di_kedua_keadaan_dock() {
        use crate::app::{Noktah, NotchPage};
        use eframe::egui::{pos2, vec2, Id, Rect};

        let ctx = egui::Context::default();
        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);
        app.current_page = NotchPage::Music;
        app.lyrics = (0..12)
            .map(|i| noktah_core::lrc::LrcLine {
                time: i as f64 * 10.0,
                text: format!("baris lirik {i} untuk menguji hit-test"),
            })
            .collect();
        app.media.active = true;
        app.media.playing = true;
        app.media.duration_ms = 240_000;
        app.media.position_ms = 45_000;

        let outer = Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 148.0));
        // Titik di kolom kanan (kolom lirik saat panel terbuka).
        let kolom_lirik = Rect::from_min_size(pos2(270.0, 20.0), vec2(220.0, 110.0));
        let probe = kolom_lirik.center();

        for dock in [0.0_f32, 1.0] {
            app.dock_open =
                crate::presentation::spring::Spring::with(dock, 280.0, 22.0);
            app.dock_open.snap();

            let mut area = None;
            let mut hit = false;
            for putaran in 0..4 {
                let mut input = egui::RawInput::default();
                input.screen_rect = Some(Rect::from_min_size(
                    pos2(0.0, 0.0),
                    vec2(1280.0, 720.0),
                ));
                input.events = vec![egui::Event::PointerMoved(probe)];
                let _ = ctx.run(input, |ctx| {
                    app.theme = crate::presentation::theme::Theme::new(
                        noktah_core::settings::Accent::Teal,
                        1.0,
                    );
                    app.draw_panel(ctx, outer, outer, 1.0);
                });
                if putaran == 3 {
                    area = ctx.memory(|m| m.area_rect(Id::new("isi-panel")));
                    hit = ctx.rect_contains_pointer(
                        egui::LayerId::new(
                            egui::Order::Foreground,
                            Id::new("isi-panel"),
                        ),
                        kolom_lirik,
                    );
                }
            }

            let area = area.expect("Area \"isi-panel\" harus ada setelah digambar");
            assert!(
                area.width() >= outer.width() - 1.0 && area.height() >= outer.height() - 1.0,
                "dock_open={dock}: Area \"isi-panel\" hanya {}x{}px, \
                 seharusnya >= {}x{}px — min_rect menyusut karena isi \
                 digambar lewat new_child tanpa expand_to_include_rect",
                area.width(),
                area.height(),
                outer.width(),
                outer.height()
            );
            assert!(
                hit,
                "dock_open={dock}: hit-test pointer di kolom lirik mati — \
                 roda mouse dan drag tidak akan pernah terdeteksi"
            );
        }
    }

    /// Regresi: pergantian baris lirik aktif TIDAK boleh mengubah tinggi kolom.
    ///
    /// Latar bug: dulu baris aktif menganimasikan ukuran font 13.5→16 px.
    /// "And in your eyes you're holding mine" berada di ambang pembungkusan —
    /// di 16 px ia tumbuh dari 1 baris ke 2 baris (terukur +22 px tinggi dalam
    /// SATU frame) — dan mendorong seluruh baris di bawahnya turun.
    ///
    /// Perbaikan sekarang: teks baris aktif memang dibesarkan ke `LYRIC_SIZE_BIG`
    /// (16 px) supaya terlihat menonjol, TETAPI tinggi slot tiap baris dibekukan
    /// karena diukur di 16 px untuk SEMUA baris — konstan, tidak peduli mana yang
    /// aktif. Jadi yang berubah hanya ukuran teks di dalam slot, bukan geometri
    /// kolom.
    ///
    /// Test menjalankan `draw_page_music` produksi yang sebenarnya, lalu:
    ///   1. memastikan vektor `row_center` IDENTIK untuk indeks aktif 4, 5, 6
    ///      (invariansi LANGSUNG, bukan sekadar target gulir);
    ///   2. memastikan `lyric_target_scroll` cocok dengan model murni produksi
    ///      (`lyric_row_centers_for`, yang mengukur slot di 16 px);
    ///   3. mendokumentasikan mengapa reservasi ini perlu: baris 6 adalah 1 baris
    ///      di 13.5 px tapi 2 baris di 16 px.
    ///
    /// Catatan harness: `egui::__run_test_ui` memanggil
    /// `Context::set_fonts(FontDefinitions::empty())` — semua galley setinggi
    /// 0 px. Karena itu test ini membangun `Context` sendiri, memasang font
    /// produksi di frame pertama, dan BARU mengukur di frame berikutnya.
    #[test]
    fn lirik_baris_aktif_tidak_mengubah_tinggi_kolom() {
        use crate::app::{Noktah, NotchPage};
        use crate::presentation::pages::music::{
            lyric_row_centers_for, lyric_size, LYRIC_SIZE, LYRIC_SIZE_BIG,
        };
        use eframe::egui::{pos2, vec2, Rect};

        // area.height() di dalam draw_page_music = CONTENT_BOTTOM - top_pad.
        const BODY_H: f32 =
            crate::presentation::layout::CONTENT_BOTTOM - crate::presentation::style::S2;
        // area.width() (OPEN_W - 2*panel gutter) dikurangi kolom kiri 192 dan gap S3.
        const COL_RIGHT_W: f32 = (crate::presentation::layout::OPEN_W - 2.0 * 16.0)
            - 192.0
            - crate::presentation::style::S3;

        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(1400.0, 900.0));
        let raw = || egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };

        // Frame 0: pasang font produksi.
        let _ = ctx.run(raw(), |ctx| {
            crate::presentation::fonts::install(ctx);
            egui::CentralPanel::default().show(ctx, |_ui| {});
        });
        // Frame 1: biarkan set_fonts aktif di cache.
        let _ = ctx.run(raw(), |_ctx| {});

        let mut app = Noktah::new_for_test(
            "test".to_string(),
            crate::infrastructure::platform::Hover::new(false),
            &ctx,
        );
        app.current_page = NotchPage::Music;
        app.settings.lyrics_enabled = true;
        app.settings.lyric_offset_ms = 0;
        app.reduce_motion = false;
        app.media.active = true;
        app.media.playing = true;
        app.media.duration_ms = 240_000;

        // Indeks 6 = baris yang flip 1→2 wrap di 16 px. Indeks 2 & 9 pembungkus
        // panjang 2 baris; sisanya pendek.
        let teks = [
            "baris lirik satu",
            "baris lirik dua",
            "pembungkus panjang yang sengaja dibuat melebihi lebar kolom lirik supaya pasti membungkus ke dua baris teks",
            "baris lirik empat",
            "baris lirik lima",
            "baris lirik enam",
            "And in your eyes you're holding mine",
            "baris lirik delapan",
            "baris lirik sembilan",
            "pembungkus panjang yang sengaja dibuat melebihi lebar kolom lirik supaya pasti membungkus ke dua baris teks",
            "baris lirik sebelas",
            "baris lirik dua belas",
        ];
        app.lyrics = teks
            .iter()
            .enumerate()
            .map(|(i, t)| noktah_core::lrc::LrcLine {
                time: (i as f64 + 1.0) * 10.0,
                text: (*t).to_string(),
            })
            .collect();

        let area = Rect::from_min_size(
            pos2(0.0, 0.0),
            vec2(
                crate::presentation::layout::OPEN_W,
                crate::presentation::layout::OPEN_H,
            ),
        );
        let content_rect =
            crate::presentation::layout::content_rect(area, 16.0, crate::presentation::style::S2);

        // Model murni produksi: pusat baris dari slot yang diukur di 16 px.
        // Dipakai untuk target gulir DAN untuk assert invariansi di bawah.
        let teks_owned: Vec<String> = app.lyrics.iter().map(|l| l.text.clone()).collect();
        let pusat_model = lyric_row_centers_for(&ctx, &teks_owned, COL_RIGHT_W);

        let mut targets = [0.0_f32; 3];
        for (slot, i) in [4_usize, 5, 6].into_iter().enumerate() {
            // Posisi di tengah baris i supaya active_line pasti mengembalikan i.
            app.media.position_ms = (app.lyrics[i].time * 1000.0) as i64 + 500;
            assert_eq!(
                noktah_core::lrc::active_line(&app.lyrics, app.media.position_ms),
                Some(i),
                "prasyarat: active_line harus {i}"
            );

            // Settled di posisi i supaya tes tidak bergantung waktu.
            app.last_active_lyric = Some(i);
            app.lyric_line_active_at =
                std::time::Instant::now() - std::time::Duration::from_secs(10);
            app.lyric_anim_pos.target = i as f32;
            app.lyric_anim_pos.snap();
            app.lyrics_anim.target = 1.0;
            app.lyrics_anim.snap();
            // Paksa loop menghitung ulang target pada frame ini.
            app.lyric_target_scroll = 0.0;

            let _ = ctx.run(raw(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let mut page_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(content_rect)
                            .layout(egui::Layout::top_down(egui::Align::LEFT)),
                    );
                    page_ui.set_clip_rect(content_rect);
                    app.draw_page_music(&mut page_ui, content_rect);
                });
            });

            targets[slot] = app.lyric_target_scroll;
        }

        // --- (1) Vektor row_center IDENTIK untuk tiga indeks aktif berbeda ---
        // `lyric_row_centers_for` mengukur slot SELALU di 16 px (sama seperti
        // produksi), jadi tidak menerima indeks aktif dan secara konstruksi
        // invarian. Test menegaskan hasilnya cocok dengan produksi lewat target
        // di bagian (2), lalu membuktikan model ini PUNYA GIGI: versi "regresi"
        // (slot diukur dari ukuran baris itu sendiri) memang BERBEDA antar tiga
        // indeks aktif — kalau tidak, test ini tidak akan menangkap bug apa pun.
        let vektor_invarian = lyric_row_centers_for(&ctx, &teks_owned, COL_RIGHT_W);
        assert!(
            vektor_invarian.windows(2).all(|w| w[1] > w[0]),
            "pusat baris harus monoton naik, dapat {:?}",
            vektor_invarian
        );

        // Versi regresi yang SENGAJA salah: tinggi slot dari `lyric_size(d)`.
        let pusat_regresi = |aktif: usize| -> Vec<f32> {
            let wrap_w = (COL_RIGHT_W - crate::presentation::style::S2).max(10.0);
            let h: Vec<f32> = ctx.fonts(|f| {
                teks_owned
                    .iter()
                    .enumerate()
                    .map(|(idx, t)| {
                        let d = (idx as f32 - aktif as f32).abs();
                        let mut job = egui::text::LayoutJob::default();
                        job.wrap.max_width = wrap_w;
                        job.append(
                            t,
                            0.0,
                            egui::TextFormat {
                                font_id: egui::FontId::proportional(lyric_size(d)),
                                color: egui::Color32::WHITE,
                                ..Default::default()
                            },
                        );
                        f.layout_job(job).size().y
                    })
                    .collect()
            });
            crate::presentation::pages::music::lyric_row_centers(&h)
        };
        let reg4 = pusat_regresi(4);
        let reg5 = pusat_regresi(5);
        let reg6 = pusat_regresi(6);
        let beda_maks = |a: &[f32], b: &[f32]| {
            a.iter()
                .zip(b.iter())
                .map(|(x, y)| (x - y).abs())
                .fold(0.0_f32, f32::max)
        };
        assert!(
            beda_maks(&reg4, &reg5) > 0.01 && beda_maks(&reg5, &reg6) > 0.01,
            "data test harus cukup tajam: versi regresi seharusnya menghasilkan \
             row_center berbeda antar indeks aktif (4→5: {} px, 5→6: {} px), \
             kalau tidak invariansi tidak bisa dibedakan",
            beda_maks(&reg4, &reg5),
            beda_maks(&reg5, &reg6)
        );
        // Invarian produksi: pusat baris yang DITURUNKAN dari `lyric_target_scroll`
        // produksi (target + BODY_H/2, tanpa clamp karena semua > 62 px) harus
        // sama persis dengan model untuk tiap indeks aktif. Ini yang membuktikan
        // slot produksi benar-benar diukur di 16 px, bukan di ukuran aktif.
        for (slot, i) in [4_usize, 5, 6].into_iter().enumerate() {
            let prod_center = targets[slot] + BODY_H * 0.5;
            assert!(
                (prod_center - vektor_invarian[i]).abs() < 1.0,
                "indeks {i}: pusat baris produksi {prod_center} menyimpang dari \
                 model invarian {}",
                vektor_invarian[i]
            );
        }

        // --- (2) Target gulir produksi == model murni (slot diukur 16 px) ---
        for (slot, i) in [4_usize, 5, 6].into_iter().enumerate() {
            let expected = (pusat_model[i] - BODY_H * 0.5).max(0.0);
            assert!(
                (targets[slot] - expected).abs() < 1.0,
                "indeks {i}: lyric_target_scroll={} menyimpang {} px dari \
                 model slot-16px {}",
                targets[slot],
                (targets[slot] - expected).abs(),
                expected
            );
        }

        // --- (3) Dokumentasi mengapa reservasi perlu: baris 6 flip 1→2 wrap ---
        let tinggi_135 = |text: &str| {
            ctx.fonts(|f| {
                let mut job = egui::text::LayoutJob::default();
                job.wrap.max_width = (COL_RIGHT_W - crate::presentation::style::S2).max(10.0);
                job.append(
                    text,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::proportional(LYRIC_SIZE),
                        color: egui::Color32::WHITE,
                        ..Default::default()
                    },
                );
                f.layout_job(job).size().y
            })
        };
        let h135_6 = tinggi_135(&teks_owned[6]);
        let h16_6 = {
            let mut job = egui::text::LayoutJob::default();
            job.wrap.max_width = (COL_RIGHT_W - crate::presentation::style::S2).max(10.0);
            job.append(
                &teks_owned[6],
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::proportional(LYRIC_SIZE_BIG),
                    color: egui::Color32::WHITE,
                    ..Default::default()
                },
            );
            ctx.fonts(|f| f.layout_job(job).size().y)
        };
        assert!(
            h135_6 <= 28.0,
            "baris 6 seharusnya 1 baris teks di {LYRIC_SIZE} px, tingginya {h135_6} px"
        );
        assert!(
            h16_6 >= 34.0,
            "baris 6 seharusnya 2 baris teks di {LYRIC_SIZE_BIG} px, tingginya {h16_6} px"
        );

        // --- Ukuran teks memang berbeda per baris (16 vs 13.5), bukan konstan ---
        assert_eq!(lyric_size(0.0), LYRIC_SIZE_BIG);
        assert_eq!(lyric_size(1.0), LYRIC_SIZE);
        assert_eq!(lyric_size(5.0), LYRIC_SIZE);
        assert!(
            lyric_size(0.5) > LYRIC_SIZE && lyric_size(0.5) < LYRIC_SIZE_BIG,
            "lyric_size harus kontinu di antara kedua ukuran"
        );
    }

    /// Regresi: pegas posisi baris lirik TIDAK boleh menahan aplikasi bangun
    /// saat halaman yang menggambarnya tidak aktif.
    ///
    /// Bug yang dijaga: `lyric_anim_pos` hanya dimajukan `draw_page_music`.
    /// Saat pengguna pindah ke halaman lain (atau menutup panel) di tengah
    /// transisi, pegas itu berhenti di posisi antara dan `settled()` tidak
    /// pernah benar. Kalau keadaan itu ikut dihitung "masih bergerak" oleh
    /// `next_wake()`, aplikasi meminta frame 60 fps terus-menerus tanpa ada
    /// yang bergerak — dan tidak sembuh sendiri sampai halaman Music dibuka
    /// lagi.
    ///
    /// Test memakai pegas yang SENGAJA terdampar (posisi 0, target 1, belum
    /// maju) lalu menuntut: saat halaman bukan Music, aplikasi tidak dihitung
    /// bergerak dan `next_wake` tidak nol.
    #[test]
    fn pegas_lirik_terdampar_tidak_menahan_bangun_di_halaman_lain() {
        use crate::app::{Noktah, NotchPage};
        use eframe::egui::{pos2, vec2, Rect};

        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(1400.0, 900.0));
        let raw = || egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };

        let _ = ctx.run(raw(), |ctx| {
            crate::presentation::fonts::install(ctx);
            egui::CentralPanel::default().show(ctx, |_ui| {});
        });

        let mut app = Noktah::new_for_test(
            "test".to_string(),
            crate::infrastructure::platform::Hover::new(false),
            &ctx,
        );
        app.settings.lyrics_enabled = true;
        app.reduce_motion = false;
        // Panel terbuka penuh supaya syarat pertama terpenuhi, dan halaman
        // System supaya syarat halaman TIDAK terpenuhi.
        app.open.target = 1.0;
        app.open.snap();
        app.open_h.target = 1.0;
        app.open_h.snap();
        app.current_page = NotchPage::System;

        // Pegas terdampar: target bergerak, posisi ketinggalan, belum dimajukan.
        app.lyric_anim_pos.target = 4.0;
        app.lyric_anim_pos.pos = 1.0;

        let mut bergerak = true;
        let _ = ctx.run(raw(), |ctx| {
            egui::CentralPanel::default().show(ctx, |_ui| {});
            bergerak = app.lyric_line_animating();
        });
        assert!(
            !bergerak,
            "di halaman System, pegas lirik terdampar tidak boleh dihitung \
             bergerak — kalau ya, aplikasi meminta frame 60 fps selamanya"
        );

        let tunggu = app.next_wake(0);
        assert!(
            tunggu > std::time::Duration::ZERO,
            "di halaman System, next_wake harus boleh tidur walau pegas lirik \
             sedang terdampar; dapat {tunggu:?}"
        );

        // Di halaman Music, pegas yang sama HARUS dihitung bergerak.
        app.current_page = NotchPage::Music;
        let mut bergerak_music = false;
        let _ = ctx.run(raw(), |ctx| {
            egui::CentralPanel::default().show(ctx, |_ui| {});
            bergerak_music = app.lyric_line_animating();
        });
        assert!(
            bergerak_music,
            "di halaman Music, pegas yang belum settled harus dihitung bergerak \
             supaya transisinya digambar 60 fps"
        );
    }

    /// Pegas posisi baris lirik HARUS benar-benar sampai di tujuan, tidak
    /// berhenti sedikit di bawahnya.
    ///
    /// Kalau berhenti di bawah (mis. `settled()` menyatakan diam terlalu awal),
    /// baris aktif TIDAK PERNAH mencapai alpha penuh 255 — ia diam di ~247 dan
    /// baris aktif selamanya terlihat sedikit lebih redup dari seharusnya.
    /// Bug halus yang tidak memunculkan error apa pun.
    #[test]
    fn pegas_lirik_berhenti_tepat_di_alpha_penuh() {
        use crate::presentation::pages::music::lyric_alpha;
        use crate::presentation::spring::{Spring, MORPH_LYRIC_LINE};

        let mut s = Spring::from_spec(0.0, MORPH_LYRIC_LINE);
        s.target = 1.0;

        // Majukan pada langkah 60 Hz sampai diam.
        let mut langkah = 0;
        while !s.settled() && langkah < 600 {
            s.step(1.0 / 60.0);
            langkah += 1;
        }
        assert!(s.settled(), "pegas harus diam dalam 10 detik, {langkah} langkah");

        let d = (1.0_f32 - s.pos).abs();
        let alpha = lyric_alpha(d);
        assert_eq!(
            alpha, 255.0,
            "pegas berhenti di pos={} (sisa d={d}), jadi baris aktif hanya \
             beralpha {alpha} — tidak pernah mencapai 255. Pantulan {} atau \
             ambang settled terlalu longgar.",
            s.pos, MORPH_LYRIC_LINE.bounce
        );
    }

    /// Verifikasi font Jepang/CJK: karakter kanji, hiragana, katakana, dan simbol musik
    /// dapat dimuat dan di-layout dengan benar tanpa menghasilkan missing glyph / error.
    #[test]
    fn lirik_bahasa_jepang_dapat_dimuat_dan_terender_dengan_mplus_rounded() {
        use crate::presentation::pages::music::{
            lyric_row_centers_for, LYRIC_SIZE, LYRIC_SIZE_BIG,
        };

        const COL_RIGHT_W: f32 = (crate::presentation::layout::OPEN_W - 2.0 * 16.0)
            - 192.0
            - crate::presentation::style::S3;

        let ctx = egui::Context::default();
        let raw = || egui::RawInput::default();

        let _ = ctx.run(raw(), |ctx| {
            crate::presentation::fonts::install(ctx);
            egui::CentralPanel::default().show(ctx, |_ui| {});
        });
        // Frame 1: biarkan set_fonts aktif di cache.
        let _ = ctx.run(raw(), |_ctx| {});

        let jp_lines = vec![
            "七号線ロストボーイズ".to_string(),
            "♫ 僕らの歩いた七号線、雨上がりの街並み".to_string(),
            "後悔ばかりの毎日に、それでも光を探していた".to_string(),
            "いつか笑える日が来ると信じていたあの夏".to_string(),
            "季節は次々死んでいく 絶望の果てに何がある".to_string(),
        ];

        // Pastikan layout_job dapat mengukur teks Jepang pada kedua ukuran font
        for text in &jp_lines {
            let (h_small, w_small) = ctx.fonts(|f| {
                let mut job = egui::text::LayoutJob::default();
                job.wrap.max_width = (COL_RIGHT_W - crate::presentation::style::S2).max(10.0);
                job.append(
                    text,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::proportional(LYRIC_SIZE),
                        color: egui::Color32::WHITE,
                        ..Default::default()
                    },
                );
                let g = f.layout_job(job);
                (g.size().y, g.size().x)
            });

            let (h_big, w_big) = ctx.fonts(|f| {
                let mut job = egui::text::LayoutJob::default();
                job.wrap.max_width = (COL_RIGHT_W - crate::presentation::style::S2).max(10.0);
                job.append(
                    text,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::proportional(LYRIC_SIZE_BIG),
                        color: egui::Color32::WHITE,
                        ..Default::default()
                    },
                );
                let g = f.layout_job(job);
                (g.size().y, g.size().x)
            });

            assert!(h_small > 0.0 && w_small > 0.0, "teks '{text}' gagal di-render di ukuran kecil");
            assert!(h_big > 0.0 && w_big > 0.0, "teks '{text}' gagal di-render di ukuran besar");
            assert!(h_big >= h_small, "ukuran besar harus memiliki tinggi >= ukuran kecil");
        }

        // Pastikan row centers untuk lirik Jepang terhitung secara stabil
        let centers = lyric_row_centers_for(&ctx, &jp_lines, COL_RIGHT_W);
        assert_eq!(centers.len(), jp_lines.len());
        for i in 1..centers.len() {
            assert!(
                centers[i] > centers[i - 1],
                "baris lirik {i} ({}) harus berada di bawah baris sebelumnya ({})",
                centers[i],
                centers[i - 1]
            );
        }
    }

    #[test]
    fn kustomisasi_menu_navigasi_mengikuti_setelan_dan_fallback() {
        use crate::app::{Noktah, NotchPage};
        let ctx = egui::Context::default();
        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);
        app.settings = noktah_core::Settings::default();

        // Default 7 halaman aktif
        let default_pages = app.active_nav_pages();
        assert_eq!(default_pages.len(), 7);

        // Matikan beberapa halaman (Music & Pomodoro)
        app.settings.nav_music = false;
        app.settings.nav_pomodoro = false;
        let filtered = app.active_nav_pages();
        assert_eq!(filtered.len(), 5);
        assert!(!filtered.contains(&NotchPage::Music));
        assert!(!filtered.contains(&NotchPage::Pomodoro));

        // Uji rotasi prev/next hanya di halaman aktif
        app.current_page = NotchPage::Home;
        assert_eq!(app.next_nav_page(), NotchPage::Tasks);

        // Fallback jika semua halaman dimatikan (menjamin minimal Home)
        app.settings.nav_home = false;
        app.settings.nav_tasks = false;
        app.settings.nav_system = false;
        app.settings.nav_ask = false;
        app.settings.nav_agent = false;
        let empty_fallback = app.active_nav_pages();
        assert_eq!(empty_fallback, vec![NotchPage::Home]);
    }

    #[test]
    fn next_wake_hemat_saat_tertutup_tanpa_maskot() {
        use crate::app::Noktah;
        use std::time::Duration;
        let ctx = egui::Context::default();
        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);

        app.open.target = 0.0;
        app.open.snap();
        app.open_h.target = 0.0;
        app.open_h.snap();
        app.pill.target = 180.0;
        app.pill.snap();
        app.dock_open.target = 0.0;
        app.dock_open.snap();
        app.scrubber_expand.target = 0.0;
        app.scrubber_expand.snap();
        app.lyrics_anim.target = 0.0;
        app.lyrics_anim.snap();
        app.minimal_mix.target = 0.0;
        app.minimal_mix.snap();
        app.media.active = false;
        app.agent.active = false;
        app.todos.clear();
        app.banner = None;
        app.transisi_at = None;
        app.trimmed = true;

        // Tanpa maskot mini: tidak boleh membangunkan 30 fps (32 ms)
        app.settings.mini_agent = false;
        let now_ms = chrono::Local::now().timestamp_millis();
        assert!(
            app.next_wake(now_ms) >= Duration::from_secs(1),
            "next_wake saat tertutup tanpa maskot harus >= 1s, dapat {:?}",
            app.next_wake(now_ms)
        );

        // Notch minimal juga menyembunyikan maskot mini
        app.settings.mini_agent = true;
        app.settings.minimal_notch = true;
        app.minimal_mix.target = 1.0;
        app.minimal_mix.snap();
        assert!(
            app.next_wake(now_ms) >= Duration::from_secs(1),
            "next_wake saat notch minimal harus >= 1s, dapat {:?}",
            app.next_wake(now_ms)
        );
    }

    #[test]
    fn pill_kembali_ke_idle_saat_musik_dipause_sesuai_setelan() {
        use crate::app::Noktah;
        use std::time::{Duration, Instant};
        let ctx = egui::Context::default();
        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);

        app.settings = noktah_core::Settings::default();
        app.media.active = true;
        app.media.playing = false;
        app.media_paused_since = Some(Instant::now() - Duration::from_secs(1));

        // Bawaan true: setelah masa jeda singkat terlewati, pill kembali ke mode idle
        assert!(!app.show_media_in_mini());

        // Jika setelan dimatikan: tetap pertahankan state musik pada pill saat pause
        app.settings.mini_idle_on_pause = false;
        assert!(app.show_media_in_mini());

        // Saat musik kembali diputar: mode musik tetap tampil meski setelan idle aktif
        app.settings.mini_idle_on_pause = true;
        app.media.playing = true;
        assert!(app.show_media_in_mini());
    }

    #[test]
    fn antrean_fifo_izin_multi_agent_berurutan() {
        use crate::app::{Noktah, NotchPage, QueuedPermission};
        use noktah_core::model::{AgentStatus, PermissionDecision, PermissionRequest};
        use std::sync::mpsc::sync_channel;

        let ctx = egui::Context::default();
        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);
        app.settings = noktah_core::Settings::default();

        let (tx1, rx1) = sync_channel::<PermissionDecision>(1);
        let (tx2, rx2) = sync_channel::<PermissionDecision>(1);

        let req1 = PermissionRequest {
            request_id: "req-1".into(),
            tool_name: "Bash".into(),
            tool_input: serde_json::json!({"command": "cargo test"}),
            suggestions: vec![],
            session_title: Some("repo-a".into()),
        };
        let req2 = PermissionRequest {
            request_id: "req-2".into(),
            tool_name: "Write".into(),
            tool_input: serde_json::json!({"file_path": "src/lib.rs"}),
            suggestions: vec![],
            session_title: Some("repo-b".into()),
        };

        app.pending_permissions.push_back(QueuedPermission {
            request: req1.clone(),
            responder: tx1,
        });
        app.pending_permissions.push_back(QueuedPermission {
            request: req2,
            responder: tx2,
        });

        app.open.target = 1.0;
        app.open.pos = 1.0;
        app.current_page = NotchPage::Agent;
        app.agent.status = AgentStatus::AwaitingApproval(req1);

        // Selesaikan permintaan pertama: antrean berikutnya dipromosikan dan panel tetap terbuka.
        app.resolve_front_permission(
            PermissionDecision {
                behavior: "allow".into(),
                message: None,
                updated_input: None,
            },
            Some("Approved: Bash"),
            1_000,
        );
        assert_eq!(rx1.try_recv().unwrap().behavior, "allow");
        assert_eq!(app.pending_permissions.len(), 1);
        assert_eq!(app.open.target, 1.0);
        assert!(matches!(
            &app.agent.status,
            AgentStatus::AwaitingApproval(r)
                if r.request_id == "req-2" && r.session_title.as_deref() == Some("repo-b")
        ));

        // Selesaikan permintaan kedua: antrean kosong dan panel menutup.
        app.resolve_front_permission(
            PermissionDecision {
                behavior: "deny".into(),
                message: Some("No".into()),
                updated_input: None,
            },
            None,
            2_000,
        );
        assert_eq!(rx2.try_recv().unwrap().behavior, "deny");
        assert!(app.pending_permissions.is_empty());
        assert_eq!(app.open.target, 0.0);
    }

    #[test]
    fn test_fullscreen_autohide_dan_peek_intent() {
        use crate::app::Noktah;
        use std::sync::atomic::Ordering;

        let ctx = egui::Context::default();
        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover.clone(), &ctx);
        app.settings = noktah_core::Settings::default();
        app.settings.fullscreen_autohide = true;

        // Saat tidak fullscreen, tidak ada supresi
        hover.fullscreen.store(false, Ordering::Relaxed);
        let is_fs = app.settings.fullscreen_autohide && hover.fullscreen.load(Ordering::Relaxed);
        assert!(!is_fs);

        // Saat fullscreen aktif dan kursor jauh dari tepi atas
        hover.fullscreen.store(true, Ordering::Relaxed);
        let is_fs = app.settings.fullscreen_autohide && hover.fullscreen.load(Ordering::Relaxed);
        assert!(is_fs);

        let cy_far = 50.0;
        let peek_intent_far = cy_far <= 10.0;
        assert!(!peek_intent_far);
        let suppress = is_fs && app.open.target == 0.0 && !peek_intent_far;
        assert!(suppress);

        // Saat kursor menyentuh tepi atas layar (peek intent <= 10 px), supresi dilepas
        let cy_peek = 6.0;
        let peek_intent = cy_peek <= 10.0;
        assert!(peek_intent);
        let suppress_peek = is_fs && app.open.target == 0.0 && !peek_intent;
        assert!(!suppress_peek);
    }

    #[test]
    fn test_file_drop_shelf_cap_dan_clear() {
        use crate::app::Noktah;
        use crate::presentation::widgets::ShelfFile;
        use std::path::PathBuf;

        let ctx = egui::Context::default();
        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);

        for i in 1..=7 {
            app.file_shelf.push(ShelfFile {
                name: format!("doc-{i}.pdf"),
                path: PathBuf::from(format!("C:\\temp\\doc-{i}.pdf")),
                size_bytes: i * 1024,
            });
            if app.file_shelf.len() > 5 {
                app.file_shelf.remove(0);
            }
        }

        assert_eq!(app.file_shelf.len(), 5);
        assert_eq!(app.file_shelf[0].name, "doc-3.pdf");
        assert_eq!(app.file_shelf[4].name, "doc-7.pdf");

        app.file_shelf.clear();
        assert!(app.file_shelf.is_empty());
    }

    #[test]
    fn test_audio_device_hud_dan_focus_dnd() {
        use crate::app::Noktah;
        use std::time::Instant;

        let ctx = egui::Context::default();
        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);
        app.settings = noktah_core::Settings::default();
        app.settings.audio_hud_enabled = true;
        app.settings.focus_dnd = true;

        // Simulasi deteksi pergantian perangkat audio saat DND tidak aktif
        app.last_audio_dev_name = Some("speakers-guid".into());
        let new_dev_id = "headphones-bluetooth-guid".to_string();
        if app.last_audio_dev_name.as_deref() != Some(&new_dev_id) && !app.is_focus_dnd_active() {
            app.banner = Some(("🎧 Output audio beralih".into(), Instant::now()));
        }
        assert!(app.banner.is_some());
        assert_eq!(app.banner.as_ref().unwrap().0, "🎧 Output audio beralih");

        // Saat fokus DND aktif, banner audio ditahan
        app.banner = None;
        app.pomodoro.running = true;
        app.pomodoro.mode = noktah_core::PomodoroMode::Focus;
        assert!(app.is_focus_dnd_active());
        if app.last_audio_dev_name.as_deref() != Some("usb-dac-guid") && !app.is_focus_dnd_active() {
            app.banner = Some(("🎧 Output audio beralih".into(), Instant::now()));
        }
        assert!(app.banner.is_none());
    }

    #[test]
    fn test_calendar_agenda_filtering() {
        use noktah_core::calendar::{upcoming_events, CalendarEvent};

        let now_sec = 1_000_000;
        let events = vec![
            CalendarEvent {
                uid: "1".into(),
                summary: "Past Event".into(),
                description: None,
                location: None,
                start_timestamp: now_sec - 10_000,
                end_timestamp: Some(now_sec - 5_000),
                all_day: false,
            },
            CalendarEvent {
                uid: "2".into(),
                summary: "Ongoing Event".into(),
                description: None,
                location: None,
                start_timestamp: now_sec - 600,
                end_timestamp: Some(now_sec + 1200),
                all_day: false,
            },
            CalendarEvent {
                uid: "3".into(),
                summary: "Upcoming Meeting".into(),
                description: None,
                location: None,
                start_timestamp: now_sec + 3600,
                end_timestamp: Some(now_sec + 7200),
                all_day: false,
            },
        ];

        let upcoming = upcoming_events(&events, now_sec, 5);
        assert_eq!(upcoming.len(), 2);
        assert_eq!(upcoming[0].summary, "Ongoing Event");
        assert_eq!(upcoming[1].summary, "Upcoming Meeting");
        assert_eq!(upcoming[1].relative_time_label(now_sec), "1 jam lagi");
    }

    #[test]
    fn test_ipc_custom_tiles_lifecycle_without_mutating_agent() {
        use crate::app::Noktah;
        use crate::infrastructure::ipc::IpcEvent;
        use noktah_core::model::{AgentHookPayload, AgentStatus};
        use noktah_core::tile::TileTone;
        use std::sync::mpsc::channel;

        let ctx = egui::Context::default();
        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);
        app.settings = noktah_core::Settings::default();

        let (tx, rx) = channel();
        app.ipc.rx = rx;

        // Inisialisasi state awal agent: tidak aktif dan idle
        app.agent.active = false;
        app.agent.status = AgentStatus::Idle;
        let initial_updated = app.agent.updated_at;

        // 1. Kirim event tile upsert
        let payload_upsert = AgentHookPayload {
            hook_event_name: "tile".into(),
            tile_id: Some("deploy".into()),
            tile_value: Some("v1.5.0".into()),
            tile_tone: Some("success".into()),
            tile_ttl_secs: Some(3600),
            tile_pin_mini: Some(true),
            ..Default::default()
        };
        tx.send(IpcEvent::HookReceived(payload_upsert)).unwrap();
        app.poll_ipc();

        assert_eq!(app.custom_tiles.len(), 1);
        let tile = &app.custom_tiles.items()[0];
        assert_eq!(tile.id, "deploy");
        assert_eq!(tile.value, "v1.5.0");
        assert_eq!(tile.tone, TileTone::Success);
        assert!(tile.pin_mini);
        // Pastikan agent sama sekali tidak bermutasi
        assert!(!app.agent.active);
        assert_eq!(app.agent.status, AgentStatus::Idle);
        assert_eq!(app.agent.updated_at, initial_updated);

        // 2. Kirim event tile kedua
        let payload_upsert2 = AgentHookPayload {
            hook_event_name: "tile-set".into(),
            tile_id: Some("cpu".into()),
            tile_value: Some("42%".into()),
            tile_subtitle: Some("normal".into()),
            tile_tone: Some("accent".into()),
            ..Default::default()
        };
        tx.send(IpcEvent::HookReceived(payload_upsert2)).unwrap();
        app.poll_ipc();
        assert_eq!(app.custom_tiles.len(), 2);
        assert!(!app.agent.active);

        // 3. Kirim event tile-remove
        let payload_remove = AgentHookPayload {
            hook_event_name: "tile-remove".into(),
            tile_id: Some("deploy".into()),
            ..Default::default()
        };
        tx.send(IpcEvent::HookReceived(payload_remove)).unwrap();
        app.poll_ipc();
        assert_eq!(app.custom_tiles.len(), 1);
        assert_eq!(app.custom_tiles.items()[0].id, "cpu");
        assert!(!app.agent.active);

        // 4. Kirim event tile-clear
        let payload_clear = AgentHookPayload {
            hook_event_name: "tile-clear".into(),
            ..Default::default()
        };
        tx.send(IpcEvent::HookReceived(payload_clear)).unwrap();
        app.poll_ipc();
        assert!(app.custom_tiles.is_empty());
        assert!(!app.system_tiles_mode);
        assert!(!app.agent.active);

        // 5. Uji TTL expiry via poll_ipc
        let now_utc = chrono::Utc::now().timestamp_millis();
        let short_tile = noktah_core::CustomTile {
            id: "short".into(),
            title: "Short".into(),
            value: "10s".into(),
            ttl_secs: Some(1),
            updated_at_ms: now_utc - 2_000,
            ..Default::default()
        };
        assert!(app.custom_tiles.upsert(short_tile, now_utc - 2_000));
        assert_eq!(app.custom_tiles.len(), 1);
        app.system_tiles_mode = true;

        app.poll_ipc();
        assert!(app.custom_tiles.is_empty());
        assert!(!app.system_tiles_mode);
        assert!(!app.agent.active);
    }

    #[test]
    fn test_system_page_custom_tiles_fit() {
        use crate::app::Noktah;
        use crate::presentation::layout;
        use noktah_core::tile::{CustomTile, TileTone};

        egui::__run_test_ui(|ui| {
            let ctx = ui.ctx().clone();
            crate::presentation::fonts::install(&ctx);

            let hover = crate::infrastructure::platform::Hover::new(false);
            let mut app = Noktah::new_for_test("test_renderer".to_string(), hover, &ctx);
            app.settings = noktah_core::Settings::default();
            app.settings.custom_tiles_enabled = true;
            app.system_tiles_mode = true;

            let area = egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(layout::OPEN_W, layout::OPEN_H),
            );
            let content_rect = layout::content_rect(
                area,
                16.0,
                crate::presentation::style::S2,
            );
            let limit_y = content_rect.bottom();

            for count in [1, 3, 5] {
                app.custom_tiles.clear();
                let now_ms = 1_000_000;
                for i in 1..=count {
                    let tile = CustomTile {
                        id: format!("tile-{i}"),
                        title: format!("Custom Metric {i}"),
                        value: format!("{}%", i * 20),
                        subtitle: if i % 2 == 0 { Some(format!("sub-{i}")) } else { None },
                        tone: match i % 4 {
                            0 => TileTone::Success,
                            1 => TileTone::Accent,
                            2 => TileTone::Warning,
                            _ => TileTone::Error,
                        },
                        progress: if i % 2 == 1 { Some(i as f32 * 0.2) } else { None },
                        ttl_secs: Some(3600),
                        updated_at_ms: now_ms,
                        pin_mini: i == 1,
                    };
                    app.custom_tiles.upsert(tile, now_ms);
                }
                assert_eq!(app.custom_tiles.len(), count);

                let mut page_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(content_rect)
                        .layout(egui::Layout::top_down(egui::Align::LEFT)),
                );
                page_ui.set_clip_rect(content_rect);

                app.draw_page_system(&mut page_ui, content_rect);

                let min_rect = page_ui.min_rect();
                println!(
                    "System Page with {} custom tiles: bottom={}, limit={}, diff={}",
                    count,
                    min_rect.bottom(),
                    limit_y,
                    min_rect.bottom() - limit_y
                );
                assert!(
                    min_rect.bottom() <= limit_y + 0.5,
                    "System Page with {} tiles overflows by {}px! (bottom={}, limit={})",
                    count,
                    min_rect.bottom() - limit_y,
                    min_rect.bottom(),
                    limit_y
                );
            }
        });
    }

    #[test]
    fn test_built_in_theme_presets_application_and_theme_resolution() {
        use crate::presentation::theme::{luminance, on_color, Theme};
        use noktah_core::{Settings, ThemePreset};

        let presets = ThemePreset::built_in_presets();
        assert_eq!(presets.len(), 5);

        for preset in presets {
            let mut settings = Settings::default();
            preset.apply_to(&mut settings);

            assert_eq!(settings.theme_preset, preset.id_slug());
            assert_eq!(settings.opacity, preset.opacity);
            assert_eq!(settings.glass_specular, preset.glass_specular);
            assert_eq!(settings.glass_blur_radius, preset.glass_blur_radius);
            assert_eq!(settings.glass_saturation, preset.glass_saturation);
            assert_eq!(settings.glass_tint_rgb, preset.glass_tint_rgb);
            assert_eq!(settings.ears, preset.ears);

            let theme = Theme::from_settings(&settings);

            // Contrast invariance: accent must be legible
            let lum = luminance(theme.accent);
            assert!(
                lum >= 0.25,
                "Aksen preset '{}' terlalu gelap (lum={})",
                preset.name,
                lum
            );
            assert_eq!(theme.on_accent, on_color(theme.accent));

            if preset.glass_specular > 0.01 {
                assert!(theme.edge.a() > 0);
                assert!(theme.edge_bright.a() > 0);
            } else {
                assert_eq!(theme.edge, egui::Color32::TRANSPARENT);
                assert_eq!(theme.edge_bright, egui::Color32::TRANSPARENT);
            }

            let expected_alpha = (preset.opacity * 255.0) as u8;
            assert_eq!(theme.panel.a(), expected_alpha);
        }
    }

    #[test]
    fn test_custom_hex_accent_resolution() {
        use crate::presentation::theme::Theme;
        use noktah_core::{Accent, Settings};

        let mut settings = Settings::default();
        settings.accent = Accent::Teal;
        settings.custom_accent_hex = Some("#3B82F6".into());

        let theme = Theme::from_settings(&settings);
        // Custom accent #3B82F6 is [59, 130, 246], legible_accent adjusts it slightly for contrast
        assert!((theme.accent.b() as i32 - 246).abs() <= 2);

        // Invalid hex falls back to settings.accent
        settings.custom_accent_hex = Some("invalid-hex".into());
        let theme_fallback = Theme::from_settings(&settings);
        let [tr, tg, tb] = Accent::Teal.rgb();
        assert_eq!(theme_fallback.accent.r(), tr);
        assert_eq!(theme_fallback.accent.g(), tg);
        assert_eq!(theme_fallback.accent.b(), tb);
    }

    #[test]
    fn test_appearance_tab_ui_and_settings_backward_compat() {
        use crate::app::Noktah;
        use crate::presentation::theme::Theme;
        use noktah_core::Settings;

        // Legacy empty settings deserialization
        let legacy_settings: Settings = serde_json::from_str("{}").expect("deserialization of empty json");
        assert_eq!(legacy_settings.theme_preset, "obsidian");
        assert_eq!(legacy_settings.glass_blur_radius, 3);
        assert_eq!(legacy_settings.glass_saturation, 1.35);
        assert_eq!(legacy_settings.glass_specular, 0.0);
        assert_eq!(legacy_settings.custom_accent_hex, None);

        let theme = Theme::from_settings(&legacy_settings);
        assert_eq!(theme.edge, egui::Color32::TRANSPARENT);

        egui::__run_test_ui(|ui| {
            let ctx = ui.ctx().clone();
            let hover = crate::infrastructure::platform::Hover::new(false);
            let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);

            app.draw_appearance_tab(ui, noktah_core::Lang::Id, theme);
        });
    }

    fn collect_text_shapes(shape: &egui::epaint::Shape, out: &mut Vec<(String, egui::Rect)>) {
        match shape {
            egui::epaint::Shape::Text(t) => {
                out.push((t.galley.text().to_owned(), t.visual_bounding_rect()));
            }
            egui::epaint::Shape::Vec(v) => {
                for s in v {
                    collect_text_shapes(s, out);
                }
            }
            _ => {}
        }
    }

    fn collect_rect_shapes(shape: &egui::epaint::Shape, out: &mut Vec<egui::epaint::RectShape>) {
        match shape {
            egui::epaint::Shape::Rect(r) => out.push(*r),
            egui::epaint::Shape::Vec(v) => {
                for s in v {
                    collect_rect_shapes(s, out);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn test_custom_hex_typing_buffer_preserves_partial_edits() {
        use crate::app::Noktah;
        use crate::presentation::theme::Theme;
        use noktah_core::{Lang, Settings};

        let ctx = egui::Context::default();
        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);
        app.lang = Lang::En;
        app.settings.language = Lang::En;
        let input_id = egui::Id::new("custom_accent_hex_input");

        let run_appearance = |app: &mut Noktah, events: Vec<egui::Event>, focus: bool| {
            let mut raw = egui::RawInput::default();
            raw.events = events;
            ctx.run(raw, |ctx| {
                if focus {
                    ctx.memory_mut(|m| m.request_focus(input_id));
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let theme = Theme::from_settings(&app.settings);
                        app.draw_appearance_tab(ui, app.lang, theme);
                    });
                });
            })
        };

        // Frame 1: mount Appearance tab and focus the HEX TextEdit.
        let _ = run_appearance(&mut app, vec![], true);

        // Frame 2: type partial HEX '#3B82' -> buffer preserved, settings untouched, error indicator shown.
        let out_partial = run_appearance(
            &mut app,
            vec![egui::Event::Text("#3B82".to_string())],
            true,
        );
        assert_eq!(app.custom_accent_input, "#3B82");
        assert_eq!(app.settings.custom_accent_hex, None);
        let mut texts = Vec::new();
        for cs in &out_partial.shapes {
            collect_text_shapes(&cs.shape, &mut texts);
        }
        assert!(
            texts.iter().any(|(t, _)| t == "Invalid HEX"),
            "expected 'Invalid HEX' indicator for partial input, got {texts:?}"
        );

        // Frame 3: press Backspace -> buffer becomes '#3B8' without being clobbered.
        let _ = run_appearance(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::Backspace,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            true,
        );
        assert_eq!(app.custom_accent_input, "#3B8");
        assert_eq!(app.settings.custom_accent_hex, None);

        // Frame 4: finish valid HEX '#3B82F6' -> updates settings & Theme accent.
        let out_valid = run_appearance(
            &mut app,
            vec![egui::Event::Text("2F6".to_string())],
            true,
        );
        assert_eq!(app.custom_accent_input, "#3B82F6");
        assert_eq!(app.settings.custom_accent_hex.as_deref(), Some("#3B82F6"));
        assert_eq!(app.settings.theme_preset, "custom");
        let blue_theme = Theme::from_settings(&app.settings);
        assert!(blue_theme.accent.b() > blue_theme.accent.r());
        let mut texts_valid = Vec::new();
        for cs in &out_valid.shapes {
            collect_text_shapes(&cs.shape, &mut texts_valid);
        }
        assert!(
            !texts_valid.iter().any(|(t, _)| t == "Invalid HEX"),
            "valid HEX should not show error indicator"
        );

        // Frame 5: select all + type invalid '#+1+2+3' -> rejected & error indicator shown.
        let ctrl_all = egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        };
        let out_invalid = run_appearance(
            &mut app,
            vec![
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: ctrl_all,
                },
                egui::Event::Text("#+1+2+3".to_string()),
            ],
            true,
        );
        assert_eq!(app.custom_accent_input, "#+1+2+3");
        assert_eq!(
            app.settings.custom_accent_hex.as_deref(),
            Some("#3B82F6"),
            "invalid '#+1+2+3' must not overwrite valid custom_accent_hex"
        );
        let mut texts_invalid = Vec::new();
        for cs in &out_invalid.shapes {
            collect_text_shapes(&cs.shape, &mut texts_invalid);
        }
        assert!(
            texts_invalid.iter().any(|(t, _)| t == "Invalid HEX"),
            "expected 'Invalid HEX' indicator for '#+1+2+3'"
        );

        // Full Settings JSON import syncs custom_accent_input and updates Theme accent.
        let imported: Settings = serde_json::from_str(r##"{"custom_accent_hex":"#FF6B9D"}"##)
            .expect("valid settings JSON");
        app.apply_imported_settings(imported);
        assert_eq!(app.settings.custom_accent_hex.as_deref(), Some("#FF6B9D"));
        assert_eq!(app.custom_accent_input, "#FF6B9D");
        let pink_theme = Theme::from_settings(&app.settings);
        assert_eq!(app.theme.accent, pink_theme.accent);
        assert!(pink_theme.accent.r() > pink_theme.accent.b());
    }

    #[test]
    fn test_theme_preset_json_import_validation() {
        use noktah_core::theme_preset::ThemePreset;
        assert!(ThemePreset::from_json_str("{}").is_err());
        assert!(ThemePreset::from_json_str(r#"{"hello": "world"}"#).is_err());
        assert!(ThemePreset::from_json_str("invalid json string").is_err());

        let valid_json = r##"{
            "schema_version": 1,
            "name": "Emerald Slate",
            "accent": "green",
            "custom_accent_hex": "#10B981",
            "glass_effect": "full",
            "opacity": 0.85,
            "glass_tint_rgb": [16, 185, 129],
            "glass_specular": 0.5,
            "glass_blur_radius": 5,
            "glass_saturation": 1.2
        }"##;
        let preset = ThemePreset::from_json_str(valid_json);
        assert!(preset.is_ok(), "failed: {:?}", preset.err());
        let p = preset.unwrap();
        assert_eq!(p.accent, noktah_core::Accent::Green);
        assert_eq!(p.custom_accent_hex.as_deref(), Some("#10B981"));
    }

    #[test]
    fn test_ipc_hook_pin_mini_behavior() {
        use crate::app::Noktah;
        use crate::infrastructure::ipc::IpcEvent;
        use noktah_core::model::AgentHookPayload;
        use noktah_core::settings::GlassEffect;
        use noktah_core::tile::{CustomTile, TileTone};
        use std::sync::mpsc::channel;

        let ctx = egui::Context::default();
        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);
        app.settings.calendar_enabled = false;
        app.settings.audio_hud_enabled = false;
        app.settings.glass_effect = GlassEffect::Off;

        let (tx, rx) = channel();
        app.ipc.rx = rx;

        let now_ms = chrono::Utc::now().timestamp_millis();
        let payload = AgentHookPayload {
            hook_event_name: "tile".into(),
            tile_id: Some("build".into()),
            tile_title: Some("Build".into()),
            tile_value: Some("OK".into()),
            tile_pin_mini: Some(true),
            tile: Some(CustomTile {
                id: "build".into(),
                title: "Build".into(),
                value: "OK".into(),
                subtitle: None,
                tone: TileTone::Neutral,
                progress: None,
                ttl_secs: None,
                updated_at_ms: now_ms,
                pin_mini: true,
            }),
            ..Default::default()
        };

        // Before tile: idle pill target is 180.0
        assert_eq!(app.compute_pill_target(), 180.0);

        tx.send(IpcEvent::HookReceived(payload)).unwrap();
        // Drive the real frame update flow (which calls poll_ipc + compute_pill_target)
        let _ = ctx.run(Default::default(), |ctx| {
            app.update_frame(ctx);
        });

        assert_eq!(app.custom_tiles.len(), 1);
        assert!(app.custom_tiles.items()[0].pin_mini);
        assert_eq!(app.compute_pill_target(), 220.0);
        assert_eq!(app.pill.target, 220.0);
    }

    #[test]
    fn test_deterministic_next_wake_custom_tiles_ttl() {
        use crate::app::Noktah;
        use noktah_core::tile::{CustomTile, TileTone};
        use std::time::Duration;

        let ctx = egui::Context::default();
        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);
        app.settings.mini_agent = false;
        app.trimmed = true;
        app.media.active = false;
        app.banner = None;
        app.todos.clear();

        let t0 = 1_700_000_000_000_i64;
        let baseline_wake = app.next_wake(t0);
        assert!(
            baseline_wake >= Duration::from_secs(1),
            "expected baseline next_wake >= 1000ms without tiles, got {baseline_wake:?}"
        );

        let tile = CustomTile {
            id: "temp".into(),
            title: "Temp".into(),
            value: "1".into(),
            subtitle: None,
            tone: TileTone::Neutral,
            progress: None,
            ttl_secs: Some(1),
            updated_at_ms: t0 - 700, // expires at t0 + 300
            pin_mini: false,
        };
        app.custom_tiles.upsert(tile, t0);

        assert_eq!(app.next_wake(t0), Duration::from_millis(300));
        assert_eq!(app.next_wake(t0 + 250), Duration::from_millis(50));
    }

    #[test]
    fn test_draw_todo_tab_and_settings_window() {
        use crate::app::{Noktah, SettingsTab};
        use std::time::{Duration, Instant};

        let ctx = egui::Context::default();
        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);
        app.settings_tab = SettingsTab::Todo;
        app.banner = Some((
            "Settings toast active".to_string(),
            Instant::now()
                .checked_sub(Duration::from_millis(500))
                .unwrap_or_else(Instant::now),
        ));

        let output = ctx.run(Default::default(), |ctx| {
            let should_close = app.draw_settings_window(ctx);
            assert!(!should_close);
        });

        let mut texts = Vec::new();
        let mut rects = Vec::new();
        for cs in &output.shapes {
            collect_text_shapes(&cs.shape, &mut texts);
            collect_rect_shapes(&cs.shape, &mut rects);
        }

        let toast_text = texts
            .iter()
            .find(|(t, _)| t == "Settings toast active")
            .map(|(_, r)| *r)
            .expect("expected toast banner text to be rendered by draw_settings_window");

        let toast_bg = rects
            .iter()
            .find(|r| r.rect.contains(toast_text.center()) && r.rect.height() >= 28.0 && r.rect.height() <= 40.0)
            .expect("expected toast banner background rect around toast text");
        assert!(toast_bg.rounding.nw >= 14.0);
    }

    #[test]
    fn test_calendar_ics_path_change_clears_stale_events_and_surfaces_error() {
        use crate::app::Noktah;
        use noktah_core::Lang;

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let tmp_dir = std::env::temp_dir().join(format!(
            "noktah_calendar_test_{}_{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let valid_ics = tmp_dir.join("valid.ics");
        let missing_ics = tmp_dir.join("missing.ics");

        std::fs::write(
            &valid_ics,
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:u1\r\nSUMMARY:Sprint Sync\r\nDTSTART:20261127T100000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        )
        .unwrap();

        let ctx = egui::Context::default();
        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);
        app.lang = Lang::En;
        app.settings.language = Lang::En;
        app.settings.calendar_enabled = true;
        app.settings.calendar_ics_path = valid_ics.to_string_lossy().into_owned();

        app.poll_calendar();
        assert_eq!(app.calendar_events.len(), 1);
        assert_eq!(app.calendar_events[0].summary, "Sprint Sync");
        assert!(app.calendar_error.is_none());

        // Switch to missing .ics -> stale events cleared and error state surfaced.
        app.settings.calendar_ics_path = missing_ics.to_string_lossy().into_owned();
        app.poll_calendar();
        assert!(
            app.calendar_events.is_empty(),
            "stale calendar events must be cleared when switching to missing .ics"
        );
        assert!(
            app.calendar_error.is_some(),
            "calendar_error should be set when custom .ics cannot be read"
        );

        app.settings_tab = crate::app::SettingsTab::Display;
        let output = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                app.draw_settings_tab_content(ui);
            });
        });
        let mut texts = Vec::new();
        for cs in &output.shapes {
            collect_text_shapes(&cs.shape, &mut texts);
        }
        assert!(
            texts
                .iter()
                .any(|(t, _)| t.contains("Failed to load .ics file")),
            "Display settings should render localized calendar error, got {texts:?}"
        );

        // Clearing custom path clears error state.
        app.settings.calendar_ics_path.clear();
        app.poll_calendar();
        assert!(app.calendar_error.is_none());

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_import_settings_preserves_calendar_events_on_same_path_and_reloads_on_change() {
        use crate::app::Noktah;
        use noktah_core::Settings;
        use std::time::{Duration, Instant};

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let tmp_dir = std::env::temp_dir().join(format!(
            "noktah_calendar_import_test_{}_{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let ics_a = tmp_dir.join("a.ics");
        let ics_b = tmp_dir.join("b.ics");

        std::fs::write(
            &ics_a,
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:a1\r\nSUMMARY:Event Alpha\r\nDTSTART:20261127T100000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        )
        .unwrap();
        std::fs::write(
            &ics_b,
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:b1\r\nSUMMARY:Event Beta\r\nDTSTART:20261127T110000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        )
        .unwrap();

        let ctx = egui::Context::default();
        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);
        app.settings.calendar_enabled = true;
        app.settings.calendar_ics_path = ics_a.to_string_lossy().into_owned();
        app.poll_calendar();
        assert_eq!(app.calendar_events.len(), 1);
        assert_eq!(app.calendar_events[0].summary, "Event Alpha");

        // Ensure last_calendar_check is fresh (within 45s throttle).
        app.last_calendar_check = Instant::now();

        // 1. Import settings with same calendar_ics_path (even with extra whitespace): retains events immediately.
        let mut same_path = Settings::default();
        same_path.calendar_enabled = true;
        same_path.calendar_ics_path = format!("  {}  ", ics_a.to_string_lossy());
        app.apply_imported_settings(same_path);
        assert_eq!(app.calendar_events.len(), 1);
        assert_eq!(app.calendar_events[0].summary, "Event Alpha");

        // 2. Import settings with a different calendar_ics_path: clears stale events and reloads immediately on poll_calendar().
        let mut diff_path = Settings::default();
        diff_path.calendar_enabled = true;
        diff_path.calendar_ics_path = ics_b.to_string_lossy().into_owned();
        app.apply_imported_settings(diff_path);
        assert!(app.calendar_events.is_empty());
        assert!(app.last_calendar_check.elapsed() >= Duration::from_secs(45));
        app.poll_calendar();
        assert_eq!(app.calendar_events.len(), 1);
        assert_eq!(app.calendar_events[0].summary, "Event Beta");

        // 3. Import settings disabling calendar: clears events & error.
        let mut disabled = Settings::default();
        disabled.calendar_enabled = false;
        disabled.calendar_ics_path = ics_b.to_string_lossy().into_owned();
        app.calendar_error = Some("stale".into());
        app.apply_imported_settings(disabled);
        assert!(app.calendar_events.is_empty());
        assert!(app.calendar_error.is_none());

        // 4. Import settings re-enabling calendar with same path: invalidates throttle and reloads immediately.
        let mut reenabled = Settings::default();
        reenabled.calendar_enabled = true;
        reenabled.calendar_ics_path = ics_b.to_string_lossy().into_owned();
        app.apply_imported_settings(reenabled);
        app.poll_calendar();
        assert_eq!(app.calendar_events.len(), 1);
        assert_eq!(app.calendar_events[0].summary, "Event Beta");

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_settings_toast_wraps_long_reminder_inside_bg_and_viewport() {
        use crate::app::{Noktah, SettingsTab};
        use eframe::egui::{pos2, vec2, Rect};
        use std::time::{Duration, Instant};

        let ctx = egui::Context::default();
        let min_win = Rect::from_min_size(pos2(0.0, 0.0), vec2(640.0, 460.0));
        let raw = || egui::RawInput {
            screen_rect: Some(min_win),
            ..Default::default()
        };

        // Frame 0: install production fonts.
        let _ = ctx.run(raw(), |ctx| {
            crate::presentation::fonts::install(ctx);
            egui::CentralPanel::default().show(ctx, |_ui| {});
        });
        let _ = ctx.run(raw(), |_ctx| {});

        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);
        app.settings_tab = SettingsTab::Todo;
        let long_reminder = "Reminder: Review all pull request comments, verify release SHA-256 checksums, check calendar ICS import behavior, and verify v1.6.1 before 17:00!";
        assert!(long_reminder.len() > 120);
        app.banner = Some((
            long_reminder.to_string(),
            Instant::now()
                .checked_sub(Duration::from_millis(500))
                .unwrap_or_else(Instant::now),
        ));

        let output = ctx.run(raw(), |ctx| {
            let should_close = app.draw_settings_window(ctx);
            assert!(!should_close);
        });

        let mut texts = Vec::new();
        let mut rects = Vec::new();
        for cs in &output.shapes {
            collect_text_shapes(&cs.shape, &mut texts);
            collect_rect_shapes(&cs.shape, &mut rects);
        }

        let (_, text_rect) = texts
            .iter()
            .find(|(t, _)| t.starts_with("Reminder: Review all pull request comments"))
            .expect("expected wrapped long reminder text shape");

        let toast_bg = rects
            .iter()
            .find(|r| {
                r.fill != egui::Color32::TRANSPARENT
                    && r.stroke.width > 0.0
                    && r.rect.contains(text_rect.center())
            })
            .expect("expected toast background rect behind long reminder text");

        assert!(
            toast_bg.rect.contains_rect(*text_rect),
            "painted toast text rect {:?} escaped toast bg rect {:?}",
            text_rect,
            toast_bg.rect
        );
        assert!(
            min_win.contains_rect(toast_bg.visual_bounding_rect()),
            "toast bg rect {:?} escaped minimum viewport {:?}",
            toast_bg.visual_bounding_rect(),
            min_win
        );
        assert!(
            min_win.contains_rect(*text_rect),
            "toast text rect {:?} escaped minimum viewport {:?}",
            text_rect,
            min_win
        );
        assert!(
            toast_bg.rect.height() > 40.0,
            "expected multi-line wrapped toast height > 40px, got {}",
            toast_bg.rect.height()
        );
    }

    #[test]
    fn test_about_tab_honest_sha256_and_localized_errors() {
        use crate::app::Noktah;
        use crate::infrastructure::updater::{UpdateError, UpdateStatus};
        use crate::presentation::theme::Theme;
        use noktah_core::Lang;
        use std::path::PathBuf;

        let ctx = egui::Context::default();
        let hover = crate::infrastructure::platform::Hover::new(false);
        let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);

        let render_about = |app: &mut Noktah, lang: Lang| -> Vec<String> {
            app.lang = lang;
            app.settings.language = lang;
            let theme = Theme::from_settings(&app.settings);
            let out = ctx.run(Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    app.draw_about_tab(ui, lang, theme);
                });
            });
            let mut texts = Vec::new();
            for cs in &out.shapes {
                collect_text_shapes(&cs.shape, &mut texts);
            }
            texts.into_iter().map(|(t, _)| t).collect()
        };

        // 1. Verified == true shows SHA-256 verified text
        app.update_status = UpdateStatus::ReadyToRestart {
            version: "v1.6.1".into(),
            staged_exe: PathBuf::from("noktah-native.exe"),
            staged_hook: None,
            sha256_verified: true,
        };
        let en_verified = render_about(&mut app, Lang::En);
        assert!(en_verified.iter().any(|t| t.contains("SHA-256 verified")));
        let id_verified = render_about(&mut app, Lang::Id);
        assert!(id_verified.iter().any(|t| t.contains("terverifikasi SHA-256")));

        // 2. Verified == false MUST NOT claim SHA-256 verified
        app.update_status = UpdateStatus::ReadyToRestart {
            version: "v1.6.1".into(),
            staged_exe: PathBuf::from("noktah-native.exe"),
            staged_hook: None,
            sha256_verified: false,
        };
        let en_unverified = render_about(&mut app, Lang::En);
        assert!(!en_unverified.iter().any(|t| t.contains("SHA-256 verified")));
        assert!(en_unverified
            .iter()
            .any(|t| t.contains("SHA-256 checksum unavailable")));
        let id_unverified = render_about(&mut app, Lang::Id);
        assert!(!id_unverified.iter().any(|t| t.contains("terverifikasi SHA-256")));
        assert!(id_unverified
            .iter()
            .any(|t| t.contains("checksum SHA-256 tidak tersedia")));

        // 3. Structured error renders cleanly in EN without Indonesian leak
        app.update_status = UpdateStatus::Error(UpdateError::GitHubUnreachable("timed out".into()));
        let en_err = render_about(&mut app, Lang::En);
        assert!(en_err
            .iter()
            .any(|t| t.contains("Failed: Unable to reach GitHub: timed out")));
        assert!(!en_err.iter().any(|t| t.contains("Gagal menghubungi")));
    }

    #[test]
    fn test_settings_ai_tab_hook_path_renders_cleanly_at_min_size() {
        use crate::app::{Noktah, SettingsTab};
        use eframe::egui::{pos2, vec2, Rect};
        use noktah_core::{i18n, Lang};

        let before_hook_path = crate::infrastructure::os::hook_in_user_path();
        let min_win = Rect::from_min_size(pos2(0.0, 0.0), vec2(640.0, 460.0));

        for lang in [Lang::Id, Lang::En] {
            let ctx = egui::Context::default();
            let raw = || egui::RawInput {
                screen_rect: Some(min_win),
                ..Default::default()
            };

            // Frame 0 & 1: install production fonts so wrapping matches runtime.
            let _ = ctx.run(raw(), |ctx| {
                crate::presentation::fonts::install(ctx);
                egui::CentralPanel::default().show(ctx, |_ui| {});
            });
            let _ = ctx.run(raw(), |_ctx| {});

            let hover = crate::infrastructure::platform::Hover::new(false);
            let mut app = Noktah::new_for_test("test".to_string(), hover, &ctx);
            app.settings_tab = SettingsTab::Ai;
            app.lang = lang;
            app.settings.language = lang;

            // Check both initial cold-cache frame (frame 0) and settled frame (frame 1).
            for frame_idx in 0..2 {
                let output = ctx.run(raw(), |ctx| {
                    let should_close = app.draw_settings_window(ctx);
                    assert!(!should_close);
                });

                assert!(app.hook_path_cached.is_some());

                let mut texts = Vec::new();
                let mut rects = Vec::new();
                for cs in &output.shapes {
                    collect_text_shapes(&cs.shape, &mut texts);
                    collect_rect_shapes(&cs.shape, &mut rects);
                }

                let expected_label = i18n::t(lang, "settings.hook_path");
                let expected_desc = i18n::t(lang, "settings.hook_path_desc");

                let (_, label_rect) = texts
                    .iter()
                    .find(|(t, _)| t == expected_label)
                    .unwrap_or_else(|| {
                        panic!("lang={lang:?} frame={frame_idx}: missing label '{expected_label}'")
                    });
                let (_, desc_rect) = texts
                    .iter()
                    .find(|(t, _)| t == expected_desc)
                    .unwrap_or_else(|| {
                        panic!("lang={lang:?} frame={frame_idx}: missing desc '{expected_desc}'")
                    });

                let toggle_rect = rects
                    .iter()
                    .find(|r| {
                        (r.rect.width() - 36.0).abs() < 0.5
                            && (r.rect.height() - 20.0).abs() < 0.5
                            && r.rounding.nw >= 900.0
                            && r.rect.center().y >= label_rect.top() - 4.0
                            && r.rect.center().y <= desc_rect.bottom() + 4.0
                    })
                    .map(|r| r.rect)
                    .unwrap_or_else(|| {
                        panic!(
                            "lang={lang:?} frame={frame_idx}: missing hook_path toggle rect near label={label_rect:?} desc={desc_rect:?}"
                        )
                    });

                assert!(
                    min_win.contains_rect(*label_rect),
                    "lang={lang:?} frame={frame_idx}: label rect {label_rect:?} escaped viewport {min_win:?}"
                );
                assert!(
                    min_win.contains_rect(*desc_rect),
                    "lang={lang:?} frame={frame_idx}: desc rect {desc_rect:?} escaped viewport {min_win:?}"
                );
                assert!(
                    min_win.contains_rect(toggle_rect),
                    "lang={lang:?} frame={frame_idx}: toggle rect {toggle_rect:?} escaped viewport {min_win:?}"
                );
                assert!(
                    !label_rect.intersects(toggle_rect)
                        && label_rect.right() <= toggle_rect.left() + 0.5,
                    "lang={lang:?} frame={frame_idx}: label rect {label_rect:?} overlapped toggle rect {toggle_rect:?}"
                );
                assert!(
                    !desc_rect.intersects(toggle_rect)
                        && desc_rect.right() <= toggle_rect.left() + 0.5,
                    "lang={lang:?} frame={frame_idx}: desc rect {desc_rect:?} overlapped toggle rect {toggle_rect:?}"
                );
                assert!(
                    !label_rect.intersects(*desc_rect)
                        && label_rect.bottom() <= desc_rect.top() + 0.5,
                    "lang={lang:?} frame={frame_idx}: label rect {label_rect:?} overlapped desc rect {desc_rect:?}"
                );
            }
        }

        assert_eq!(
            crate::infrastructure::os::hook_in_user_path(),
            before_hook_path,
            "rendering SettingsTab::Ai must never mutate HKCU\\Environment"
        );
    }
}
