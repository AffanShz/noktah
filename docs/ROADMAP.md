# Roadmap Noktah

Dokumen kerja untuk arah pengembangan **Noktah** (overlay Dynamic Island native Windows berbasis Rust + `egui`/`eframe`). Urutan disusun berdasarkan **urgensi stabilitas & dampak harian ÷ usaha teknis**, dengan prinsip inti: **100% native tanpa WebView, satu biner ringan, local-first, tanpa telemetri**.

Legenda usaha: 🟢 beberapa jam · 🟡 beberapa hari · 🔴 butuh interop Windows tingkat rendah dan pengujian di perangkat asli.

---

## Sudah selesai (`v1.0.0` – `v2.0.0`)

- **Distribusi Publik Terpercaya: CI Code Signing, `winget` & Setup PATH (`v2.0.0`)** — penandatanganan digital biner rilis dual-mode di CI (`Azure Trusted Signing` / `PFX signtool` / `unsigned fallback`), generator + validator manifest `winget` skema `1.6.0` dengan alias ganda `noktah` & `noktah-hook` (`winget validate` lulus tanpa peringatan), embedding `VERSIONINFO` pada `noktah-hook.exe`, serta saklar registrasi User `PATH` di *Settings → AI* + `docs/DISTRIBUTION.md`.
- **Stabilisasi Liquid Glass, IPC Tile, UX & Isolasi Test (`v1.6.1`)** — perbaikan gradien *single-premultiplication* & *screen-space UV* Liquid Glass, pengingat inline `@HH:MM`, input path kalender `.ics`, toast *Settings* dwibahasa terikat token desain, `store::load_strict`, status SHA-256 *updater* yang jujur, prioritas prompt izin atas *hover-exit*, serta isolasi penuh test dari `%APPDATA%` & Registry.
- **Liquid Glass GPU Shader Stabil & Preset Tema (`v1.6.0`)** — stabilisasi efek backdrop Liquid Glass (inpainting strip pill agar bebas noda hitam, saturasi BT.709, alpha dinamis, gradien tint, rim highlight specular, sinkronisasi `glass_dirty`) serta `ThemePreset` dengan 5 preset kurasi, kustomisasi aksen `#RRGGBB`, dan impor/ekspor JSON clipboard + berkas (`theme-preset.json`).
- **Custom Status Tiles via `noktah-hook` (`v1.5.0`)** — skema IPC `CustomTile`/`TileStore` (maks 6 tile, TTL otomatis di `next_wake`, 5 nada status), perintah CLI `noktah-hook tile` / `tile-remove` / `tile-clear`, penyematan di pill minimal (`--pin-mini`), dan switcher kartu di halaman `System`.
- **Aplikasi Rust native (`egui`/`eframe`)** — tanpa WebView/runtime tambahan, terbagi menjadi `noktah-core`, `noktah-native`, dan `noktah-hook`.
- **Dynamic Island & Morphing** — kurva spring terkalibrasi ([MOTION.md](MOTION.md)), squircle, liquid ears, pill minimized elastis yang dapat dikustomisasi (termasuk `mini_idle_on_pause` saat audio dijeda), serta hover intent.
- **Dock 7-Halaman & Kustomisasi** — Home (SMTC + Day Progress + kuota Antigravity), Music (SMTC + lirik tersinkron multi-provider LRCLIB/BetterLyrics/NetEase/KuGou + seekable lyrics + aksen album dinamis), Focus (Pomodoro dinamis + Focus DND + rekap 7 hari), Tasks (pengingat + auto-banner + Scratchpad autosave + File Drop Shelf), System (CPU/RAM/cuaca Open-Meteo), Ask (AI chat streaming 6 provider + Quick AI Actions), dan Agent (maskot Noki + gaze tracking + `noktah-hook` universal).
- **Integrasi Mendalam Windows & Perangkat Keras (`v1.4.0`)** — penempatan layar multi-monitor (`Primary` atau `CursorFollow`), deteksi fullscreen auto-hide pintar dengan peek intent (4 px tepi atas), File Drop Shelf penampung berkas sementara, notifikasi mikro HUD audio saat headset/speaker beralih, dan parser kalender lokal `.ics` (RFC 5545) zero cloud dependency.
- **Jendela Setelan Mandiri & Harian (`v1.3.0`)** — halaman default saat dibuka + pintasan `Alt+1..7`/`Esc`, antrean izin multi-agent FIFO dengan nama workspace, serta One-Click Self-Updater dengan verifikasi SHA-256 terhadap `BACA-DULU.txt`.
- **Stabilitas & Diagnostik (`v1.2.2`)** — perbaikan bug penulisan `tasks.json` tiap frame, gating idle wakeup & polling `sysinfo`, konfirmasi reset 2-klik beserta pembersihan penuh `store::wipe`, logger lokal berputar (`logs/noktah.log`) + penampil log di Settings, lokalisasi label dock/dialog, dan gate unit test di workflow rilis CI.

---

## Selanjutnya

*Seluruh target roadmap hingga `v2.0.0` telah selesai dirilis. Rencana rilis berikutnya akan disusun berdasarkan prioritas stabilitas dan umpan balik penggunaan harian.*

---

## Yang sengaja tidak dikerjakan

- **Kembali ke WebView / Electron / Tauri** — bertentangan dengan target biner tunggal ringan dan memori minimal.
- **Spotify Web API** — Windows SMTC sudah memberi kontrol dan metadata lintas pemutar tanpa OAuth atau token tambahan.
- **Telemetri & Akun Cloud** — aplikasi tidak boleh mengirim data analitik atau menyimpan data pengguna ke server pihak ketiga.
