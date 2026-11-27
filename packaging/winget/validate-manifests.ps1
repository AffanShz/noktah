param(
    [string]$ManifestDir = "packaging/winget/out",
    [string]$ExpectedVersion = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $ManifestDir)) {
    throw "Direktori manifest tidak ditemukan: $ManifestDir"
}

$required = @(
    @{ File = "AffanShz.Noktah.yaml"; Type = "version" },
    @{ File = "AffanShz.Noktah.locale.en-US.yaml"; Type = "defaultLocale" },
    @{ File = "AffanShz.Noktah.locale.id-ID.yaml"; Type = "locale" },
    @{ File = "AffanShz.Noktah.installer.yaml"; Type = "installer" }
)

$cleanExpected = if ([string]::IsNullOrWhiteSpace($ExpectedVersion)) {
    ""
} else {
    $ExpectedVersion.Trim().TrimStart('v')
}

$seenVersions = @()
foreach ($item in $required) {
    $path = Join-Path $ManifestDir $item.File
    if (-not (Test-Path -LiteralPath $path)) {
        throw "Manifest wajib hilang: $path"
    }

    $bytes = [System.IO.File]::ReadAllBytes($path)
    if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) {
        throw "Manifest tidak boleh memiliki UTF-8 BOM: $($item.File)"
    }

    $text = [System.Text.Encoding]::UTF8.GetString($bytes)
    if ($text -cmatch '\{\{|__|TODO') {
        throw "Placeholder belum terisi ditemukan di $($item.File)"
    }
    if ($text -cnotmatch '(?m)^PackageIdentifier:\s*AffanShz\.Noktah\r?$') {
        throw "PackageIdentifier tidak valid di $($item.File)"
    }
    if ($text -cnotmatch '(?m)^ManifestVersion:\s*1\.6\.0\r?$') {
        throw "ManifestVersion harus 1.6.0 di $($item.File)"
    }
    if ($text -cnotmatch "(?m)^ManifestType:\s*$([regex]::Escape($item.Type))\r?$") {
        throw "ManifestType harus '$($item.Type)' di $($item.File)"
    }
    if ($text -cnotmatch '(?m)^PackageVersion:\s*([^\r\n]+?)\s*$') {
        throw "PackageVersion hilang di $($item.File)"
    }
    $pkgVer = $Matches[1].Trim()
    if ([string]::IsNullOrWhiteSpace($pkgVer)) {
        throw "PackageVersion kosong di $($item.File)"
    }
    if ($cleanExpected -ne "" -and $pkgVer -ne $cleanExpected) {
        throw "PackageVersion '$pkgVer' tidak cocok dengan ExpectedVersion '$cleanExpected' di $($item.File)"
    }
    $seenVersions += $pkgVer
}

if (@($seenVersions | Select-Object -Unique).Count -ne 1) {
    throw "PackageVersion tidak konsisten antar manifest: $($seenVersions -join ', ')"
}

$installerPath = Join-Path $ManifestDir "AffanShz.Noktah.installer.yaml"
$installerText = [System.IO.File]::ReadAllText($installerPath, [System.Text.Encoding]::UTF8)

if ($installerText -cnotmatch '(?m)^InstallerType:\s*zip\r?$') {
    throw "InstallerType harus 'zip' di AffanShz.Noktah.installer.yaml"
}
if ($installerText -cnotmatch '(?m)^NestedInstallerType:\s*portable\r?$') {
    throw "NestedInstallerType harus 'portable' di AffanShz.Noktah.installer.yaml"
}
if ($installerText -cnotmatch '(?m)^\s*InstallerSha256:\s*([A-F0-9]{64})\r?$') {
    throw "InstallerSha256 tidak valid (harus 64 huruf besar hex) di AffanShz.Noktah.installer.yaml"
}
if ($installerText -cnotmatch '(?m)^\s*InstallerUrl:\s*(https://github\.com/AffanShz/noktah/releases/download/[^\s]+)\r?$') {
    throw "InstallerUrl harus berawalan https://github.com/AffanShz/noktah/releases/download/ di AffanShz.Noktah.installer.yaml"
}
if ($installerText -cnotmatch '(?m)^\s*-\s*RelativeFilePath:\s*noktah-native\.exe\r?$') {
    throw "RelativeFilePath: noktah-native.exe tidak ditemukan di AffanShz.Noktah.installer.yaml"
}
if ($installerText -cnotmatch '(?m)^\s*PortableCommandAlias:\s*noktah\r?$') {
    throw "PortableCommandAlias: noktah tidak ditemukan di AffanShz.Noktah.installer.yaml"
}
if ($installerText -cnotmatch '(?m)^\s*-\s*RelativeFilePath:\s*noktah-hook\.exe\r?$') {
    throw "RelativeFilePath: noktah-hook.exe tidak ditemukan di AffanShz.Noktah.installer.yaml"
}
if ($installerText -cnotmatch '(?m)^\s*PortableCommandAlias:\s*noktah-hook\r?$') {
    throw "PortableCommandAlias: noktah-hook tidak ditemukan di AffanShz.Noktah.installer.yaml"
}

Write-Host ("Validasi manifest winget 1.6.0 berhasil ({0}, versi {1})." -f $ManifestDir, $seenVersions[0])
