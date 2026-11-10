//! Siklus hidup per-frame dan penjadwalan aplikasi Noktah (`eframe::App`).
//!
//! Mengatur:
//! - Pembaruan fisika animasi spring setiap frame (morph panel, pill, minimal line)
//! - Deteksi hover intent (kursor berhenti sejenak di atas notch)
//! - Sinkronisasi region jendela Windows dan mode klik-tembus
//! - Pemanggilan pipeline polling worker dan pemutakhiran timer (daya, Pomodoro)
//! - Penjadwalan repaint pintar (`next_wake`) agar hemat daya saat diam

use crate::app::{Noktah, NotchPage};
use crate::presentation::layout;
use crate::presentation::spring::*;
use crate::presentation::theme::Theme;
use chrono::Timelike;
use eframe::egui;
use noktah_core::model::PomodoroMode;
use noktah_core::reminders;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

impl Noktah {
    /// Pegas posisi baris lirik sedang bergerak DI HALAMAN yang menggambarnya?
    ///
    /// `lyric_anim_pos` hanya dimajukan `draw_page_music`. Di halaman lain atau
    /// saat panel tertutup, posisinya tidak dibaca siapa pun — jadi ia TIDAK
    /// boleh dihitung "bergerak". Kalau dihitung, ia terdampar di posisi antara
    /// (`settled()` tak pernah benar) dan aplikasi meminta frame 60 fps terus
    /// tanpa ada yang bergerak. Dua pemakai di berkas ini harus memakai definisi
    /// yang sama supaya tidak bisa melenceng.
    ///
    /// Ambangnya `open_h.pos > 0.5`, bukan ambang gambar `panel_alpha > 0.01`
    /// (`panel_alpha` baru positif di `open_h.pos > 0.2065`). Pita di antaranya
    /// hanya membuat pegas dimajukan tanpa dihitung bergerak — arahnya aman,
    /// karena `draw_page_music` meminta repaint sendiri saat pegas belum diam.
    pub(crate) fn lyric_line_animating(&self) -> bool {
        self.open_h.pos > 0.5
            && self.current_page == NotchPage::Music
            && !self.lyric_anim_pos.settled()
    }

    /// Selisih waktu sampai repaint berikutnya yang berguna.
    ///
    /// Selama bentuk masih bergerak, hasilnya nol: frame berikutnya diminta
    /// secepat vsync, bukan lewat timer.
    pub(crate) fn next_wake(&self, now_ms: i64) -> Duration {
        if !self.open.settled()
            || !self.open_h.settled()
            || !self.pill.settled()
            || !self.minimal_mix.settled()
            || !self.scrubber_expand.settled()
            || !self.dock_open.settled()
            || !self.lyrics_anim.settled()
            || self.lyric_line_animating()
        {
            return Duration::ZERO;
        }
        // Transisi antar-lagu sedang menahan garis tipis: bangun sesering
        // mungkin supaya begitu lirik datang, bentuknya langsung membesar.
        if self.transisi_at.is_some() {
            return Duration::from_millis(50);
        }
        // Maskot idle atau agent aktif butuh ~30 fps untuk breathing & gaze tracking
        if !self.media.active || self.agent.active {
            if let noktah_core::model::AgentStatus::AwaitingApproval(_) = self.agent.status {
                return Duration::from_millis(16); // 60fps untuk bouncy mascot hop
            }
            if let noktah_core::model::AgentStatus::ToolRunning { .. } = self.agent.status {
                return Duration::from_millis(32); // 30fps untuk animasi komet & timer
            }
            if self.mascot.is_sleeping {
                return Duration::from_millis(500);
            }
            return Duration::from_millis(32); // 30fps untuk breathing & gaze tracking mascot
        }
        if self.mascot.dizzy_timer > 0.0 || !self.mascot.squish.settled() {
            return Duration::from_millis(16);
        }
        if self.open.pos > 0.5 {
            // Pendar vokal lirik saat baru aktif (Mochy-Music AnimatedWordV2)
            if self.current_page == NotchPage::Music
                && self.lyric_line_active_at.elapsed() < Duration::from_millis(800)
            {
                return Duration::from_millis(16);
            }
            if self.media.active && self.media.playing {
                return Duration::from_millis(80);
            }
            return Duration::from_millis(500);
        }

        // Pill yang sedang memutar musik dan menampilkan equalizer ATAU aura
        // progres: denyut keduanya butuh ~30 fps. Tanpa syarat ini, pill yang
        // sedang putar lagu tapi equalizernya dimatikan jatuh ke cabang
        // "pergantian menit" dan denyutnya membeku.
        if self.media.active && self.media.playing && (self.settings.mini_wave || self.settings.mini_progress) {
            return Duration::from_millis(32);
        }

        // Pill yang sedang menggulir lirik butuh ~60 fps saat playing; selebihnya cukup
        // bangun di pergantian menit.
        if self.media.active
            && self.media.playing
            && (self.settings.mini_lyrics || self.settings.pill_show_lyrics)
            && !self.lyrics.is_empty()
        {
            return Duration::from_millis(16);
        }

        // Saat tertutup: bangun di pergantian menit, atau tepat saat pengingat
        // berikutnya jatuh tempo.
        let ke_menit = 60 - chrono::Local::now().second() as u64;
        let mut tunggu = Duration::from_secs(ke_menit.max(1));

        if let Some(next) = reminders::next_due(&self.todos, now_ms) {
            if next.in_ms > 0 {
                let d = Duration::from_millis(next.in_ms as u64);
                if d < tunggu {
                    tunggu = d;
                }
            } else {
                tunggu = Duration::from_millis(16);
            }
        }

        // Bangun lebih awal jika belum di-trim saat pertama kali masuk kondisi tertutup stabil
        if !self.trimmed && self.open.pos < 0.01 && !self.show_settings {
            tunggu = tunggu.min(Duration::from_millis(3200));
        }

        tunggu
    }
}

impl eframe::App for Noktah {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.show_settings {
            let mut close_settings = false;
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("noktah_settings_window"),
                egui::ViewportBuilder::default()
                    .with_title("Noktah Settings")
                    .with_inner_size([740.0, 520.0])
                    .with_min_inner_size([640.0, 460.0])
                    .with_decorations(true)
                    .with_resizable(true)
                    // Jendela setelan selalu di atas: begitu dibuka ia langsung
                    // terlihat, tidak tertutup jendela lain atau notch sendiri.
                    .with_always_on_top(),
                |ctx, _class| {
                    if self.draw_settings_window(ctx) {
                        close_settings = true;
                    }
                },
            );
            if close_settings {
                self.show_settings = false;
            }
        }
        if self.frames == 0 {
            let mut visuals = egui::Visuals::dark();
            // Background warna gelap elegan untuk input TextEdit (#1C1D22) agar batas kotaknya jelas di atas latar hitam
            visuals.extreme_bg_color = egui::Color32::from_rgb(28, 29, 34);
            // Tombol dan widget bawaan (button, textedit, slider)
            visuals.widgets.inactive.bg_fill =
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 24);
            visuals.widgets.inactive.fg_stroke =
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(240, 242, 248));
            visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
            visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);

            visuals.widgets.hovered.bg_fill =
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 42);
            visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
            visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
            visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);

            visuals.widgets.active.bg_fill =
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 56);
            visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
            visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
            visuals.widgets.active.rounding = egui::Rounding::same(8.0);

            visuals.widgets.open.bg_stroke = egui::Stroke::NONE;
            visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
            visuals.widgets.noninteractive.fg_stroke =
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(185, 190, 204));
            visuals.window_stroke = egui::Stroke::NONE;

            ctx.set_visuals(visuals);
            crate::presentation::fonts::install(ctx);
        }

        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32().clamp(0.0, 1.0 / 20.0);
        self.last_frame = now;
        self.frames += 1;

        // Periksa status daya / baterai setiap 4 detik
        if self.last_power_check.elapsed() >= Duration::from_secs(4) {
            self.last_power_check = now;
            let (charging, pct) = crate::infrastructure::os::power_status();
            self.is_charging = charging;
            self.battery_pct = pct;
        }

        // Jalankan ticking Pomodoro setiap 1 detik jika aktif
        if self.last_pomodoro_tick.elapsed() >= Duration::from_secs(1) {
            self.last_pomodoro_tick = now;
            if self.pomodoro.tick() {
                let msg = match self.pomodoro.mode {
                    PomodoroMode::Focus => "Sesi fokus selesai! Waktunya istirahat.",
                    _ => "Waktu istirahat selesai! Siap untuk kembali fokus?",
                };
                self.banner = Some((msg.to_string(), Instant::now()));
                self.save_pomodoro();
            }
        }

        // --- morph ------------------------------------------------------
        let screen = ctx.screen_rect();

        // Garis tipis dipakai saat setelan "notch minimal" menyala, ATAU saat
        // lagu sedang berganti. Yang kedua bersifat sementara: menyusut dulu,
        // lalu membesar lagi begitu lirik lagu baru siap.
        let transisi = self.transisi_at.is_some();
        let minimal_target = if self.settings.minimal_notch || transisi {
            1.0
        } else {
            0.0
        };
        self.minimal_mix.target = minimal_target;
        self.minimal_mix.set_spec(if minimal_target > 0.5 {
            MORPH_MINIMAL_IN
        } else {
            MORPH_MINIMAL_OUT
        });
        if self.reduce_motion {
            self.minimal_mix.snap();
        } else {
            self.minimal_mix.step(dt);
        }
        let mix = self.minimal_mix.pos;

        // Uji hover memakai bentuk frame sebelumnya. Ini disengaja: kalau uji
        // hover memakai bentuk yang baru saja digerakkan oleh hover itu
        // sendiri, hasilnya bisa berayun di tepi (masuk → membesar → kursor
        // kini di dalam → keluar → mengecil → …).
        let (open_w, open_h) = self.target_open_size();
        let target_r = self.target_radius();
        let previous_base = layout::notch_rect(
            self.open.pos,
            self.open_h.pos,
            self.pill.pos,
            mix,
            screen,
            open_w,
            open_h,
        );
        let previous_outer = layout::outer_shell_rect(previous_base, self.dock_open.pos);
        let (cx, cy) = self.hover.get();
        let cursor = egui::pos2(cx, cy);
        let inside = previous_outer.contains(cursor);

        // Satu zona dock mencakup pita tetap di panel dasar dan seluruh menu
        // yang telah tumbuh di bawahnya. Ini membuat perjalanan kursor dari
        // strip ke tab tetap mempertahankan spring dan linger yang sama.
        if layout::dock_hover_zone(previous_base, self.dock_open.pos).contains(cursor) {
            self.dock_hover_since.get_or_insert(now);
            self.dock_linger = now;
        } else {
            self.dock_hover_since = None;
        }

        // Hover intent: panel tidak membuka begitu kursor menyentuh. Ia
        // menunggu kursor diam selama `hover_delay_ms` — jalan kaki yang
        // sekadar lewat di atas notch tidak lagi membukanya.
        if !inside {
            self.suppress_hover = false;
        }
        let can_hover = !self.suppress_hover;

        let mut berubah = false;
        if inside && !self.hovered && can_hover {
            let since = *self.hover_since.get_or_insert(Instant::now());
            let jeda = Duration::from_millis(self.settings.hover_delay_ms as u64);
            if since.elapsed() >= jeda {
                self.hover_since = None;
                berubah = true;
            }
        } else {
            if !inside {
                self.hover_since = None;
            }
            if inside != self.hovered {
                berubah = true;
            }
        }

        // Guard anti-stuck: jika cursor di luar dan tidak ada pending approval,
        // panel wajib menutup kembali ke minimize.
        if !inside && self.pending_permission.is_none() && self.open.target > 0.0 {
            self.open.target = 0.0;
            self.open_h.target = 0.0;
            self.hovered = false;
            self.hover_since = None;
            self.open_h.set_spec(MORPH_CLOSE_H);
            self.open.set_spec(MORPH_CLOSE_W);
            berubah = false;
        }

        if berubah {
            self.hovered = inside;
            let target = if inside { 1.0 } else { 0.0 };
            self.open.target = target;
            self.open_h.target = target;
            // Buka: lebar dulu, tinggi menyusul (terasa melar).
            // Tutup: tinggi dulu, lebar menyusul (terasa mengempis).
            if inside {
                self.open.set_spec(MORPH_OPEN_W);
                self.open_h.set_spec(MORPH_OPEN_H);
            } else {
                self.open_h.set_spec(MORPH_CLOSE_H);
                self.open.set_spec(MORPH_CLOSE_W);
            }
        }
        self.hover
            .expanded
            .store(self.open.target > 0.5, Ordering::Relaxed);

        // Saat idle, aplikasi tidur sampai 500 ms. Tanpa ini, frame pertama
        // animasi sudah melompat 50 ms ke depan — bentuknya "meletus" alih-alih
        // mulai dari diam.
        let anim_dt = if berubah { 0.0 } else { dt };

        // Lebar pill ikut dianimasikan: isinya berubah saat lagu mulai/berhenti,
        // dan lompatan 193 → 489 px itu yang paling terasa kasar.
        //
        // Sneak peek: selama masa intip setelah lagu berganti, pill ditahan
        // pada lebar "sedang memutar dengan judul" — judul + artis lagu baru
        // terbaca, lalu menyusut sendiri.
        let peek = self.peek_until.map(|t| Instant::now() < t).unwrap_or(false);
        self.pill.target = if peek {
            self.settings.mini_width(true, true)
        } else if self.media.active {
            self.settings.mini_width(true, self.has_lyrics())
        } else if self.agent.active && self.settings.mini_agent {
            240.0
        } else {
            180.0
        };

        // Dock terbuka selama kursor berada di daerahnya, dengan jeda 250 ms
        // setelah kursor terakhir terlihat. Jeda itu yang membuat kursor bisa
        // berpindah dari strip ke ikon tanpa dock menutup di tengah jalan.
        //
        // Saat panel tertutup, dock tidak relevan — targetnya 0.
        let panel_open = self.open_h.pos > 0.5;
        let dock_hover =
            self.dock_hover_since.is_some() || self.dock_linger.elapsed() < layout::DOCK_LINGER;
        self.dock_open.target = if panel_open && dock_hover { 1.0 } else { 0.0 };
        self.lyrics_anim.target = if self.settings.lyrics_enabled { 1.0 } else { 0.0 };

        // Pegas posisi baris lirik HANYA dimajukan `draw_page_music` — sengaja
        // hanya di satu tempat, supaya tidak mungkin maju dua kali dalam satu
        // frame. Karena itu ia tidak boleh dihitung "bergerak" di halaman lain:
        // lihat `lyric_line_animating`.
        let music_lirik_bergerak = self.lyric_line_animating();

        if self.reduce_motion {
            // Langsung ke tujuan, tanpa pegas.
            self.open.snap();
            self.open_h.snap();
            self.pill.snap();
            self.scrubber_expand.snap();
            self.dock_open.snap();
            self.lyrics_anim.snap();
        } else {
            self.open.step(anim_dt);
            self.open_h.step(anim_dt);
            self.pill.step(anim_dt);
            self.scrubber_expand.step(anim_dt);
            self.dock_open.step(anim_dt);
            self.lyrics_anim.step(anim_dt);
        }

        // Bentuk yang BENAR-BENAR digambar frame ini. Region jendela dan uji
        // klik memakai angka ini, bukan bentuk tujuan — kalau memakai tujuan,
        // panel terpotong di tengah animasi.
        let current_base = layout::notch_rect(
            self.open.pos,
            self.open_h.pos,
            self.pill.pos,
            mix,
            screen,
            open_w,
            open_h,
        );
        let current_outer = layout::outer_shell_rect(current_base, self.dock_open.pos);
        self.hover
            .minimal_mix
            .store(mix.to_bits(), Ordering::Relaxed);
        self.hover
            .pill_w
            .store(self.pill.pos.to_bits(), Ordering::Relaxed);
        self.hover.ears.store(self.settings.ears, Ordering::Relaxed);
        self.hover.open_w.store(open_w.to_bits(), Ordering::Relaxed);
        self.hover.open_h.store(open_h.to_bits(), Ordering::Relaxed);
        self.hover
            .dock_open
            .store(self.dock_open.pos.to_bits(), Ordering::Relaxed);
        if let Ok(mut cur) = self.hover.cur.lock() {
            *cur = (
                current_outer.left(),
                current_outer.top(),
                current_outer.right(),
                current_outer.bottom(),
            );
        }
        self.hover.cur_r.store(
            layout::radius(self.open_h.pos, mix, target_r).to_bits(),
            Ordering::Relaxed,
        );
        // Region dan klik-tembus dipasang dari sini, bukan dari thread
        // pemantau: bentuk yang menerima klik harus identik dengan bentuk yang
        // baru digambar.
        #[cfg(windows)]
        {
            let scale = self.hover.scale.load(Ordering::Relaxed) as f32 / 96.0;
            let scale = if scale > 0.0 { scale } else { 1.0 };
            crate::infrastructure::platform::sync_region(&self.hover, current_outer, scale);
            // `inside` dihitung dari bentuk frame SEBELUMNYA; pakai bentuk
            // yang baru saja digambar supaya klik pertama tidak tembus.
            let (mx, my) = self.hover.get();
            crate::infrastructure::platform::set_click_through(
                &self.hover,
                !current_outer.contains(egui::pos2(mx, my)),
            );
        }

        // Minta frame berikutnya secepat mungkin selama masih ada yang
        // bergerak. `request_repaint_after(16ms)` hanya menambah timer 16 ms di
        // atas waktu frame yang sudah dipakai — hasilnya 20–33 ms per frame,
        // yaitu 30–50 fps, bukan 60. `request_repaint()` menyerahkan irama ke
        // vsync dan itu yang dipakai di sini.
        let animating = !self.open.settled()
            || !self.open_h.settled()
            || !self.pill.settled()
            || !self.minimal_mix.settled()
            || !self.scrubber_expand.settled()
            || !self.dock_open.settled()
            || music_lirik_bergerak
            // Sneak peek menahan pill melebar; setelah lewat, pill menyusut
            // lagi — itu masih gerak, jadi jangan tidur sebelum selesai.
            || peek
            || (self.mini_lyric_at.elapsed().as_secs_f32() < 0.35 && self.media.active);
        // Pemantau kursor memotong region jendela. Kalau ia tetap menunggu
        // 40 ms, tepi panel yang sedang membesar terpotong 2–3 frame. Selama
        // morph berjalan, ia dipacu lebih rapat.
        self.hover.animating.store(animating, Ordering::Relaxed);

        // Perbarui tema aktif (opsi aksen dinamis dari album art ala Mochy-Music)
        let base_theme = Theme::new(self.settings.accent, self.settings.opacity);
        self.theme =
            if self.settings.dynamic_accent && self.media.active && self.album_accent.is_some() {
                base_theme.with_accent(self.album_accent.unwrap())
            } else {
                base_theme
            };

        // --- data -------------------------------------------------------
        if now.duration_since(self.stats_at) > Duration::from_secs(1) {
            self.sys.tick();
            self.stats_at = now;
        }

        // Visualizer hanya perlu hidup saat equalizer benar-benar terlihat DAN
        // ada lagu yang diputar. Tanpa syarat "sedang diputar", thread ini
        // membaca audio sistem 35 kali per detik selamanya — terukur ~40% satu
        // core saat aplikasi diam, karena tiap pembacaan meminta repaint.
        let equalizer_tampil = self.media.active
            && self.media.playing
            && self.open.pos < 0.35
            && self.minimal_mix.pos < 0.5
            && self.settings.mini_wave;
        self.bands.set_active(equalizer_tampil);

        while let Ok(bands) = self.bands.rx.try_recv() {
            self.levels = bands;
        }

        let now_ms = chrono::Local::now().timestamp_millis();
        self.poll_media(ctx);
        self.poll_ai();
        self.poll_weather();
        self.poll_quota();
        self.poll_ipc();

        // Pembaruan fisika & ekspresi mascot Noki
        let (mx, my) = self.hover.get();
        let cursor_pos = Some(egui::pos2(mx, my));
        let is_minimized = self.open.pos < 0.2;
        let mascot_center = if is_minimized {
            egui::pos2(current_base.left() + 32.0, current_base.center().y)
        } else {
            current_base.center()
        };
        let anim_t = self.anim.elapsed().as_secs_f32();
        self.mascot.update(
            dt,
            anim_t,
            cursor_pos,
            mascot_center,
            &self.agent.status,
            true,
        );

        // Posisi lagu dihaluskan di antara dua pembacaan SMTC (500 ms),
        // supaya lirik tidak melompat tiap setengah detik.
        if self.media.playing {
            self.media.position_ms += (dt * 1000.0) as i64;
            if self.media.duration_ms > 0 {
                self.media.position_ms = self.media.position_ms.min(self.media.duration_ms);
            }
        }

        self.check_reminders(now_ms);
        self.flush_saves(now);

        // Mode Penuh: tangkap layar sekali saat panel mulai membuka, selagi
        // jendela masih berupa pill kecil sehingga yang tertangkap hampir
        // seluruhnya isi layar.
        if self.settings.glass_effect.uses_blur() && self.open.target > 0.5 && self.open.pos < 0.1 {
            if let Some((x, y, w, h)) = crate::infrastructure::platform::window_rect() {
                if let Some(gambar) = crate::infrastructure::blur::capture(x, y, w, h) {
                    self.glass_tex =
                        Some(ctx.load_texture("glass-bg", gambar, egui::TextureOptions::LINEAR));
                }
            }
        }

        // --- gambar -----------------------------------------------------
        self.draw_shell(ctx, current_outer);
        let panel_alpha = ((self.open_h.pos - 0.20) / 0.65).clamp(0.0, 1.0);
        if panel_alpha > 0.01 {
            self.draw_panel(ctx, current_base, current_outer, panel_alpha);
        }

        // --- hemat memori saat bentuk tertutup dan stabil (trim working set) -
        let is_idle_closed = !animating
            && self.open.pos < 0.01
            && !self.show_settings;

        if is_idle_closed {
            let since = *self.idle_since.get_or_insert(now);
            if since.elapsed() >= Duration::from_millis(3000) && !self.trimmed {
                crate::infrastructure::os::trim_working_set();
                self.trimmed = true;
            }
        } else {
            self.idle_since = None;
            self.trimmed = false;
        }

        if animating {
            // Tanpa timer: egui/winit langsung menyusun frame berikutnya pada
            // vsync, jadi 60/120/144 Hz semua terpakai penuh.
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(self.next_wake(now_ms));
        }
    }
}
