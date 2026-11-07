# Gerak (Motion) — riset dan rencana

> **Catatan status.** Dokumen ini ditulis pada era frontend React + CSS
> (v0.1.5) dan dipertahankan sebagai catatan riset. Aplikasi sekarang menggambar
> sendiri dengan egui, jadi istilah CSS di bawah (`linear()`, `will-change`,
> `PerformanceObserver`) tidak lagi menggambarkan implementasi berjalan:
> solver spring-nya kini ada di `native/app/src/presentation/spring.rs`.
> Bagian 9 tetap berguna karena menyimpan hasil pengukuran aslinya.

Dokumen kerja. Hasil riset Apple HIG + WWDC23 "Animate with springs", ditambah
pengukuran langsung pada Noktah 0.1.5. **Belum ada kode yang diubah.**

---

## 1. Kondisi sekarang (terukur)

Diukur di preview Chromium, transisi buka panel 1,1 detik:

| Metrik | Nilai | Arti |
| --- | --- | --- |
| Properti yang dianimasikan | `width, height, border-radius` | **semuanya memicu layout tiap frame** |
| Kurva | `cubic-bezier(0.22, 1, 0.36, 1)` | ease-out biasa, tidak bisa overshoot |
| Durasi | `0.46s / 0.46s / 0.38s` | tetap, tidak peduli jarak |
| `will-change` | `auto` | tidak dipromosikan ke layer sendiri |
| p50 frame | 16,7 ms | 60 fps di antara jeda |
| **p95 frame** | **33,4 ms** | 1 frame hilang |
| **frame terburuk** | **50 ms** | 3 frame hilang sekaligus — inilah yang terasa "patah" |
| Frame > 20 ms | 3 dari 60 | 5% frame jelek |

Jadi keluhannya benar dan bisa dijelaskan: **3 frame jank + satu jeda 50 ms**
di tengah morph. Penyebabnya bukan durasi, tapi properti yang dianimasikan.

---

## 2. Kenapa animasi Apple terasa berbeda

Dari WWDC23 *Animate with springs* dan HIG:

1. **Apple memakai spring, bukan kurva waktu.** Sejak iOS 17, `withAnimation`
   default-nya spring. Spring punya dua parameter saja: `duration` dan `bounce`.
2. **Velocity dipertahankan saat animasi diinterupsi.** Kalau target berubah di
   tengah jalan, spring memakai kecepatan saat itu sebagai kecepatan awal.
   Inilah yang membuat gerakan terasa "hidup" — dan ini **tidak mungkin** dengan
   `cubic-bezier`, karena kurva tidak menyimpan state.
3. **`cubic-bezier` hanya bisa overshoot sekali.** Spring berosilasi: lewat
   target, balik, lewat lagi lebih kecil, lalu diam. Butuh banyak titik.
4. **Bentuk bermorph, isi di-cross-fade.** Dynamic Island tidak "membuka" dengan
   melebarkan kotak berisi teks; bentuknya berubah dulu, teks muncul belakangan.
5. **Bounce itu sedikit.** Nilai acuan Apple: `bounce = 0` untuk hampir semua UI,
   `0.3` sudah terasa jelas, `> 0.4` berlebihan.
6. **Reduce Motion dihormati.** Bukan dimatikan total, tapi diganti cross-fade.

---

## 3. Lima masalah teknis di Noktah

| # | Masalah | Akibat yang terlihat |
| --- | --- | --- |
| 1 | `width`/`height` dianimasikan | layout dihitung ulang tiap frame → p95 33 ms |
| 2 | `cubic-bezier` untuk morph | tidak ada overshoot/settle, terasa mekanis |
| 3 | Tidak ada penanganan interupsi | hover masuk-keluar cepat terasa tersentak |
| 4 | Durasi tetap | morph 96px→300px dan 300px→489px terasa sama |
| 5 | Tidak ada `will-change` | tidak ada layer compositor |

---

## 4. Rancangan

### 4.1 Token spring

Mengikuti penamaan Apple, dengan `damping ratio ζ = 1 − bounce`:

| Token | duration | bounce | ζ | Dipakai untuk |
| --- | --- | --- | --- | --- |
| `--spring-snappy` | 0.5 s | 0.15 | 0.85 | buka/tutup notch, hover pill |
| `--spring-smooth` | 0.5 s | 0 | 1.0 | panel, tab, daftar |
| `--spring-bouncy` | 0.5 s | 0.30 | 0.7 | hanya umpan balik kecil (centang tugas) |

Jarak memendekkan durasi: `durasi = base × clamp(0.45, 0.35 + 0.65·(jarak/300px), 1)`.
Morph 96→300 px jadi lebih cepat daripada 96→489 px, seperti di iOS.

### 4.2 Tiga lapis implementasi

| Lapis | Alat | Kapan dipakai |
| --- | --- | --- |
| A | CSS `linear()` berisi sampel spring | transisi satu arah yang tidak perlu diinterupsi (fade, chip, hover) |
| B | Solver spring di JS + rAF, menulis CSS variable | morph notch — wajib bisa diinterupsi & sadar velocity |
| C | Web Animations API (`composite: "add"`) | animasi yang perlu dibatalkan/di-*reverse* di tengah |

`linear()` didukung Chromium 113+, jadi aman untuk WebView2 modern
(WebView2 sudah jauh di atas itu). Fallback: `cubic-bezier` yang sama seperti
sekarang.

### 4.3 Morph notch tanpa layout

Kunci perbaikannya: **jangan animasikan ukuran, animasikan `transform`.**

```
shell  : ukuran tetap (mis. 489×540), transform: scale(sx, sy), origin: top center
radius : border-radius: calc(var(--radius) / var(--s))   ← kompensasi skala
isi    : opacity 0 → 1, translateY 4px → 0, mulai di 40% durasi
```

Karena isi masih transparan saat bentuk berubah, distorsi teks akibat
`scale` tidak terlihat — sama seperti Dynamic Island. Hanya `transform` dan
`opacity` yang berubah, keduanya compositor-only.

Solver (semi-implisit Euler, ~30 baris):

```ts
const w = (2 * Math.PI) / duration;   // ω
const k = w * w;
const c = 2 * damping * w;
// per frame
const a = -k * (x - target) - c * v;
v += a * dt;
x += v * dt;
// berhenti kalau |x-target| < 0.5px dan |v| < 0.5px/s
```

### 4.4 Aksesibilitas

`prefers-reduced-motion: reduce` → durasi 0, ganti dengan cross-fade 120 ms.
Bukan dimatikan, karena hilangnya umpan balik juga membingungkan.

---

## 5. Rencana bertahap

### Tahap 1 — fondasi (tanpa mengubah tampilan)

- [ ] `src/lib/spring.ts`: solver + `springTo()` yang mengembalikan handle
      `{ retarget, cancel }`, plus konversi `bounce → ζ`.
- [ ] `src/lib/spring.test.ts`: uji numerik — nilai akhir konvergen ke target,
      tidak pernah overshoot saat `bounce = 0`, overshoot < 2% saat `bounce = 0.15`,
      `retarget` mempertahankan kecepatan (tanda kecepatan tidak berbalik).
- [ ] `src/lib/motion.ts`: token + fungsi `durationForDistance()`.
- [ ] Tambah `@media (prefers-reduced-motion: reduce)` di CSS.

### Tahap 2 — morph notch

- [ ] Shell notch: ukuran tetap + `transform: scale()` + radius kompensasi.
- [ ] Isi panel: cross-fade berjenjang (`opacity` + `translateY`).
- [ ] `will-change: transform` hanya selama animasi (dilepas saat selesai).
- [ ] Hover masuk-keluar cepat: `retarget()` dipanggil, bukan restart.

### Tahap 3 — sisanya

- [ ] Pill: ganti lebar animasi ke `scaleX` + isi di-cross-fade.
- [ ] Tab: transisi masuk `translateY(6px)` + opacity, spring `smooth`.
- [ ] Daftar tugas: tambah/hapus pakai FLIP, bukan tinggi animasi.
- [ ] Bar progres & ring fokus: biarkan linear (sesuai saran Apple — progres
      tidak boleh memantul).

---

## 6. Kriteria selesai

Terukur, bukan "kelihatan halus":

| Metrik | Sekarang | Target |
| --- | --- | --- |
| p95 frame | 33,4 ms | **≤ 16,8 ms** |
| Frame terburuk | 50 ms | **≤ 20 ms** |
| Frame > 20 ms | 3 dari 60 | **0** |
| Properti dianimasikan | width/height | hanya `transform`, `opacity` |
| Interupsi | restart | lanjut dengan velocity |
| Reduced motion | tidak ada | cross-fade 120 ms |

Alat ukur: `PerformanceObserver` dengan `long-animation-frame` (Chromium 123+)
untuk melihat waktu *style & layout* per frame, bukan hanya selisih rAF —
supaya perbaikan tidak bisa "terlihat bagus" karena kebetulan mesinnya cepat.

---

## 7. Risiko

1. **`scale` mendistorsi bayangan & border.** Perlu kompensasi radius dan
   bayangan (`box-shadow` ikut terskala). Mitigasi: bayangan dipindah ke
   pseudo-element berskala terbalik.
2. **Teks buram saat scale.** Diatasi karena isi transparan selama morph;
   kalau tetap terlihat, pakai `will-change` dan hindari `filter` di dalam.
3. **`linear()` tidak ada di WebView2 lama.** Windows 10 dengan WebView2 usang
   bisa tidak mendukung; fallback `cubic-bezier` wajib ada.
4. **Solver JS menambah satu rAF loop permanen.** Harus berhenti total saat
   idle — kalau tidak, kita menukar jank dengan baterai.

---

## 9. Hasil implementasi

**Koreksi diagnosis awal.** Pengukuran pertama menyimpulkan `width`/`height`
sebagai penyebab jank. Setelah diuji ulang dengan A/B (mengubah class langsung
vs lewat React), ternyata **animasi CSS-nya sendiri sudah 16,7 ms rata**.
Lonjakan 50 ms hanya terjadi pada **pembukaan pertama** setelah halaman dimuat —
biaya mount awal, font, dan decode gambar — bukan biaya per-animasi.
Jadi masalah sesungguhnya bukan properti, melainkan **kurvanya**.

Yang diimplementasikan:

- `src/lib/spring.ts` — solver spring (oscillator teredam) + `springCss()` yang
  menghasilkan `linear()` berisi 24 titik sampel. 15 unit test.
- `src/lib/motion.ts` — token `morph` (0,42 s / bounce 0,12),
  `morphOut` (0,34 s / bounce 0,05), `smooth`, `bouncy`; ditulis ke `:root`
  sekali saat aplikasi mulai.
- `.notch` memakai `var(--spring-morph)` untuk buka dan `--spring-morphOut`
  untuk tutup, dengan fallback `cubic-bezier` bila `linear()` tak didukung.
- Cross-fade isi panel dipercepat (opacity 0,2 s mulai 0,06 s) supaya isi
  tidak muncul saat bentuk masih setengah jalan.

Hasil terukur (Chromium, jalur React, spring aktif):

| Metrik | Sebelum | Sesudah |
| --- | --- | --- |
| Buka: p95 | 33,4 ms | **16,7 ms** |
| Buka: frame terburuk | 50 ms | **16,8 ms** |
| Buka: frame > 20 ms | 3 | **0** |
| Tutup: p95 | 33,4 ms | **16,8 ms** |
| Tutup: frame > 20 ms | — | 1 (33,4 ms) |

Sisa satu frame jatuh saat menutup masih ada; kandidat perbaikan berikutnya
adalah mematikan `backdrop-filter` lebih awal pada jalur tutup.

**Yang belum dikerjakan** dari rencana awal: morph berbasis `transform: scale`
(tahap 2 di atas) tidak jadi dipakai, karena setelah diukur tidak diperlukan —
properti ukuran bukan penyebab jank. Prinsip "jangan optimalkan yang bukan
masalah" lebih penting daripada mengikuti rencana secara harfiah.


## 8. Sumber

- Apple HIG — Motion: <https://developer.apple.com/design/human-interface-guidelines/motion>
- WWDC23 — *Animate with springs*: <https://developer.apple.com/videos/play/wwdc2023/10158>
- Catatan WWDC23 (durasi perseptual, velocity saat retarget):
  <https://wwdcnotes.com/documentation/wwdc23-10158-animate-with-springs>
- Nilai preset `.smooth` / `.snappy` / `.bouncy` dan panduan bounce:
  <https://github.com/GetStream/swiftui-spring-animations>
- `spring(response:dampingFraction:)` bawaan 0.5 / 0.825:
  <https://developer.apple.com/documentation/swiftui/animation/spring(response:dampingfraction:blendduration:)>
- Spring lewat CSS `linear()`: <https://modern-css-layouts.com/css-only-micro-interactions-animations/easing-and-motion-design/linear-easing-function-for-springs>
- Reduced Motion: <https://developer.apple.com/help/app-store-connect/manage-app-accessibility/reduced-motion-evaluation-criteria/>
