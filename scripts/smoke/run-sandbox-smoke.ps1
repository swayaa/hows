<#
.SYNOPSIS
  End-to-end smoke test of a Hows Windows build inside Windows Sandbox.

.DESCRIPTION
  Starts a disposable Windows Sandbox without network, installs the NSIS
  build there, seeds its settings file, records a short session with
  synthetic input into a test window, and checks the seeded settings,
  hotkeys, step detection, the typed-text privacy rule, the title
  suggestion, recents after saving, and HTML/PDF/JSON/Markdown exports.
  All input happens inside the sandbox; the host desktop is not touched.
  The WebView2 runtime offline installer is downloaded from Microsoft once,
  signature-checked, cached under %LOCALAPPDATA%\hows-smoke\cache and
  installed in the sandbox before Hows.

  Requires Windows 10/11 Pro or Enterprise with the optional feature
  "Windows Sandbox" enabled (admin, then reboot):
    Enable-WindowsOptionalFeature -Online -FeatureName Containers-DisposableClientVM -All

.PARAMETER Bundle
  Folder with `Hows_*-setup.exe`, `Hows_*-portable.zip`, and/or `hows.exe`,
  e.g. the unzipped `windows-installers` CI artifact. A portable ZIP is
  unpacked so the sandbox can start `hows.exe`. Default: the local
  `npm run tauri build` output.

.PARAMETER KeepOpen
  Leave the sandbox running after the test for manual inspection.

.EXAMPLE
  pwsh scripts/smoke/run-sandbox-smoke.ps1
#>
[CmdletBinding()]
param(
    [string]$Bundle,
    [int]$TimeoutMinutes = 10,
    [switch]$KeepOpen
)

$ErrorActionPreference = 'Stop'

$sandboxExe = Join-Path $env:windir 'System32\WindowsSandbox.exe'
if (-not (Test-Path $sandboxExe)) {
    throw ('Windows Sandbox is not enabled. As administrator run ' +
        '"Enable-WindowsOptionalFeature -Online -FeatureName Containers-DisposableClientVM -All" and reboot.')
}
$running = Get-Process -Name WindowsSandbox, WindowsSandboxClient, WindowsSandboxRemoteSession -ErrorAction SilentlyContinue
if ($running) { throw 'A Windows Sandbox is already running; only one can run at a time. Close it first.' }

$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$work = Join-Path $env:LOCALAPPDATA ('hows-smoke\' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
$bundleDir = Join-Path $work 'bundle'
$resultsDir = Join-Path $work 'results'
New-Item -ItemType Directory -Force $bundleDir, $resultsDir | Out-Null

if ($Bundle) {
    $sources = Get-ChildItem -Path $Bundle -Recurse -File -Include 'Hows_*-setup.exe', 'Hows_*-portable.zip', 'hows.exe'
} else {
    $release = Join-Path $repoRoot 'app\src-tauri\target\release'
    $sources = @(
        Get-ChildItem (Join-Path $release 'bundle\nsis\Hows_*-setup.exe') -ErrorAction SilentlyContinue |
            Sort-Object LastWriteTime -Descending | Select-Object -First 1
        Get-Item (Join-Path $release 'hows.exe') -ErrorAction SilentlyContinue
    ) | Where-Object { $_ }
}
if (-not $sources) {
    throw 'No build found. Run "npm run tauri build" in app/ or pass -Bundle <folder with Hows_*-setup.exe>.'
}
$sources | Copy-Item -Destination $bundleDir
Add-Type -AssemblyName System.IO.Compression.FileSystem
Get-ChildItem -Path $bundleDir -Filter 'Hows_*-portable.zip' | ForEach-Object {
    $exeInBundle = Join-Path $bundleDir 'hows.exe'
    if (Test-Path -LiteralPath $exeInBundle) { return }
    $zip = [IO.Compression.ZipFile]::OpenRead($_.FullName)
    try {
        $entry = $zip.GetEntry('hows.exe')
        if (-not $entry) { throw "Portable ZIP has no hows.exe: $($_.FullName)" }
        [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $exeInBundle, $false)
    }
    finally { $zip.Dispose() }
}
# The mark check compares the exported colors with their source.
Copy-Item (Join-Path $repoRoot 'core\store\marks.json') $bundleDir

# Windows Sandbox ships without the WebView2 runtime, and the installer's
# bootstrapper cannot download it with networking disabled.
$cacheDir = Join-Path $env:LOCALAPPDATA 'hows-smoke\cache'
$runtimeSetup = Join-Path $cacheDir 'MicrosoftEdgeWebView2RuntimeInstallerX64.exe'
if (-not (Test-Path $runtimeSetup)) {
    New-Item -ItemType Directory -Force $cacheDir | Out-Null
    Write-Host 'Downloading the WebView2 runtime offline installer (once) ...'
    Invoke-WebRequest -Uri 'https://go.microsoft.com/fwlink/?linkid=2124701' -OutFile "$runtimeSetup.part"
    Move-Item "$runtimeSetup.part" $runtimeSetup
}
$signature = Get-AuthenticodeSignature $runtimeSetup
if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation') {
    Remove-Item $runtimeSetup
    throw "WebView2 installer signature is not a valid Microsoft signature ($($signature.Status)); deleted, run again."
}
$runtimeDir = Join-Path $bundleDir 'webview2'
New-Item -ItemType Directory -Force $runtimeDir | Out-Null
Copy-Item $runtimeSetup $runtimeDir
if ($KeepOpen) { New-Item -ItemType File (Join-Path $resultsDir 'keep-open') | Out-Null }

$wsb = Join-Path $work 'smoke.wsb'
@"
<Configuration>
  <Networking>Disable</Networking>
  <ClipboardRedirection>Disable</ClipboardRedirection>
  <AudioInput>Disable</AudioInput>
  <VideoInput>Disable</VideoInput>
  <PrinterRedirection>Disable</PrinterRedirection>
  <MemoryInMB>4096</MemoryInMB>
  <MappedFolders>
    <MappedFolder>
      <HostFolder>$bundleDir</HostFolder>
      <SandboxFolder>C:\smoke\bundle</SandboxFolder>
      <ReadOnly>true</ReadOnly>
    </MappedFolder>
    <MappedFolder>
      <HostFolder>$PSScriptRoot</HostFolder>
      <SandboxFolder>C:\smoke\scripts</SandboxFolder>
      <ReadOnly>true</ReadOnly>
    </MappedFolder>
    <MappedFolder>
      <HostFolder>$resultsDir</HostFolder>
      <SandboxFolder>C:\smoke\results</SandboxFolder>
      <ReadOnly>false</ReadOnly>
    </MappedFolder>
  </MappedFolders>
  <LogonCommand>
    <Command>powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\smoke\scripts\in-sandbox.ps1</Command>
  </LogonCommand>
</Configuration>
"@ | Set-Content -Path $wsb -Encoding UTF8

Write-Host "Starting Windows Sandbox (work folder: $work) ..."
$startedAt = Get-Date
Start-Process -FilePath $sandboxExe -ArgumentList "`"$wsb`""

$resultFile = Join-Path $resultsDir 'result.json'
$deadline = (Get-Date).AddMinutes($TimeoutMinutes)
while (-not (Test-Path $resultFile) -and (Get-Date) -lt $deadline) { Start-Sleep -Seconds 5 }
if (-not (Test-Path $resultFile)) {
    if (-not $KeepOpen) {
        # Only the client processes; stopping WindowsSandboxServer breaks the next start.
        Get-Process -Name WindowsSandbox, WindowsSandboxClient, WindowsSandboxRemoteSession -ErrorAction SilentlyContinue |
            Where-Object { $_.StartTime -ge $startedAt.AddSeconds(-5) } |
            Stop-Process -Force -ErrorAction SilentlyContinue
    }
    throw "No result after $TimeoutMinutes minutes. Log: $(Join-Path $resultsDir 'smoke.log')"
}
Start-Sleep -Seconds 1

$result = Get-Content -Raw -Path $resultFile | ConvertFrom-Json
foreach ($check in $result.checks) {
    $status = if ($check.ok) { 'PASS' } else { 'FAIL' }
    Write-Host ('{0}  {1}  {2}' -f $status, $check.name, $check.detail)
}
Write-Host "Log, screenshots and exports: $resultsDir"
if (-not $result.ok) { exit 1 }
