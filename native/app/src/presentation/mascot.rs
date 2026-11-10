//! Mascot Engine ("Noki" - Procedural Blob Mascot).
//!
//! Terinspirasi dari karakter Mochi di Coucou:
//! - Tubuh superellipse/squircle parametrik (n ≈ 2.7) yang kenyal dan elastis
//! - Mata 3D yang diproyeksikan pada elipsoid virtual (gaze tracking mengikuti kursor)
//! - Animasi bernapas (sinusoidal volume preservation) & kedip mata alami
//! - Reaksi fisik pegas: squish saat diklik, bouncing saat menunggu izin
//! - Ragam ekspresi dan partikel berdasarkan state hook Claude Code:
//!   * Idle: bernapas tenang, menatap kursor, sesekali berkedip
//!   * Thinking / Prompt: tubuh ungu violet (#8B5CF6), mata melirik ke kanan-atas
//!   * Working / Tool: tubuh zamrud-sian (#10B981), getaran kinetik tekun
//!   * Awaiting Approval: tubuh emas-amber (#F59E0B), mata membelalak lebar, melompat ritmis
//!   * Finished: tubuh toska cerah, mata tersenyum (^ ^), pipi merona merah muda, partikel bintang
//!   * Error: tubuh merah (#EF4444), menggelengkan kepala
//!   * Sleeping (inactivity > 45s): mata terpejam tenang (- -), napas lambat, partikel "z"

use crate::presentation::spring::Spring;
use eframe::egui::{self, Color32, Painter, Pos2, Rect, Rounding, Stroke, Vec2};
use noktah_core::model::AgentStatus;

/// Gaya mata mascot berdasarkan mood dan state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EyeStyle {
    Pill,
    Wide,
    Happy,
    Thinking,
    Closed,
    Dizzy,
}

/// Partikel visual yang melayang di sekitar mascot (bintang selebrasi, z tidur, keringat).
#[derive(Clone, Debug)]
pub struct MascotParticle {
    pub pos: Pos2,
    pub vel: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub kind: ParticleKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleKind {
    Star,
    SleepZ,
    Spark,
}

/// State animasi dan fisika mascot Noktah.
#[derive(Clone, Debug)]
pub struct MascotState {
    pub eye_yaw: f32,
    pub eye_pitch: f32,
    pub current_yaw: f32,
    pub current_pitch: f32,

    pub blink: f32,
    pub blink_timer: f32,
    pub next_blink_sec: f32,

    pub squish: Spring,
    pub bounce_y: f32,
    pub hop_vel: f32,

    pub click_count: u32,
    pub last_click_sec: f32,
    pub dizzy_timer: f32,

    pub particles: Vec<MascotParticle>,
    pub was_active: bool,
    pub last_status_tag: &'static str,
    pub is_happy: bool,
    pub is_surprised: bool,
    pub is_sleeping: bool,
    pub last_cursor: Option<Pos2>,
    pub last_move_sec: f32,
}

impl Default for MascotState {
    fn default() -> Self {
        Self::new()
    }
}

impl MascotState {
    pub fn new() -> Self {
        Self {
            eye_yaw: 0.0,
            eye_pitch: 0.0,
            current_yaw: 0.0,
            current_pitch: 0.0,
            blink: 1.0,
            blink_timer: 0.0,
            next_blink_sec: 2.8,
            squish: Spring::with(1.0, 180.0, 14.0),
            bounce_y: 0.0,
            hop_vel: 0.0,
            click_count: 0,
            last_click_sec: 0.0,
            dizzy_timer: 0.0,
            particles: Vec::new(),
            was_active: false,
            last_status_tag: "idle",
            is_happy: false,
            is_surprised: false,
            is_sleeping: false,
            last_cursor: None,
            last_move_sec: 0.0,
        }
    }

    /// Memicu efek kenyal/squish saat mascot diklik.
    pub fn trigger_squish(&mut self, now_sec: f32) {
        self.squish.pos = 0.68;
        self.squish.target = 1.0;

        if now_sec - self.last_click_sec < 1.4 {
            self.click_count += 1;
            if self.click_count >= 3 {
                self.dizzy_timer = 3.2;
                self.click_count = 0;
            }
        } else {
            self.click_count = 1;
        }
        self.last_click_sec = now_sec;
    }

    /// Melangkah satu tick simulasi fisika, gaze tracking, dan ekspresi.
    pub fn update(
        &mut self,
        dt: f32,
        t: f32,
        cursor_pos: Option<Pos2>,
        mascot_center: Pos2,
        status: &AgentStatus,
        active: bool,
    ) {
        let dt_clamped = dt.clamp(0.001, 0.05);

        // 1. Integrasikan spring squish
        self.squish.step(dt_clamped);

        // 2. Timer Dizzy
        if self.dizzy_timer > 0.0 {
            self.dizzy_timer = (self.dizzy_timer - dt_clamped).max(0.0);
        }

        // Deteksi gerakan dan posisi kursor
        if let Some(cursor) = cursor_pos {
            if let Some(last_pos) = self.last_cursor {
                let moved_dist = (cursor - last_pos).length();
                if moved_dist > 1.5 {
                    self.last_move_sec = t;
                    self.is_sleeping = false;
                    let speed = moved_dist / dt_clamped;
                    if speed > 1500.0 {
                        self.is_surprised = true;
                    } else if speed < 500.0 {
                        self.is_surprised = false;
                    }
                }
            } else {
                self.last_move_sec = t;
            }
            self.last_cursor = Some(cursor);

            let dx = cursor.x - mascot_center.x;
            let dy = cursor.y - mascot_center.y;
            let dist = (dx * dx + dy * dy).sqrt();

            // Hover dekat maskot (< 42 px): ekspresi tersenyum ceria (^ ^)
            self.is_happy = dist < 42.0;

            // Inaktivitas kursor > 45 detik: tidur tenang (- -)
            if t - self.last_move_sec > 45.0 {
                self.is_sleeping = true;
                self.is_happy = false;
                self.is_surprised = false;
            }
        }

        // 3. Tentukan Target Gaze & Mood berdasarkan Status Agent
        let (target_yaw, target_pitch) = if self.dizzy_timer > 0.0 {
            ((t * 14.0).sin() * 0.85, (t * 8.0).cos() * 0.4)
        } else if self.is_sleeping {
            // Sleeping / Deep Idle
            (0.0, 0.35)
        } else if let Some(cursor) = cursor_pos {
            // SEMUA MASCOT IKUTI KURSOR:
            // Baik saat idle, prompt, tool berjalan, awaiting approval, maupun selesai!
            let dx = cursor.x - mascot_center.x;
            let dy = cursor.y - mascot_center.y;
            let dist = (dx * dx + dy * dy).sqrt().max(10.0);
            let yaw = (dx / dist).clamp(-0.85, 0.85);
            let pitch = (dy / dist).clamp(-0.75, 0.75);
            (yaw, pitch)
        } else {
            match status {
                AgentStatus::PromptSubmitted { .. } => (0.55, -0.45),
                AgentStatus::ToolRunning { .. } => (((t * 3.5).sin() * 0.35), 0.40),
                AgentStatus::AwaitingApproval(_) => (0.0, -0.15),
                AgentStatus::Completed { .. } => (0.0, -0.30),
                _ => (((t * 0.6).sin() * 0.25), ((t * 0.4).cos() * 0.15)),
            }
        };

        self.eye_yaw = target_yaw;
        self.eye_pitch = target_pitch;

        // Smoothing tatapan mata (lerp)
        let gaze_speed = if status.is_awaiting_approval() { 16.0 } else { 9.0 };
        self.current_yaw += (self.eye_yaw - self.current_yaw) * (dt_clamped * gaze_speed).min(1.0);
        self.current_pitch += (self.eye_pitch - self.current_pitch) * (dt_clamped * gaze_speed).min(1.0);

        // 4. Siklus Kedipan Alami
        if self.dizzy_timer > 0.0 || !active || self.is_sleeping {
            self.blink = if !active || self.is_sleeping { 0.05 } else { 0.9 };
        } else {
            self.blink_timer += dt_clamped;
            if self.blink_timer >= self.next_blink_sec {
                // Kedip selama 160 ms (70ms tutup, 90ms buka)
                let blink_dur = 0.16;
                let progress = (self.blink_timer - self.next_blink_sec) / blink_dur;
                if progress >= 1.0 {
                    self.blink = 1.0;
                    self.blink_timer = 0.0;
                    // Jadwalkan kedipan berikutnya (2.2s .. 4.8s)
                    let pseudo_rand = (t * 11.3).sin().abs();
                    self.next_blink_sec = 2.2 + pseudo_rand * 2.6;
                } else if progress < 0.45 {
                    self.blink = (1.0 - progress / 0.45).clamp(0.06, 1.0);
                } else {
                    self.blink = ((progress - 0.45) / 0.55).clamp(0.06, 1.0);
                }
            } else {
                self.blink = 1.0;
            }
        }

        // 5. Pantulan dan Lompatan (Hop Physics)
        match status {
            AgentStatus::AwaitingApproval(_) => {
                // Bouncing terus menerus saat butuh persetujuan user
                let hop = (t * 6.8).sin().abs() * 7.5;
                self.bounce_y = -hop;
            }
            AgentStatus::Completed { .. } => {
                let hop = ((t * 4.5).sin().max(0.0)) * 6.0;
                self.bounce_y = -hop;
            }
            _ => {
                self.bounce_y = 0.0;
            }
        }

        // 6. Spawn Partikel Selebrasi atau Status
        let current_tag = match status {
            AgentStatus::Completed { .. } => "completed",
            AgentStatus::AwaitingApproval(_) => "approval",
            AgentStatus::ToolRunning { .. } => "tool",
            AgentStatus::PromptSubmitted { .. } => "thinking",
            _ => "idle",
        };

        if current_tag == "completed" && self.last_status_tag != "completed" {
            // Baru saja selesai -> semburkan 6 partikel bintang selebrasi!
            for i in 0..6 {
                let ang = std::f32::consts::TAU * (i as f32) / 6.0;
                let spd = 24.0 + (i as f32 * 5.0);
                self.particles.push(MascotParticle {
                    pos: mascot_center,
                    vel: Vec2::new(ang.cos() * spd, ang.sin() * spd - 20.0),
                    life: 0.0,
                    max_life: 0.9,
                    kind: ParticleKind::Star,
                });
            }
        } else if !active && self.particles.len() < 2 {
            // Partikel tidur "Z" sesekali
            if (t * 1.5).fract() < dt_clamped * 1.5 {
                self.particles.push(MascotParticle {
                    pos: mascot_center + Vec2::new(8.0, -6.0),
                    vel: Vec2::new(4.0, -12.0),
                    life: 0.0,
                    max_life: 1.8,
                    kind: ParticleKind::SleepZ,
                });
            }
        }
        self.last_status_tag = current_tag;
        self.was_active = active;

        // 7. Perbarui partikel
        self.particles.retain_mut(|p| {
            p.life += dt_clamped;
            p.pos += p.vel * dt_clamped;
            p.vel.y += 9.8 * dt_clamped; // gravitasi halus
            p.life < p.max_life
        });
    }

    /// Evaluasi palet warna tubuh dan aksen berdasarkan status agent.
    pub fn get_palette(&self, status: &AgentStatus, active: bool) -> (Color32, Color32, EyeStyle) {
        if self.dizzy_timer > 0.0 {
            (
                Color32::from_rgb(244, 114, 182), // Pink pusing
                Color32::from_rgb(219, 39, 119),
                EyeStyle::Dizzy,
            )
        } else {
            match status {
                AgentStatus::Completed { .. } => (
                    Color32::from_rgb(20, 184, 166), // Toska ceria selesai
                    Color32::from_rgb(45, 212, 191),
                    EyeStyle::Happy,
                ),
                AgentStatus::AwaitingApproval(_) => (
                    Color32::from_rgb(245, 158, 11), // Emas amber butuh izin
                    Color32::from_rgb(251, 191, 36),
                    EyeStyle::Wide,
                ),
                AgentStatus::ToolRunning { .. } => (
                    Color32::from_rgb(16, 185, 129), // Hijau zamrud eksekusi
                    Color32::from_rgb(52, 211, 153),
                    EyeStyle::Pill,
                ),
                AgentStatus::PromptSubmitted { .. } => (
                    Color32::from_rgb(139, 92, 246), // Violet berpikir
                    Color32::from_rgb(192, 132, 252),
                    EyeStyle::Thinking,
                ),
                _ => {
                    if !active {
                        (
                            Color32::from_rgb(100, 116, 139), // Slate tidur
                            Color32::from_rgb(71, 85, 105),
                            EyeStyle::Closed,
                        )
                    } else if self.is_sleeping {
                        (
                            Color32::from_rgb(148, 163, 184), // Slate tidur lembut
                            Color32::from_rgb(100, 116, 139),
                            EyeStyle::Closed,
                        )
                    } else if self.is_happy {
                        (
                            Color32::from_rgb(241, 245, 249),
                            Color32::from_rgb(203, 213, 225),
                            EyeStyle::Happy,
                        )
                    } else if self.is_surprised {
                        (
                            Color32::from_rgb(241, 245, 249),
                            Color32::from_rgb(203, 213, 225),
                            EyeStyle::Wide,
                        )
                    } else {
                        (
                            Color32::from_rgb(241, 245, 249), // Putih susu lembut saat idle
                            Color32::from_rgb(203, 213, 225),
                            EyeStyle::Pill,
                        )
                    }
                }
            }
        }
    }
}

/// Renderer visual untuk Blob Mascot.
pub struct MascotRenderer;

impl MascotRenderer {
    /// Menggambar avatar blob kompak (~18-22 px) untuk baris minimized notch.
    pub fn draw_mini(
        painter: &Painter,
        rect: Rect,
        state: &MascotState,
        status: &AgentStatus,
        active: bool,
        t: f32,
    ) {
        let (body_col, _rim_col, eye_style) = state.get_palette(status, active);
        let center = rect.center() + Vec2::new(0.0, state.bounce_y * 0.45);
        let rx = rect.width() * 0.45;
        let ry = rect.height() * 0.45;

        // Cek kedekatan kursor ke maskot ini secara independen
        let is_near = if let Some(cursor) = state.last_cursor {
            (cursor - center).length() < 36.0
        } else {
            false
        };
        let effective_eye_style = if state.dizzy_timer > 0.0 {
            EyeStyle::Dizzy
        } else if state.is_sleeping {
            EyeStyle::Closed
        } else if is_near {
            EyeStyle::Happy
        } else {
            eye_style
        };

        // Gaze tracking independen terhadap pusat mini mascot ini
        let (mini_yaw, mini_pitch) = if state.dizzy_timer > 0.0 {
            ((t * 14.0).sin() * 0.85, (t * 8.0).cos() * 0.4)
        } else if state.is_sleeping {
            (0.0, 0.35)
        } else if let Some(cursor) = state.last_cursor {
            let dx = cursor.x - center.x;
            let dy = cursor.y - center.y;
            let dist = (dx * dx + dy * dy).sqrt().max(10.0);
            ((dx / dist).clamp(-0.85, 0.85), (dy / dist).clamp(-0.75, 0.75))
        } else {
            (state.current_yaw, state.current_pitch)
        };

        // Superellipse tubuh mini (32 titik untuk efisiensi)
        let points = Self::superellipse_points(center, rx, ry, 32, 0.0);
        painter.add(egui::Shape::convex_polygon(
            points,
            body_col,
            Stroke::new(1.0, Color32::from_white_alpha(45)),
        ));

        // Pipi merona ceria saat tersenyum senang
        if effective_eye_style == EyeStyle::Happy {
            let blush_col = Color32::from_rgba_unmultiplied(251, 113, 133, 160);
            painter.circle_filled(center + Vec2::new(-rx * 0.52, ry * 0.22), 1.6, blush_col);
            painter.circle_filled(center + Vec2::new(rx * 0.52, ry * 0.22), 1.6, blush_col);
        }

        // Mata 3D kompak
        Self::draw_eyes(
            painter,
            center,
            rx,
            ry,
            mini_yaw,
            mini_pitch,
            state.blink,
            effective_eye_style,
            true,
        );

        // Badge kecil di pojok jika butuh izin
        if status.is_awaiting_approval() {
            let badge_pos = rect.right_top() + Vec2::new(-2.0, 2.0);
            painter.circle_filled(badge_pos, 3.5, Color32::from_rgb(239, 68, 68));
            let pulse = (t * 6.0).sin().abs() * 2.5;
            painter.circle_stroke(
                badge_pos,
                3.5 + pulse,
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(239, 68, 68, 140)),
            );
        }
    }

    /// Menggambar mascot interaktif penuh (~48-64 px) di halaman Agent.
    pub fn draw_card(
        painter: &Painter,
        rect: Rect,
        state: &MascotState,
        status: &AgentStatus,
        active: bool,
        t: f32,
    ) {
        let (body_col, rim_col, eye_style) = state.get_palette(status, active);

        // Breathing sinusoidal volume preservation
        let breath = (t * 2.0).sin() * 0.032;
        let scale_y = (1.0 + breath) * state.squish.pos;
        let scale_x = (1.0 - breath * 0.52) / state.squish.pos.max(0.4);

        let rot = if state.dizzy_timer > 0.0 {
            (t * 12.0).sin() * 0.12
        } else {
            0.0
        };

        let jitter_x = if matches!(status, AgentStatus::ToolRunning { .. }) {
            (t * 28.0).sin() * 0.6
        } else {
            0.0
        };

        let center = rect.center() + Vec2::new(jitter_x, state.bounce_y);
        let rx = rect.width() * 0.44 * scale_x;
        let ry = rect.height() * 0.44 * scale_y;

        // 1. Bayangan di bawah tubuh
        let shadow_y = rect.bottom() - 3.0;
        let shadow_w = rx * 1.35 * (1.0 - (state.bounce_y.abs() / 20.0).min(0.5));
        painter.rect_filled(
            Rect::from_center_size(Pos2::new(center.x, shadow_y), Vec2::new(shadow_w * 2.0, 6.0)),
            Rounding::same(3.0),
            Color32::from_black_alpha(45),
        );

        // 2. Tubuh Superellipse Utama (48 titik presisi tinggi)
        let points = Self::superellipse_points(center, rx, ry, 48, rot);
        painter.add(egui::Shape::convex_polygon(
            points,
            body_col,
            Stroke::new(1.4, rim_col),
        ));

        // 3. Highlight 3D Lembut di dahi atas (tidak menimpa mata)
        let shine_pos = center + Vec2::new(0.0, -ry * 0.68);
        painter.rect_filled(
            Rect::from_center_size(shine_pos, Vec2::new(rx * 0.55, ry * 0.14)),
            Rounding::same(ry * 0.07),
            Color32::from_white_alpha(35),
        );

        // Cek kedekatan kursor ke maskot kartu ini secara independen
        let is_near = if let Some(cursor) = state.last_cursor {
            (cursor - center).length() < 48.0
        } else {
            false
        };
        let effective_eye_style = if state.dizzy_timer > 0.0 {
            EyeStyle::Dizzy
        } else if state.is_sleeping {
            EyeStyle::Closed
        } else if is_near {
            EyeStyle::Happy
        } else {
            eye_style
        };

        // Gaze tracking independen terhadap pusat card mascot ini
        let (card_yaw, card_pitch) = if state.dizzy_timer > 0.0 {
            ((t * 14.0).sin() * 0.85, (t * 8.0).cos() * 0.4)
        } else if state.is_sleeping {
            (0.0, 0.35)
        } else if let Some(cursor) = state.last_cursor {
            let dx = cursor.x - center.x;
            let dy = cursor.y - center.y;
            let dist = (dx * dx + dy * dy).sqrt().max(10.0);
            ((dx / dist).clamp(-0.85, 0.85), (dy / dist).clamp(-0.75, 0.75))
        } else {
            (state.current_yaw, state.current_pitch)
        };

        // 4. Pipi Merona Merah Muda (Cheek Blush)
        let blush_alpha = match effective_eye_style {
            EyeStyle::Happy => 190,
            EyeStyle::Wide => 160,
            EyeStyle::Closed => 60,
            _ => 110,
        };
        let blush_col = Color32::from_rgba_unmultiplied(251, 113, 133, blush_alpha);
        let cheek_offset_x = rx * 0.52;
        let cheek_offset_y = ry * 0.22;
        let cheek_w = rx * 0.28;
        let cheek_h = ry * 0.16;
        painter.rect_filled(
            Rect::from_center_size(center + Vec2::new(-cheek_offset_x, cheek_offset_y), Vec2::new(cheek_w, cheek_h)),
            Rounding::same(cheek_h * 0.5),
            blush_col,
        );
        painter.rect_filled(
            Rect::from_center_size(center + Vec2::new(cheek_offset_x, cheek_offset_y), Vec2::new(cheek_w, cheek_h)),
            Rounding::same(cheek_h * 0.5),
            blush_col,
        );

        // 5. Mata 3D Terproyeksi Bola
        Self::draw_eyes(
            painter,
            center,
            rx,
            ry,
            card_yaw,
            card_pitch,
            state.blink,
            effective_eye_style,
            false,
        );

        // 6. Mulut Ekspresif
        Self::draw_mouth(painter, center, rx, ry, effective_eye_style, status);

        // 7. Gambar Partikel Melayang
        for p in &state.particles {
            let frac = 1.0 - (p.life / p.max_life);
            let alpha = (frac * 255.0) as u8;
            match p.kind {
                ParticleKind::Star => {
                    painter.circle_filled(
                        p.pos,
                        2.5 * frac,
                        Color32::from_rgba_unmultiplied(251, 191, 36, alpha),
                    );
                }
                ParticleKind::SleepZ => {
                    painter.text(
                        p.pos,
                        egui::Align2::CENTER_CENTER,
                        "z",
                        egui::FontId::proportional(11.0),
                        Color32::from_rgba_unmultiplied(148, 163, 184, alpha),
                    );
                }
                ParticleKind::Spark => {
                    painter.circle_filled(
                        p.pos,
                        1.8 * frac,
                        Color32::from_rgba_unmultiplied(56, 189, 248, alpha),
                    );
                }
            }
        }
    }

    /// Menghasilkan titik-titik superellipse parametrik (|x/rx|^n + |y/ry|^n = 1 dengan n=2.7).
    fn superellipse_points(
        center: Pos2,
        rx: f32,
        ry: f32,
        segments: usize,
        rot: f32,
    ) -> Vec<Pos2> {
        let n_exp = 2.0 / 2.7; // Exponent superellipse squircle
        let mut pts = Vec::with_capacity(segments);
        let (sin_r, cos_r) = rot.sin_cos();

        for i in 0..segments {
            let theta = std::f32::consts::TAU * (i as f32) / (segments as f32);
            let cos_t = theta.cos();
            let sin_t = theta.sin();

            let px = rx * cos_t.signum() * cos_t.abs().powf(n_exp);
            let py = ry * sin_t.signum() * sin_t.abs().powf(n_exp);

            let (rot_x, rot_y) = if rot.abs() > 0.001 {
                (px * cos_r - py * sin_r, px * sin_r + py * cos_r)
            } else {
                (px, py)
            };

            pts.push(Pos2::new(center.x + rot_x, center.y + rot_y));
        }
        pts
    }

    /// Menggambar sepasang mata 3D dengan foreshortening perspektif.
    fn draw_eyes(
        painter: &Painter,
        center: Pos2,
        rx: f32,
        ry: f32,
        yaw: f32,
        pitch: f32,
        blink: f32,
        style: EyeStyle,
        is_mini: bool,
    ) {
        let eye_color = Color32::from_rgb(15, 23, 42); // slate-900 pekat
        let eye_spacing_yaw = 0.36;
        let eye_pitch_offset = -0.06;

        for is_right in [false, true] {
            let side_sign = if is_right { 1.0 } else { -1.0 };
            let eye_yaw = yaw + side_sign * eye_spacing_yaw;
            let eye_pitch = pitch + eye_pitch_offset;

            // Proyeksi koordinat 3D bola ke bidang 2D
            let ex = center.x + eye_yaw.sin() * eye_pitch.cos() * rx * 0.78;
            let ey = center.y + eye_pitch.sin() * ry * 0.78;

            // Foreshortening horizontal & vertikal
            let scale_h = eye_yaw.cos().max(0.20);
            let scale_v = eye_pitch.cos().max(0.20);

            match style {
                EyeStyle::Happy => {
                    // Mata tersenyum busur ceria (^ ^)
                    let w = (if is_mini { 3.8 } else { 8.0 }) * scale_h;
                    let h = if is_mini { 2.4 } else { 4.8 };
                    let stroke_w = if is_mini { 1.3 } else { 2.4 };
                    let p0 = Pos2::new(ex - w * 0.5, ey + h * 0.35);
                    let p1 = Pos2::new(ex - w * 0.22, ey - h * 0.40);
                    let p2 = Pos2::new(ex, ey - h * 0.50);
                    let p3 = Pos2::new(ex + w * 0.22, ey - h * 0.40);
                    let p4 = Pos2::new(ex + w * 0.5, ey + h * 0.35);
                    painter.line_segment([p0, p1], Stroke::new(stroke_w, eye_color));
                    painter.line_segment([p1, p2], Stroke::new(stroke_w, eye_color));
                    painter.line_segment([p2, p3], Stroke::new(stroke_w, eye_color));
                    painter.line_segment([p3, p4], Stroke::new(stroke_w, eye_color));
                }
                EyeStyle::Closed => {
                    // Mata terpejam tidur tenang (- -)
                    let w = (if is_mini { 3.4 } else { 7.2 }) * scale_h;
                    let h = if is_mini { 1.5 } else { 3.0 };
                    let stroke_w = if is_mini { 1.2 } else { 2.2 };
                    let p0 = Pos2::new(ex - w * 0.5, ey - h * 0.25);
                    let p1 = Pos2::new(ex, ey + h * 0.35);
                    let p2 = Pos2::new(ex + w * 0.5, ey - h * 0.25);
                    painter.line_segment([p0, p1], Stroke::new(stroke_w, eye_color));
                    painter.line_segment([p1, p2], Stroke::new(stroke_w, eye_color));
                }
                EyeStyle::Dizzy => {
                    let s = if is_mini { 2.0 } else { 4.0 };
                    let stroke_w = if is_mini { 1.2 } else { 2.0 };
                    painter.line_segment(
                        [Pos2::new(ex - s, ey - s), Pos2::new(ex + s, ey + s)],
                        Stroke::new(stroke_w, eye_color),
                    );
                    painter.line_segment(
                        [Pos2::new(ex - s, ey + s), Pos2::new(ex + s, ey - s)],
                        Stroke::new(stroke_w, eye_color),
                    );
                }
                EyeStyle::Wide => {
                    let base_w = if is_mini { 3.2 } else { 7.6 };
                    let base_h = if is_mini { 4.6 } else { 10.5 };
                    let ew = base_w * scale_h;
                    let eh = (base_h * scale_v * blink).max(ew * 0.35);
                    let eye_rect = Rect::from_center_size(Pos2::new(ex, ey), Vec2::new(ew, eh));
                    painter.rect_filled(eye_rect, Rounding::same(ew * 0.5), eye_color);

                    if !is_mini && blink > 0.4 && eh > 5.0 {
                        let spark1 = Pos2::new(ex - ew * 0.18, ey - eh * 0.24);
                        painter.circle_filled(spark1, 1.8, Color32::WHITE);
                        let spark2 = Pos2::new(ex + ew * 0.22, ey + eh * 0.20);
                        painter.circle_filled(spark2, 1.0, Color32::WHITE);
                    }
                }
                _ => {
                    // Pill standar & Thinking
                    let base_w = if is_mini { 3.0 } else { 6.2 };
                    let base_h = if is_mini { 4.4 } else { 9.0 };
                    let ew = base_w * scale_h;
                    let eh = (base_h * scale_v * blink).max(ew * 0.35);
                    let eye_rect = Rect::from_center_size(Pos2::new(ex, ey), Vec2::new(ew, eh));
                    painter.rect_filled(eye_rect, Rounding::same(ew * 0.5), eye_color);

                    if blink > 0.4 && eh > 3.0 {
                        let spark_r = if is_mini { 0.75 } else { 1.4 };
                        let spark1 = Pos2::new(ex + ew * 0.18, ey - eh * 0.22);
                        painter.circle_filled(spark1, spark_r, Color32::WHITE);
                        if !is_mini && eh > 5.0 {
                            let spark2 = Pos2::new(ex - ew * 0.16, ey + eh * 0.20);
                            painter.circle_filled(spark2, 0.75, Color32::from_white_alpha(190));
                        }
                    }
                }
            }
        }
    }

    /// Menggambar mulut kecil ekspresif.
    fn draw_mouth(
        painter: &Painter,
        center: Pos2,
        rx: f32,
        ry: f32,
        style: EyeStyle,
        status: &AgentStatus,
    ) {
        let mouth_color = Color32::from_rgb(15, 23, 42);
        let my = center.y + ry * 0.36;

        match style {
            EyeStyle::Happy => {
                // Senyum melengkung manis
                let w = rx * 0.26;
                let h = ry * 0.15;
                let left = Pos2::new(center.x - w * 0.5, my - h * 0.2);
                let mid = Pos2::new(center.x, my + h * 0.8);
                let right = Pos2::new(center.x + w * 0.5, my - h * 0.2);
                painter.line_segment([left, mid], Stroke::new(1.8, mouth_color));
                painter.line_segment([mid, right], Stroke::new(1.8, mouth_color));
            }
            EyeStyle::Wide => {
                // Mulut "o" memohon
                let mw = rx * 0.16;
                let mh = ry * 0.20;
                painter.rect_filled(
                    Rect::from_center_size(Pos2::new(center.x, my + 1.0), Vec2::new(mw, mh)),
                    Rounding::same(mw * 0.5),
                    mouth_color,
                );
            }
            _ => {
                if status.is_awaiting_approval() {
                    painter.circle_filled(Pos2::new(center.x, my), 2.2, mouth_color);
                } else {
                    // Garis senyum lembut
                    let w = rx * 0.18;
                    let h = ry * 0.08;
                    let left = Pos2::new(center.x - w * 0.5, my);
                    let mid = Pos2::new(center.x, my + h);
                    let right = Pos2::new(center.x + w * 0.5, my);
                    painter.line_segment([left, mid], Stroke::new(1.5, mouth_color));
                    painter.line_segment([mid, right], Stroke::new(1.5, mouth_color));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superellipse_points_count() {
        let pts = MascotRenderer::superellipse_points(Pos2::new(50.0, 50.0), 20.0, 20.0, 48, 0.0);
        assert_eq!(pts.len(), 48);
        for p in pts {
            assert!(p.x >= 30.0 && p.x <= 70.0);
            assert!(p.y >= 30.0 && p.y <= 70.0);
        }
    }

    #[test]
    fn test_mascot_palette_transitions() {
        let state = MascotState::new();

        let (idle_col, _, idle_style) = state.get_palette(&AgentStatus::Idle, true);
        assert_eq!(idle_style, EyeStyle::Pill);
        assert_eq!(idle_col, Color32::from_rgb(241, 245, 249));

        let req = noktah_core::model::PermissionRequest {
            request_id: "test-req".into(),
            tool_name: "Bash".into(),
            tool_input: serde_json::json!({}),
            suggestions: Vec::new(),
        };
        let (appr_col, _, appr_style) =
            state.get_palette(&AgentStatus::AwaitingApproval(req), true);
        assert_eq!(appr_style, EyeStyle::Wide);
        assert_eq!(appr_col, Color32::from_rgb(245, 158, 11));

        let (sleep_col, _, sleep_style) = state.get_palette(&AgentStatus::Idle, false);
        assert_eq!(sleep_style, EyeStyle::Closed);
        assert_eq!(sleep_col, Color32::from_rgb(100, 116, 139));
    }

    #[test]
    fn test_mascot_squish_and_dizzy_trigger() {
        let mut state = MascotState::new();
        assert_eq!(state.squish.pos, 1.0);

        state.trigger_squish(1.0);
        assert!(state.squish.pos < 0.75);
        assert_eq!(state.click_count, 1);

        state.trigger_squish(1.2);
        assert_eq!(state.click_count, 2);

        state.trigger_squish(1.4);
        assert_eq!(state.click_count, 0);
        assert!(state.dizzy_timer > 3.0);
    }

    #[test]
    fn test_mascot_tracks_cursor_in_all_states() {
        let mut state = MascotState::new();
        let center = Pos2::new(100.0, 100.0);

        // Kursor di kanan bawah
        let cursor_rb = Some(Pos2::new(200.0, 200.0));

        let req = noktah_core::model::PermissionRequest {
            request_id: "test-req".into(),
            tool_name: "Bash".into(),
            tool_input: serde_json::json!({}),
            suggestions: Vec::new(),
        };

        let statuses = vec![
            AgentStatus::Idle,
            AgentStatus::AwaitingApproval(req),
            AgentStatus::ToolRunning {
                tool: "cargo".into(),
                input: serde_json::json!({}),
                start_time: 0,
            },
            AgentStatus::PromptSubmitted {
                prompt: "thinking".into(),
                timestamp: 0,
            },
            AgentStatus::Completed {
                summary: "done".into(),
                last_message: "ok".into(),
                timestamp: 0,
            },
        ];

        for s in statuses {
            state.update(0.016, 1.0, cursor_rb, center, &s, true);
            // Kursor di kanan (dx > 0) -> target yaw harus positif
            assert!(state.eye_yaw > 0.5, "eye_yaw should be positive for {:?}", s);
            // Kursor di bawah (dy > 0) -> target pitch harus positif
            assert!(state.eye_pitch > 0.5, "eye_pitch should be positive for {:?}", s);
        }
    }
}
