# Rencana P1 — empat perbaikan dari analisa fitur

Status: **disetujui dan dikerjakan** (2026-10-26). Keputusan yang diambil:
notch minimal 6 px dengan isi disembunyikan, pengingat berupa pita di dalam
notch (bukan toast Windows), waktu pengingat dipilih lewat jam, dan auto-scroll
lirik berhenti 4 detik setelah baris diklik.

Empat item ini
dipilih karena murah, tidak butuh interop Win32 baru, dan dampaknya langsung
terasa. Halaman unduhan sengaja dikeluarkan sesuai permintaan.

Target rilis: **v0.1.6**, dikerjakan di `develop`, `main` tidak disentuh sampai
kamu bilang rilis.

---

## 1. Klik baris lirik untuk lompat

**Masalah.** Lirik sudah tersinkron dan sudah tahu timestamp tiap baris, tapi
hanya bisa dibaca. Untuk mengulang satu baris, pengguna harus menyeret progress
bar dan menebak.

**Perilaku yang diinginkan.**
- Hanya aktif saat `status === "ready"` (ada timestamp). Lirik polos tetap
  tidak bisa diklik — tidak ada patokan waktu.
- Klik satu baris → pemutaran lompat ke detik baris itu.
- Baris yang bisa diklik memberi tanda saat hover: warna naik, kursor pointer,
  dan `translateX` 2px sebagai isyarat "ini bisa ditekan".
- Klik di baris yang sama dengan baris aktif → tidak melakukan apa-apa
  (menghindari lompatan sia-sia beberapa milidetik).

**Teknis.**
- `useMedia` dapat fungsi baru `seekTo(seconds)`; `seek(ratio)` yang lama tetap
  ada untuk progress bar.
- `MediaTab` menerima `onSeekTo`. Rasio dihitung `detik / (durationMs/1000)`,
  dijepit 0..1.
- **Perhatikan offset lirik.** Kalau `lyricOffsetMs` = +200 ms, lirik digeser
  maju, jadi seek harus memakai `line.time - offset/1000`. Ini titik paling
  mudah salah; akan ada test-nya.
- Durasi 0 (SMTC belum melaporkan) → klik diabaikan.

**Aksesibilitas.** Baris lirik jadi `role="button"` + `tabIndex={-1}`: bisa
diklik dan dibaca pembaca layar, tapi tidak menambah 100+ titik tab. Navigasi
keyboard tetap lewat panah pada kontainer.

**Usaha.** 🟢 ~45 menit termasuk test.

---

## 2. Minimal notch

**Masalah.** Di laptop berbezel tebal, notch setinggi 40 px terlihat seperti
kotak hitam mengambang, bukan poni.

**Perilaku.**
- Saklar baru di Setelan → Tampilan: **"Notch minimal"**.
- Saat panel tertutup: notch jadi garis tipis di tengah atas, tanpa jam, cuaca,
  atau tugas.
- Saat kursor mendekat: tetap membuka penuh seperti biasa. Lirik dan pengingat
  tetap jalan.

**Teknis.**
- Kelas `is-minimal` pada `.notch` hanya ketika `!expanded && settings.minimalNotch`.
- CSS: tinggi 6 px, lebar 168 px, `border-radius` hanya di bawah, isi `.pill`
  `opacity: 0` dan `pointer-events: none`.
- `computePill()` dilewati saat minimal — lebar dipatok, bukan dihitung.
- **Titik rawan:** hit-test kursor di Rust memakai geometri dari
  `set_expanded(expanded, height)`. Kalau lebarnya tidak ikut dikirim, area
  tangkap hover tetap selebar pill lama (193–489 px) sehingga notch terbuka
  padahal kursor masih jauh dari garis. Perlu memeriksa `lib.rs` dan
  mengirim lebar juga.
- Transisi masuk/keluar memakai `--spring-morph` yang sudah ada.

**Usaha.** 🟢 ~30 menit, tapi **tidak bisa aku verifikasi** tanpa Windows
(hit-test ada di sisi Rust).

---

## 3. Start with Windows

**Masalah.** Aplikasi yang gunanya selalu siaga, tapi harus dibuka manual tiap
kali login.

**Perilaku.**
- Saklar di Setelan → Tampilan: **"Jalankan saat Windows menyala"**.
- Status saklar dibaca dari sistem saat aplikasi mulai, bukan dari localStorage
  saja — supaya tidak berbeda kalau pengguna mematikannya lewat Task Manager.

**Teknis.**
- Tambah `tauri-plugin-autostart` v2 di `Cargo.toml`, daftarkan di `lib.rs`,
  izin di `capabilities/default.json`.
- `enable()` / `disable()` saat saklar diubah; `isEnabled()` saat mount.
- Di Windows plugin ini menulis ke `HKCU\...\Run` — tidak butuh admin.
- Kalau plugin gagal (misal di mode dev), saklar kembali ke posisi semula dan
  muncul catatan kecil, bukan gagal diam-diam.

**Usaha.** 🟢 ~30 menit. Perlu build Windows untuk memastikan plugin
ter-compile dan berjalan.

---

## 4. Pengingat tugas

**Masalah.** Tugas hanya daftar. Tidak ada yang mengingatkan saat waktunya tiba,
padahal itu inti dari "tugas".

**Perilaku.**
- Tiap tugas bisa diberi waktu pengingat lewat ikon jam di barisnya
  (`<input type="time">`, default hari ini).
- Saat waktunya tiba: notch menampilkan pita kecil **"⏰ {tugas}"** selama
  beberapa detik, tanpa memaksa panel terbuka.
- Tugas yang waktunya lewat saat aplikasi mati ditandai **"terlewat"** saat
  aplikasi dinyalakan lagi — bukan didiamkan.
- Pengingat yang sudah berbunyi tidak berbunyi ulang.

**Teknis.**
- `Todo` dapat `dueAt?: number` dan `notifiedAt?: number`.
- Hook baru `useReminders`: satu `setTimeout` yang diarahkan ke tugas terdekat,
  dipasang ulang setiap kali daftar berubah. **Bukan** interval 1 detik —
  sejalan dengan target "idle hemat" di roadmap.
- Pita pengingat dirender di `Notch`, di luar panel, supaya tidak mengganggu
  tinggi panel.
- **Keputusan yang perlu persetujuanmu:** pita di dalam notch (tanpa dependensi
  baru, sesuai karakter aplikasi) atau toast Windows (`tauri-plugin-notification`,
  muncul walau aplikasi tidak fokus). Rencana ini memakai pita notch dulu.

**Usaha.** 🟡 ~3 jam termasuk test untuk penjadwalan.

---

## Urutan kerja

| # | Item | Alasan urutan |
| --- | --- | --- |
| 1 | Klik lirik | Tanpa dependensi baru, dampak langsung, mudah diuji |
| 2 | Minimal notch | CSS murni, tapi perlu cek hit-test Rust |
| 3 | Start with Windows | Dependensi baru, perlu verifikasi build Windows |
| 4 | Pengingat tugas | Paling besar, menyentuh model data |

## Kriteria selesai

- `npm test` hijau, termasuk test baru: offset lirik saat seek, penjadwalan
  pengingat (termasuk tugas terlewat), dan `computePill` yang tidak dipakai
  saat mode minimal.
- `npm run build` dan `cargo check --target x86_64-pc-windows-msvc` hijau.
- Keempat saklar tersimpan dan bertahan setelah aplikasi ditutup.
- CHANGELOG `[Unreleased]` terisi; versi tetap `0.1.5` sampai kamu minta rilis.

## Yang tidak bisa aku jamin

Autostart, hit-test notch minimal, dan pengingat saat aplikasi di latar
belakang hanya bisa dipastikan di PC Windows-mu. Aku bisa memastikan kodenya
compile dan logikanya benar lewat test, tapi bukan perilaku runtime Windows.

## Pertanyaan sebelum mulai

1. **Notch minimal:** tinggi 6 px sudah cukup tipis, atau mau 4 px? Isi
   (jam/cuaca) disembunyikan total, atau jam tetap tampil kecil?
2. **Pengingat:** pita di dalam notch, atau toast Windows?
3. **Waktu pengingat:** pilih jam tertentu (14:30), atau durasi cepat
   ("30 menit lagi")? Bisa dua-duanya, tapi menambah kerja.
4. **Setelah klik lirik:** auto-scroll lirik sebaiknya berhenti ~4 detik supaya
   pengguna sempat membaca baris yang dia pilih, atau langsung ikut lagi?
