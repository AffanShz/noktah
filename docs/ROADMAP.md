# Roadmap

Dokumen kerja, bukan janji. Urutan ditentukan oleh **dampak harian ÷ usaha**,
bukan oleh seberapa keren fiturnya.

Legenda usaha: 🟢 beberapa jam · 🟡 beberapa hari · 🔴 butuh interop Windows
tingkat rendah dan pengujian di perangkat asli.

---

## Sudah selesai

- **Lirik** — LRCLIB dengan pencarian bertingkat dan retry 429/5xx.
- **Bahasa Indonesia/English** — kamus terpusat di `native/core/src/i18n.rs`.
- **Aplikasi Rust native** — digambar sendiri dengan `egui`/`eframe`, tanpa WebView.
- **Test** — 157 test Rust: 81 di `native/core` (lirik, LRC, format, i18n,
  model, setelan, pill, pengingat) dan 76 di `native/app` (geometri, tema,
  tipografi, mini player). Dijalankan di CI pada tiap push ke `native/`.
- **Gerak ala Apple** — kurva spring sungguhan, diimplementasikan di
  `native/app/src/presentation/spring.rs` ([riset](MOTION.md)).
- **Halaman Agent** — status Claude Code dan persetujuan izin langsung dari
  notch, lewat biner pendamping `native/hook`.
- **Halaman Hidrasi** dan **Now Playing**.

---

## P0 — fondasi sebelum menambah fitur

Kalau ini belum ada, setiap fitur baru menambah beban perawatan.

| Item | Kenapa penting | Usaha |
| --- | --- | --- |
| **Auto-update** | Tanpa ini, setiap perbaikan berarti pengguna harus unduh ulang installer. Ini syarat minimum sebuah app desktop bisa dirawat. Butuh signing key + secret di repo; mekanismenya harus ditulis sendiri karena aplikasi ini tidak lagi memakai Tauri. | 🟡 |
| **Modul on/off** | 9 halaman sekarang; menambah fitur tanpa saklar akan membuat UI penuh. Sekalian jadi dasar "halaman default". | 🟢 |
| **Log lokal + halaman diagnosa** | Kalau pengguna melaporkan "musik tidak muncul", sekarang tidak ada cara melihat kenapa. Cukup tulis log ke `%APPDATA%\Noktah\noktah.log`, tampilkan 200 baris terakhir di Setelan. | 🟢 |
| **Test UI native** | 76 test di `native/app` masih berpusat pada geometri, tema, dan tipografi; interaksi panel dan alur persetujuan Agent belum diuji. | 🟡 |

## P1 — nilai harian, usaha kecil

| Item | Isi | Usaha |
| --- | --- | --- |
| **Pengingat kesehatan** | Minum, istirahat gerak, jeda mata 20-20-20. Semua hanya butuh timer + notifikasi. | 🟢 |
| **Day progress** | Seberapa jauh hari ini berjalan, ditampilkan di Beranda. | 🟢 |
| **Terjemahan cepat** | Infrastruktur AI sudah ada; tinggal tab dengan input + Ctrl+Enter. | 🟢 |
| **Catatan cepat** | Satu textarea, autosave. Melengkapi Tugas. | 🟢 |
| **Tab default** | Buka notch langsung ke tab favorit. | 🟢 |

## P2 — fitur berat, nilai tinggi

| Item | Isi | Usaha |
| --- | --- | --- |
| **Lirik** | Butuh sumber lirik (LRCLIB gratis) + sinkronisasi ke posisi SMTC. Bisa dikerjakan tanpa interop baru. | 🟡 |
| **Visualizer** | Butuh capture audio loopback (WASAPI) lalu FFT. Berat, dan menambah konsumsi CPU saat musik jalan. | 🔴 |
| **Kalender iCal** | Parse file/URL `.ics`, tampilkan rapat berikutnya. Google/Outlook OAuth ditunda. | 🟡 |
| **File shelf** | Drag & drop ke notch, simpan sementara, zip, konversi gambar. Di aplikasi native ini butuh interop `IDropTarget` Win32 sendiri. | 🔴 |
| **Bluetooth & baterai perangkat** | Butuh WinRT `Windows.Devices.Bluetooth` + `Enumeration`. | 🔴 |
| **Wi-Fi** | Butuh `WlanAPI`, dan di Windows 11 menuntut izin lokasi. Sensitif secara privasi. | 🔴 |

## P3 — eksperimen, jangan dikerjakan sebelum P0–P2 rapi

- **Multi-monitor**: satu notch per layar. Butuh refactor penempatan jendela.
- **Posisi bebas**: notch bisa digeser, tidak selalu tengah atas.
- **Snap zones**: butuh manajemen jendela Win32.
- **Maskot 3D**: memproyeksikan maskot 2D procedural yang sudah ada ke ruang 3D.

---

## Utang teknis yang diketahui

1. **Test UI native belum merata.** Logika murni sudah tertutup rapi (81 test di
   `native/core`) dan `native/app` punya 76 test, tapi interaksi panel dan alur
   persetujuan Agent belum diuji.
2. **Polling kursor.** Region hover disegarkan dari thread UI tiap frame, bukan
   dari thread pemantau — ini sudah menggantikan polling 40 ms yang lama. Sisa
   utangnya: memastikan tidak ada polling saat idle total.
3. **Statistik sistem dipoll berkala** meski panel tertutup; idealnya hanya saat
   halaman Sistem dibuka.
4. **Belum ada isolasi panic per halaman.** Satu panic di dalam `draw` akan
   menjatuhkan seluruh proses, bukan hanya halaman itu.
5. **Belum ada code signing**, jadi SmartScreen selalu memperingatkan.

## Yang sengaja tidak dikerjakan

- **Spotify Web API** — SMTC sudah memberi kontrol penuh tanpa OAuth, client ID,
  dan refresh token. Menambahkannya hanya menambah permukaan kegagalan.
- **Telemetri** — app ini tidak boleh mengirim data apa pun tanpa izin eksplisit.
- **Akun & sinkronisasi cloud** — bertentangan dengan premis "data tinggal di PC".
