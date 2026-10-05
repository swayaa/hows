# Packs the portable distribution. The published file is the ZIP, not hows.exe alone.
[CmdletBinding()]
param(
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path,
    [string]$Exe,
    [string]$OutDir
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$confPath = Join-Path $RepoRoot 'app\src-tauri\tauri.conf.json'
$version = (Get-Content -Raw $confPath | ConvertFrom-Json).version
if (-not $version) { throw 'tauri.conf.json has no version.' }

if (-not $Exe) {
    $Exe = Join-Path $RepoRoot 'app\src-tauri\target\release\hows.exe'
}
if (-not $OutDir) {
    $OutDir = Join-Path $RepoRoot 'app\src-tauri\target\release\bundle'
}
if (-not (Test-Path -LiteralPath $Exe)) { throw "Portable executable is missing: $Exe" }

$license = Join-Path $RepoRoot 'LICENSE'
$notices = Join-Path $RepoRoot 'THIRD-PARTY-NOTICES.md'
foreach ($path in @($license, $notices)) {
    if (-not (Test-Path -LiteralPath $path)) { throw "Missing $path" }
}
$licenseText = [IO.File]::ReadAllText($license)
if ($licenseText -notmatch 'Copyright \(c\) 2026 Hows contributors') {
    throw 'Root LICENSE does not name the Hows contributors.'
}
if ($licenseText -match 'Copyright \(c\) 2026 swaya\b') {
    throw 'Root LICENSE is the empty public-repo placeholder license.'
}

New-Item -ItemType Directory -Force $OutDir | Out-Null
$zipPath = Join-Path $OutDir "Hows_${version}_x64-portable.zip"
if (Test-Path -LiteralPath $zipPath) { Remove-Item -LiteralPath $zipPath -Force }

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [IO.Compression.ZipFile]::Open($zipPath, [IO.Compression.ZipArchiveMode]::Create)
try {
    [void][IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $Exe, 'hows.exe')
    [void][IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $license, 'LICENSE')
    [void][IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $notices, 'THIRD-PARTY-NOTICES.md')
}
finally {
    $zip.Dispose()
}

$check = [IO.Compression.ZipFile]::OpenRead($zipPath)
try {
    $names = @($check.Entries | ForEach-Object { $_.FullName })
}
finally {
    $check.Dispose()
}
$expected = @('hows.exe', 'LICENSE', 'THIRD-PARTY-NOTICES.md')
$diff = Compare-Object $expected $names
if ($diff) { throw "Portable ZIP must contain exactly hows.exe, LICENSE, and THIRD-PARTY-NOTICES.md. Found: $($names -join ', ')" }

Write-Host $zipPath
