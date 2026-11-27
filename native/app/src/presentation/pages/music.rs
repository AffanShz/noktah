//! Halaman Musik — tata letak dua kolom dari prototipe `noktah-redesign.html`:
//! kolom kiri 272 px (sampul, judul, scrubber, transport), kolom kanan lirik
//! dengan mask gradasi atas-bawah.
//!
//! Perbedaan penting dari versi sebelumnya: sampul 68 px (bukan 40) dengan
//! lengkung 16 px, judul 17 px semi-bold, dan transport memakai pola grid
//! `36px 1fr 36px` supaya tombol tengah selalu benar-benar di tengah kolom —
//! bukan hasil pembagian yang bergeser saat lebar label berubah.

use crate::app::{LyricsStatus, Noktah};
use crate::presentation::{icons::Icon, style, widgets};
use eframe::egui::{self, Color32, Id, Rect, Rounding, Sense, Stroke};
use noktah_core::format;

/// Ukuran huruf SEMUA baris lirik — tetap, tidak pernah dianimasikan.
///
/// Sama dengan `style::T_TITLE`. Ukuran TIDAK boleh dianimasikan: baris yang
/// panjangnya berada di ambang pembungkusan bisa berubah jumlah baris wrap di
/// tengah animasi ("And in your eyes you're holding mine" = 1 baris/22.0 px
/// @13.5 tapi 2 baris/44.0 px @16, flip di 15.51), yaitu +22.0 px tinggi dalam
/// SATU frame — itulah lompatan yang harus dibunuh. Hierarki dibawa alpha.
pub(crate) const LYRIC_SIZE: f32 = 13.5;

/// Ukuran huruf baris lirik yang SEDANG aktif.
///
/// Ini nilai "besar" yang dulu dianimasikan 13.5→16 px. Sekarang penekanannya
/// kembali, tetapi geometrinya (tinggi slot tiap baris) dibekukan di ukuran
/// BESAR ini — lihat `lyric_size` dan komentar reservasi di `draw_page_music`.
/// Baris non-aktif tetap 13.5 px.
pub(crate) const LYRIC_SIZE_BIG: f32 = 16.0;

/// Ukuran huruf baris lirik sebagai fungsi jarak `d = |idx − posisi_aktif|`.
///
/// Kontinu: baris baru membesar dan baris lama menyusut dari SATU pegas yang
/// sama, bukan dari dua animasi terpisah. Baris aktif (d=0) = 16 px, tetangga
/// (d≥1) = 13.5 px, di antaranya interpolasi linier.
pub(crate) fn lyric_size(d: f32) -> f32 {
    let t = 1.0 - d.clamp(0.0, 1.0); // d=0 → 1 (paling besar), d>=1 → 0
    LYRIC_SIZE + (LYRIC_SIZE_BIG - LYRIC_SIZE) * t
}

/// Tinggi area lirik yang terlihat, dalam koordinat lokal galley.
///
/// Dipakai bersama oleh produksi (`draw_page_music`) dan tes. Galley egui
/// mengukur lebih tinggi dari ukuran huruf: ada kelonggaran ascent/descent dan
/// baris galley mengembalikan tinggi yang berbeda pada 13.5 px vs 16 px, jadi
/// selisih ini tidak boleh dianggap sepele. Dipisah supaya tes invariansi
/// memakai angka yang BENAR-BENAR sama dengan produksi, bukan perkiraan.
pub(crate) const LYRIC_TEXT_SLACK: f32 = 6.0;

/// Tinggi slot satu baris lirik dari tinggi galley GEOMETRI-nya.
///
/// Galley geometri SELALU diukur pada `LYRIC_SIZE_BIG` (16 px), apa pun
/// keadaan baris itu, sehingga `slot_h` tidak pernah berubah saat baris aktif
/// berganti. Karena `galley.size().y` tidak menurun ketika ukuran naik, slot
/// ini selalu cukup untuk ukuran berapa pun antara 13.5 dan 16 px.
pub(crate) fn lyric_slot_h(galley_big_h: f32) -> f32 {
    galley_big_h + LYRIC_TEXT_SLACK
}

/// Jarak vertikal antar-slot lirik (`ui.spacing_mut().item_spacing.y`).
pub(crate) const LYRIC_ROW_GAP: f32 = 4.0;

/// Pusat vertikal tiap baris lirik dari tinggi galley slot-nya.
///
/// Murni (tanpa egui) supaya produksi dan tes memakai model yang SAMA, bukan
/// perkiraan paralel. `slot_galley_h[i]` = tinggi galley baris `i` pada ukuran
/// slot — di produksi selalu `LYRIC_SIZE_BIG`, jadi hasilnya invarian terhadap
/// indeks aktif. Kalau tinggi slot justru diukur dari ukuran baris itu sendiri,
/// vektor ini berubah saat baris aktif berganti — itulah regresi yang dijaga
/// tes invariansi.
pub(crate) fn lyric_row_centers(slot_galley_h: &[f32]) -> Vec<f32> {
    let mut out = Vec::with_capacity(slot_galley_h.len());
    let mut cur_y = 0.0_f32;
    for &h in slot_galley_h {
        let slot_h = lyric_slot_h(h);
        out.push(cur_y + slot_h * 0.5);
        cur_y += slot_h + LYRIC_ROW_GAP;
    }
    out
}

/// Pusat tiap baris dari daftar teks + lebar kolom, diukur pada galley
/// GEOMETRI (`LYRIC_SIZE_BIG`) — model yang SAMA dipakai produksi.
///
/// Disediakan untuk tes invariansi: karena galley geometri selalu 16 px,
/// vektor ini TIDAK bergantung pada indeks baris aktif. Kalau tinggi slot
/// justru diukur dari ukuran baris itu sendiri, vektor ini akan berubah saat
/// baris aktif berganti — itulah regresi yang dijaga tes. Dipakai bersama
/// `lyric_row_centers` supaya tidak ada model kedua yang bisa menyimpang.
#[allow(dead_code)]
pub(crate) fn lyric_row_centers_for(
    ctx: &egui::Context,
    texts: &[String],
    col_right_w: f32,
) -> Vec<f32> {
    let wrap_w = (col_right_w - style::S2).max(10.0);
    let slot_galley_h: Vec<f32> = ctx.fonts(|f| {
        texts
            .iter()
            .map(|t| {
                let mut job = egui::text::LayoutJob::default();
                job.wrap.max_width = wrap_w;
                job.append(
                    t,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::proportional(LYRIC_SIZE_BIG),
                        color: egui::Color32::PLACEHOLDER,
                        ..Default::default()
                    },
                );
                f.layout_job(job).size().y
            })
            .collect()
    });
    lyric_row_centers(&slot_galley_h)
}

/// Alpha baris lirik sebagai fungsi jarak `d` = |indeks_baris − posisi_aktif|.
///
/// Reproduksi PERSIS nilai istirahat yang lama (d=0 → 255, d=1 → 110,
/// d≥1.5 → 55) tetapi kontinu dengan kemiringan terbatas, sehingga baris
/// tetangga "dijalani" — bukan dilompati dalam satu frame seperti tabel
/// peran lama. Baris aktif memakai 255 (paling terang); baris tetangga 110;
/// selebihnya 55.
pub(crate) fn lyric_alpha(d: f32) -> f32 {
    if d <= 1.0 {
        255.0 + (110.0 - 255.0) * d
    } else if d <= 1.5 {
        110.0 + (55.0 - 110.0) * ((d - 1.0) / 0.5)
    } else {
        55.0
    }
}

impl Noktah {
    /// Halaman 0: Music & Synced Lyrics split view.
    pub(crate) fn draw_page_music(&mut self, ui: &mut egui::Ui, area: Rect) {
        let theme = self.theme;
        let aktif = noktah_core::lrc::active_line(
            &self.lyrics,
            self.media.position_ms + self.settings.lyric_offset_ms,
        );
        // Pergantian baris aktif menggerakkan SATU spring (indeks baris) alih-alih
        // menganimasikan ukuran huruf. Ukuran dulu dianimasikan 13.5→16 px dan itu
        // +22.0 px tinggi dalam satu frame untuk baris di ambang pembungkusan.
        // Spring membawa kecepatan: pergantian cepat beruntun melanjutkan gerak,
        // bukan restart dari nol.
        //
        // INI SATU-SATUNYA TEMPAT `lyric_anim_pos` DIMAJUKAN. `frame.rs` sengaja
        // tidak memajukannya: dua pemanggil `step()` pada frame yang sama membuat
        // pegas maju 2x — durasi 0.34 s jadi efektif ~0.17 s, tanpa error apa pun.
        // Yang dilakukan `frame.rs` hanya membaca `lyric_line_animating()` untuk
        // penjadwalan, bukan memajukan pegasnya.
        let berubah_baris = aktif != self.last_active_lyric;
        if berubah_baris {
            let was_none = self.last_active_lyric.is_none();
            self.last_active_lyric = aktif;
            self.lyric_line_active_at = std::time::Instant::now();
            self.lyric_anim_pos.target = aktif.map(|i| i as f32).unwrap_or(0.0);
            // Akuisisi pertama, reduce_motion, dan seek/lompat jauh harus instan
            // (sama seperti `t_trans` dulu dipaksa 1.0). Hanya pergantian antar
            // baris berdekatan yang boleh beranimasi.
            if was_none
                || self.reduce_motion
                || (self.lyric_anim_pos.target - self.lyric_anim_pos.pos).abs() > 3.0
            {
                self.lyric_anim_pos.snap();
            }
        }

        // Pada frame pergantian JANGAN `step()`: kalau frame sebelumnya berjarak
        // 80 ms, langkah pertama dengan dt penuh akan besar. Frame pertama sesudah
        // pergantian mulai dari kecepatan nol, jadi tidak ada lompatan.
        if self.reduce_motion {
            self.lyric_anim_pos.snap();
        } else if !berubah_baris {
            self.lyric_anim_pos.step(ui.input(|i| i.stable_dt).min(0.1));
        }
        if !self.lyric_anim_pos.settled() {
            ui.ctx().request_repaint();
        }

        let body_h = area.height().max(60.0);
        let elapsed_manual = self.lyric_manual_at.elapsed();
        let auto_scroll = self.is_lyrics_synced() && elapsed_manual >= std::time::Duration::from_secs(3);
        if self.is_lyrics_synced() && !auto_scroll {
            let remain = std::time::Duration::from_secs(3) - elapsed_manual;
            ui.ctx().request_repaint_after(remain);
        } else if auto_scroll {
            if self.reduce_motion {
                self.lyric_scroll_pos = self.lyric_target_scroll;
            } else {
                let dt = ui.input(|i| i.stable_dt).min(0.1);
                let speed = 8.5; // Smooth Apple-like spring/damping
                let factor = (1.0 - (-speed * dt).exp()).clamp(0.0, 1.0);
                self.lyric_scroll_pos += (self.lyric_target_scroll - self.lyric_scroll_pos) * factor;
                if (self.lyric_target_scroll - self.lyric_scroll_pos).abs() > 0.5 {
                    ui.ctx().request_repaint();
                }
            }
        }
        let mut seek: Option<i64> = None;

        // Buka-tutup kolom lirik dengan animasi pegas yang smooth:
        let gap_max = style::S3;
        let can_show_lyrics = (area.width() - 192.0 - gap_max).max(0.0) >= 100.0;
        self.lyrics_anim.target = if self.settings.lyrics_enabled && can_show_lyrics { 1.0 } else { 0.0 };

        let t_lyr = if self.reduce_motion {
            self.lyrics_anim.target
        } else {
            self.lyrics_anim.pos.clamp(0.0, 1.0)
        };

        if !self.lyrics_anim.settled() {
            ui.ctx().request_repaint();
        }

        let col_left_w = if can_show_lyrics {
            egui::lerp(area.width()..=192.0, t_lyr)
        } else {
            area.width()
        };
        let gap = if can_show_lyrics { gap_max * t_lyr } else { 0.0 };
        let col_right_w = (area.width() - col_left_w - gap).max(0.0);

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            // ---- Kiri: sampul, judul, scrubber, transport ----
            const ART: f32 = 44.0;
            const SCRUBBER: f32 = 12.0;
            const PLAY: f32 = 36.0;
            const TOTAL_H: f32 = ART + SCRUBBER + PLAY; // 92.0 px
            const IB: f32 = 28.0;
            let gap_y = ((body_h - TOTAL_H) / 2.0).clamp(4.0, 10.0);

            ui.allocate_ui_with_layout(
                egui::vec2(col_left_w, body_h),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;

                    // Sampul 44 px, radius 10.
                    const ART_R: f32 = 10.0;
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = style::S2;
                        if let Some(tex) = &self.art {
                            let img = egui::Image::new(tex)
                                .fit_to_exact_size(egui::vec2(ART, ART))
                                .rounding(Rounding::same(ART_R));
                            let resp = ui.add(img);
                            ui.painter().rect_stroke(
                                resp.rect,
                                Rounding::same(ART_R),
                                Stroke::new(0.5, Color32::from_white_alpha(64)),
                            );
                        } else {
                            let (kotak, _) =
                                ui.allocate_exact_size(egui::vec2(ART, ART), Sense::hover());
                            ui.painter().rect_filled(
                                kotak,
                                Rounding::same(ART_R),
                                Color32::from_white_alpha(31),
                            );
                            widgets::glyph(
                                ui.painter(),
                                Icon::Music,
                                Rect::from_center_size(kotak.center(), egui::vec2(22.0, 22.0)),
                                Color32::from_white_alpha(235),
                                1.7,
                            );
                        }

                        let meta_w = (col_left_w - ART - style::S2).max(40.0);
                        ui.allocate_ui_with_layout(
                            egui::vec2(meta_w, ART),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                ui.spacing_mut().item_spacing.y = 1.0;
                                ui.add_space(style::S1);
                                widgets::marquee_title(
                                    ui,
                                    &self.media.title,
                                    egui::FontId::proportional(style::T_TITLE_LG),
                                    theme.ink,
                                    meta_w,
                                );
                                let artis = widgets::ellipsis_sized(ui, &self.media.artist, meta_w, style::T_SUBTITLE);
                                ui.label(style::subtitle(artis, theme.dim));
                            },
                        );
                    });

                    ui.add_space(gap_y);

                    // ---- Scrubber ----
                    let dur_s = (self.media.duration_ms as f32 / 1000.0).max(1.0);
                    // Saat seek belum dikonfirmasi SMTC, tahan thumb di target
                    // supaya tidak memantul ke posisi lama (siklus ≤ 500 ms).
                    let pos_disp = if let Some((t, _)) = self.seek_at {
                        (t as f32 / 1000.0).clamp(0.0, dur_s)
                    } else {
                        (self.media.position_ms as f32 / 1000.0).clamp(0.0, dur_s)
                    };
                    let mut pos_s = pos_disp;
                    let elapsed = format::mmss(pos_s as i64);
                    let remain = format::mmss((dur_s - pos_s).max(0.0) as i64);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = style::S1;
                        ui.add_sized(
                            egui::vec2(32.0, 12.0),
                            egui::Label::new(style::micro_mono(elapsed.clone(), theme.dim)),
                        );

                        let track_w =
                            (col_left_w - 32.0 * 2.0 - style::S1 * 2.0).max(40.0);
                        let (jalur_rect, resp) = ui.allocate_exact_size(
                            egui::vec2(track_w, 12.0),
                            Sense::click_and_drag(),
                        );
                        let frac = (pos_s / dur_s).clamp(0.0, 1.0);

                        let hover = resp.hovered() || resp.dragged();
                        let jalur_h = if hover { 5.0 } else { 3.5 };
                        let jalur = Rect::from_center_size(
                            jalur_rect.center(),
                            egui::vec2(jalur_rect.width(), jalur_h),
                        );
                        ui.painter().rect_filled(
                            jalur,
                            Rounding::same(jalur_h / 2.0),
                            Color32::from_white_alpha(51),
                        );
                        ui.painter().rect_filled(
                            Rect::from_min_size(
                                jalur.min,
                                egui::vec2(jalur.width() * frac, jalur.height()),
                            ),
                            Rounding::same(jalur_h / 2.0),
                            Color32::from_white_alpha(230),
                        );

                        if hover {
                            ui.painter().circle_filled(
                                egui::pos2(jalur.left() + jalur.width() * frac, jalur.center().y),
                                5.0,
                                Color32::WHITE,
                            );
                        }

                        if (resp.dragged() || resp.clicked())
                            && resp.interact_pointer_pos().is_some()
                        {
                            let p = resp.interact_pointer_pos().unwrap();
                            let f = ((p.x - jalur.left()) / jalur.width()).clamp(0.0, 1.0);
                            pos_s = f * dur_s;
                            self.lyric_manual_at = std::time::Instant::now();
                        }
                        if resp.drag_stopped() || resp.clicked() {
                            // Klik langsung DAN lepas drag sama-sama melompat ke
                            // posisi kursor. Ditahan di `seek_at` supaya thumb
                            // tidak memantul balik sebelum SMTC mengonfirmasi.
                            let target_ms = (pos_s * 1000.0) as i64;
                            self.seek_at = Some((target_ms, std::time::Instant::now()));
                            self.media_cmd(crate::infrastructure::music::Cmd::Seek(
                                target_ms,
                            ));
                            self.lyric_manual_at = std::time::Instant::now();
                        }

                        ui.add_sized(
                            egui::vec2(32.0, 12.0),
                            egui::Label::new(style::micro_mono(
                                format!("-{remain}"),
                                theme.dim,
                            )),
                        );
                    });

                    ui.add_space(gap_y);

                    // ---- Transport: 4 tombol lirik/playback + 1 tombol speaker ----
                    // Meluncur mulus antara posisi closed (3-slot simetris) dan open (4-slot padat)
                    let (baris, _) =
                        ui.allocate_exact_size(egui::vec2(col_left_w, PLAY), Sense::hover());

                    // 1. Lyrics (28.0 px) -> kiri tetap
                    let lyr_rect = Rect::from_center_size(
                        egui::pos2(baris.left() + 14.0, baris.center().y),
                        egui::vec2(IB, IB),
                    );
                    let mut lyr_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(lyr_rect)
                            .layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight)),
                    );
                    lyr_ui.spacing_mut().item_spacing.x = 0.0;
                    let lyr_on = self.settings.lyrics_enabled;
                    let lyr_icon = if lyr_on { Icon::LyricsOn } else { Icon::LyricsOff };
                    if widgets::icon_button(&mut lyr_ui, lyr_icon, IB, lyr_on, &theme) {
                        self.settings.lyrics_enabled = !lyr_on;
                        self.touch_settings();
                    }

                    // 2. Trio tengah: Prev, Play, Next meluncur halus
                    let closed_cx = col_left_w * 0.5;
                    let prev_cx = egui::lerp((closed_cx - 48.0)..=66.0, t_lyr);
                    let play_cx = egui::lerp(closed_cx..=122.0, t_lyr);
                    let next_cx = egui::lerp((closed_cx + 48.0)..=178.0, t_lyr);

                    // Prev (28.0 px)
                    let prev_rect = Rect::from_center_size(
                        egui::pos2(baris.left() + prev_cx, baris.center().y),
                        egui::vec2(IB, IB),
                    );
                    let mut prev_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(prev_rect)
                            .layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight)),
                    );
                    prev_ui.spacing_mut().item_spacing.x = 0.0;
                    if widgets::icon_button(&mut prev_ui, Icon::Prev, IB, false, &theme) {
                        self.media_cmd(crate::infrastructure::music::Cmd::Prev);
                    }

                    // Play (36.0 px)
                    let play_rect = Rect::from_center_size(
                        egui::pos2(baris.left() + play_cx, baris.center().y),
                        egui::vec2(PLAY, PLAY),
                    );
                    let mut play_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(play_rect)
                            .layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight)),
                    );
                    play_ui.spacing_mut().item_spacing.x = 0.0;
                    let play_icon = if self.media.playing {
                        Icon::Pause
                    } else {
                        Icon::Play
                    };
                    if widgets::play_button(&mut play_ui, play_icon, PLAY, &theme) {
                        self.media_cmd(crate::infrastructure::music::Cmd::PlayPause);
                    }

                    // Next (28.0 px)
                    let next_rect = Rect::from_center_size(
                        egui::pos2(baris.left() + next_cx, baris.center().y),
                        egui::vec2(IB, IB),
                    );
                    let mut next_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(next_rect)
                            .layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight)),
                    );
                    next_ui.spacing_mut().item_spacing.x = 0.0;
                    if widgets::icon_button(&mut next_ui, Icon::Next, IB, false, &theme) {
                        self.media_cmd(crate::infrastructure::music::Cmd::Next);
                    }

                    // 3. Speaker (28.0 px) - memudar halus saat kolom lirik dibuka
                    let speaker_alpha = (1.0 - t_lyr * 2.5).clamp(0.0, 1.0);
                    if speaker_alpha > 0.01 {
                        let spk_rect = Rect::from_center_size(
                            egui::pos2(baris.left() + col_left_w - 14.0, baris.center().y),
                            egui::vec2(IB, IB),
                        );
                        let mut spk_ui = ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(spk_rect)
                                .layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight)),
                        );
                        spk_ui.spacing_mut().item_spacing.x = 0.0;
                        let mut spk_theme = theme;
                        spk_theme.ink = Color32::from_rgba_unmultiplied(
                            theme.ink.r(),
                            theme.ink.g(),
                            theme.ink.b(),
                            ((theme.ink.a() as f32) * speaker_alpha) as u8,
                        );
                        spk_theme.surface = Color32::from_rgba_unmultiplied(
                            theme.surface.r(),
                            theme.surface.g(),
                            theme.surface.b(),
                            ((theme.surface.a() as f32) * speaker_alpha) as u8,
                        );
                        if widgets::icon_button(&mut spk_ui, Icon::Speaker, IB, false, &spk_theme) {
                            crate::infrastructure::os::toggle_volume_mute();
                        }
                    }
                },
            );

            // ---- Pemisah tipis ----
            if col_right_w > 1.0 && t_lyr > 0.01 {
                ui.add_space(gap * 0.5);
                let div_x = ui.cursor().min.x;
                let stroke_alpha = ((18.0 * t_lyr) as u8).clamp(0, 255);
                ui.painter().line_segment(
                    [
                        egui::pos2(div_x, ui.cursor().min.y),
                        egui::pos2(div_x, ui.cursor().min.y + body_h),
                    ],
                    Stroke::new(1.0, Color32::from_white_alpha(stroke_alpha)),
                );
                ui.add_space(gap * 0.5);
            }

            // ---- Kanan: lirik tersinkron ----
            if col_right_w > 10.0 && t_lyr > 0.01 {
                let lyrics_area = Rect::from_min_size(ui.cursor().min, egui::vec2(col_right_w, body_h));
                let pointer_in_lyrics = ui.rect_contains_pointer(lyrics_area);
                let scrolled_wheel = pointer_in_lyrics && ui.input(|i| i.raw_scroll_delta.y.abs() > 0.1);
                let dragged_lyrics = pointer_in_lyrics && ui.input(|i| i.pointer.is_decidedly_dragging());
                if scrolled_wheel || dragged_lyrics {
                    self.lyric_manual_at = std::time::Instant::now();
                }

                let mut lyrics_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(lyrics_area)
                        .layout(egui::Layout::top_down(egui::Align::LEFT)),
                );
                lyrics_ui.set_clip_rect(lyrics_area.intersect(ui.clip_rect()));

                if self.lyrics.is_empty() {
                    lyrics_ui.vertical_centered(|ui| {
                        ui.add_space(body_h * 0.28);
                        let (icon, msg) = match self.lyrics_status {
                            LyricsStatus::Searching => {
                                (Icon::Sparkles, "Searching lyrics…")
                            }
                            LyricsStatus::NotFound => {
                                (Icon::Music, "No lyrics for this track")
                            }
                            _ => (Icon::Music, "Start a track to see lyrics"),
                        };
                        let (g, _) = ui.allocate_exact_size(
                            egui::vec2(22.0, 22.0),
                            Sense::hover(),
                        );
                        let faint_alpha = ((theme.faint.a() as f32) * t_lyr) as u8;
                        let faint_color = Color32::from_rgba_unmultiplied(
                            theme.faint.r(),
                            theme.faint.g(),
                            theme.faint.b(),
                            faint_alpha,
                        );
                        widgets::glyph(ui.painter(), icon, g, faint_color, 1.4);
                        ui.add_space(style::S2);
                        ui.label(style::text(msg, faint_color));
                    });
                } else {
                    let mut scroll_area = egui::ScrollArea::vertical()
                        .id_salt("page_lyrics_scroll")
                        .max_height(body_h)
                        .auto_shrink([false, false])
                        .drag_to_scroll(true)
                        .scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                        );

                    if auto_scroll {
                        scroll_area = scroll_area.vertical_scroll_offset(self.lyric_scroll_pos);
                    }

                    // Pusat tiap baris (indeks-lokal) untuk interpolasi posisi spring,
                    // dihitung dari tinggi slot GEOMETRI (selalu LYRIC_SIZE_BIG) di
                    // luar loop menggambar supaya invarian terhadap indeks aktif.
                    let mut row_center: Vec<f32> = Vec::with_capacity(self.lyrics.len());

                    let scroll_output = scroll_area.show(&mut lyrics_ui, |ui| {
                        ui.set_width(col_right_w);
                        ui.spacing_mut().item_spacing.y = LYRIC_ROW_GAP;

                        // Jarak galley slot tiap baris, diukur pada ukuran BESAR yang
                        // konstan — inilah yang membuat tinggi kolom tidak bergantung
                        // pada baris mana yang aktif.
                        let mut slot_galley_h: Vec<f32> = Vec::with_capacity(self.lyrics.len());

                        for (idx, line) in self.lyrics.iter().enumerate() {
                            // Hierarki baris dibawa ALPHA, bukan typeface dan bukan
                            // ukuran: dulu baris aktif memakai Plus Jakarta Bold
                            // sehingga terlihat milik aplikasi lain, lalu diganti
                            // animasi ukuran yang justru mengubah tinggi kolom.
                            // Semua baris kini Skynight dengan ukuran TETAP
                            // (fonts.rs: Proportional dimulai dari "skynight").
                            // Alpha murni nilai paint — tidak bisa mengganggu geometri.
                            let alpha = match aktif {
                                None => if self.is_lyrics_synced() { 110.0 } else { 190.0 },
                                Some(_) => lyric_alpha((idx as f32 - self.lyric_anim_pos.pos).abs()),
                            };

                            // Ukuran huruf baris ini sebagai fungsi jarak dari posisi
                            // aktif (kontinu). Baris tetangga/selebihnya = 13.5 px.
                            let d = (idx as f32 - self.lyric_anim_pos.pos).abs();
                            let size_teks = if self.is_lyrics_synced() {
                                lyric_size(d)
                            } else {
                                LYRIC_SIZE
                            };
                            let font_id = egui::FontId::proportional(size_teks);

                            let item_alpha = (alpha * t_lyr).clamp(0.0, 255.0);
                            let warna = Color32::from_rgba_unmultiplied(
                                theme.ink.r(),
                                theme.ink.g(),
                                theme.ink.b(),
                                item_alpha.round() as u8,
                            );

                            let wrap_w = (col_right_w - style::S2).max(10.0);

                            // GEOMETRI: diukur SELALU di LYRIC_SIZE_BIG. Tinggi slot
                            // tidak boleh menurun saat ukuran naik, jadi ini batas atas
                            // yang selalu cukup untuk teks 13.5..16 px.
                            let mut job_geom = egui::text::LayoutJob::default();
                            job_geom.wrap.max_width = wrap_w;
                            job_geom.append(
                                &line.text,
                                0.0,
                                egui::TextFormat {
                                    font_id: egui::FontId::proportional(LYRIC_SIZE_BIG),
                                    color: egui::Color32::PLACEHOLDER,
                                    ..Default::default()
                                },
                            );
                            let galley_geom = ui.painter().layout_job(job_geom);
                            slot_galley_h.push(galley_geom.size().y);
                            let slot_h = lyric_slot_h(galley_geom.size().y);

                            // TEKS: digambar di ukuran baris itu sendiri, dipusatkan di
                            // slot. Galley ini di-cache egui; baris jauh konstan 13.5 px
                            // sehingga cache-hit. Wrapping alami ke newline, bukan ellipsis.
                            let mut job = egui::text::LayoutJob::default();
                            job.wrap.max_width = wrap_w;
                            job.append(
                                &line.text,
                                0.0,
                                egui::TextFormat {
                                    font_id,
                                    color: warna,
                                    ..Default::default()
                                },
                            );

                            let galley = ui.painter().layout_job(job);

                            let (row, _) = ui.allocate_exact_size(
                                egui::vec2(col_right_w, slot_h),
                                Sense::hover(),
                            );
                            // Pusatkan teks vertikal di slot: di 13.5 px teks lebih
                            // pendek dari slot 16 px, jadi ada sedikit celah atas-bawah.
                            // Itu harga jujur dari reservasi tinggi konstan ini — kolom
                            // lirik jadi lebih longgar beberapa px per baris.
                            let ty = row.top() + ((slot_h - galley.size().y) * 0.5).max(0.0);
                            ui.painter().galley(
                                egui::pos2(row.left(), ty),
                                galley,
                                warna,
                            );
                            let line_resp = ui.interact(
                                row,
                                Id::new("p_lyric_click").with(idx),
                                Sense::click(),
                            );
                            if line_resp.clicked() && line.time >= 0.0 {
                                seek = Some(noktah_core::lrc::seek_target_ms(
                                    line.time,
                                    self.settings.lyric_offset_ms,
                                ));
                                // Klik pada baris lirik: langsung sinkron dan auto-follow baris tersebut
                                self.lyric_manual_at = std::time::Instant::now() - std::time::Duration::from_secs(10);
                            }
                        }

                        // Pusat baris dari tinggi galley slot (model murni bersama tes).
                        row_center = lyric_row_centers(&slot_galley_h);

                        ui.add_space(body_h * 0.5);
                    });

                    // Target gulir diinterpolasi dari posisi spring (indeks pecahan),
                    // bukan dari indeks aktif bulat. Dulu target melompat 35.5 px
                    // hampir seluruhnya pada frame pergantian (kecepatan 0,60,0,0,60),
                    // jadi easing mengejar fungsi tangga. Kini target bergerak mulus
                    // sejak frame 0 sehingga easing `1 - exp(-8.5*dt)` yang ada
                    // benar-benar meredam ramp, bukan lompatan.
                    let n = row_center.len();
                    if n > 0 && self.is_lyrics_synced() && aktif.is_some() {
                        let p = self.lyric_anim_pos.pos.clamp(0.0, (n - 1) as f32);
                        let i0 = p.floor() as usize;
                        let i1 = (i0 + 1).min(n - 1);
                        let f = p - i0 as f32;
                        let cy = egui::lerp(row_center[i0]..=row_center[i1], f);
                        let target = (cy - body_h * 0.5).max(0.0);
                        if (self.lyric_target_scroll - target).abs() > 0.5 {
                            self.lyric_target_scroll = target;
                            if auto_scroll {
                                ui.ctx().request_repaint();
                            }
                        }
                    }

                    if !auto_scroll {
                        if (scroll_output.state.offset.y - self.lyric_scroll_pos).abs() > 0.5 {
                            self.lyric_manual_at = std::time::Instant::now();
                        }
                        self.lyric_scroll_pos = scroll_output.state.offset.y;
                    }

                    // Gradient fade mask lembut (12px) di atas & bawah kolom lirik
                    const MASK_H: f32 = 12.0;
                    let mask_a = ((theme.panel.a().max(220) as f32) * t_lyr) as u8;
                    let mask_color = Color32::from_rgba_unmultiplied(
                        0,
                        0,
                        0,
                        mask_a,
                    );

                    let top_rect = Rect::from_min_size(lyrics_area.min, egui::vec2(col_right_w, MASK_H));
                    let mut top_mesh = egui::Mesh::default();
                    top_mesh.colored_vertex(top_rect.left_top(), mask_color);
                    top_mesh.colored_vertex(top_rect.right_top(), mask_color);
                    top_mesh.colored_vertex(top_rect.right_bottom(), Color32::TRANSPARENT);
                    top_mesh.colored_vertex(top_rect.left_bottom(), Color32::TRANSPARENT);
                    top_mesh.add_triangle(0, 1, 2);
                    top_mesh.add_triangle(0, 2, 3);
                    ui.painter().add(egui::Shape::mesh(top_mesh));

                    let bot_rect = Rect::from_min_size(
                        egui::pos2(lyrics_area.left(), lyrics_area.bottom() - MASK_H),
                        egui::vec2(col_right_w, MASK_H),
                    );
                    let mut bot_mesh = egui::Mesh::default();
                    bot_mesh.colored_vertex(bot_rect.left_top(), Color32::TRANSPARENT);
                    bot_mesh.colored_vertex(bot_rect.right_top(), Color32::TRANSPARENT);
                    bot_mesh.colored_vertex(bot_rect.right_bottom(), mask_color);
                    bot_mesh.colored_vertex(bot_rect.left_bottom(), mask_color);
                    bot_mesh.add_triangle(0, 1, 2);
                    bot_mesh.add_triangle(0, 2, 3);
                    ui.painter().add(egui::Shape::mesh(bot_mesh));
                }
            }
        });

        if let Some(target) = seek {
            self.lyric_manual_at = std::time::Instant::now() - std::time::Duration::from_secs(10);
            self.request_seek(target);
        }
    }
}
