# Roadmap Noktah

Dokumen kerja untuk arah pengembangan **Noktah** (overlay Dynamic Island native Windows berbasis Rust + `egui`/`eframe`). Urutan disusun berdasarkan **urgensi stabilitas & dampak harian ÷ usaha teknis**, dengan prinsip inti: **100% native tanpa WebView, satu biner ringan, local-first, tanpa telemetri**.

Legenda usaha: 🟢 beberapa jam · 🟡 beberapa hari · 🔴 butuh interop Windows tingkat rendah dan pengujian di perangkat asli.

---

## Sudah selesai (`v1.0.0` – `v1.2.2`)

- **Aplikasi Rust native (`egui`/`eframe`)** — tanpa WebView/runtime tambahan, terbagi menjadi `noktah-core`, `noktah-native`, dan `noktah-hook`.
- **Dynamic Island & Morphing** — kurva spring terkalibrasi ([MOTION.md](MOTION.md)), squircle, liquid ears, pill minimized elastis yang dapat dikustomisasi, serta hover intent.
- **Dock 7-Halaman & Kustomisasi** — Home (SMTC + kuota Antigravity), Music (SMTC + lirik tersinkron multi-provider LRCLIB/BetterLyrics/NetEase/KuGou + seekable lyrics + aksen album dinamis), Focus (Pomodoro dinamis), Tasks (pengingat + auto-banner), System (CPU/RAM/cuaca Open-Meteo), Ask (AI chat streaming 6 provider dengan enkripsi DPAPI), dan Agent (maskot Noki + gaze tracking + `noktah-hook` universal untuk Claude Code, Antigravity, dan CLI).
- **Jendela Setelan Mandiri** — secondary viewport native *always-on-top*, kontrol visibilitas tab dock (`nav_*`), salin konfigurasi hook instan, serta in-app update checker GitHub.
- **Stabilitas & Diagnostik (`v1.2.2`)** — perbaikan bug penulisan `tasks.json` tiap frame, gating idle wakeup & polling `sysinfo`, konfirmasi reset 2-klik beserta pembersihan penuh `store::wipe`, logger lokal berputar (`logs/noktah.log`) + penampil log di Settings, lokalisasi label dock/dialog, dan gate unit test di workflow rilis CI.

---

## `v1.3.0` — Produktivitas Harian, Agent Queue & Self-Update

*Fokus: memaksimalkan kenyamanan pemakaian harian dan alur kerja AI/developer memakai infrastruktur yang sudah ada.*

| Item | Isi | Usaha |
| --- | --- | --- |
| **One-Click Self-Updater** | Unduh `.zip` rilis dari GitHub, verifikasi SHA-256 otomatis, ganti biner (`noktah-native.exe` + `noktah-hook.exe`), lalu restart mulus. | 🟡 |
| **Multi-Agent Permission Queue** | Antrean FIFO permintaan izin untuk beberapa sesi agent paralel (`Claude Code` / `Antigravity`), lengkap dengan label workspace. | 🟡 |
| **Halaman Default & Pintasan Cepat** | Pilih tab bawaan saat notch dibuka serta navigasi cepat keyboard (`Alt+1..7`, `Esc`). | 🟢 |
| **Quick AI Actions** | Aksi kilat di tab Ask: terjemahkan clipboard (ID↔EN), ringkas teks, dan jelaskan error memakai kunci DPAPI aktif. | 🟢 |
| **Scratchpad (Catatan Cepat)** | Area catatan instan dengan autosave ter-debounce di samping daftar tugas. | 🟢 |
| **Focus DND & Rekap 7 Hari** | Mode senyap banner non-esensial selama sesi Fokus berlangsung dan statistik fokus mingguan. | 🟢 |

---

## `v1.4.0` — Integrasi Mendalam Windows & Perangkat Keras

*Fokus: kesadaran konteks layar penuh, multi-monitor, dan perangkat eksternal.*

| Item | Isi | Usaha |
| --- | --- | --- |
| **Multi-Monitor & Per-Monitor DPI** | Pilih layar utama/aktif/mengikuti kursor serta penanganan `WM_DPICHANGED` dan `WM_DISPLAYCHANGE` (100%–200%). | 🔴 |
| **Fullscreen Auto-Hide** | Sembunyikan notch otomatis saat game, video, atau presentasi berjalan dalam mode layar penuh. | 🟡 |
| **File Drop Shelf** | Dukungan `IDropTarget` Win32 untuk menampung berkas sementara lewat drag & drop. | 🔴 |
| **HUD Perangkat Audio & Bluetooth** | Notifikasi mikro saat TWS/headset tersambung beserta indikator baterai perangkat via WinRT. | 🔴 |
| **Kalender `.ics` Lokal** | Parse file/URL `.ics` untuk menampilkan pengingat rapat terdekat tanpa OAuth cloud. | 🟡 |

---

## `v2.0.0` — Distribusi Terpercaya & Ekosistem Ekstensibel

*Fokus: distribusi publik tanpa friksi dan ekstensibilitas pihak pengguna.*

- **Code Signing & `winget`** — penandatanganan digital biner rilis di CI untuk menghindari peringatan SmartScreen serta paket `winget` dengan registrasi `PATH` otomatis bagi `noktah-hook.exe`.
- **Custom Status Tile via `noktah-hook`** — skema payload IPC agar skrip pengguna dapat menaruh status/metrik kustom di notch (mis. status CI, container, atau profil daya).
- **Liquid Glass GPU Shader Stabil & Preset Tema** — penyempurnaan mode kaca penuh dan impor/ekspor preset tampilan.

---

## Yang sengaja tidak dikerjakan

- **Kembali ke WebView / Electron / Tauri** — bertentangan dengan target biner tunggal ringan dan memori minimal.
- **Spotify Web API** — Windows SMTC sudah memberi kontrol dan metadata lintas pemutar tanpa OAuth atau token tambahan.
- **Telemetri & Akun Cloud** — aplikasi tidak boleh mengirim data analitik atau menyimpan data pengguna ke server pihak ketiga.
