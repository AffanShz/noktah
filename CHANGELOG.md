# Changelog

Format mengikuti [Keep a Changelog](https://keepachangelog.com/id/1.1.0/),
versi mengikuti [SemVer](https://semver.org/lang/id/).

## [1.4.2] - 2026-11-22

### Diperbaiki
- **Autostart Gagal Saat Boot (`C:\Windows\System32`)** — Proses yang dijalankan Windows lewat registry `Run` mewarisi working directory `C:\Windows\System32`, sehingga penulisan file relatif gagal. Directory kerja kini dinormalisasi ke folder executable di awal `main()`.
- **Pencekalan Task Manager Tidak Terhapus** — Menyalakan kembali *Start with Windows* kini ikut menghapus nilai disabled di `HKCU\...\Explorer\StartupApproved\Run`. Sebelumnya Windows tetap menolak menjalankan Noktah saat boot meski entri `Run` sudah ada, karena status disabled dari Task Manager tidak pernah disinkronkan.
- **Status Autostart Salah Baca** — `autostart_enabled()` kini memeriksa byte status `StartupApproved` (hanya untuk nilai bertipe `REG_BINARY`), jadi sakelar di Setelan mencerminkan keadaan sebenarnya.
- **Prefix Verbatim `\\?\` di Path Registry** — Path dari `current_exe()` dibersihkan dari prefix `\\?\` sebelum ditulis sebagai nilai `Run`, karena sebagian shell boot gagal mem-parse-nya.
- **Build Test Mendaftarkan Dirinya Sebagai Autostart** — Konstruktor `Noktah::new` menyinkronkan registry `Run` setiap start, dan harness layout test memanggil konstruktor yang sama. Akibatnya setiap `cargo test` menulis path binary test (`target\debug\deps\noktah_native-<hash>.exe`) ke registry pengguna, lalu Windows menjalankan test suite saat boot. Sinkronisasi dipindah ke `main.rs` lewat `os::sync_autostart` yang dijaga `cfg!(test)` sekaligus deteksi nama binary test; `set_autostart` juga menolak path tersebut.
- **Pembersihan `StartupApproved` Sebelum Penulisan Berhasil** — Nilai disabled kini hanya dihapus setelah penulisan `Run` benar-benar sukses. Dulu penghapusan berjalan walau penulisan gagal, menyisakan entri lama dalam keadaan aktif dengan path basi.
- **Handle Registry Bocor** — Path executable disiapkan sebelum kunci `Run` dibuka, sehingga kegagalan `current_exe()` tidak lagi meninggalkan handle registry terbuka.

### Diubah
- **Test Regresi Autostart** — Dua test baru: deteksi nama binary test dan penegasan bahwa jalur test tidak menulis registry. Total 191 unit test (91 `noktah-core` + 100 `noktah-native`), naik dari 189.

## [1.4.1] - 2026-11-20

### Diperbaiki
- **Deteksi Layar Penuh (Fullscreen) Akurat & Bebas False Positive** — Menyaring dan mengabaikan jendela aplikasi standar yang dimaksimalkan (*maximized*, seperti Chrome, VS Code, File Explorer) melalui pengecekan gaya Win32 `WS_MAXIMIZE` dan `WS_CAPTION`. Supresi auto-hide kini hanya berlaku pada game atau video layar penuh (F11) sejati yang menutupi monitor tanpa titlebar/caption standar.
- **Pencegahan Notch Hilang Saat Minimize** — Memastikan notch Noktah dalam keadaan minimize (`open.target == 0.0`) selalu tampak dan stabil di atas layar tanpa mewajibkan kursor diarahkan (*hover*) terlebih dahulu.
- **Isolasi Monitor Jendela Aktif** — Memastikan deteksi layar penuh hanya memeriksa jendela yang berada pada monitor yang sama dengan jendela Noktah (`fg_mon == notch_mon`), mencegah jendela di monitor sekunder menyembunyikan notch di monitor utama.
- **Pengecualian Kelas Shell & Desktop Windows 11** — Menambahkan filter pencegahan supresi untuk desktop dan shell host Windows 11 (`Shell_SecondaryTrayWnd`, `XamlExplorerHostIslandWindow`, `SysListView32`, dll).
- **Ambang Gestur Peek Lebih Responsif** — Melonggarkan ambang batas *peek intent* menjadi `<= 10.0 px` dari tepi atas monitor agar notch lebih mudah dimunculkan kembali saat pengguna memainkan game layar penuh.
- **Setelan Bawaan Aman (Opt-In)** — Mengubah nilai default `fullscreen_autohide` menjadi `false` agar notch tidak pernah bersembunyi secara tidak terduga pada instalasi awal.

## [1.4.0] - 2026-11-19

### Ditambahkan
- **Kalender `.ics` Lokal (Zero Cloud Dependency)** — Parser RFC 5545 murni Rust (`noktah-core::calendar`) fail-safe tanpa ketergantungan library eksternal; mendukung pembacaan agenda rapat/jadwal lokal dari `%APPDATA%\Noktah\calendar.ics` atau path custom, ekstraksi `VEVENT` (`DTSTART`, `DTEND`, `SUMMARY`, `LOCATION`), serta integrasi agenda terdekat di kartu Home.
- **Fullscreen Auto-Hide & Peek Intent** — Deteksi cerdas jendela aktif (foreground) berukuran layar penuh via Win32 `GetForegroundWindow` & `rcMonitor`; otomatis menyusutkan/menyembunyikan notch ke atas layar saat bermain game atau memutar video layar penuh tanpa mengganggu pengguna, dengan kemampuan peek intent saat kursor diarahkan ke 4 px tepi atas layar.
- **Multi-Monitor & DPI Awareness** — Dukungan penempatan notch pada layar utama (`Primary`) maupun mode dinamis mengikuti kursor (`CursorFollow` via `MonitorFromPoint`), lengkap dengan penanganan scaling DPI dan rekonsiliasi posisi instan saat berpindah monitor.
- **File Drop Shelf (Rak Berkas Sementara)** — Rak penampung berkas sementara di tab Tasks yang menangkap berkas yang di-drag & drop langsung ke area notch (`ctx.input raw.dropped_files`), menampilkan pratinjau berkas (nama, ukuran, ikon), tombol salin path clipboard, buka di File Explorer, dan bersihkan shelf.
- **HUD Notifikasi Audio & Headset** — Deteksi otomatis pergantian default audio render endpoint via Windows Core Audio (`IMMDeviceEnumerator`), memunculkan banner HUD mikro animasi di notch saat headset/speaker beralih dengan proteksi Focus DND.
- **Setelan Baru di Settings → Window** — Penambahan toggle saklar `Fullscreen Auto-Hide`, pemilih `Screen Placement` (`Layar Utama` / `Ikuti Kursor`), `Audio Device HUD`, `File Drop Shelf`, dan `Kalender .ics Lokal`.
- **Test Suite Lanjutan** — Penambahan unit test parser RFC 5545, unfolding baris panjang, penapisan agenda waktu relatif, logika auto-hide fullscreen & peek intent, pembatasan kuota rak berkas, serta isolasi banner audio pada mode Focus DND (total 189 unit test lulus).

## [1.3.0] - 2026-11-18

### Ditambahkan
- **One-Click In-App Self-Updater** — Pengunduhan arsip `-win64.zip` resmi GitHub Releases dari Settings → About, verifikasi otomatis SHA-256 terhadap `BACA-DULU.txt`, ekstraksi staging ke folder lokal, penggantian biner `noktah-native.exe` + `noktah-hook.exe`, dan restart otomatis aplikasi.
- **Multi-Agent FIFO Permission Queue** — Mengganti slot tunggal dengan antrean FIFO `VecDeque<QueuedPermission>`, label antrean `(1/N)` pada pill minimal & modal Agent, serta label sesi/workspace (`session_title`) otomatis dari argumen `--title` atau nama folder kerja (`cwd`) di `noktah-hook`.
- **Halaman Default & Pintasan Keyboard Cepat** — Pengaturan `default_page` di Settings → General (*Last opened*, *Home*, *Music*, *Focus*, *Tasks*, *System*, *Ask*, *Agent*) serta pintasan `Alt+1..7` untuk berpindah tab dock dan `Esc` untuk menutup panel.
- **Quick AI Actions di Halaman Ask** — Tombol aksi kilat satu klik (`ID↔EN`, `Ringkas`, `Jelaskan`, `Bersihkan`) yang mengambil teks dari kolom input atau clipboard Windows melalui kunci DPAPI aktif.
- **Scratchpad / Catatan Cepat di Halaman Tasks** — Toggle `Tugas | Catatan` beserta tombol `Copy` dan autosave ter-debounce 400 ms ke `notes.json` (ikut dibersihkan pada *Reset all data*).
- **Focus Mode DND, Rekap Fokus 7 Hari & Home Day Progress** — Opsi `focus_dnd` di Settings → Pomodoro untuk menahan banner pengingat dan intip lagu saat timer Fokus aktif, indikator `DND` di pill/Fokus, arsip riwayat fokus 7 hari terakhir (`weekly_total()`), serta persentase & hairline *Day Progress* pada kartu Home.
- **Setelan Kembali ke Mode Idle saat Musik Dijeda (`mini_idle_on_pause`) & Single-Instance Handoff** — Pilihan untuk menciutkan pill kembali ke tampilan jam & Noki saat audio dijeda atau tidak aktif (termasuk handoff antar instance aplikasi lewat `Local\NoktahSingleInstance`).

### Diubah & Diperbaiki
- **Penghapusan Tombol Settings Ganda di Halaman `SYSTEM`** — Menghapus tombol chip `Settings` di kanan atas header halaman System karena ikon Settings sudah tersedia permanen di dock navigasi.
- **Unit & Layout Test Tambahan** — Penambahan test untuk `day_progress_pct`, riwayat fokus 7 hari + `Scratchpad`, `DefaultPage` & `focus_dnd`, parser & verifikator SHA-256 self-updater, pembersihan `notes.json`, `mini_idle_on_pause`, serta alur antrean FIFO multi-agent.

## [1.2.2] - 2026-11-18

### Ditambahkan
- **Logger Diagnostik Lokal Berputar (`logs/noktah.log`)** — Pencatatan otomatis event startup, panic, error AI, fallback lirik, error kuota, dan pengecekan pembaruan dengan rotasi maksimal 256 KB ke `noktah.old.log`.
- **Bagian Diagnostik & Log di Settings → About** — Tombol *Buka Folder Data*, *Lihat/Sembunyikan Log* (200 baris terakhir + `crash.log`), *Segarkan*, *Salin Log*, dan *Bersihkan Log*.
- **Konfirmasi Reset Data 2-Klik** — Tombol *Hapus semua data* kini meminta konfirmasi klik kedua dalam jeda 5 detik sebelum menghapus data.
- **Gate Kualitas di CI Release (`release-native.yml`)** — Menambahkan verifikasi konsistensi versi lintas 3 `Cargo.toml` + tag rilis serta menjalankan `cargo test --workspace` sebelum build rilis.
- **Roadmap Multi-Versi (`docs/ROADMAP.md`)** — Pembaruan peta jalan pengembangan terstruktur untuk `v1.3.0`, `v1.4.0`, dan `v2.0.0`.

### Diperbaiki
- **Bug Penulisan Disk Setiap Frame (`tasks.json`)** — `check_reminders()` di `persistence.rs` kini hanya menulis ke disk ketika ada status pengingat yang benar-benar berubah, menghentikan penulisan file berulang setiap frame.
- **Efisiensi Idle CPU & Polling Sistem** — Gating wakeup maskot 30 fps di `next_wake()` dan pembaruan `sysinfo` hanya saat elemen terkait terlihat atau sedang aktif; penambahan unit test `next_wake_hemat_saat_tertutup_tanpa_maskot`.
- **Kelengkapan Pembersihan Data (`store::wipe`)** — Menghapus berkas persisten nyata (`settings.json`, `tasks.json`, `chat.json`, `pomodoro.json`, `settings-export.json`, `secret.bin`) dan folder cache `lyrics/`, serta mengosongkan state `pomodoro`, `api_key`, dan lirik di memori.
- **Lokalisasi Antarmuka & Manifest Dinamis** — Memindahkan label dock, notifikasi selesai Pomodoro, reset harian, serta bagian Pembaruan/Data di About ke `i18n.rs`, dan menyelaraskan versi manifest XML di `build.rs` dengan `CARGO_PKG_VERSION`.

### Dihapus
- **Pembersihan Dead Code & Modul Transisi** — Menghapus berkas `native/app/src/tabs.rs`, alias modul transisi di `main.rs`, metode `NotchPage` yang tak terpakai, wrapper mati `tab_settings`, serta konstanta usang `DOCK_GROUP_W`.

## [1.2.1] - 2026-11-16

### Ditambahkan
- **Integrasi Hook Antigravity (`agy` IDE/CLI)** — Dukungan event `PreInvocation`, `PostInvocation`, dan normalisasi payload `toolCall` pada `noktah-hook`, disertai tombol salin konfigurasi `hooks.json` di tab AI Assistant (Settings).
- **Pembaca Kuota Lokal `agy-quota.json`** — Dukungan membaca snapshot kuota lokal dari direktori aplikasi atau `%APPDATA%\Noktah\agy-quota.json` sebelum fallback ke `9Router`.
- **Toggle Visibilitas Navigasi Dock** — Pengaturan untuk menampilkan/menyembunyikan halaman dock (`nav_home`, `nav_music`, `nav_pomodoro`, `nav_tasks`, `nav_system`, `nav_ask`, `nav_agent`).

### Diubah
- **Label Status Agent Generik** — Judul halaman dan status bar/pill diubah dari `"Claude Code"` menjadi `"Agents status"` / `"Agent sedang memproses..."` / `"Agent Berpikir..."` agar netral lintas agen.
- **Penyederhanaan Halaman Dock** — Menghapus modul `now_playing` dan `water` yang tidak lagi digunakan.

## [1.2.0] - 2026-11-15

### Ditambahkan
- **In-App Update Checker** — Pemeriksa pembaruan otomatis terintegrasi GitHub Releases API di tab Tentang (Settings) dengan deteksi versi, catatan rilis (changelog), dan link download rilis langsung.
- **HUD Banner Notch** — Notifikasi visual notch pill yang berekspansi mikro (340 px) saat sesi Pomodoro selesai atau pengingat hidrasi tiba, serta floating toast di dasar panel terbuka.
- **Aktivasi Modul Water & NowPlaying di Switcher** — Siklus dock lengkap 9 halaman (Home, Music, NowPlaying, Pomodoro, Water, Tasks, System, Ask, Agent) dengan rotasi memutar dan ikon khusus.
- **Background Ticker & Auto-Reset Harian Hidrasi** — Reset otomatis kuota air minum harian dan sesi Pomodoro di tengah malam kalender lokal, disertai pengingat hidrasi berkala.
- **Pengaturan Durasi Pomodoro Dinamis** — Dukungan kustomisasi menit fokus dan istirahat pada slider pengaturan yang langsung tersinkronisasi ke timer aktif.
- **Local Crash & Panic Logger** — Pencatatan panic hook otomatis ke berkas `%APPDATA%\Noktah\logs\crash.log`.

### Diperbaiki
- **Eliminasi Glitch Putih Secondary Viewport** — Pemindahan eksekusi render secondary viewport (Settings) ke akhir frame pass sehingga tidak memotong rendering Glow GL context notch utama.
- **Polesan Gaze Tracking Maskot Noki saat Idle** — Integrasi lerp smoothing pada tatapan mata maskot di miniatur maupun kartu, serta penambahan wandering gaze organik saat kursor diam santai.
- **Pembersihan Dead Fields & Personalisasi To-Do** — Penghapusan field compiler warning yang tak terpakai (`todo_due`, `applied_glass`) dan penyambungan nama pengguna `user_name` ke judul header dan salam tugas.

## [1.1.0] - 2026-11-14

### Ditambahkan
- **Maskot Noki Interaktif & Universal Gaze Tracking** — Avatar superellipse ekspresif dengan pelacakan kursor global lintas aplikasi, respons dinamis terhadap musik/lirik, serta animasi squish, pusing (dizzy), kedip alami, dan tidur otomatis (>45 detik tidak aktif).
- **Redesain Antarmuka Setelan** — Panel pengaturan modern berbasis secondary viewport native (*always-on-top*) dengan sidebar berkategori rapi (General, Display, Appearance, Music & Lyrics, Tasks, Pomodoro, AI Assistant, About).
- **Fallback Multi-Provider Lirik** — Integrasi pencarian lirik tersinkronisasi lintas penyedia (LRCLIB, KuGou, NetEase) sebelum jatuh ke fallback lirik teks polos.

### Diperbaiki
- **Slot Lirik Dinamis Anti-Tabrakan** — Batas slot lirik di baris compact dihitung dinamis di antara komponen kiri dan kanan, dilengkapi pemotongan rapi (ellipsize) dan clipping agar teks lirik panjang tidak pernah menabrak maskot Noki maupun kontrol.
- **Evaluasi Binary Search Lirik Polos** — Memperbaiki penanganan lirik tanpa timestamp (`time < 0.0`) agar tidak terkunci di baris terakhir dan tidak menarik scroll paksa ke dasar.
- **Tampilan Status Claude Code di Compact Notch** — Pembersihan identifier panjang MCP (misal `mcp__terminal__...` disederhanakan) serta penataan layout agar label status agen tidak tumpang-tindih dengan avatar atau equalizer.
- **Auto-Minimize Setelah Approval** — Panel notch otomatis menutup rapi setelah pengguna menekan tombol persetujuan izin pada halaman Agent, dilengkapi pencegahan stuck hover state.

## [1.0.0] - 2026-11-08

Rilis pertama. Aplikasi ini adalah overlay notch untuk Windows 10/11 yang
digambar sendiri dengan `egui`/`eframe` — tanpa WebView, satu exe, tanpa runtime
tambahan. Nomor versi dimulai ulang dari 1: versi 0.x sebelumnya milik aplikasi
Tauri + React yang sudah tidak ada di repo ini.

### Halaman
- **Beranda** — jam besar, tanggal Indonesia, lagu yang sedang diputar, dan sisa
  kuota Antigravity dari 9Router.
- **Musik** — pemutar dari SMTC: sampul, judul/artis/album, progres bisa diklik,
  play/pause, next/prev, dan lirik tersinkron dari LRCLIB.
- **Now Playing** — tampilan mini satu kolom, posisi tombol transport identik
  dengan halaman Musik.
- **Tugas** — tambah/centang/hapus, dengan pengingat berjam yang memunculkan
  pita di bawah notch selama 8 detik. Tugas yang terlewat ditandai.
- **Fokus** — Pomodoro dengan cincin progres dan hitungan sesi harian.
- **Hidrasi** — pencatat minum harian.
- **Sistem** — cuaca, CPU, dan memori sebagai tiga kartu dengan sparkline.
- **Tanya** — chat AI streaming enam penyedia (Gemini, OpenAI, Groq, OpenRouter,
  DeepSeek, Ollama), kunci dienkripsi DPAPI.
- **Agent** — status Claude Code dan persetujuan izin langsung dari notch, lewat
  biner pendamping `noktah-hook.exe`.

### Perilaku notch
- Notch minimal 168×6 px saat tertutup; hover membuka panel penuh.
- Tinggi panel menyesuaikan isi halaman dengan animasi pegas sungguhan
  (solver di `native/app/src/presentation/spring.rs`).
- Hit-test kursor: area transparan di bawah notch tidak menelan klik milik
  aplikasi lain.
- Baris minimized elastis dengan tujuh item yang bisa dinyalakan/dimatikan,
  empat preset, dan lirik yang bergulir saat teks tidak muat.
- Tray icon + menu, hotkey **Alt+N**, autostart, dan satu instance.
- Menghormati setelan "kurangi animasi" Windows.

### Teknis
- Workspace Cargo: `native/core` (logika murni), `native/app` (UI dan adapter
  Windows), `native/hook` (biner pendamping).
- 157 test Rust (`cargo test --workspace`), dijalankan di CI sebelum build.
- Cuaca lewat Open-Meteo tanpa API key; media lewat Windows SMTC, jadi Spotify,
  YouTube Music, dan VLC terbaca tanpa plugin.
- Data pengguna tinggal di `%APPDATA%\Noktah`. Tidak ada telemetri.

### Diketahui belum
- Mode kaca Penuh (tangkap layar + blur) masih eksperimental.
- Belum ada code signing, jadi SmartScreen dan pemindai antivirus bisa
  memperingatkan saat pertama dijalankan. Paket rilis menyertakan SHA-256.

## [0.1.5] — 2026-10-26

- Tab Musik kembali ke layout vertikal: sampul 96px + meta sebaris, progres, kontrol, lalu blok lirik selebar panel di bawahnya.
- Setelan dikelompokkan ulang (Bahasa, Tampilan, Lirik, Pill, Kota, AI, Data, Tentang); pemilih penyedia AI yang dobel dihapus.
- Pengaturan bahasa Indonesia/English untuk seluruh antarmuka, termasuk format tanggal.

## [0.1.4] — 2026-10-26

### Diperbaiki
- **Banyak lirik dilaporkan "tidak tersedia" padahal ada di LRCLIB.** Tiga sebab:
  - `/api/get` menuntut durasi hampir identik — "Runtuh" (Feby Putri) 404 hanya
    karena selisih 8 detik dari entri database.
  - `album_name` ikut dikirim, padahal album dari SMTC sering kosong atau beda
    edisi; parameter itu justru membuat permintaan gagal.
  - Balasan 429/503 (server sibuk) diperlakukan sama dengan 404, sehingga
    gangguan sementara tampil sebagai "lirik tidak ada".
- Pencarian sekarang bertingkat: exact match → fuzzy search → query bebas,
  dengan skor berbasis judul, artis utama, dan selisih durasi, plus retry
  untuk 429/5xx.
- Lirik tanpa timestamp tetap ditampilkan (status `plain`), tidak lagi
  dianggap kosong.

## [0.1.3] — 2026-10-26

### Ditambahkan
- **Lirik di pill.** Saat lagu diputar, notch kecil menampilkan baris lirik yang
  sedang aktif di kanan judul lagu. Lebar pill dihitung dari segmen yang aktif,
  jadi ikut menyusut kalau sebagian dimatikan.
- **Setelan → Pill saat memutar**: saklar terpisah untuk jam, cuaca, judul lagu,
  dan lirik. Matikan semua kecuali lirik kalau mau notch jadi pemutar lirik murni.

### Diubah
- Hook lirik dipindah ke level `App`, bukan di dalam tab Musik. Sebelumnya lirik
  berhenti dihitung begitu kamu pindah tab, dan pill tidak mungkin memakainya.

## [0.1.2] — 2026-10-26

### Diubah
- **Animasi morph notch mencapai 60 fps.** Sebelumnya `p95` 33.3 ms dengan
  7 frame jatuh saat expand (efektif 55 fps). Sesudahnya `p95` 16.8 ms di
  seluruh pengukuran, 58–60 fps, tanpa frame di atas 34 ms.
  - Panel dan pill sekarang berukuran tetap (`--panel-h`, 560 px) sehingga
    isinya tidak di-reflow tiap frame — shell hanya memotong (clip).
  - `backdrop-filter` dimatikan selama morph (`.is-morphing`) dan dipulihkan
    setelah animasi selesai, karena blur harus di-raster ulang di setiap ukuran.
  - `box-shadow` disamakan antara state pill dan panel supaya toggle tidak
    memicu repaint area besar.
  - `contain: layout paint`, `will-change` hanya aktif selama morph,
    durasi 0.46 s.

### Diperbaiki
- **RAM selalu terbaca 0%** di status bar. Struct `Stats` di Rust mengirim
  `mem_used`/`mem_total` (snake_case), sementara frontend membaca `memUsed`/
  `memTotal`. Ditambahkan `#[serde(rename_all = "camelCase")]`.
- **Lirik telat dari audio.** SMTC hanya memperbarui posisi sesekali; sekarang
  posisi dikoreksi dengan selisih `LastUpdatedTime` → sekarang, dan interval
  polling diturunkan 700 ms → 400 ms.

### Ditambahkan
- **Layout now-playing dua kolom**: judul/artis/sampul di kiri, lirik di kanan.
- **Setelan → Lirik**: saklar hidup/mati dan penggeser sinkronisasi ±1000 ms
  untuk mengoreksi kalau lirik masih terasa maju/mundur.
- **Setelan → Tentang**: versi, lisensi, deskripsi singkat, dan tautan
  repositori/rilis/laporan bug (dibuka di browser, bukan di webview).

### Diubah
- Saat musik diputar, pill menyembunyikan jam dan cuaca — hanya equalizer dan
  judul lagu yang tampil, dan lebarnya mengecil 372 → 300 px.

## [0.1.1] — 2026-10-26

### Ditambahkan
- **Lirik tersinkron** di tab Musik, diambil dari [LRCLIB](https://lrclib.net)
  (gratis, tanpa API key) lalu dicocokkan ke posisi lagu. Baris aktif disorot
  putih tebal, blok lirik auto-scroll dan memudar di tepi atas/bawah.
- Cache lirik per lagu di localStorage, jadi lagu yang sama tidak di-fetch ulang.

### Diubah
- **Layout pemutar** jadi horizontal (sampul 96px di kiri, metadata di kanan)
  supaya ada ruang untuk lirik.
- **Animasi notch**: `backdrop-filter` diturunkan 30px→16px dan `saturate`
  165%→120%, transisi `box-shadow` dihapus (memicu repaint area besar tiap
  frame), ditambah `contain: layout paint style`, durasi 0.42s→0.5s.
- **Sudut panel** memakai `corner-shape: squircle` (superellipse) dengan radius
  30px, bukan busur lingkaran biasa.
- `sysinfo` polling 2s → 15s saat panel tertutup.
- `base64` 0.22 → 0.23.

### Direncanakan
- Auto-update lewat `tauri-plugin-updater`
- Modul on/off dan tab default di Setelan

## [0.1.0] — 2026-10-25

Rilis pertama.

### Ditambahkan
- **Notch shell**: jendela transparan selalu di atas, morph pill ↔ panel dengan
  animasi pegas, tinggi panel menyesuaikan isi tab.
- **Hit-test kursor di Rust**: area transparan di bawah notch tidak menelan klik
  milik aplikasi lain (`set_ignore_cursor_events` + polling 70 ms).
- **Beranda**: jam besar, tanggal Indonesia, cuaca sekarang, prakiraan 5 hari,
  strip per jam, tugas berikutnya.
- **Musik via Windows SMTC**: judul, artis, album, sampul (stream → base64),
  posisi, play/pause, next/prev, seek. Jalan dengan Spotify, YouTube Music, VLC,
  dan aplikasi lain yang terdaftar di kontrol media Windows.
- **Tugas**: tambah/centang/hapus, filter Semua/Aktif/Selesai, tersimpan lokal.
- **Fokus**: Pomodoro 25/5/15 dengan ring progres dan hitungan sesi.
- **Sistem**: CPU, RAM, inti, uptime lewat `sysinfo`.
- **AI chat**: penyedia OpenAI-compatible (Gemini, OpenAI, Groq, OpenRouter,
  DeepSeek, Ollama) dengan API key milik pengguna.
- **Setelan**: aksen warna, ketebalan kaca, selalu di atas, kota cuaca, penyedia AI.
- Tray icon + menu, hotkey global **Alt+N**.
- Cuaca memakai Open-Meteo (tanpa API key).
- CI: installer NSIS + GitHub Release otomatis pada tag `v*`.

### Catatan teknis
- Kode Rust diverifikasi dengan `cargo check --target x86_64-pc-windows-msvc`.
- Belum ada code signing, jadi SmartScreen memberi peringatan saat pertama jalan.
- Belum ada test otomatis (unit, widget, maupun golden).
