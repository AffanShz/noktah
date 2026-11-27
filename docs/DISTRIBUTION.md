# Panduan Distribusi Noktah (`winget`, `PATH`, dan CI Code Signing)

Dokumen ini menjelaskan cara memasang Noktah melalui `winget`, mendaftarkan `noktah-hook.exe` ke User `PATH`, membuat dan memvalidasi manifest `winget-pkgs`, serta mengonfigurasi penandatanganan digital (Code Signing) di GitHub Actions.

---

## 1. Instalasi via `winget`

Noktah dikemas dalam format arsip `.zip` dengan tipe *nested portable installer* yang dipasang ke cakupan pengguna (*user scope*) secara bawaan (dan mendukung flag `--scope` secara dinamis di `winget`, tanpa perlu mendeklarasikan `Scope` di manifest) sehingga tidak membutuhkan hak administrator:

```powershell
winget install AffanShz.Noktah
```

Untuk memperbarui versi terpasang:

```powershell
winget upgrade AffanShz.Noktah
```

Saat dipasang melalui `winget`, kedua alias perintah berikut langsung didaftarkan otomatis oleh `winget` ke folder link portabel pengguna:

- `noktah` → `noktah-native.exe`
- `noktah-hook` → `noktah-hook.exe`

---

## 2. Registrasi `noktah-hook.exe` ke User `PATH` (Instalasi ZIP Manual)

Jika kamu mengunduh paket `noktah-<versi>-win64.zip` secara manual dari halaman GitHub Releases:

### Lewat Aplikasi (Rekomendasi)
1. Ekstrak `noktah-native.exe` dan `noktah-hook.exe` ke folder yang sama (misalnya `%LOCALAPPDATA%\Noktah`).
2. Jalankan `noktah-native.exe`, lalu buka **Setelan → AI → Integrasi Maskot & Agent Hooks (Noki)**.
3. Nyalakan saklar **Daftarkan noktah-hook ke PATH** (*Register noktah-hook in PATH*).
4. Noktah akan menambahkan folder aplikasi ke `HKCU\Environment\Path` (`REG_EXPAND_SZ`) dan menyiarkan `WM_SETTINGCHANGE`. Buka ulang terminal untuk langsung memanggil `noktah-hook` dari mana saja.

### Lewat Terminal PowerShell (Manual)
```powershell
$appDir = "C:\Path\Ke\Folder\Noktah"
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$appDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$appDir", "User")
}
```

Verifikasi di terminal baru:

```powershell
where.exe noktah-hook
noktah-hook --help
```

---

## 3. Manifest `winget` (`packaging/winget/`) & Pengajuan ke `winget-pkgs`

Skema manifest menggunakan **winget 1.6.0** (4 berkas UTF-8 tanpa BOM untuk `AffanShz.Noktah`):

- `AffanShz.Noktah.yaml` (`ManifestType: version`)
- `AffanShz.Noktah.locale.en-US.yaml` (`ManifestType: defaultLocale`)
- `AffanShz.Noktah.locale.id-ID.yaml` (`ManifestType: locale`)
- `AffanShz.Noktah.installer.yaml` (`ManifestType: installer`, mencakup alias `noktah` dan `noktah-hook`)

### Generate & Validasi Secara Lokal

```powershell
./packaging/winget/generate-manifests.ps1 `
  -Version "2.0.0" `
  -InstallerUrl "https://github.com/AffanShz/noktah/releases/download/v2.0.0/noktah-v2.0.0-win64.zip" `
  -InstallerSha256 "<SHA256_ZIP_64_HEX>" `
  -OutputDir "packaging/winget/out"

./packaging/winget/validate-manifests.ps1 `
  -ManifestDir "packaging/winget/out" `
  -ExpectedVersion "2.0.0"
```

### Pengajuan PR ke `microsoft/winget-pkgs`

1. Setiap rilis tag `v*` di GitHub Actions otomatis menghasilkan dan melampirkan artefak `noktah-<tag>-winget-manifests.zip` di halaman Release.
2. Ekstrak keempat berkas YAML tersebut ke fork `microsoft/winget-pkgs` pada jalur:
   `manifests/a/AffanShz/Noktah/<versi>/`
3. Verifikasi lokal (bila `winget` terpasang):
   ```powershell
   winget validate --manifest manifests/a/AffanShz/Noktah/<versi>/
   ```
4. Buat Pull Request ke repositori `microsoft/winget-pkgs`.

---

## 4. Konfigurasi CI Code Signing (`.github/workflows/release-native.yml`)

Workflow rilis mendukung **tiga mode penandatanganan otomatis** sebelum perhitungan SHA-256 dan pengemasan ZIP:

### Mode 1 — Azure Trusted Signing (Prioritas Utama)
Menggunakan `azure/trusted-signing-action@v0.5.0` untuk menandatangani `noktah-native.exe` dan `noktah-hook.exe` dengan digest `SHA256` dan timestamp RFC 3161 (`http://timestamp.acs.microsoft.com`).

Tambahkan seluruh **Repository Secrets** berikut di GitHub (*Settings → Secrets and variables → Actions*):

| Secret | Keterangan |
| --- | --- |
| `AZURE_TRUSTED_SIGNING_ENDPOINT` | URL endpoint region akun Trusted Signing (mis. `https://eus.codesigning.azure.net/`) |
| `AZURE_CLIENT_ID` | Application (client) ID Service Principal Entra ID |
| `AZURE_TENANT_ID` | Directory (tenant) ID Entra ID |
| `AZURE_CLIENT_SECRET` | Client secret Service Principal (pastikan berperan *Trusted Signing Certificate Profile Signer*) |
| `AZURE_CODE_SIGNING_ACCOUNT` | Nama akun Azure Trusted Signing |
| `AZURE_CERT_PROFILE` | Nama Certificate Profile yang aktif |

### Mode 2 — Sertifikat PFX Authenticode (`signtool.exe`)
Dipakai otomatis bila secret Azure tidak diisi, tetapi secret sertifikat `.pfx` tersedia:

| Secret | Keterangan |
| --- | --- |
| `WINDOWS_SIGNING_CERT_BASE64` | Isi berkas `.pfx` yang telah di-encode Base64 (`[Convert]::ToBase64String([IO.File]::ReadAllBytes("cert.pfx"))`) |
| `WINDOWS_SIGNING_CERT_PASSWORD` | Kata sandi berkas `.pfx` (opsional bila sertifikat tanpa password) |
| `WINDOWS_TIMESTAMP_URL` | Opsional; default ke `http://timestamp.digicert.com` |

CI mendekode `.pfx` sementara ke `$env:RUNNER_TEMP\cert.pfx`, menandatangani dan memverifikasi kedua biner memakai `signtool.exe` (Windows Kits 10 x64), lalu selalu menghapus file `.pfx` di blok `finally`.

### Mode 3 — Fallback Unsigned
Bila tidak ada secret Code Signing yang dikonfigurasi, workflow tetap berjalan normal (`SIGNING_STATUS=unsigned`) dan menulis statusnya ke `BACA-DULU.txt` beserta checksum SHA-256 kedua biner.

### Verifikasi Tanda Tangan di Windows Pengguna

```powershell
Get-AuthenticodeSignature .\noktah-native.exe, .\noktah-hook.exe | Format-Table Path, Status, SignerCertificate
```
