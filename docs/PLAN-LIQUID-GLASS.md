# Rencana implementasi "Liquid Glass Notch"

Sumber: `noktah_liquid_glass_notch_ui.html` (2.469 baris, prototipe Tailwind + JS).
Tujuan: memindahkan bahasa desainnya ke aplikasi Rust native (egui) yang sudah
ada, tanpa kehilangan perilaku yang sudah berjalan.

---

## 1. Apa yang sebenarnya ada di file itu

Bukan sekadar mockup — ini prototipe yang bisa diklik, dengan state machine,
geometri elastis, dan tujuh tab berisi data contoh.

### Bahasa desain

| Lapisan | Detail |
|---|---|
| Permukaan kaca | gradien 165° tiga titik (30,38,54 → 13,17,26 → 18,25,38) dengan alfa 0,72–0,82 |
| Blur latar | `backdrop-filter: blur(32px) saturate(190%) contrast(105%)` |
| Pantulan tepi | empat `inset box-shadow` putih tipis: bawah 0,28 / kiri 0,14 / kanan 0,14 |
| Cahaya bawah | garis 1,5 px gradien transparan → cyan 0,45 → putih 0,65 → cyan → transparan |
| Kilau | `::before` radial putih 0,12 di tengah atas, menutupi 200% area |
| Telinga cair | SVG 28×28, kurva `M28 28V0H0C15.464 0 28 12.536 28 28Z` — sudut cekung yang menyambung ke tepi layar |
| Sudut | atas 0 (menempel layar), bawah 38 px saat terbuka, 24 px saat mengecil |

### Tiga keadaan

1. **Minimized** — tinggi tetap 44 px, lebar **elastis** 160–720 px dihitung dari
   item mana yang aktif.
2. **Extended** — 530×465, sudut bawah 38 px, isi panel penuh.
3. **Detached** — varian mengambang (top 16 px, telinga hilang, radius 38 px semua sisi).

### Baris minimized (7 item, bisa dinyalakan/dimatikan)

| Item | Ukuran | Isi |
|---|---|---|
| `art` | 28 px | sampul bulat, berputar 12 detik/putaran |
| `title` | ≤175 px | judul tebal + "—" + artis warna sky |
| `lyrics` | 245 px (310 jika judul mati) | chip ♪ + satu baris lirik, animasi slide-in |
| `controls` | 58 px | tombol putar dan berikutnya |
| `wave` | 34 px | 4 bar equalizer, tinggi berdenyut |
| `clock` | 68 px | chip jam monospace |
| `progress` | 2 px | garis di tepi bawah, gradien cyan |

Pembatas kaca otomatis disembunyikan kalau salah satu sisinya kosong.
Empat preset: **Standar**, **Mode Lirik**, **Fokus Lirik Saja**, **Lengkap**.

### Panel terbuka

Header (ikon kilau + "Noktah" + badge DND + tombol "Mini Standar"/"Mini + Lirik" +
badge bahasa + jam) → bilah tab dengan **indikator pil yang meluncur** → isi tab →
toast di atas panel.

### Fitur yang belum ada di versi native kita

- Acak dan ulangi pada pemutar
- Filter tugas (semua / aktif / selesai) + hapus yang selesai
- Preset pomodoro (fokus/istirahat, durasi berbeda)
- Ganti kota cuaca dengan satu klik
- Suara ambient (white noise + filter, untuk menemani fokus)
- Badge DND, ganti mode jam (nyata/demo), quick controls

---

## 2. Pemetaan CSS → native

| CSS | Pendekatan di egui | Risiko |
|---|---|---|
| `linear-gradient` latar | mesh 4 sudut dengan warna per vertex | rendah |
| `inset box-shadow` | garis tepi 1 px mengikuti rounding, digambar 4 sisi | rendah |
| garis cahaya bawah | mesh gradien horizontal 5 titik | rendah |
| kilau radial `::before` | 12–16 lingkaran konsentris dengan alfa menurun | sedang (banding) |
| telinga SVG | `egui::Mesh` manual: sampel bezier jadi 24 titik, triangulasi kipas | sedang |
| `transition` width/height/radius | spring per properti (sudah ada) | rendah |
| `filter: blur(2px)` pada teks | tidak ada di egui — ganti opacity + geser 4 px | rendah (aproksimasi) |
| font Plus Jakarta Sans / JetBrains Mono | bundel TTF (OFL), daftarkan di `FontDefinitions` | rendah |
| toggle switch, slider, tombol kaca | widget kustom dengan `painter` + `interact` | rendah |
| equalizer | 4 rect, tinggi = sinus(fase + offset) | rendah |
| indikator pil tab | rect dengan spring x/width di belakang label aktif | rendah |
| **`backdrop-filter: blur(32px)`** | **lihat bagian 3** | **tinggi** |

---

## 3. Keputusan tersulit: blur latar

Ini satu-satunya bagian yang tidak bisa ditiru langsung, dan menentukan
seberapa "kaca" hasil akhirnya. Tiga jalur:

### A. DWM acrylic — `SetWindowCompositionAttribute(ACCENT_ENABLE_ACRYLICBLURBEHIND)`

- **Kelebihan:** satu panggilan API, blur asli, tanpa beban CPU, sudah termasuk
  tint dan noise bawaan Windows.
- **Masalah:** blur berlaku untuk **seluruh persegi jendela**, bukan bentuk notch.
  Jendela kita 560×588 sementara notch lebih kecil — area transparan ikut
  diburamkan. Solusinya: jendela diubah ukurannya mengikuti bentuk notch tiap
  frame. Itu menimbulkan biaya resize saat animasi dan bisa berkedip.
- **Catatan:** di banyak laporan, acrylic tidak bekerja bersama `WS_EX_LAYERED`
  (yang kita pakai untuk transparansi per piksel).

### B. Tangkap layar + blur sendiri

- **Cara:** `BitBlt` area di belakang notch → perkecil ¼ → box blur 3 lintasan →
  naikkan saturasi → jadikan tekstur latar panel.
- **Kelebihan:** sepenuhnya terkendali, tetap per-piksel-alpha, bentuk apa pun.
- **Masalah:** hasil tangkapan **ikut memuat jendela kita sendiri** (dari frame
  sebelumnya), dan menyembunyikan jendela sesaat akan berkedip. Bisa dikurangi
  dengan menangkap saat panel baru dibuka saja, lalu dibekukan selama terbuka.

### C. Tanpa blur asli

- Translucent gelap + gradien + noise halus. Di wallpaper gelap hasilnya sudah
  mendekati; di wallpaper terang jelas terlihat "tempelan".
- **Kelebihan:** nol risiko, nol biaya.

### Rekomendasi

**Spike 2 jam di awal, sebelum menulis UI apa pun:**

1. Buat jendela uji 300×200, coba acrylic. Kalau blur muncul **dan** transparansi
   per piksel tetap benar → pakai A.
2. Kalau gagal → coba B dengan pembekuan gambar saat panel dibuka.
3. Kalau keduanya gagal → C, dan desainnya disesuaikan (gelap lebih pekat,
   tambah noise dan vignette supaya tidak terlihat datar).

Jangan menulis seluruh UI dulu baru menguji blur — itu kesalahan yang mahal.

---

## 4. Fase implementasi

### Fase 1 — Fondasi visual (1 hari)
- [ ] Bundel font Plus Jakarta Sans (400/500/600/700) + JetBrains Mono, daftarkan ke egui
- [ ] Modul `theme`: token warna kaca, radius, spasi, skala tipografi
- [ ] `glass.rs`: gambar permukaan kaca lengkap (gradien, tepi specular, cahaya bawah, kilau)
- [ ] Panel memakai permukaan baru; tab lama tetap jalan
- **Selesai bila:** panel terlihat seperti kaca meski blur masih C

### Fase 2 — Bentuk dan telinga (0,5 hari)
- [ ] `ears.rs`: mesh telinga kiri/kanan, ikut mengecil saat morph
- [ ] Radius atas 0 saat menempel, 38 px saat mengambang
- [ ] Mode detached (setelan `floating`)
- **Selesai bila:** sudut atas menyambung mulus ke tepi layar tanpa celah

### Fase 3 — Baris minimized elastis (1 hari)
- [ ] Tujuh item dengan animasi muncul/hilang (lebar + opacity)
- [ ] Hitung lebar elastis 160–720 px seperti `updateMinimizedGeometry`
- [ ] Pembatas otomatis sembunyi
- [ ] Empat preset + setelan per item di tab Setelan
- [ ] Equalizer berdenyut, sampul berputar, lirik slide-in
- **Selesai bila:** mematikan judul melebarkan lirik, seperti di prototipe

### Fase 4 — Kerangka panel terbuka (1 hari)
- [ ] Header baru: ikon, DND, dua tombol mini, badge bahasa, jam
- [ ] Bilah tab dengan indikator pil meluncur (spring)
- [ ] Tombol kaca, switch kaca, slider kaca
- [ ] Toast menggantikan banner biasa
- **Selesai bila:** perpindahan tab terasa seperti prototipe

### Fase 5 — Blur (1–2 hari, tergantung hasil spike)
- [ ] Terapkan jalur yang lolos spike
- [ ] Fallback otomatis kalau gagal (deteksi: bandingkan hasil, atau setelan manual)
- [ ] Setelan "efek kaca: penuh / ringan / mati"
- **Selesai bila:** panel memperlihatkan isi layar di belakangnya, terblur

### Fase 6 — Fitur baru dari prototipe (2–3 hari)
- [ ] Acak & ulangi pada tab Musik
- [ ] Filter tugas + hapus yang selesai
- [ ] Preset pomodoro + durasi kustom
- [ ] Ganti kota cuaca cepat
- [ ] Suara ambient (white noise + low-pass) untuk menemani fokus
- [ ] Badge DND (menahan toast saat fokus)
- **Selesai bila:** semua kontrol di prototipe punya pasangan yang berfungsi

### Fase 7 — Poles (1 hari)
- [ ] Animasi slide-in lirik saat baris berganti
- [ ] Transisi isi panel (scale 0,92 + geser) saat morph
- [ ] Hormati "kurangi animasi" Windows
- [ ] Uji di 100% / 125% / 150% DPI
- **Selesai bila:** tidak ada teks terpotong di ketiga skala

---

## 5. Yang sengaja tidak ditiru

| Di prototipe | Alasan |
|---|---|
| Orb ambient & latar kotak-kotak | hanya alat demo untuk menguji transparansi |
| `filter: blur()` pada teks | egui tidak punya blur teks; diganti opacity + geser |
| `backdrop-filter: saturate(190%)` | hanya mungkin kalau blur sendiri yang dipakai (jalur B) |
| Tailwind CDN | tidak relevan di native |

---

## 6. Perubahan model data

`Settings` bertambah:

```rust
// baris minimized
pub mini_art: bool,
pub mini_title: bool,
pub mini_lyrics: bool,
pub mini_controls: bool,
pub mini_wave: bool,
pub mini_progress: bool,
pub mini_clock: bool,
// tampilan
pub floating: bool,          // mode mengambang, bukan menempel
pub glass_effect: GlassEffect, // Full | Light | Off
pub reduce_motion: bool,     // ikut setelan Windows, bisa ditimpa
// fitur
pub shuffle: bool,
pub repeat: Repeat,          // Off | All | One
pub ambient_sound: bool,
```

Semua dengan `#[serde(default)]` supaya setelan lama tetap terbaca.

---

## 7. Urutan kerja

1. Spike blur (2 jam) — **keputusan sebelum menulis UI**
2. Fase 1 → 2 → 3 → 4 (visual, tanpa fitur baru)
3. Fase 5 (blur) sesuai hasil spike
4. Fase 6 (fitur baru)
5. Fase 7 (poles + DPI)

Setiap fase: `cargo build` untuk target Windows → uji di mesinmu → commit.
Tes `core` harus tetap hijau (66 tes sekarang).

---

## 8. Perkiraan

| Fase | Perkiraan |
|---|---|
| Spike blur | 2 jam |
| 1–4 (visual) | 3,5 hari |
| 5 (blur) | 1–2 hari |
| 6 (fitur) | 2–3 hari |
| 7 (poles) | 1 hari |
| **Total** | **8–10 hari kerja** |

Risiko terbesar ada di Fase 5. Kalau blur tidak bisa diwujudkan, sisa 7 hari
tetap berjalan — hanya hasilnya "kaca gelap" bukan "kaca tembus pandang".
