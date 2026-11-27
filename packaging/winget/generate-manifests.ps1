param(
    [Parameter(Mandatory = $true)]
    [string]$Version,

    [Parameter(Mandatory = $true)]
    [string]$InstallerUrl,

    [Parameter(Mandatory = $true)]
    [string]$InstallerSha256,

    [string]$OutputDir = "packaging/winget/out"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$cleanVer = $Version.Trim().TrimStart('v')
if ([string]::IsNullOrWhiteSpace($cleanVer) -or $cleanVer -cnotmatch '^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$') {
    throw "Versi tidak valid: '$Version'"
}

$shaUpper = $InstallerSha256.Trim().ToUpperInvariant()
if ($shaUpper -cnotmatch '^[A-F0-9]{64}$') {
    throw "InstallerSha256 harus berupa 64 karakter hex: '$InstallerSha256'"
}

$cleanUrl = $InstallerUrl.Trim()
if (-not $cleanUrl.StartsWith("https://")) {
    throw "InstallerUrl harus menggunakan HTTPS: '$InstallerUrl'"
}

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$resolvedOut = (Resolve-Path -LiteralPath $OutputDir).Path
$utf8NoBom = [System.Text.UTF8Encoding]::new($false)

function Write-ManifestFile {
    param(
        [string]$FileName,
        [string]$Body
    )
    $path = Join-Path $resolvedOut $FileName
    $normalized = ($Body -replace "`r`n", "`n").TrimEnd("`n") + "`n"
    [System.IO.File]::WriteAllText($path, $normalized, $utf8NoBom)
}

$versionManifest = @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.version.1.6.0.schema.json
PackageIdentifier: AffanShz.Noktah
PackageVersion: $cleanVer
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.6.0
"@

$enLocaleManifest = @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.defaultLocale.1.6.0.schema.json
PackageIdentifier: AffanShz.Noktah
PackageVersion: $cleanVer
PackageLocale: en-US
Publisher: AffanShz
PublisherUrl: https://github.com/AffanShz
PublisherSupportUrl: https://github.com/AffanShz/noktah/issues
PackageName: Noktah
PackageUrl: https://github.com/AffanShz/noktah
License: MIT
LicenseUrl: https://github.com/AffanShz/noktah/blob/main/LICENSE
ShortDescription: Native Windows Dynamic Island & Notch overlay with music, lyrics, mascot, tasks, focus timer, system monitor, and AI agent hooks.
Description: |-
  Noktah is a lightweight native Rust Windows 10/11 Dynamic Island & Notch overlay built with egui/eframe (zero WebView, zero telemetry). It integrates Windows SMTC playback controls, multi-provider synced lyrics, the interactive Noki mascot with global gaze tracking, tasks and reminders, Pomodoro focus timer, CPU/RAM system monitor, custom status tiles, AI chat with DPAPI key encryption, and Claude Code/Antigravity/CLI agent hooks via noktah-hook.
Tags:
  - dynamic-island
  - notch
  - windows
  - overlay
  - rust
  - egui
  - productivity
  - lyrics
  - pomodoro
  - claude-code
ReleaseNotesUrl: https://github.com/AffanShz/noktah/releases/tag/v$cleanVer
ManifestType: defaultLocale
ManifestVersion: 1.6.0
"@

$idLocaleManifest = @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.locale.1.6.0.schema.json
PackageIdentifier: AffanShz.Noktah
PackageVersion: $cleanVer
PackageLocale: id-ID
Publisher: AffanShz
PublisherUrl: https://github.com/AffanShz
PublisherSupportUrl: https://github.com/AffanShz/noktah/issues
PackageName: Noktah
PackageUrl: https://github.com/AffanShz/noktah
License: MIT
LicenseUrl: https://github.com/AffanShz/noktah/blob/main/LICENSE
ShortDescription: Overlay Dynamic Island & Notch native untuk Windows dengan musik, lirik, maskot Noki, tugas, fokus, monitor sistem, dan hook agent AI.
Description: |-
  Noktah adalah overlay Dynamic Island & Notch native Windows 10/11 berbasis Rust (egui/eframe) tanpa WebView dan tanpa telemetri. Mendukung kontrol media Windows SMTC, lirik tersinkron multi-penyedia, maskot Noki dengan pelacakan kursor, daftar tugas & pengingat, timer fokus Pomodoro, monitor CPU/RAM, tile status kustom, obrolan AI dengan enkripsi DPAPI, serta integrasi agent Claude Code/Antigravity/CLI melalui noktah-hook.
ReleaseNotesUrl: https://github.com/AffanShz/noktah/releases/tag/v$cleanVer
ManifestType: locale
ManifestVersion: 1.6.0
"@

$installerManifest = @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.installer.1.6.0.schema.json
PackageIdentifier: AffanShz.Noktah
PackageVersion: $cleanVer
InstallModes:
  - silent
  - silentWithProgress
InstallerType: zip
NestedInstallerType: portable
NestedInstallerFiles:
  - RelativeFilePath: noktah-native.exe
    PortableCommandAlias: noktah
  - RelativeFilePath: noktah-hook.exe
    PortableCommandAlias: noktah-hook
UpgradeBehavior: install
Installers:
  - Architecture: x64
    InstallerUrl: $cleanUrl
    InstallerSha256: $shaUpper
ManifestType: installer
ManifestVersion: 1.6.0
"@

Write-ManifestFile -FileName "AffanShz.Noktah.yaml" -Body $versionManifest
Write-ManifestFile -FileName "AffanShz.Noktah.locale.en-US.yaml" -Body $enLocaleManifest
Write-ManifestFile -FileName "AffanShz.Noktah.locale.id-ID.yaml" -Body $idLocaleManifest
Write-ManifestFile -FileName "AffanShz.Noktah.installer.yaml" -Body $installerManifest

Write-Host "Berhasil membuat 4 manifest winget 1.6.0 di $resolvedOut (versi $cleanVer)"
