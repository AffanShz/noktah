# AGENTS.md — Instruksi kerja agen di repo Noktah

Peta arsitektur + aturan kerja untuk agen (Openbot/Claude Code/dll). Tujuannya:
agen tidak perlu `glob`/`grep` buta. Setiap klaim di bawah punya `path:line`.

Repo: `D:\noktah-develop\noktah` · workspace Cargo di `native/` (`core`, `app`, `hook`).
Toolchain wajib **MSVC** `x86_64-pc-windows-msvc` (README.md:178-181).

---

## 0. Aturan keras (baca dulu, jangan dilanggar)

1. **Verifikasi jujur.** Sebelum bilang "selesai": `cd native && cargo test --workspace`
   dan `cargo check --target x86_64-pc-windows-msvc` (CONTRIBUTING.md:113, CI: `.github/workflows/native-preview.yml:43`).
2. **Jangan commit langsung ke `main`** (CONTRIBUTING.md:110). Branch `feat/...`/`fix/...`, PR ke `develop`.
3. **Format commit:** `[AS] type: message`, type = conventional commit (CONTRIBUTING.md:36-48).
4. **Jangan pernah** menulis API key/token ke file mana pun. Kunci AI lewat DPAPI, bukan `settings.json` (native/core/src/settings.rs:1-5).
5. **Jangan sentuh `native/target/`** (build artifact, ribuan file `.rs` hasil `build.rs` crate lain).
6. **Baca file target sebelum mengubah.** Banyak perilaku dikunci komentar panjang yang menjelaskan *kenapa*; mengabaikannya akan mengulang bug yang sudah diperbaiki.
7. **Jangan menulis angka mentah** di UI; pakai token `style.rs` (native/app/src/presentation/widgets.rs:3-5).
8. **Jangan tambah halaman tanpa 5 titik registrasi** (lihat §4) — kalau tidak, halaman tidak muncul / test gagal.

---

## 1. Empat lapisan + arah dependensi

```
presentation/   egui, warna, token          (cara menggambar)
     ↓
application/    use case + trait port        (apa yang boleh terjadi)
     ↓
infrastructure/ adapter nyata: disk, Win32, HTTP, pipe
     ↓
noktah_core     aturan murni, tanpa I/O, tanpa egui
```

Bukti aturan: `native/app/src/main.rs:6-22` (diagram + "panah hanya menunjuk ke dalam"),
`native/app/src/presentation/mod.rs:7-8` ("lapisan ini boleh tahu `application` dan `domain`;
tidak sebaliknya"), `native/app/src/application/ports.rs:1-6` (tidak ada egui/`std::fs`/`windows`
di file port), `native/app/src/infrastructure/mod.rs:1-6` (satu-satunya tempat menyentuh
disk/registry/socket/SMTC), `native/core/src/lib.rs:1-4` (tanpa egui, tanpa windows).

### Siapa boleh impor siapa

| Dari | Boleh impor | Catatan |
|---|---|---|
| `noktah_core` | hanya std/serde/chrono | jangan pernah `egui`/`windows` |
| `application` | `noktah_core`, `ports` | tidak boleh `egui`, tidak boleh `infrastructure` |
| `infrastructure` | `application::ports`, `noktah_core`, `presentation::layout` | implementasi trait |
| `presentation` | `application`, `noktah_core`, `app::Noktah` | **praktiknya juga memanggil `infrastructure` langsung** |

**Kenyataan yang harus diketahui agen:** `presentation` memanggil `infrastructure` langsung
di 25 tempat — mis. `pages/music.rs:368,491`, `pages/home.rs:316-323`, `pages/settings.rs:39,78,527-542`,
`shell/frame.rs:199,419,423`, `shell/persistence.rs:26-49`, `pages/system.rs:52,74`.
Itu **utang arsitektur yang disengaja**, bukan pola yang perlu ditiru untuk logika baru.
Aturan: efek samping *baru* yang punya aturan → taruh di `application` + port + fake;
panggilan OS satu baris dari UI (toggle mute, set always-on-top) masih diterima seperti pola yang ada.

### Modul legacy (belum berlapis)

`native/app/src/main.rs:34-37`: `mod app; mod tabs;` — "setiap penghapusan di daftar ini
adalah satu langkah refactor yang selesai".

- `native/app/src/app.rs` — **jantung state**: struct `Noktah` (app.rs:44-164) + `enum NotchPage`
  (app.rs:167-246). Bukan di `presentation`, tapi presentation mengimpornya.
- `native/app/src/tabs.rs` — hanya `pub use crate::presentation::pages::*;` (tabs.rs:7). Dulu 2025 baris.
- Alias transisi `app_layer`/`infra`/`ui` di main.rs:45-50 (belum dipakai di mana pun).

---

## 2. "Mau ubah X → sentuh file mana"

### 2a. Halaman notch (konten)
- Satu halaman = satu file: `native/app/src/presentation/pages/*.rs` (pages/mod.rs:1-5).
- Pola: `impl Noktah { pub(crate) fn draw_page_x(&mut self, ui: &mut egui::Ui, area: Rect) }`
  — mis. music.rs:138, home.rs:119, tasks.rs:20, focus.rs:18, system.rs:19, ask.rs:21, agent.rs:10.
- Nama file ≠ nama method: `focus.rs` → `draw_page_pomodoro`, `ask.rs` → `draw_page_ai`,
  `tasks.rs` → `draw_page_todo` (grep: `draw_page_` di pages/).
- `pages/seek.rs` **bukan halaman** — helper lompat posisi (seek.rs:1-4), tidak punya varian enum.
- Tata letak bersama: `widgets::page_header` (widgets.rs:827-866, mengembalikan tinggi terpakai),
  `layout::content_rect` (layout.rs:131-136).

### 2b. Dock / chevron / navigasi
- Gambar + interaksi dock: `pages/notch.rs:79-311` (strip 291-310, kapsul 118-209, chevron+gear 211-287).
- Urutan tab: **selalu** dari `NotchPage::SWITCHER` (notch.rs:25-41, app.rs:188-196).
- Siklus chevron: `prev()`/`next()` (app.rs:209-222); halaman di luar SWITCHER kembali ke Music (app.rs:201-206).
- Geometri dock: `layout.rs:58-91` (`DOCK_STRIP_H`, `DOCK_MENU_H`, `DOCK_TAB_W`, `dock_group_w()`,
  `DOCK_LINGER`, `CONTENT_BOTTOM`), `outer_shell_rect` 148, `dock_menu_band` 157, `dock_hover_zone` 165.
- Hover/linger dock dimutasi per-frame di `shell/frame.rs:265-270` + `341-349` (bukan di notch.rs).
- ⚠️ `layout::DOCK_GROUP_W` (layout.rs:82) dihitung untuk **5 tab** padahal SWITCHER berisi 7 —
  konstanta basi, jangan dipakai; produksi memakai `dock_group_w(halaman.len())` (notch.rs:121).

### 2c. Shell morph / geometri notch
- Semua angka bentuk: `presentation/layout.rs` — `notch_rect` 101-121, `radius` 179-184,
  `hit_shape` 188-198, konstanta `PILL_W` 20, `PILL_MAX` 47, `WINDOW_W` 51, `MINIMAL_*` 54-56,
  `OPEN_W/H` 34-35, `OUTER_OPEN_H` 67, `WINDOW_H` 69.
- Fisika/urutan animasi: `presentation/spring.rs:41-85` (`MORPH_OPEN_W/H`, `MORPH_CLOSE_*`,
  `MORPH_MINIMAL_*`, `MORPH_DOCK`, `MORPH_LYRICS`, `SNEAK_PEEK`, `TRANSISI_*`).
- Per-frame: `shell/frame.rs:217-373` (target, hover intent, step/snap pegas), gambar:
  `shell/render.rs:176` `draw_shell`, `295` `draw_panel`.
- Region jendela + klik-tembus: `frame.rs:375-427` → `infrastructure/platform.rs:341` `sync_region`,
  `401` `set_click_through`; pemantau kursor `platform.rs:427` (thread terpisah, karena jendela
  transparan non-aktif tidak menerima event mouse — platform.rs:1-7).
- ⚠️ **Aturan performa:** setiap pegas/animasi baru **wajib** ditambahkan ke dua tempat:
  daftar `animating` (frame.rs:434-444) dan `next_wake` (frame.rs:45-128). Lupa → animasi beku
  atau aplikasi 60 fps selamanya. Contoh penjelasan: frame.rs:22-34 (`lyric_line_animating`).
- Reduce motion: frame.rs:358-373; sumber `os::animations_enabled()` (os.rs:460) + `settings.force_animation`.

### 2d. Token desain (spasi, radius, tipografi, warna)
- Skala: `presentation/style.rs` — `S1..S4` 26-32, `PAD` 36, `CARD_PAD` 39, `R_*` 47-57,
  `concentric()` 61, `T_*` 68-84, helper `text/cap/title/heading/page_title` 102-170.
- Warna: `presentation/theme.rs` — `Theme` 150-176, `Theme::new` 179, `with_accent` 221,
  `legible_accent` 139-147 (aksen gelap dicampur putih agar kontras ≥ ~6:1).
- Komponen: `presentation/widgets.rs` (daftar fungsi: `toggle` 18, `row` 67, `row_desc` 82,
  `section` 60, `chip_button` 119, `pill` 350, `segmented` 517, `page_header` 827,
  `spark_tile` 875, `stat_tile` 967, `progress_ring` 1001, `marquee_title` 1082).
- Aturan tertulis: spasi kelipatan 4, empat ukuran huruf, radius konsentris (style.rs:9-19);
  "jangan menulis angka mentah di sini" (widgets.rs:3-5).
- `Theme::new` dipanggil **setiap frame** (frame.rs:451) — struct kecil, jangan di-cache.

### 2e. Setelan baru
Lihat langkah lengkap §5.

### 2f. Adapter OS / jaringan / worker
- Kontrak: `application/ports.rs` — `Store` 16, `SecretStore` 37, `Network` 49, `SystemInfo` 92,
  `SystemActions` 102, `MediaSource` 112, `Clock` 129, agregat `Deps` 141, nama file 164-170.
- Implementasi nyata: `infrastructure/*.rs` (mod.rs:8-22). Pola worker:
  `pub fn spawn(ctx: egui::Context) -> XHandle` + `ctx.request_repaint()` — ai.rs:30, audio.rs:37,
  ipc.rs:331, lyrics_client.rs:108, music.rs:32, quota.rs:524, weather.rs:16, platform.rs:427.
- Hasil worker ditarik di UI thread: `shell/sync.rs` (`poll_media` 21, `poll_ai` 125,
  `poll_quota` 184, `poll_weather` 195, `poll_ipc` 255), dipanggil frame.rs:481-485.
- Handle di-spawn di `Noktah::new` (app.rs:313-328, 355, 365).
- Versi palsu **wajib** ditambah di `infrastructure/fake.rs` (lihat §3).
- OS Windows: `infrastructure/os.rs` (autostart 8, power 102, tray/hotkey 414, single instance 427,
  trim 198, animations 460), `infrastructure/platform.rs` (hover 122-160, posisi 253, region 341,
  click-through 401, window_rect 565). Semua punya stub non-Windows (mis. os.rs:56, platform.rs:117).

### 2g. Logika murni
- `native/core/src/*.rs` + `pub mod` di lib.rs:6-13. Yang ada: `format`, `i18n`, `lrc`, `lyrics`,
  `model`, `pill`, `reminders`, `settings`.
- Kriteria pindah ke core: "kalau keputusan bisa diuji tanpa membuka jendela" (presentation/mod.rs:3-5).

### 2h. Hook Claude Code (halaman Agent)
- Biner: `native/hook/src/main.rs` — `main` 142; baca JSON stdin 145-166; nama pipe per-SID 29-66;
  fail-open `exit(0)` bila Noktah mati 215-219; frame protokol 222-224; timeout 150 ms / 110 s
  untuk `PermissionRequest` 236.
- Server: `native/app/src/infrastructure/ipc.rs` — `PIPE_BUF_SIZE` 35, `IpcEvent` 41-51,
  `IpcHandle` 54, `get_user_pipe_name` 74, `spawn` 331; panjang payload `u32` LE (ipc.rs:221).
- Payload: `noktah_core::model::AgentHookPayload` (model.rs:321-347), status `AgentStatus` (263),
  state `AgentState` (307).
- Konsumsi: `sync.rs:255-410` — auto-decay 45 s (259-267), match event 269-357, permission 358-390.
- UI: `pages/agent.rs:10` `draw_page_agent`.
- Protokol: **4 byte panjang LE + JSON UTF-8**, balasan opsional (ACK 0-byte / JSON keputusan),
  batas balasan 10 MB (hook.rs:222-250). Menambah field cukup di `AgentHookPayload` dengan
  `#[serde(default)]` — biner hook meneruskan JSON mentah, tidak perlu diubah kecuali semantik izin.

---

## 3. Test: di mana diletakkan, peran `fake.rs`, cara menghindari I/O

**Tiga jenis, tiga lokasi:**

1. **Inline `#[cfg(test)] mod tests` di dasar file** — mayoritas. 8 file core, 11 file app.
   Contoh: `core/src/i18n.rs:210`, `core/src/settings.rs:490`, `app/src/application/tasks.rs:132`,
   `app/src/presentation/pages/home.rs:536`, `app/src/infrastructure/quota.rs:585`.
   Aturan: test milik modul itu, memakai `use super::*`.
2. **File harness terpisah, didaftarkan di `main.rs:39-43`:**
   - `app/src/layout_test.rs` (1133 baris) — mengukur `Rect` sungguhan lewat `egui::__run_test_ui`
     (layout_test.rs:1-7, 43-58), termasuk semua 9 halaman presisi (297-400) dan pegas lirik.
   - `app/src/page_fit_test.rs` (324 baris) — anggaran tinggi halaman vs `CONTENT_BOTTOM`
     (page_fit_test.rs:1-10, `page_needs` 39-60, 11 test).
   - Kapan pakai yang mana: geometri/hit-test → `layout_test`; anggaran vertikal → `page_fit_test`;
     anggaran satu halaman yang punya angka sendiri → inline di file halaman (home.rs:536-560).
3. **Test use case** — di modul use case, dengan fake (`application/tasks.rs:132-225`, 8 test).

**Peran `fake.rs`:** `app/src/infrastructure/fake.rs` menyediakan implementasi in-memory untuk
semua port — `FakeStore` 20, `FakeSecret` 64, `FakeNetwork` 81, `FakeSystem` 123, `FakeActions` 152,
`FakeMedia` 175, `FakeClock` 186. Ditaruh di `infrastructure` (bukan modul test) supaya use case
bisa mengimpornya di `#[cfg(test)]` tanpa menarik `std::fs`/Win32 (fake.rs:1-6).
`FakeStore::written()` (fake.rs:53-58) memeriksa file apa yang benar-benar ditulis.

**Cara menghindari I/O:** use case generik atas trait (`TaskList<S: Store>`, tasks.rs:17-22),
jadi test menyuntik `FakeStore` dan tidak pernah menyentuh disk — lihat `tersimpan_dan_dimuat_ulang`
(tasks.rs:216-225). Test yang butuh waktu memakai `FakeClock`, bukan `Instant::now()`.
UI diuji dengan harness egui, bukan dengan membuka jendela (layout_test.rs:3-7).

**Jumlah saat ini:** 81 `#[test]` di `native/core`, 77 di `native/app` (grep `#\[test\]`).

---

## 4. Menambah halaman notch — langkah lengkap

Halaman punya **dua kategori**:
- **SWITCHER** (bisa dicapai dock + chevron): 7 halaman (app.rs:188-196).
- **Sekunder** (tidak di SWITCHER; `prev/next` memperlakukannya sebagai "kembali ke Music"):
  `NowPlaying`, `Water` (app.rs:201-206). **Keduanya saat ini tidak bisa dibuka dari UI** —
  tidak ada satu pun penugasan `current_page = NotchPage::Water/NowPlaying` di kode produksi
  (hanya digambar oleh test). Kalau menambah halaman sekunder, sediakan pemicunya sendiri;
  jangan mengandalkan chevron.

### Langkah (semua wajib)

1. **Enum** — `app.rs:167-177`: tambah varian + diskriminan.
2. **SWITCHER** — `app.rs:188-196`: tambah ke daftar **dan ubah tipe array** `[NotchPage; 7]`
   menjadi jumlah baru. `COUNT` (199) otomatis ikut; jangan hardcode jumlah tab di tempat lain
   (layout.rs:70-73 memperingatkan ini).
3. **Ikon** — `app.rs:233-245`: tambah lengan `icon()`. Ikon tersedia di `presentation/icons.rs`.
4. **Modul** — `pages/mod.rs:7-18`: `pub mod namafile;`.
5. **File halaman** — `pages/<namafile>.rs` dengan `impl Noktah { pub(crate) fn draw_page_x(...) }`.
   Salin pola header + `page_header` dari tasks.rs:1-32; hitung tinggi dari `area`/`CONTENT_BOTTOM`.
6. **Label dock** — `notch.rs:28-38`: lengan `match` label (string ini masih hardcoded Inggris).
7. **Dispatch** — `notch.rs:67-77`: lengan `match self.current_page`.
8. **layout_test** — `layout_test.rs:318-328` (array halaman) dan `364-382` (lengan match).
9. **page_fit_test** — `page_fit_test.rs:39-60`: tambah entri `page_needs()` dan ubah tipe
   `[(&str, f32); 7]`; kalau halaman tidak memakai `page_header`, buat test inline seperti
   home.rs:553-560.
10. **i18n (opsional tapi disarankan)** — label/tab di `core/src/i18n.rs:9-187`.

**Halaman sekunder** (tidak muncul di dock): lewati langkah 2, tapi **tetap** lakukan 1, 3, 4, 5, 6,
7, 8. Kalau tidak, match jadi non-exhaustive → build gagal.

⚠️ Jangan lupa: `draw_page_*` yang dipanggil `notch.rs` harus `pub(crate)`/`pub` dan menerima
`(&mut self, ui, area)`; nama method **tidak** harus sama dengan nama file.

---

## 5. Menambah setelan baru end-to-end

1. **Field struct** — `core/src/settings.rs:225-289`. Struct memakai `#[serde(default)]` di level
   container (settings.rs:224), artinya field yang hilang di JSON lama memakai
   `Default` **tipe**, bukan `Settings::default()`.
   → Untuk bool yang bawaannya `true`, **wajib** `#[serde(default = "default_true")]`
   (contoh: settings.rs:228, 259, 270, 277-284). Tanpa itu pengguna lama diam-diam dapat `false`.
   Untuk nilai khusus, buat fn kecil seperti `default_hover_delay()` (settings.rs:264, 216-219).
2. **Default** — `settings.rs:291-334`: isi nilai awal.
3. **Sanitize** (kalau ada rentang) — `settings.rs:472-487`: clamp + isi string kosong.
   Test pola: settings.rs:707-720.
4. **i18n** — `core/src/i18n.rs:9-187`: tambah `("settings.kunci", "ID", "EN")` + pasangan `_desc`.
   Test otomatis menolak kunci kosong/duplikat (i18n.rs:214-229). Lookup `i18n::t` 191-203;
   kunci tak dikenal mengembalikan `"(?)"` — itu sinyal kunci salah.
5. **UI** — `pages/settings.rs`: pilih tab yang sesuai
   (`draw_general_tab` 30, `draw_display_tab` 70, `draw_appearance_tab` 122, `draw_music_tab` 187,
   `draw_mini_row_settings` 246, `draw_todo_tab` 390, `draw_pomodoro_tab` 430, `draw_ai_tab` 484,
   `draw_about_tab` 512), lalu `widgets::row_desc(ui, tr(label), tr(desc), &theme, |ui| { ... })`
   dan `widgets::toggle(...)` — contoh persis: settings.rs:94-104.
   Setiap perubahan **wajib** memanggil `self.touch_settings()` (persistence.rs:16-23):
   itu menyegarkan `lang`/`theme`, menyalin `minimal_notch` ke atomic `Hover`, dan menjadwalkan
   simpan 400 ms (persistence.rs:46-53).
6. **Kalau menambah tab setelan** (bukan sekadar field): varian `SettingsTab` (app.rs:33-42),
   array ikon tab (render.rs:70-79), dan lengan match (settings.rs:17-26).
7. **Titik baca** — pakai `self.settings.<field>` di frame/sync/mini/pages. Kalau memengaruhi
   hover/minimal, salin ke atomic `Hover` (frame.rs:388-399) karena pemantau kursor di thread lain.
8. **Kalau memengaruhi geometri** — ubah konstanta `layout.rs`, lalu sesuaikan `page_fit_test.rs`.

⚠️ Jangan menyimpan rahasia di `Settings`. Kunci API ada di `SecretStore` (ports.rs:32-40,
implementasi `infrastructure/secret.rs`).

---

## 6. Batas ukuran / aturan arsitektur yang tertulis di kode

Tidak ada batas LOC numerik yang dinyatakan. Yang tertulis adalah aturan bentuk:

| Aturan | Bukti |
|---|---|
| Satu halaman = satu file (dulu `tabs.rs` 2025 baris dipecah) | `tabs.rs:3-5`, `pages/mod.rs:1-5` |
| Modul legacy hanya boleh menyusut, bukan bertambah | `main.rs:34-35` |
| Spasi kelipatan 4, hanya 4 ukuran huruf, radius konsentris | `style.rs:9-19` |
| Tidak ada angka mentah di komponen; semua dari `style` | `widgets.rs:3-5` |
| Satu sumber geometri agar UI dan hit-test tidak berbeda | `layout.rs:1-2`, `layout.rs:93-100` |
| Jumlah tab tidak boleh di-hardcode; pakai `SWITCHER.len()`/`dock_group_w(n)` | `layout.rs:70-79`, `notch.rs:112,121` |
| Satu sumber urutan halaman untuk dock dan chevron | `app.rs:180-187` |
| Isi halaman berakhir di `CONTENT_BOTTOM`; tinggi cadangan dock tidak boleh ikut pegas | `layout.rs:123-136` |
| Angka test terpisah dari kode halaman; tambah baris → sesuaikan anggaran | `page_fit_test.rs:8-10` |
| Setiap animasi baru masuk `animating` + `next_wake` | `frame.rs:434-444`, `frame.rs:45-128` |
| Setiap pegas punya alasan tertulis (kenapa bukan boolean) | `app.rs:94-96,137-141`, `layout.rs:98-100` |
| Write file atomik (tmp + rename), file rusak disalin `.rusak` | `infrastructure/store.rs:1-4,29-51` |
| Field setelan baru harus aman untuk JSON lama | `settings.rs:224`, `default_true` 216 |

---

## 7. Jebakan yang sudah terverifikasi (jangan ulangi)

1. **Komentar basi** — `app.rs:147` masih menulis "0: Music, 1: Tasks, 2: Pomodoro, 3: System",
   padahal ada 9 halaman. Percayai `SWITCHER` (app.rs:188), bukan komentar itu.
2. **`DOCK_GROUP_W` basi** — dihitung untuk 5 tab (layout.rs:82); SWITCHER 7. Jangan dipakai.
3. **`NotchPage::from_index` tidak dipakai** (app.rs:224, satu-satunya kemunculan) — kandidat hapus.
4. **`store::wipe` tidak lengkap** — daftar file di store.rs:60 berisi `focus.json` (tidak pernah
   ditulis) dan **tidak** berisi `water.json`/`pomodoro.json`. Reset di settings.rs:541-545 juga
   hanya membersihkan todos/chat. Hati-hati saat menyentuh fitur reset.
5. **`tab_settings` adalah wrapper mati** — settings.rs:552-561 hanya memanggil
   `draw_settings_tab_content`; komentar "Legacy flat settings panel" menyesatkan.
6. **i18n belum menyeluruh** — label dock (notch.rs:29-37) dan beberapa deskripsi setelan
   (mis. settings.rs:97) masih hardcoded Inggris/Indonesia. Jangan asumsikan semua teks punya kunci.
7. **`presentation` → `infrastructure`** sudah terjadi 25×; jangan "perbaiki" massal tanpa diminta,
   dan jangan tambah pemanggilan baru untuk logika yang punya aturan.
8. **`Water` dan `NowPlaying` adalah halaman yatim** — punya enum, ikon, modul, dan test, tapi
   tidak ada jalur UI yang menetapkan `current_page` ke sana. Jangan bingung mencari tombolnya.
9. **Jangan pakai `WS_EX_LAYERED`** untuk transparansi — membuat kotak hitam; alpha ditangani
   `platform::ensure_alpha_visible` (platform.rs:74-115) + `sync_region`.

---

## 8. Perintah

```bash
cd native
cargo test --workspace                                  # README.md:199
cargo check --target x86_64-pc-windows-msvc             # CONTRIBUTING.md:113
cargo run --release --target x86_64-pc-windows-msvc     # README.md:185-195
cargo build --release --target x86_64-pc-windows-msvc   # CI: native-preview.yml:47
```

Hasil build: `native/target/x86_64-pc-windows-msvc/release/noktah-native.exe` +
`noktah-hook.exe` (README.md:206-212). CI memeriksa `VERSIONINFO` — build tanpa resource ditolak
(README.md:220-225).

---

## 9. Ringkasan satu layar

```
State & halaman        app.rs (Noktah, NotchPage, SWITCHER)
Gambar tiap halaman    presentation/pages/<x>.rs      (impl Noktah)
Dock & chevron         presentation/pages/notch.rs
Bentuk & angka         presentation/layout.rs
Per-frame & pegas      presentation/shell/frame.rs + presentation/spring.rs
Shell/panel/setelan UI presentation/shell/render.rs + pages/settings.rs
Warna/tipografi        presentation/theme.rs + style.rs
Komponen               presentation/widgets.rs
Use case & port        application/{tasks,ports}.rs
Adapter nyata          infrastructure/*.rs        (satu file per port)
Fake                   infrastructure/fake.rs
Aturan murni           core/src/*.rs              (tanpa egui/windows)
Hook Claude Code       hook/src/main.rs  ↔  infrastructure/ipc.rs  →  sync.rs:255
Test harness           layout_test.rs, page_fit_test.rs  (main.rs:39-43)
```
