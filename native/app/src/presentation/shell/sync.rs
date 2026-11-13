//! Polling hasil kerja latar belakang (worker background threads) dan sinkronisasi.
//!
//! Menangani update dari:
//! - SMTC Windows Media Control (playback, judul, artis, album art)
//! - Klien LRCLIB (lirik tersinkron)
//! - Klien AI (streaming respon SSE)
//! - Layanan cuaca (Open-Meteo)

use crate::app::{LyricsStatus, Noktah};
use crate::infrastructure::ai::{self, AiEvent};
use crate::infrastructure::lyrics_client;
use crate::infrastructure::music::Cmd as MediaCmd;
use crate::infrastructure::secret;
use crate::presentation::spring::{SNEAK_PEEK, TRANSISI_HOLD, TRANSISI_TIMEOUT};
use eframe::egui;
use noktah_core::model::ChatMessage;
use std::time::Instant;

impl Noktah {
    /// Menarik hasil dari thread SMTC dan klien lirik.
    pub(crate) fn poll_media(&mut self, ctx: &egui::Context) {
        while let Ok(update) = self.music.rx.try_recv() {
            // Konfirmasi seek: posisi baru dari SMTC sudah melewati target
            // (lagu berjalan terus), jadi tahanan slider boleh dilepas.
            if let Some((target, at)) = self.seek_at {
                let confirm =
                    update.state.position_ms >= target.saturating_sub(250) && update.state.active;
                let kadaluarsa = at.elapsed() >= std::time::Duration::from_secs(3);
                if confirm || kadaluarsa {
                    self.seek_at = None;
                }
            }

            self.media = update.state;

            if let Some(bytes) = update.art {
                if update.track_id != self.art_for {
                    self.art_for = update.track_id.clone();
                    if let Some((image, dominant)) = Noktah::decode_art(&bytes) {
                        self.art =
                            Some(ctx.load_texture("album-art", image, egui::TextureOptions::LINEAR));
                        self.album_accent = dominant;
                    }
                }
            }

            let track = format!("{}|{}", self.media.title, self.media.artist);
            if self.media.active && self.settings.lyrics_enabled && track != self.lyrics_for {
                // Lagu berganti (atau baru mulai): bentuk menyusut ke garis
                // tipis dulu, lalu membesar lagi begitu liriknya siap.
                if !self.lyrics_for.is_empty() {
                    self.transisi_at = Some(Instant::now());
                }
                // Sneak peek: setelah transisi selesai, pill tetap melebar
                // sesaat menampilkan judul lagu baru. Hanya untuk pergantian
                // di antara dua lagu — lagu pertama tidak ada yang perlu
                // "diintip".
                if self.settings.sneak_peek && !self.lyrics_for.is_empty() {
                    self.peek_until = Some(Instant::now() + SNEAK_PEEK);
                }
                self.lyrics_for = track.clone();
                self.lyrics_status = LyricsStatus::Searching;
                self.lyrics_provider.clear();
                self.lyrics.clear();
                self.lyrics_synced = false;
                self.lyric_scroll_pos = 0.0;
                self.lyric_target_scroll = 0.0;
                self.last_active_lyric = None;
                self.mini_prev_lyric = None;
                self.mini_curr_lyric = None;
                let _ = self.lyrics_client.tx.send(lyrics_client::Request {
                    track_id: track,
                    title: self.media.title.clone(),
                    artist: self.media.artist.clone(),
                    duration: self.media.duration_ms / 1000,
                    album: self.media.album.clone(),
                });
            }

            if !self.media.active {
                self.art = None;
                self.art_for.clear();
                self.album_accent = None;
                self.lyrics.clear();
                self.lyric_scroll_pos = 0.0;
                self.lyric_target_scroll = 0.0;
                self.last_active_lyric = None;
                self.mini_prev_lyric = None;
                self.mini_curr_lyric = None;
                self.lyrics_status = LyricsStatus::Idle;
                self.lyrics_provider.clear();
                self.lyrics_for.clear();
                self.lyrics_synced = false;
            }
        }

        while let Ok(result) = self.lyrics_client.rx.try_recv() {
            if result.track_id == self.lyrics_for {
                self.lyrics_status = if result.lines.is_empty() {
                    LyricsStatus::NotFound
                } else {
                    LyricsStatus::Found
                };
                self.lyrics = result.lines;
                self.lyrics_synced = result.synced;
                self.lyrics_provider = result.provider;
            }
        }

        // Transisi selesai saat lirik sudah di tangan (baik ketemu maupun
        // tidak) — menunggu lebih lama hanya menahan bentuk di garis tipis
        // tanpa manfaat. Batas waktu menjaga lagu tanpa lirik tidak
        // menggantung selamanya.
        if let Some(at) = self.transisi_at {
            let siap = self.lyrics_status != LyricsStatus::Searching;
            let sabar = at.elapsed() >= TRANSISI_HOLD;
            let habis = at.elapsed() >= TRANSISI_TIMEOUT;
            if (siap && sabar) || habis {
                self.transisi_at = None;
            }
        }
    }

    pub fn media_cmd(&self, cmd: MediaCmd) {
        let _ = self.music.tx.send(cmd);
    }

    pub(crate) fn poll_ai(&mut self) {
        while let Ok(event) = self.ai.rx.try_recv() {
            match event {
                AiEvent::Delta(text) => {
                    if let Some(last) = self.chat.last_mut() {
                        last.content.push_str(&text);
                    }
                }
                AiEvent::Done => {
                    self.ai_busy = false;
                    self.save_chat();
                }
                AiEvent::Error(pesan) => {
                    self.ai_busy = false;
                    self.banner = Some((format!("AI: {pesan}"), Instant::now()));
                }
            }
        }
    }

    pub fn send_chat(&mut self) {
        let text = self.chat_input.trim().to_string();
        if text.is_empty() || self.ai_busy {
            return;
        }

        let now = chrono::Local::now().timestamp_millis();
        self.chat.push(ChatMessage {
            id: format!("{now:x}u"),
            role: noktah_core::model::Role::User,
            content: text,
            at: now,
        });
        self.chat.push(ChatMessage {
            id: format!("{now:x}a"),
            role: noktah_core::model::Role::Assistant,
            content: String::new(),
            at: now,
        });
        self.chat_input.clear();
        self.ai_busy = true;

        let messages: Vec<(noktah_core::model::Role, String)> = self
            .chat
            .iter()
            .filter(|m| !m.content.is_empty())
            .map(|m| (m.role, m.content.clone()))
            .collect();

        let _ = self.ai.tx.send(ai::AiRequest {
            base_url: self.settings.base_url.clone(),
            model: self.settings.model.clone(),
            api_key: self.api_key.clone(),
            messages,
        });
    }

    /// Menarik hasil worker kuota. Hanya satu snapshot terbaru yang disimpan;
    /// kalau ada dua datang berurutan, yang terakhir menang.
    pub(crate) fn poll_quota(&mut self) {
        while let Ok(snap) = self.quota_worker.rx.try_recv() {
            self.quota = Some(snap);
        }
    }

    /// Minta worker kuota membaca ulang sekarang, tanpa menunggu 3 menit.
    pub fn refresh_quota(&self) {
        let _ = self.quota_worker.tx.send(());
    }

    pub(crate) fn poll_weather(&mut self) {
        if self.settings.city != self.weather_city {
            self.weather_city = self.settings.city.clone();
            let _ = self.weather.tx.send(self.weather_city.clone());
        }
        while let Ok(cuaca) = self.weather.rx.try_recv() {
            self.weather_now = Some(cuaca);
        }
    }

    pub fn set_api_key(&mut self, key: String) {
        self.api_key = key;
        let _ = secret::save(&self.api_key);
    }

    pub fn decode_art(bytes: &[u8]) -> Option<(egui::ColorImage, Option<egui::Color32>)> {
        // Sampul dari SMTC bisa 1000x1000 atau lebih; sebagai tekstur itu
        // 4 MB di GPU untuk gambar yang cuma tampil 88 px. Dikecilkan dulu.
        let img = image::load_from_memory(bytes).ok()?.thumbnail(256, 256).to_rgba8();
        let dominant = Noktah::extract_vibrant_color(&img);
        let size = [img.width() as usize, img.height() as usize];
        Some((egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()), dominant))
    }

    /// Ekstraksi warna aksen paling hidup dari sampul album (Mochy-Music AlbumGradient).
    pub fn extract_vibrant_color(img: &image::RgbaImage) -> Option<egui::Color32> {
        let mut best_score = 0.0_f32;
        let mut best_color = None;

        for pixel in img.pixels().step_by(4) {
            let r = pixel[0] as f32 / 255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;
            let a = pixel[3] as f32 / 255.0;
            if a < 0.6 {
                continue;
            }

            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            let delta = max - min;
            let v = max;
            let s = if max > 0.0 { delta / max } else { 0.0 };

            // Hindari warna hitam pekat, putih pucat, atau abu-abu mati
            if v < 0.28 || s < 0.28 || v > 0.98 {
                continue;
            }

            let score = s * 1.8 + (v - 0.55).abs().mul_add(-0.6, 0.8);
            if score > best_score {
                best_score = score;
                best_color = Some(egui::Color32::from_rgb(pixel[0], pixel[1], pixel[2]));
            }
        }

        best_color
    }

    /// Menarik update dari server IPC Named Pipe dan sinkronisasi status agent Claude Code.
    pub(crate) fn poll_ipc(&mut self) {
        let now = chrono::Utc::now().timestamp_millis();

        // 1. Auto-decay heartbeat: jika tidak ada aktivitas selama 45 detik,
        // reset agent.active ke false agar CPU Noktah kembali tidur 60s per frame
        const AGENT_TIMEOUT_MS: i64 = 45_000;
        if self.agent.active && (now - self.agent.updated_at) > AGENT_TIMEOUT_MS {
            self.agent.active = false;
            self.agent.status = noktah_core::model::AgentStatus::Idle;
            self.pending_permission = None;
        }

        // 2. Tarik event dari IPC channel
        while let Ok(event) = self.ipc.rx.try_recv() {
            match event {
                crate::infrastructure::ipc::IpcEvent::HookReceived(payload) => {
                    self.agent.updated_at = now;
                    if let Some(sid) = payload.session_id {
                        self.agent.session_id = sid;
                    }
                    if let Some(title) = payload.session_title {
                        self.agent.session_title = title;
                    }
                    if let Some(mode) = payload.permission_mode {
                        self.agent.permission_mode = mode;
                    }

                    match payload.hook_event_name.as_str() {
                        "SessionStart" => {
                            self.agent.active = true;
                            self.agent.status = noktah_core::model::AgentStatus::Idle;
                            self.agent.logs.push("[Session Started]".into());
                        }
                        "UserPromptSubmit" => {
                            self.agent.active = true;
                            let prompt = payload.prompt.unwrap_or_default();
                            self.agent.logs.push(format!("> {}", prompt));
                            self.agent.status = noktah_core::model::AgentStatus::PromptSubmitted {
                                prompt,
                                timestamp: now,
                            };
                        }
                        "PreToolUse" => {
                            self.agent.active = true;
                            let tool = payload.tool_name.unwrap_or_else(|| "Tool".into());
                            let input = payload.tool_input.unwrap_or_default();
                            self.agent.status = noktah_core::model::AgentStatus::ToolRunning {
                                tool,
                                input,
                                start_time: now,
                            };
                        }
                        "PostToolUse" => {
                            self.agent.active = true;
                            let dur = payload.duration_ms.unwrap_or(0);
                            let tool = payload.tool_name.unwrap_or_default();
                            self.agent.logs.push(format!("✔ {} ({}ms)", tool, dur));
                            self.agent.status = noktah_core::model::AgentStatus::PromptSubmitted {
                                prompt: "Processing tool response...".into(),
                                timestamp: now,
                            };
                        }
                        "SubagentStart" => {
                            self.agent.active = true;
                            if let (Some(id), Some(typ)) = (payload.agent_id, payload.agent_type) {
                                self.agent.subagents.push(noktah_core::model::SubagentInfo {
                                    agent_id: id,
                                    agent_type: typ,
                                    status: "running".into(),
                                });
                                self.agent.status = noktah_core::model::AgentStatus::Subagents(
                                    self.agent.subagents.clone(),
                                );
                            }
                        }
                        "SubagentStop" => {
                            self.agent.active = true;
                            if let Some(id) = payload.agent_id {
                                self.agent.subagents.retain(|s| s.agent_id != id);
                                if self.agent.subagents.is_empty() {
                                    self.agent.status = noktah_core::model::AgentStatus::PromptSubmitted {
                                        prompt: "Subagents completed".into(),
                                        timestamp: now,
                                    };
                                } else {
                                    self.agent.status = noktah_core::model::AgentStatus::Subagents(
                                        self.agent.subagents.clone(),
                                    );
                                }
                            }
                        }
                        "Stop" => {
                            self.agent.active = true;
                            let last_msg = payload.last_assistant_message.unwrap_or_default();
                            self.agent.status = noktah_core::model::AgentStatus::Completed {
                                summary: "Turn finished".into(),
                                last_message: last_msg,
                                timestamp: now,
                            };
                        }
                        _ => {}
                    }
                }
                crate::infrastructure::ipc::IpcEvent::PermissionRequest {
                    request_id,
                    tool_name,
                    tool_input,
                    suggestions,
                    responder,
                } => {
                    self.agent.active = true;
                    self.agent.updated_at = now;
                    self.pending_permission = Some((request_id.clone(), responder));
                    self.agent.status = noktah_core::model::AgentStatus::AwaitingApproval(
                        noktah_core::model::PermissionRequest {
                            request_id,
                            tool_name,
                            tool_input,
                            suggestions,
                        },
                    );

                    // Buka notch otomatis untuk menampilkan modal approval
                    self.open.target = 1.0;
                    self.open_h.target = 1.0;
                    self.current_page = crate::app::NotchPage::Agent;
                }
                crate::infrastructure::ipc::IpcEvent::PermissionCanceled(req_id) => {
                    if let Some((pending_id, _)) = &self.pending_permission {
                        if pending_id == &req_id {
                            self.pending_permission = None;
                            self.agent.status = noktah_core::model::AgentStatus::Idle;
                            // Tutup notch jika tidak sedang di-hover kursor
                            if self.hover_since.is_none() {
                                self.open.target = 0.0;
                                self.open_h.target = 0.0;
                            }
                        }
                    }
                }
            }
        }
    }
}
