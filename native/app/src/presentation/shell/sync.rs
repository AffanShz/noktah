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
                if self.settings.sneak_peek && !self.lyrics_for.is_empty() && !self.is_focus_dnd_active() {
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
                    crate::infrastructure::logger::warn(
                        "lyrics",
                        &format!("No lyrics found for '{}'", result.track_id),
                    );
                    LyricsStatus::NotFound
                } else {
                    crate::infrastructure::logger::info(
                        "lyrics",
                        &format!("Lyrics loaded via {}", result.provider),
                    );
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
                    crate::infrastructure::logger::error("ai", &pesan);
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
        self.chat_input.clear();
        self.send_prompt(text);
    }

    /// Mengirim prompt apa pun (dari kolom tulis atau aksi kilat) ke penyedia AI aktif.
    pub fn send_prompt(&mut self, text: String) {
        let text = text.trim().to_string();
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

    /// Minta worker kuota membaca ulang sekarang, tanpa menunggu 3 menit.
    ///
    /// Menarik hasil worker kuota. Hanya satu snapshot terbaru yang disimpan;
    /// kalau ada dua datang berurutan, yang terakhir menang.
    pub(crate) fn poll_quota(&mut self) {
        while let Ok(snap) = self.quota_worker.rx.try_recv() {
            if let Some(err) = snap.error_key {
                let prev = self.quota.as_ref().and_then(|q| q.error_key);
                if prev != Some(err) {
                    crate::infrastructure::logger::warn("quota", err);
                }
            }
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
        if self.custom_tiles.prune_expired(now) && self.custom_tiles.is_empty() {
            self.system_tiles_mode = false;
        }

        // 1. Auto-decay heartbeat: jika tidak ada aktivitas selama 45 detik dan tidak ada antrean izin,
        // reset agent.active ke false agar CPU Noktah kembali tidur 60s per frame
        const AGENT_TIMEOUT_MS: i64 = 45_000;
        if self.agent.active
            && self.pending_permissions.is_empty()
            && (now - self.agent.updated_at) > AGENT_TIMEOUT_MS
        {
            self.agent.active = false;
            self.agent.status = noktah_core::model::AgentStatus::Idle;
        }

        // 2. Tarik event dari IPC channel
        while let Ok(event) = self.ipc.rx.try_recv() {
            match event {
                crate::infrastructure::ipc::IpcEvent::HookReceived(payload) => {
                    let ev_clean = payload.hook_event_name.trim().to_lowercase();

                    // Tangani event custom tile tanpa mengubah state self.agent
                    match ev_clean.as_str() {
                        "tile" | "tile-set" | "tile_set" | "custom_tile" | "status_tile" => {
                            if self.settings.custom_tiles_enabled {
                                if let Some(tile) = payload.into_custom_tile(now) {
                                    let _ = self.custom_tiles.upsert(tile, now);
                                }
                            }
                            continue;
                        }
                        "tile-remove" | "tile_remove" | "tile-del" | "tile_del" => {
                            let id = payload
                                .tile_id
                                .or(payload.tile_title)
                                .or_else(|| payload.tile.map(|t| t.id))
                                .unwrap_or_default();
                            if !id.is_empty() {
                                self.custom_tiles.remove(&id);
                                if self.custom_tiles.is_empty() {
                                    self.system_tiles_mode = false;
                                }
                            }
                            continue;
                        }
                        "tile-clear" | "tile_clear" => {
                            self.custom_tiles.clear();
                            self.system_tiles_mode = false;
                            continue;
                        }
                        _ => {}
                    }

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

                    let ev_clean = payload.hook_event_name.trim().to_lowercase();
                    if matches!(
                        ev_clean.as_str(),
                        "sessionstart"
                            | "start"
                            | "preinvocation"
                            | "postinvocation"
                            | "stop"
                            | "done"
                            | "completed"
                            | "finish"
                            | "finished"
                            | "success"
                    ) {
                        self.refresh_quota();
                    }
                    match ev_clean.as_str() {
                        "sessionstart" | "start" | "idle" | "ready" | "reset" => {
                            self.agent.active = true;
                            self.agent.status = noktah_core::model::AgentStatus::Idle;
                            self.agent.logs.push("[Session Started]".into());
                        }
                        "userpromptsubmit" | "preinvocation" | "prompt" | "query" | "thinking" | "plan" | "planning" | "request" => {
                            self.agent.active = true;
                            let prompt = payload.prompt.or(payload.last_assistant_message).unwrap_or_default();
                            if !prompt.is_empty() {
                                self.agent.logs.push(format!("> {}", prompt));
                            }
                            self.agent.status = noktah_core::model::AgentStatus::PromptSubmitted {
                                prompt: if prompt.is_empty() {
                                    "Thinking...".into()
                                } else {
                                    prompt
                                },
                                timestamp: now,
                            };
                        }
                        "postinvocation" => {
                            self.agent.active = true;
                            self.agent.status = noktah_core::model::AgentStatus::PromptSubmitted {
                                prompt: "Processing response...".into(),
                                timestamp: now,
                            };
                        }
                        "pretooluse" | "tool" | "working" | "busy" | "run" | "running" | "build" | "building" | "executing" | "work" => {
                            self.agent.active = true;
                            let tool = payload.tool_name.or(payload.prompt).unwrap_or_else(|| "Tool".into());
                            let input = payload.tool_input.unwrap_or_default();
                            self.agent.status = noktah_core::model::AgentStatus::ToolRunning {
                                tool,
                                input,
                                start_time: now,
                            };
                        }
                        "posttooluse" | "tool_done" | "tool_finish" | "tool_completed" => {
                            self.agent.active = true;
                            let dur = payload.duration_ms.unwrap_or(0);
                            let tool = payload.tool_name.unwrap_or_default();
                            self.agent.logs.push(format!("✔ {} ({}ms)", tool, dur));
                            self.agent.status = noktah_core::model::AgentStatus::PromptSubmitted {
                                prompt: "Processing tool response...".into(),
                                timestamp: now,
                            };
                        }
                        "subagentstart" | "subagent_start" | "spawn" => {
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
                        "subagentstop" | "subagent_stop" => {
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
                        "stop" | "done" | "completed" | "finish" | "finished" | "success" => {
                            self.agent.active = true;
                            let last_msg = payload.last_assistant_message.or(payload.prompt).unwrap_or_default();
                            self.agent.status = noktah_core::model::AgentStatus::Completed {
                                summary: "Task finished".into(),
                                last_message: last_msg,
                                timestamp: now,
                            };
                        }
                        "error" | "fail" | "failed" | "crash" | "panic" | "exception" => {
                            self.agent.active = true;
                            let msg = payload.last_assistant_message
                                .or(payload.prompt)
                                .or(payload.tool_name)
                                .unwrap_or_else(|| "Error occurred".into());
                            self.agent.logs.push(format!("✖ Error: {}", msg));
                            self.agent.status = noktah_core::model::AgentStatus::Error {
                                message: msg,
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
                    session_title,
                    responder,
                } => {
                    self.agent.active = true;
                    self.agent.updated_at = now;
                    let req = noktah_core::model::PermissionRequest {
                        request_id,
                        tool_name,
                        tool_input,
                        suggestions,
                        session_title,
                    };
                    self.pending_permissions.push_back(crate::app::QueuedPermission {
                        request: req,
                        responder,
                    });
                    if let Some(front) = self.pending_permissions.front() {
                        self.agent.status = noktah_core::model::AgentStatus::AwaitingApproval(
                            front.request.clone(),
                        );
                    }

                    // Buka notch otomatis untuk menampilkan modal approval
                    self.open.target = 1.0;
                    self.open_h.target = 1.0;
                    self.current_page = crate::app::NotchPage::Agent;
                }
                crate::infrastructure::ipc::IpcEvent::PermissionCanceled(req_id) => {
                    let before = self.pending_permissions.len();
                    self.pending_permissions
                        .retain(|item| item.request.request_id != req_id);
                    if self.pending_permissions.len() != before {
                        if let Some(next) = self.pending_permissions.front() {
                            self.agent.status = noktah_core::model::AgentStatus::AwaitingApproval(
                                next.request.clone(),
                            );
                        } else {
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

    pub(crate) fn poll_updater(&mut self) {
        while let Ok(st) = self.updater.rx.try_recv() {
            self.update_status = st;
        }
    }

    pub(crate) fn poll_calendar(&mut self) {
        if !self.settings.calendar_enabled {
            self.calendar_error = None;
            return;
        }

        let path_changed = self.settings.calendar_ics_path.trim() != self.calendar_path.trim();
        if path_changed {
            self.calendar_path = self.settings.calendar_ics_path.clone();
            self.calendar_events.clear();
            if self.calendar_path.trim().is_empty() {
                self.calendar_error = None;
            }
        } else if self.last_calendar_check.elapsed() < std::time::Duration::from_secs(45) {
            return;
        }
        self.last_calendar_check = std::time::Instant::now();

        let custom_path = self.settings.calendar_ics_path.trim();
        if !custom_path.is_empty() {
            match std::fs::read_to_string(custom_path) {
                Ok(content) => {
                    self.calendar_events = noktah_core::calendar::parse_ics(&content);
                    self.calendar_error = None;
                }
                Err(err) => {
                    self.calendar_events.clear();
                    let msg = err.to_string();
                    crate::infrastructure::logger::warn(
                        "calendar",
                        &format!("Failed to read custom .ics '{}': {}", custom_path, msg),
                    );
                    self.calendar_error = Some(msg);
                }
            }
        } else {
            self.calendar_error = None;
            if let Some(appdata) = std::env::var_os("APPDATA") {
                let path = std::path::PathBuf::from(appdata).join("Noktah").join("calendar.ics");
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        self.calendar_events = noktah_core::calendar::parse_ics(&content);
                    }
                    Err(_) => {
                        self.calendar_events.clear();
                    }
                }
            }
        }
    }

    pub(crate) fn poll_audio_device(&mut self) {
        if !self.settings.audio_hud_enabled {
            return;
        }

        if self.last_audio_dev_check.elapsed() < std::time::Duration::from_secs(3) {
            return;
        }
        self.last_audio_dev_check = std::time::Instant::now();

        if let Some(cur_id) = crate::infrastructure::audio::get_default_render_device_id() {
            if let Some(prev) = &self.last_audio_dev_name {
                if prev != &cur_id && !self.is_focus_dnd_active() {
                    self.banner = Some((
                        "🎧 Output audio beralih".to_string(),
                        std::time::Instant::now(),
                    ));
                }
            }
            self.last_audio_dev_name = Some(cur_id);
        }
    }

    pub(crate) fn poll_file_drop(&mut self, ctx: &egui::Context) {
        if !self.settings.file_shelf_enabled {
            return;
        }

        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        let mut added = false;
        for df in dropped {
            if let Some(path) = df.path {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "File".into());
                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                if !self.file_shelf.iter().any(|f| f.path == path) {
                    self.file_shelf.push(crate::presentation::widgets::ShelfFile {
                        name,
                        path,
                        size_bytes: size,
                    });
                    if self.file_shelf.len() > 5 {
                        self.file_shelf.remove(0);
                    }
                    added = true;
                }
            }
        }

        if added && !self.is_focus_dnd_active() {
            self.banner = Some((
                "📁 Berkas ditambahkan ke Drop Shelf".to_string(),
                std::time::Instant::now(),
            ));
        }
    }
}
