# Rencana migrasi penuh Noktah ke Rust native

Status: **dikerjakan**
Target: semua fitur versi Tauri (v0.1.5) berjalan di aplikasi Rust native,
dengan RAM jauh lebih rendah dan tanpa WebView.

Gerbang keputusan sudah lewat: 29 MB saat diam, GPU asli
(Intel RaptorLake-S), klik tidak tertelan, panel center.

---

## Arsitektur

Satu crate besar berisi UI dan logika sekaligus sulit diuji — tesnya butuh
jendela. Karena itu dipecah dua:

```
native/
  Cargo.toml            workspace
  core/                 logika murni, tanpa GUI — bisa diuji di Linux
    lrc, lyrics, format, reminders, settings, i18n, pill, weather, stats
  app/                  egui + Win32
    main, theme, tabs/*, platform/*
```

`core` tidak boleh menyentuh `egui` atau `windows`. Semua yang bisa diuji
tanpa layar masuk ke sana, supaya `cargo test` jalan dalam hitungan detik.

---

## Fase

### Fase 0 — Fondasi
- [x] Ubah `native/` jadi workspace `core` + `app`
- [x] Pindahkan model setelan, i18n, format, lrc, reminders, pill ke `core`
- [x] Unit test untuk semua modul `core` (target: ≥40 tes, jalan di Linux)
- [x] `cargo check --target x86_64-pc-windows-msvc` tetap hijau

### Fase 1 — Kerangka UI
- [x] Tema: aksen (amber/mint/violet/ice), opasitas, radius, tipografi
- [x] Bilah tab dengan ikon, bisa diklik, bisa dipindah dengan keyboard
- [x] i18n ID/EN menyeluruh, ganti bahasa tanpa restart
- [x] Setelan disimpan ke `%APPDATA%\Noktah\settings.json` (debounce 400 ms)
- [x] Pill: jam, cuaca, judul lagu, lirik berjalan — tiap bagian bisa dimatikan
- [x] Notch minimal (garis 168×6) sesuai setelan

### Fase 2 — Sistem + Beranda
- [x] CPU (`GetSystemTimes` delta), RAM (`GlobalMemoryStatusEx`), uptime, core
- [x] Riwayat 60 detik untuk sparkline
- [x] Beranda: sapaan sesuai jam + nama, cuaca ringkas, tugas terdekat, CTA fokus

### Fase 3 — Cuaca
- [x] Open-Meteo: geocoding kota + prakiraan 3 hari + per jam
- [x] Cache 10 menit, gagal jaringan tidak mengosongkan data lama
- [x] Pemetaan kode cuaca → ikon dan label ID/EN

### Fase 4 — Tugas + Pengingat
- [x] CRUD tugas, tandai selesai, hapus, urut tenggat
- [x] Tenggat + pengingat: satu timer ke tugas terdekat (bukan interval)
- [x] Banner pengingat di dalam notch + suara sistem
- [x] Simpan ke `tasks.json`

### Fase 5 — Fokus
- [x] Pomodoro 25/5, jeda panjang tiap 4 sesi
- [x] Cincin progres digambar dengan busur
- [x] Sesi selesai → notifikasi + catat statistik harian
- [x] Setelan durasi kerja/istirahat

### Fase 6 — Musik (bagian terberat)
- [x] SMTC lewat WinRT: sesi aktif, judul, artis, album, sumber, status
- [x] Artwork dari `thumbnail` → buffer gambar → tekstur egui
- [x] Kontrol: putar/jeda, sebelumnya, berikutnya, seek
- [x] Lirik LRCLIB: exact → fuzzy → pencarian bebas, skor judul/artis/durasi
- [x] Lirik tersinkron: baris aktif menyala, auto-scroll berhenti 4 detik
      setelah sentuhan manual, klik baris untuk melompat
- [x] Cache lirik di disk, retry 429/5xx
- [x] Pill menampilkan lirik berjalan saat panel tertutup

### Fase 7 — AI
- [x] 6 penyedia (OpenAI, Gemini, OpenRouter, Groq, DeepSeek, Ollama)
- [x] Streaming SSE, tombol berhenti, riwayat percakapan
- [x] Kunci API di Windows Credential Manager, bukan file teks
- [x] Penanda waktu dan salin jawaban

### Fase 8 — Setelan
- [x] Semua grup: Bahasa, Tampilan, Lirik, Pill, Kota, AI, Data, Tentang
- [x] Aksen, opasitas, selalu di atas, mulai terbuka, notch minimal
- [x] Offset lirik, matikan lirik
- [x] Mulai bersama Windows (registry Run)
- [x] Reset data + ekspor/impor setelan

### Fase 9 — Integrasi OS
- [x] Tray icon + menu (Tampilkan, Sembunyikan, Keluar)
- [x] Hotkey global Alt+N
- [x] Satu instance saja (named mutex)
- [x] Mulai otomatis saat login

### Fase 10 — Poles
- [x] Kaca blur — jalur tangkap layar + kotak blur (acrylic DWM dicabut karena membuat jendela ber-alpha jadi kotak hitam)
- [x] Sudut squircle, bukan rounded rect biasa
- [x] Marquee untuk judul panjang
- [x] Hormati "kurangi animasi" Windows

### Fase 11 — Rilis
- [x] Workflow CI membangun MSVC + artifact
- [x] Ganti aplikasi Tauri di `main` setelah semua tab setara
- [x] Tag `v0.2.0`, catatan rilis, dokumentasi

---

## Aturan kerja

1. Tiap fase: tulis → `cargo build` untuk target Windows → commit → lanjut.
2. Fitur yang belum selesai ditandai jujur di UI, bukan tombol mati.
3. Tidak ada kunci API di dalam source atau repo.
4. Tes `core` harus tetap hijau sebelum commit fase berikutnya.
5. Commit memakai format `[AS] type: pesan`.
