# Panduan Kontribusi

## Struktur branch

| Branch | Peran | Aturan |
| --- | --- | --- |
| `main` | **Production** | Hanya menerima merge dari `develop`. Setiap tag `v*` di sini memicu rilis otomatis. Tidak boleh commit langsung. |
| `develop` | **Integrasi** | Semua pekerjaan masuk ke sini dulu. Target PR Dependabot juga ke sini. |
| `<tipe>/<deskripsi>` | **Kerja** | Dibuat dari `develop`, dihapus setelah merge. |

## Penamaan branch

Pola: `tipe/deskripsi-singkat` — huruf kecil, pakai tanda hubung, maksimal ~60 karakter.

| Prefix | Untuk |
| --- | --- |
| `feat/` | Fitur baru |
| `fix/` | Perbaikan bug |
| `hotfix/` | Perbaikan darurat di production (bercabang dari `main`) |
| `refactor/` | Merapikan kode tanpa mengubah perilaku |
| `docs/` | Dokumentasi |
| `style/` | UI/CSS atau formatting |
| `chore/` | Konfigurasi, dependensi, CI |

Contoh:

```
feat/lirik-tersinkron
fix/pill-overflow-saat-judul-panjang
chore/typescript
docs/roadmap-p2
hotfix/kursor-hit-test
```

## Konvensi commit

```
[AS] type: message
```

`type` mengikuti [Conventional Commits](https://www.conventionalcommits.org/):
`feat`, `fix`, `chore`, `docs`, `style`, `refactor`, `test`, `ci`, `perf`, `release`.

Contoh:

```
[AS] feat(musik): lirik tersinkron dari LRCLIB
[AS] fix(ci): lockfile menunjuk mirror Tiongkok
[AS] chore(deps): bump base64 0.22 -> 0.23
[AS] release: v1.0.0
```

Scope dalam kurung opsional, dipakai kalau perubahannya spesifik ke satu area
(`musik`, `notch`, `ci`, `deps`).

## Alur kerja

```bash
git checkout develop
git pull origin develop
git checkout -b feat/nama-fitur

# ... kerjakan, commit ...

git push -u origin feat/nama-fitur
```

Lalu buat Pull Request **ke `develop`** di GitHub. Setelah review dan merge,
hapus branch kerjanya.

### Kalau darurat (production rusak)

```bash
git checkout main
git checkout -b hotfix/nama-masalah
# perbaiki, commit, push, PR ke main
```

Setelah merge ke `main`, **wajib** di-merge balik ke `develop` supaya tidak
hilang di rilis berikutnya.

## Rilis

1. Pastikan `develop` stabil dan CI hijau
2. Buat PR `develop` → `main`, merge
3. Naikkan versi di tiga manifest native: `native/app/Cargo.toml`,
   `native/core/Cargo.toml`, dan `native/hook/Cargo.toml`
4. Perbarui `CHANGELOG.md`
5. Tag dari `main`:

```bash
git checkout main && git pull
git tag -a v0.3.1 -m "Noktah 0.3.1 — ringkasan singkat"
git push origin v0.3.1
```

CI akan build aplikasi native dan membuat GitHub Release otomatis dengan paket
`noktah-<tag>-win64.zip` terlampir.

## Dependabot

- PR otomatis dibuat ke `develop`, nama branch `chore/<nama-paket>`
- Update minor/patch dikelompokkan dalam satu PR
- Naik **major** untuk `windows`, `eframe`, `egui`, dan `rusqlite` sengaja
  diabaikan — crate itu selalu mengubah API dan harus diuji manual di Windows
- TypeScript 7 ditahan: itu port native (Go), bukan update biasa

## Aturan keras

- **Jangan** commit langsung ke `main`
- **Jangan** commit API key, token, `secret.json`, atau `secret.bin`
- Setiap perubahan yang menyentuh Rust **wajib** lolos
  `cargo test` dan `cargo check --target x86_64-pc-windows-msvc` sebelum push
