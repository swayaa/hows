# Fetches the WiX 3.14.1 UI extension source that corresponds to the MSI dialogs.
# A URL in the notices is not this archive. The bytes travel next to the MSI.
[CmdletBinding()]
param(
    [string]$OutDir
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$sha = 'b40e9a32c24033e11b77baf2c91a704382f898ed'
$tag = 'wix3141rtm'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
if (-not $OutDir) {
    $OutDir = Join-Path $repoRoot 'app\src-tauri\target\release\bundle'
}

$tmp = Join-Path ([IO.Path]::GetTempPath()) "wix3-$sha"
if (Test-Path -LiteralPath $tmp) { Remove-Item -LiteralPath $tmp -Recurse -Force }

& git clone --filter=blob:none --sparse --depth 1 --branch $tag https://github.com/wixtoolset/wix3.git $tmp
if ($LASTEXITCODE -ne 0) { throw 'git clone of wix3 failed.' }
& git -C $tmp sparse-checkout set --no-cone src/ext/UIExtension /LICENSE.TXT
if ($LASTEXITCODE -ne 0) { throw 'git sparse-checkout failed.' }
& git -C $tmp checkout --detach $sha
if ($LASTEXITCODE -ne 0) { throw "git checkout $sha failed." }

$got = (& git -C $tmp rev-parse HEAD).Trim()
if ($got -ne $sha) { throw "WiX checkout is $got, expected $sha." }

$extension = Join-Path $tmp 'src\ext\UIExtension'
$wixLicense = Join-Path $tmp 'LICENSE.TXT'
if (-not (Test-Path -LiteralPath $extension -PathType Container)) { throw 'src/ext/UIExtension is missing.' }
$sourceFiles = @(Get-ChildItem -LiteralPath $extension -Recurse -File)
if ($sourceFiles.Count -lt 1) { throw 'UIExtension source is empty.' }
if (-not (Test-Path -LiteralPath $wixLicense)) { throw 'WiX LICENSE.TXT is missing.' }
$licenseText = [IO.File]::ReadAllText($wixLicense)
if ($licenseText -notmatch 'Microsoft Reciprocal License') {
    throw 'WiX LICENSE.TXT is not the Microsoft Reciprocal License.'
}

New-Item -ItemType Directory -Force $OutDir | Out-Null
$zipPath = Join-Path $OutDir 'wix-UIExtension-wix3141rtm.zip'
if (Test-Path -LiteralPath $zipPath) { Remove-Item -LiteralPath $zipPath -Force }

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [IO.Compression.ZipFile]::Open($zipPath, [IO.Compression.ZipArchiveMode]::Create)
try {
    [void][IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $wixLicense, 'LICENSE.TXT')
    foreach ($file in $sourceFiles) {
        $relative = $file.FullName.Substring($extension.Length).TrimStart('\', '/')
        $entryName = 'UIExtension/' + ($relative -replace '\\', '/')
        [void][IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $file.FullName, $entryName)
    }
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
if ($names -notcontains 'LICENSE.TXT') { throw 'WiX archive is missing LICENSE.TXT.' }
if (-not ($names | Where-Object { $_ -like 'UIExtension/*' })) { throw 'WiX archive is missing UIExtension source.' }

Remove-Item -LiteralPath $tmp -Recurse -Force
Write-Host $zipPath
