<#
  Runs inside Windows Sandbox as its logon command (see run-sandbox-smoke.ps1).
  Installs Hows, seeds its settings file, drives the
  app over the WebView2 DevTools protocol plus synthetic mouse and keyboard
  input, and writes result.json, smoke.log and a screenshot to C:\smoke\results.
  Afterwards it walks every surface in each language (light, plus dark for en
  and de) and saves one PNG per surface to C:\smoke\results\tour.

  It moves the mouse and types, so it refuses to run outside the sandbox.
  Windows PowerShell 5.1 only: the sandbox has nothing else installed.
#>
$ErrorActionPreference = 'Stop'

# The sandbox always logs on as WDAGUtilityAccount. Anywhere else this script
# would move the real mouse and shut the machine down at the end.
$inSandbox = $env:USERNAME -eq 'WDAGUtilityAccount'
if (-not $inSandbox) {
    [Console]::Error.WriteLine("Refusing to run outside Windows Sandbox (user '$env:USERNAME'). Use run-sandbox-smoke.ps1.")
    exit 1
}

$root = 'C:\smoke'
$results = Join-Path $root 'results'
$log = Join-Path $results 'smoke.log'
$checks = New-Object System.Collections.Generic.List[object]
$typedSecret = 'hello-smoke-secret'
$formTitle = 'Smoke Form Window'
# Widest capture pill accepted in any language, in CSS pixels. Before PR 5b it was 737.
$pillMaxWidth = 560

function Write-Log([string]$message) {
    Add-Content -Path $log -Value ('{0:HH:mm:ss} {1}' -f (Get-Date), $message) -Encoding UTF8
}

function Add-Check([string]$name, [bool]$ok, $detail) {
    $checks.Add([pscustomobject]@{ name = $name; ok = $ok; detail = "$detail" })
    $status = if ($ok) { 'PASS' } else { 'FAIL' }
    Write-Log "$status $name : $detail"
}

function Save-Screenshot([string]$name) {
    $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
    $bitmap.Save((Join-Path $results $name), [System.Drawing.Imaging.ImageFormat]::Png)
    $graphics.Dispose()
    $bitmap.Dispose()
}

function Wait-Until([scriptblock]$condition, [int]$seconds, [string]$what) {
    $deadline = (Get-Date).AddSeconds($seconds)
    $lastError = ''
    while ((Get-Date) -lt $deadline) {
        try { $value = & $condition; if ($value) { return $value } } catch { $lastError = $_.Exception.Message }
        Start-Sleep -Milliseconds 500
    }
    throw "Timeout after ${seconds}s: $what $lastError"
}

function Get-WebView2Version {
    $client = 'Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
    foreach ($key in "HKLM:\SOFTWARE\WOW6432Node\$client", "HKLM:\SOFTWARE\$client", "HKCU:\SOFTWARE\$client") {
        $version = (Get-ItemProperty $key -ErrorAction SilentlyContinue).pv
        if ($version -and $version -ne '0.0.0.0') { return $version }
    }
}

# --- DevTools protocol over a plain WebSocket --------------------------------

$script:cdpId = 0
$script:socket = $null
$script:cdpEvents = New-Object System.Collections.Generic.List[object]

function Connect-Cdp([int]$port) {
    try {
        $page = Wait-Until {
            Invoke-RestMethod -UseBasicParsing "http://127.0.0.1:$port/json" |
                Where-Object { $_.type -eq 'page' } | Select-Object -First 1
        } 90 'WebView2 DevTools endpoint'
    } catch {
        Get-CimInstance Win32_Process -Filter "Name='hows.exe'" |
            ForEach-Object { Write-Log "app process: pid $($_.ProcessId) parent $($_.ParentProcessId) started $($_.CreationDate)" }
        Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe'" |
            Where-Object { $_.CommandLine -notmatch '--type=' } |
            ForEach-Object { Write-Log "webview browser: parent $($_.ParentProcessId) $($_.CommandLine)" }
        Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue |
            ForEach-Object { Write-Log "listening: $($_.LocalAddress):$($_.LocalPort) pid $($_.OwningProcess)" }
        throw
    }
    $script:socket = New-Object System.Net.WebSockets.ClientWebSocket
    $script:socket.ConnectAsync([Uri]$page.webSocketDebuggerUrl, [Threading.CancellationToken]::None).Wait()
}

function Send-Cdp([string]$method, [hashtable]$params) {
    $script:cdpId++
    $id = $script:cdpId
    $payload = @{ id = $id; method = $method; params = $params } | ConvertTo-Json -Depth 10 -Compress
    $bytes = [Text.Encoding]::UTF8.GetBytes($payload)
    $script:socket.SendAsync(
        (New-Object 'ArraySegment[byte]' (, $bytes)),
        [System.Net.WebSockets.WebSocketMessageType]::Text, $true,
        [Threading.CancellationToken]::None).Wait()
    $buffer = New-Object byte[] 65536
    while ($true) {
        $stream = New-Object IO.MemoryStream
        do {
            $task = $script:socket.ReceiveAsync(
                (New-Object 'ArraySegment[byte]' (, $buffer)), [Threading.CancellationToken]::None)
            if (-not $task.Wait(60000)) { throw "CDP timeout: $method" }
            $stream.Write($buffer, 0, $task.Result.Count)
        } while (-not $task.Result.EndOfMessage)
        $message = [Text.Encoding]::UTF8.GetString($stream.ToArray()) | ConvertFrom-Json
        if ($message.PSObject.Properties['id'] -and $message.id -eq $id) { return $message }
        if ($message.PSObject.Properties['method']) { $script:cdpEvents.Add($message) }
    }
}

function Invoke-Js([string]$expression) {
    $response = Send-Cdp 'Runtime.evaluate' @{ expression = $expression; awaitPromise = $true; returnByValue = $true }
    if ($response.result.PSObject.Properties['exceptionDetails']) {
        throw "JS: $($response.result.exceptionDetails.exception.description)"
    }
    return $response.result.result.value
}

function Invoke-Tauri([string]$command, [string]$argsJson = '{}') {
    return Invoke-Js "window.__TAURI_INTERNALS__.invoke('$command', $argsJson)"
}

# --- Screenshot tour over every surface ---------------------------------------

$tourDir = Join-Path $results 'tour'
$script:tourShots = New-Object System.Collections.Generic.List[string]

function Save-Surface([string]$name) {
    $shot = Send-Cdp 'Page.captureScreenshot' @{ format = 'png' }
    [IO.File]::WriteAllBytes((Join-Path $tourDir "$name.png"), [Convert]::FromBase64String($shot.result.data))
    $script:tourShots.Add($name)
}

function Invoke-Action([string]$action) {
    $found = Invoke-Js "(() => { const el = document.querySelector('[data-action=`"$action`"]'); if (el) el.click(); return !!el; })()"
    if (-not $found) { throw "no element for data-action '$action'" }
    Start-Sleep -Milliseconds 400
}

function Wait-Surface([string]$surface) {
    Wait-Until { (Invoke-Js "document.querySelector('main')?.dataset.surface ?? ''") -eq $surface } 15 "surface $surface" | Out-Null
    Start-Sleep -Milliseconds 500
}

function Set-Choice([string]$selectId, [string]$value) {
    $ok = Invoke-Js "(() => { const s = document.getElementById('$selectId'); if (!s || ![...s.options].some(o => o.value === '$value')) return false; s.value = '$value'; s.dispatchEvent(new Event('change', { bubbles: true })); return true; })()"
    if (-not $ok) { throw "no option '$value' in #$selectId" }
    Start-Sleep -Milliseconds 600
}

function Open-Advanced {
    $ok = Invoke-Js "(() => { const d = document.querySelector('[data-section=advanced]'); if (!d) return false; d.open = true; d.scrollIntoView(); return true; })()"
    if (-not $ok) { throw 'no advanced settings section' }
    Start-Sleep -Milliseconds 400
}

function Set-Field([string]$setting, [string]$value) {
    $ok = Invoke-Js "(() => { const el = document.querySelector('[data-setting=`"$setting`"]'); if (!el) return false; el.value = '$value'; el.dispatchEvent(new Event('change', { bubbles: true })); return true; })()"
    if (-not $ok) { throw "no field for data-setting '$setting'" }
    Start-Sleep -Milliseconds 600
}

$script:pillWidths = New-Object System.Collections.Generic.List[object]
$script:hiddenToolbars = New-Object System.Collections.Generic.List[string]

# --- Export contents ----------------------------------------------------------

function Read-Latin1([string]$path) {
    if (-not (Test-Path $path)) { return '' }
    return [Text.Encoding]::GetEncoding(28591).GetString([IO.File]::ReadAllBytes($path))
}

# Text of a Hows PDF: glyph ids of every Tj decoded through the ToUnicode
# map of its font (F1 regular, F2 bold, in that order in the file).
function Get-PdfText([string]$path) {
    $raw = Read-Latin1 $path
    $maps = @(foreach ($cmap in [regex]::Matches($raw, 'begincmap(.*?)endcmap', 'Singleline')) {
        $map = @{}
        foreach ($entry in [regex]::Matches($cmap.Groups[1].Value, '<([0-9A-Fa-f]{4})> <([0-9A-Fa-f]{4,8})>')) {
            $units = $entry.Groups[2].Value
            $text = -join (0..($units.Length / 4 - 1) | ForEach-Object { [char][Convert]::ToInt32($units.Substring($_ * 4, 4), 16) })
            $map[$entry.Groups[1].Value.ToUpper()] = $text
        }
        , $map
    })
    $lines = foreach ($show in [regex]::Matches($raw, '/F(\d) [\d.]+ Tf [-\d.]+ [-\d.]+ Td <([0-9A-Fa-f]*)> Tj')) {
        $map = $maps[[int]$show.Groups[1].Value - 1]
        $hex = $show.Groups[2].Value
        -join (0..($hex.Length / 4 - 1) | Where-Object { $hex.Length -ge 4 } | ForEach-Object { $map[$hex.Substring($_ * 4, 4).ToUpper()] })
    }
    return ($lines -join "`n")
}

# Exports the open guide as HTML, PDF and Markdown and copies them to the results.
function Export-Documents([string]$tag) {
    $files = @{
        html = "$(Invoke-Tauri 'export_html')"
        pdf = "$(Invoke-Tauri 'export_pdf')"
        markdown = "$(Invoke-Tauri 'export_markdown')"
    }
    Copy-Item $files.html (Join-Path $results "export-$tag.html") -ErrorAction SilentlyContinue
    Copy-Item $files.pdf (Join-Path $results "export-$tag.pdf") -ErrorAction SilentlyContinue
    Copy-Item $files.markdown (Join-Path $results "export-$tag.md") -ErrorAction SilentlyContinue
    return @{
        html = if (Test-Path $files.html) { [IO.File]::ReadAllText($files.html) } else { '' }
        pdfRaw = Read-Latin1 $files.pdf
        pdfText = Get-PdfText $files.pdf
        markdown = if (Test-Path $files.markdown) { [IO.File]::ReadAllText($files.markdown) } else { '' }
    }
}

# --- Mark colors in exported images -------------------------------------------

Add-Type @'
public static class SmokePixels {
    // Pixels of exactly rgb; r, g and b are the channel offsets inside each step-byte pixel.
    public static int Count(byte[] data, int step, int r, int g, int b, byte[] rgb) {
        int n = 0;
        for (int i = 0; i + step <= data.Length; i += step) {
            if (data[i + r] == rgb[0] && data[i + g] == rgb[1] && data[i + b] == rgb[2]) n++;
        }
        return n;
    }
}
'@

function ConvertTo-Rgb([string]$hex) {
    $value = [Convert]::ToInt32($hex.TrimStart('#'), 16)
    return , ([byte[]]@((($value -shr 16) -band 255), (($value -shr 8) -band 255), ($value -band 255)))
}

function Measure-PngColor([byte[]]$png, [byte[]]$rgb) {
    $stream = New-Object IO.MemoryStream (, $png)
    $bitmap = New-Object System.Drawing.Bitmap $stream
    try {
        $rect = New-Object System.Drawing.Rectangle 0, 0, $bitmap.Width, $bitmap.Height
        $data = $bitmap.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::ReadOnly, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
        $bytes = New-Object byte[] ($data.Stride * $bitmap.Height)
        [Runtime.InteropServices.Marshal]::Copy($data.Scan0, $bytes, 0, $bytes.Length)
        $bitmap.UnlockBits($data)
        return [SmokePixels]::Count($bytes, 4, 2, 1, 0, $rgb)
    } finally {
        $bitmap.Dispose()
        $stream.Dispose()
    }
}

# Pixels of $rgb in every step image of an HTML export; keeps the PNG with the most as $saveAs.
function Measure-HtmlColor([string]$html, [byte[]]$rgb, [string]$saveAs) {
    $total = 0
    $best = 0
    foreach ($match in [regex]::Matches($html, 'data:image/png;base64,([A-Za-z0-9+/=]+)')) {
        $png = [Convert]::FromBase64String($match.Groups[1].Value)
        $count = Measure-PngColor $png $rgb
        $total += $count
        if ($saveAs -and $count -gt $best) {
            $best = $count
            [IO.File]::WriteAllBytes($saveAs, $png)
        }
    }
    return $total
}

# Pixels of $rgb in every DeviceRGB image of a Hows PDF. The streams are zlib:
# two header bytes, then raw deflate.
function Measure-PdfColor([string]$raw, [byte[]]$rgb) {
    $latin1 = [Text.Encoding]::GetEncoding(28591)
    $total = 0
    $pattern = '/Subtype /Image /Width \d+ /Height \d+ /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /Length (\d+) >>\nstream\n'
    foreach ($match in [regex]::Matches($raw, $pattern)) {
        $packed = $latin1.GetBytes($raw.Substring($match.Index + $match.Length + 2, [int]$match.Groups[1].Value - 2))
        $source = New-Object IO.MemoryStream (, $packed)
        $inflate = New-Object IO.Compression.DeflateStream $source, ([IO.Compression.CompressionMode]::Decompress)
        $pixels = New-Object IO.MemoryStream
        $inflate.CopyTo($pixels)
        $inflate.Dispose()
        $total += [SmokePixels]::Count($pixels.ToArray(), 3, 0, 1, 2, $rgb)
    }
    return $total
}

# Real mouse events inside the page, in CSS pixels of the viewport.
function Send-Mouse([string]$type, [double]$x, [double]$y, [int]$buttons) {
    Send-Cdp 'Input.dispatchMouseEvent' @{ type = $type; x = $x; y = $y; button = 'left'; buttons = $buttons; clickCount = 1 } | Out-Null
    Start-Sleep -Milliseconds 60
}

# Starts and ends on the library with at least one recent guide.
function Save-TourPass([string]$tag) {
    Wait-Surface 'library'
    Wait-Until { Invoke-Js "!!document.querySelector('[data-part=recent-thumbnail]')" } 10 'library thumbnail' | Out-Null
    Save-Surface "library-$tag"
    # A recording without input, discarded at once, shows the pill in this language.
    Invoke-Tauri 'recorder_command' "{name:'start'}" | Out-Null
    Wait-Surface 'capture'
    $width = Invoke-Js "Math.ceil(document.querySelector('.sl-capture-pill').getBoundingClientRect().width)"
    $script:pillWidths.Add([pscustomobject]@{ tag = $tag; width = [int]$width })
    Save-Surface "capture-$tag"
    Invoke-Tauri 'recorder_command' "{name:'discard'}" | Out-Null
    Wait-Surface 'library'
    Invoke-Action 'open-settings'; Wait-Surface 'settings'
    Save-Surface "settings-$tag"
    Open-Advanced
    Save-Surface "settings-advanced-$tag"
    Invoke-Action 'close-settings'; Wait-Surface 'library'
    Invoke-Action 'open-recent'; Wait-Surface 'editor'
    Wait-Until { Invoke-Js "!!document.querySelector('main img, main canvas')" } 15 'step image' | Out-Null
    Start-Sleep -Milliseconds 600
    Save-Surface "editor-$tag"
    Invoke-Action 'toggle-annotate'
    $toolbarShown = Invoke-Js "(() => { const r = document.querySelector('[role=toolbar]')?.getBoundingClientRect(); return !!r && r.top >= 0 && r.bottom <= window.innerHeight; })()"
    if (-not $toolbarShown) { $script:hiddenToolbars.Add($tag) }
    Save-Surface "annotate-$tag"
    Invoke-Action 'toggle-annotate'
    Invoke-Action 'open-export'
    Save-Surface "export-$tag"
    Invoke-Action 'close-export'
    Invoke-Action 'back-library'
}

function Invoke-Tour {
    Invoke-Action 'open-settings'; Wait-Surface 'settings'
    $languages = @(Invoke-Js "[...document.getElementById('language-select').options].map(o => o.value).filter(v => v !== 'system')")
    $brands = @(Invoke-Js "[...document.getElementById('brand-select').options].map(o => o.value)")
    Invoke-Action 'close-settings'
    $defaultBrand = (Invoke-Tauri 'get_settings_defaults').defaults.brand
    $passes = @(foreach ($language in $languages) {
        @{ language = $language; theme = 'light'; brand = $defaultBrand }
        if ($language -in 'en', 'de') { @{ language = $language; theme = 'dark'; brand = $defaultBrand } }
    })
    # The other presets once in English, light and dark, to choose the default from.
    $passes += @(foreach ($brand in $brands | Where-Object { $_ -ne $defaultBrand }) {
        foreach ($theme in 'light', 'dark') { @{ language = 'en'; theme = $theme; brand = $brand } }
    })
    $staleTitles = New-Object System.Collections.Generic.List[string]
    foreach ($pass in $passes) {
        Wait-Surface 'library'
        Invoke-Action 'open-settings'; Wait-Surface 'settings'
        Set-Choice 'language-select' $pass.language
        Set-Choice 'theme-select' $pass.theme
        Set-Choice 'brand-select' $pass.brand
        $shown = Invoke-Js "document.getElementById('setting-title_template')?.placeholder ?? ''"
        if ($shown -ne (Invoke-Tauri 'get_settings').language_title_template) { $staleTitles.Add("$($pass.language): $shown") }
        Invoke-Action 'close-settings'
        $tag = "$($pass.language)-$($pass.theme)"
        if ($pass.brand -ne $defaultBrand) { $tag = "$($pass.brand)-$tag" }
        Save-TourPass $tag
    }
    Add-Check 'settings: title template follows the language' ($staleTitles.Count -eq 0) ($staleTitles -join '; ')
    $widest = ($script:pillWidths | Measure-Object -Property width -Maximum).Maximum
    Add-Check 'capture pill: compact in every language' ($widest -le $pillMaxWidth) ("max $widest px of $pillMaxWidth; " + (($script:pillWidths | ForEach-Object { "$($_.tag) $($_.width)" }) -join ', '))
    Add-Check 'annotate: toolbar visible without scrolling' ($script:hiddenToolbars.Count -eq 0) ($script:hiddenToolbars -join ', ')
    return $languages
}

# --- Synthetic input into our own test window --------------------------------

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Threading;
public static class SmokeInput {
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] static extern void mouse_event(uint flags, int dx, int dy, int data, UIntPtr extra);
    public static void Click(int x, int y) {
        SetCursorPos(x, y); Thread.Sleep(80);
        mouse_event(0x0002, 0, 0, 0, UIntPtr.Zero); Thread.Sleep(60);
        mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero);
    }
    public static void Wheel(int x, int y, int delta) {
        SetCursorPos(x, y); Thread.Sleep(80);
        mouse_event(0x0800, 0, 0, delta, UIntPtr.Zero);
    }
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out Rect rect);
    [DllImport("user32.dll")] public static extern bool GetWindowDisplayAffinity(IntPtr hwnd, out uint affinity);
}
'@

# 0x11 is WDA_EXCLUDEFROMCAPTURE: the window stays out of every screenshot.
function Get-AppWindow {
    $handle = (Get-Process -Id $app.Id).MainWindowHandle
    $affinity = [uint32]0
    [void][SmokeInput]::GetWindowDisplayAffinity($handle, [ref]$affinity)
    $rect = New-Object 'SmokeInput+Rect'
    [void][SmokeInput]::GetWindowRect($handle, [ref]$rect)
    return [pscustomobject]@{
        affinity = $affinity; left = $rect.Left; top = $rect.Top
        width = $rect.Right - $rect.Left; height = $rect.Bottom - $rect.Top
    }
}

function Format-AppWindow($window) {
    return ('affinity 0x{0:X} at {1},{2} size {3}x{4}' -f $window.affinity, $window.left, $window.top, $window.width, $window.height)
}

function Invoke-Pump([int]$milliseconds) {
    $until = (Get-Date).AddMilliseconds($milliseconds)
    while ((Get-Date) -lt $until) {
        [System.Windows.Forms.Application]::DoEvents()
        Start-Sleep -Milliseconds 15
    }
}

function Get-Center($control) {
    $center = New-Object System.Drawing.Point ([int]($control.Width / 2)), ([int]($control.Height / 2))
    return $control.PointToScreen($center)
}

function New-TestForm {
    $form = New-Object System.Windows.Forms.Form
    $form.Text = $formTitle
    $form.TopMost = $true
    $form.StartPosition = 'Manual'
    $form.Location = New-Object System.Drawing.Point 40, 40
    $form.Size = New-Object System.Drawing.Size 440, 380
    $save = New-Object System.Windows.Forms.Button
    $save.Text = 'Save'
    $save.Location = New-Object System.Drawing.Point 20, 20
    $save.Size = New-Object System.Drawing.Size 120, 36
    $box = New-Object System.Windows.Forms.TextBox
    $box.Location = New-Object System.Drawing.Point 20, 80
    $box.Size = New-Object System.Drawing.Size 380, 24
    $list = New-Object System.Windows.Forms.ListBox
    $list.Location = New-Object System.Drawing.Point 20, 120
    $list.Size = New-Object System.Drawing.Size 380, 200
    1..200 | ForEach-Object { [void]$list.Items.Add("Item $_") }
    $form.Controls.AddRange(@($save, $box, $list))
    return @{ form = $form; save = $save; box = $box; list = $list }
}

# --- Test run -----------------------------------------------------------------

$app = $null
try {
    [SmokeInput]::SetProcessDPIAware() | Out-Null
    Write-Log 'smoke start'
    Wait-Until { Get-Process explorer -ErrorAction SilentlyContinue } 120 'desktop' | Out-Null

    $configDir = Join-Path $env:APPDATA 'how.hows'
    New-Item -ItemType Directory -Force $configDir | Out-Null
    [IO.File]::WriteAllText((Join-Path $configDir 'settings.json'),
        '{"hotkey":"Ctrl+Shift+F9","pause_hotkey":"Ctrl+Shift+F10","language":"en"}')

    $webview = Get-WebView2Version
    Write-Log "webview2 before: $webview"
    $runtimeSetup = Get-ChildItem (Join-Path $root 'bundle\webview2\*.exe') -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $webview -and $runtimeSetup) {
        Start-Process $runtimeSetup.FullName -ArgumentList '/silent', '/install' -Wait | Out-Null
        $webview = Get-WebView2Version
    }
    Add-Check 'webview2: runtime present' ([bool]$webview) "$webview"

    $setup = Get-ChildItem (Join-Path $root 'bundle\Hows_*-setup.exe') -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($setup) {
        $install = Start-Process $setup.FullName -ArgumentList '/S' -Wait -PassThru
        Add-Check 'install: NSIS silent install' ($install.ExitCode -eq 0) "exit $($install.ExitCode), $($setup.Name)"
    }
    $exe = @(
        (Join-Path $env:LOCALAPPDATA 'Hows\hows.exe'),
        (Join-Path $env:ProgramFiles 'Hows\hows.exe'),
        (Join-Path $root 'bundle\hows.exe')
    ) | Where-Object { Test-Path $_ } | Select-Object -First 1
    if (-not $exe) { throw 'hows.exe not found (neither installed nor portable).' }
    Write-Log "exe $exe"

    # An instance started by the installer would own the WebView2 profile
    # without the debugging port.
    Start-Sleep -Seconds 3
    $early = @(Get-Process hows -ErrorAction SilentlyContinue)
    Write-Log "instances before launch: $($early.Count)"
    $early | Stop-Process -Force
    Start-Sleep -Seconds 2
    # The sandbox user is elevated; since runtime 150 WebView2 then ignores
    # WEBVIEW2_* variables and HKCU policies and only honors the HKLM policy.
    $policy = 'HKLM:\SOFTWARE\Policies\Microsoft\Edge\WebView2\AdditionalBrowserArguments'
    New-Item -Path $policy -Force | Out-Null
    New-ItemProperty -Path $policy -Name 'hows.exe' -PropertyType String -Value '--remote-debugging-port=9333' -Force | Out-Null
    # A zone far from UTC shows that {date} and {time} use local time.
    Set-TimeZone -Id 'Tokyo Standard Time'
    [TimeZoneInfo]::ClearCachedData()
    $app = Start-Process $exe -PassThru `
        -RedirectStandardOutput (Join-Path $results 'app.stdout.log') `
        -RedirectStandardError (Join-Path $results 'app.stderr.log')
    Connect-Cdp 9333
    Wait-Until { (Invoke-Js 'typeof window.__TAURI_INTERNALS__') -eq 'object' } 60 'Tauri bridge' | Out-Null
    # Both domains first replay what the page logged since it loaded, so CSP
    # refusals during startup are counted too.
    Send-Cdp 'Log.enable' @{} | Out-Null
    Send-Cdp 'Runtime.enable' @{} | Out-Null

    $settings = Invoke-Tauri 'get_settings'
    Add-Check 'settings: hotkey taken from the seeded file' ($settings.hotkey -eq 'Ctrl+Shift+F9') $settings.hotkey
    Add-Check 'hotkeys: all registered' (@($settings.unregistered_hotkeys).Count -eq 0) (@($settings.unregistered_hotkeys) -join ',')
    Add-Check 'export folder: not cloud-synced' (-not $settings.export_folder_synced) $settings.export_folder_display
    $title = Wait-Until { $t = Invoke-Js 'document.title'; if ($t -like 'Hows*') { $t } } 30 'window title'
    Add-Check 'window title: Hows' $true $title
    New-Item -ItemType Directory -Force -Path $tourDir | Out-Null
    Save-Surface 'library-empty'

    # The app starts in English. Switching while it runs proves that the
    # recorder, not only the UI, picks up the new language.
    Invoke-Action 'open-settings'; Wait-Surface 'settings'
    # "Match Windows" comes first and resolves to the display language of the sandbox.
    $firstChoice = Invoke-Js "document.getElementById('language-select').options[0].value"
    Set-Choice 'language-select' 'system'
    $windowsLanguage = (Get-UICulture).TwoLetterISOLanguageName
    $expectedLanguage = if ($windowsLanguage -in 'en', 'de', 'fr', 'es', 'it', 'pt', 'nl') { $windowsLanguage } else { 'en' }
    $system = Wait-Until { $s = Invoke-Tauri 'get_settings'; if ($s.language -eq 'system') { $s } } 10 'language system'
    $shownChoice = Invoke-Js "document.getElementById('language-select').value"
    $uiLanguage = Invoke-Js "document.querySelector('[data-action=close-settings]')?.getAttribute('aria-label') ?? ''"
    Add-Check 'language: system follows Windows' ($firstChoice -eq 'system' -and $shownChoice -eq 'system' -and $system.resolved_language -eq $expectedLanguage) "first $firstChoice, shown $shownChoice, resolved $($system.resolved_language), Windows $windowsLanguage, $uiLanguage"
    Save-Surface 'settings-language-system'
    Set-Choice 'language-select' 'de'
    Invoke-Action 'close-settings'; Wait-Surface 'library'
    $language = Wait-Until { $l = (Invoke-Tauri 'get_settings').language; if ($l -eq 'de') { $l } } 10 'language de'
    Add-Check 'language: switched to de while running' ($language -eq 'de') $language

    $ui = New-TestForm
    $ui.form.Show()
    Invoke-Pump 800

    $windowBefore = Get-AppWindow
    Invoke-Tauri 'recorder_command' "{name:'start'}" | Out-Null
    Invoke-Pump 1500
    $pillHint = Invoke-Js "['hotkey-pause', 'hotkey-stop'].map(p => document.querySelector('[data-part=' + p + ']')?.textContent.trim() ?? '').join(' | ')"
    Add-Check 'capture pill: shows configured hotkeys' ($pillHint -eq 'Strg+Umschalt+F10 | Strg+Umschalt+F9') $pillHint
    $windowDuring = Wait-Until { $w = Get-AppWindow; if ($w.height -lt 120) { $w } } 10 'compact capture window'
    $screenWidth = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width
    $centered = [math]::Abs(($windowDuring.left + $windowDuring.width / 2) - $screenWidth / 2) -le 4
    $compact = $windowDuring.affinity -eq 0x11 -and $windowDuring.width -lt 800 -and $windowDuring.top -lt 60 -and $centered
    Add-Check 'capture pill: compact, top-center, excluded from capture' $compact ("before $(Format-AppWindow $windowBefore), during $(Format-AppWindow $windowDuring)")
    Save-Surface 'capture'
    Save-Screenshot 'capture-desktop.png'

    $point = Get-Center $ui.save
    [SmokeInput]::Click($point.X, $point.Y)
    Invoke-Pump 1200
    $point = Get-Center $ui.box
    [SmokeInput]::Click($point.X, $point.Y)
    Invoke-Pump 800
    [System.Windows.Forms.SendKeys]::SendWait($typedSecret)
    Invoke-Pump 800
    [System.Windows.Forms.SendKeys]::SendWait('^s')
    Invoke-Pump 800
    $point = Get-Center $ui.list
    foreach ($notch in 1..3) {
        [SmokeInput]::Wheel($point.X, $point.Y, -120)
        Invoke-Pump 150
    }
    Invoke-Pump 1000

    Invoke-Tauri 'recorder_command' "{name:'stop'}" | Out-Null
    $guide = Wait-Until { $g = Invoke-Tauri 'get_guide'; if ($g -and @($g.steps).Count -gt 0) { $g } } 30 'recorded guide'
    $windowAfter = Wait-Until { $w = Get-AppWindow; if ($w.height -ge 120) { $w } } 10 'restored main window'
    $restored = $windowAfter.affinity -eq 0 -and
        [math]::Abs($windowAfter.left - $windowBefore.left) -le 2 -and [math]::Abs($windowAfter.top - $windowBefore.top) -le 2 -and
        [math]::Abs($windowAfter.width - $windowBefore.width) -le 2 -and [math]::Abs($windowAfter.height - $windowBefore.height) -le 2
    Add-Check 'capture pill: window restored after stop' $restored ("before $(Format-AppWindow $windowBefore), after $(Format-AppWindow $windowAfter)")
    Save-Screenshot 'after-recording.png'
    $ui.form.Close()

    $steps = @($guide.steps)
    $actions = @($steps | ForEach-Object { $_.action })
    Write-Log ('steps: ' + (($steps | ForEach-Object { "$($_.action) | $($_.text)" }) -join ' || '))
    $saveClick = $steps | Where-Object { $_.action -eq 'click' -and $_.text -match 'Save' } | Select-Object -First 1
    Add-Check 'record: click names the button' ([bool]$saveClick) ($actions -join ', ')
    Add-Check 'record: typing step' ($actions -contains 'text_input') ''
    Add-Check 'record: shortcut step' (@($actions | Where-Object { $_ -like 'key_combo Ctrl+S*' }).Count -eq 1) ''
    Add-Check 'record: scroll merged into one step' (@($actions | Where-Object { $_ -like 'scroll Down*' }).Count -eq 1) (@($actions | Where-Object { $_ -like 'scroll*' }) -join ', ')
    Add-Check 'privacy: typed text not stored' (-not ($guide | ConvertTo-Json -Depth 8).Contains($typedSecret)) ''
    Add-Check 'title: no foreign window title' (-not "$($guide.title)".Contains($formTitle)) $guide.title
    Add-Check 'language: steps follow the switch' ("$($saveClick.text)" -like 'Klicke auf*') "$($saveClick.text)"
    Add-Check 'language: title follows the switch' ("$($guide.title)" -like 'Anleitung f*') $guide.title
    # The test form runs inside powershell.exe; the title must use its product name.
    Add-Check 'title: readable app name' ("$($guide.title)".Contains('Windows PowerShell')) $guide.title
    if ($saveClick) {
        $prefix = Invoke-Js "window.__TAURI_INTERNALS__.invoke('get_step_image', {id:'$($saveClick.id)'}).then(u => u ? u.slice(0, 22) : null)"
        Add-Check 'screenshot: click step has image' ($prefix -eq 'data:image/png;base64,') $prefix
    }

    $saved = Invoke-Tauri 'save_guide'
    Add-Check 'save: .steps written' (Test-Path $saved) $saved
    $recent = @(Invoke-Tauri 'get_recent_guides')
    Add-Check 'save: listed first in recents' ($recent.Count -gt 0 -and $recent[0].path -eq $saved) ''

    $html = Invoke-Tauri 'export_html'
    Add-Check 'export: html' ((Test-Path $html) -and -not ([IO.File]::ReadAllText($html)).Contains($typedSecret)) $html
    $pdf = Invoke-Tauri 'export_pdf'
    $header = if (Test-Path $pdf) { [Text.Encoding]::ASCII.GetString([IO.File]::ReadAllBytes($pdf), 0, 8) } else { '' }
    Add-Check 'export: pdf' ($header -eq '%PDF-1.4') "$pdf ($header)"
    $json = Invoke-Tauri 'export_json'
    Add-Check 'export: json' (Test-Path $json) $json
    $markdown = Invoke-Tauri 'export_markdown'
    Add-Check 'export: markdown' ((Test-Path $markdown) -and -not ([IO.File]::ReadAllText($markdown)).Contains($typedSecret)) $markdown
    Copy-Item -Path $saved, $html, $pdf, $json -Destination $results -ErrorAction SilentlyContinue
    Copy-Item -Path (Split-Path $markdown) -Destination (Join-Path $results 'markdown') -Recurse -ErrorAction SilentlyContinue

    # A taken name gets a free sibling name, so only a folder that refuses new files
    # makes the export fail. The interface names the problem in its own language,
    # never with the operating system's words.
    $exportFolder = Split-Path $html
    icacls $exportFolder /deny '*S-1-1-0:(WD,AD)' | Out-Null
    try {
        Invoke-Action 'open-export'
        Invoke-Action 'run-export'
        $exportError = Wait-Until { Invoke-Js "(() => { const s = document.querySelector('.sl-status'); return s && s.textContent.includes('Exportordner schreiben') ? s.textContent.trim() : ''; })()" } 10 'export error message'
        Save-Surface 'export-error'
        Invoke-Action 'close-export'
    } finally {
        icacls $exportFolder /remove:d '*S-1-1-0' | Out-Null
    }
    Add-Check 'errors: failed export explained in the interface language' ("$exportError" -notmatch 'os error|\(\d+\)|Error') "$exportError"

    $newer = Join-Path $env:TEMP 'hows-smoke-newer.steps'
    Add-Type -AssemblyName System.IO.Compression, System.IO.Compression.FileSystem
    $zip = [IO.Compression.ZipFile]::Open($newer, 'Create')
    $writer = New-Object IO.StreamWriter(($zip.CreateEntry('guide.json')).Open())
    $writer.Write('{"schema_version":2,"title":"from the future","steps":[]}')
    $writer.Dispose(); $zip.Dispose()
    $reply = Invoke-Js "window.__TAURI_INTERNALS__.invoke('open_guide', { path: $(ConvertTo-Json $newer) }).then(() => 'opened', (e) => JSON.stringify(e))"
    Remove-Item $newer
    $newerCode = try { ($reply | ConvertFrom-Json).code } catch { '' }
    Add-Check 'errors: newer guide answers with a code, not a sentence' ($newerCode -eq 'newer_version') "$reply"

    # save_guide came from outside the UI; the guide still counts as saved.
    $dialogOpen = "!!document.querySelector('[data-part=confirm-dialog]')"
    Invoke-Action 'back-library'
    Wait-Surface 'library'
    Add-Check 'save: back after saving asks nothing' (-not (Invoke-Js $dialogOpen)) ''
    $listed = Wait-Until { Invoke-Js "!!document.querySelector('[data-action=open-recent]')" } 10 'recent guide in library'
    Add-Check 'library: saved guide listed after returning' ([bool]$listed) ''
    $thumbWidth = Wait-Until { Invoke-Js "document.querySelector('[data-part=recent-thumbnail]')?.naturalWidth ?? 0" } 10 'library thumbnail'
    $card = Invoke-Js "(() => { const c = document.querySelector('[data-action=open-recent]'); return c.querySelector('.sl-recent-name').textContent.trim() + ' | ' + c.querySelector('[data-part=recent-meta]').textContent.trim(); })()"
    $cardOk = $thumbWidth -gt 0 -and $thumbWidth -le 960 -and "$card" -like "$($guide.title) | $(@($guide.steps).Count) Schritte *"
    Add-Check 'library: card shows thumbnail, title, step count and date' $cardOk "thumbnail $thumbWidth px, $card"

    # A guide that was deleted since stays listed as missing until it is opened once.
    $gone = Join-Path $env:TEMP 'hows-smoke-gone.steps'
    Copy-Item $saved $gone
    Invoke-Tauri 'open_guide' "{path:$(ConvertTo-Json $gone)}" | Out-Null
    Wait-Surface 'editor'
    Remove-Item $gone
    Invoke-Action 'back-library'; Wait-Surface 'library'
    $missingCard = Wait-Until { Invoke-Js "(() => { const c = document.querySelector('[data-action=open-recent].missing'); return c ? c.querySelector('[data-part=recent-meta]').textContent.trim() : ''; })()" } 10 'missing card'
    $missingBadge = Invoke-Js "!!document.querySelector('[data-action=open-recent].missing .sl-recent-badge')"
    Save-Surface 'library-missing'
    Invoke-Action 'open-recent'
    $pruned = Wait-Until { Invoke-Js "!document.querySelector('[data-action=open-recent].missing') && (document.querySelector('.sl-status')?.textContent ?? '').includes('nicht gefunden')" } 10 'missing guide pruned'
    Add-Check 'library: missing guide marked, then dropped when opened' ("$missingCard" -eq 'Datei fehlt' -and -not $missingBadge -and [bool]$pruned) "$missingCard, badge $missingBadge"

    # Advanced settings through the page itself: file name template and paper
    # size reach the export, and the reset buttons restore the defaults.
    Invoke-Action 'open-settings'; Wait-Surface 'settings'
    $controls = Invoke-Js "(() => [...document.querySelectorAll('.sl-settings-control select, .sl-hotkey-capture, .sl-settings-path')].map(el => { const r = el.getBoundingClientRect(); return Math.round(r.width) + '@' + Math.round(r.right); }))()"
    $shapes = @($controls | Sort-Object -Unique)
    $folder = Invoke-Js "(() => { const p = document.querySelector('[data-part=folder-path]'); return { shown: p.textContent.trim(), title: p.title, clipped: p.scrollWidth > p.clientWidth }; })()"
    $folderOk = -not $folder.clipped -and $folder.title -eq $settings.export_folder_display -and
        $folder.shown.Split('\')[-1] -eq $folder.title.Split('\')[-1]
    Add-Check 'settings: controls share one width, folder path readable' ($shapes.Count -eq 1 -and @($controls).Count -ge 7 -and $folderOk) ("$(@($controls).Count) controls as $($shapes -join ', '); folder '$($folder.shown)' of '$($folder.title)'")
    Open-Advanced
    Set-Field 'file_name_template' '{date} {time} {title}'
    Set-Field 'pdf_paper' 'letter'
    Save-Surface 'settings-advanced-changed'
    Invoke-Action 'close-settings'; Wait-Surface 'library'
    Invoke-Action 'open-recent'; Wait-Surface 'editor'
    $letterPdf = "$(Invoke-Tauri 'export_pdf')"
    $stem = [IO.Path]::GetFileNameWithoutExtension($letterPdf)
    $invariant = [Globalization.CultureInfo]::InvariantCulture
    $createdAt = [DateTimeOffset]::FromUnixTimeMilliseconds([int64]$recent[0].created_at_ms)
    $localStamp = $createdAt.ToLocalTime().ToString('yyyy-MM-dd HH.mm', $invariant)
    $utcStamp = $createdAt.UtcDateTime.ToString('yyyy-MM-dd HH.mm', $invariant)
    Add-Check 'settings: file name fills {date} {time} in local time' ($stem.StartsWith("$localStamp ") -and $localStamp -ne $utcStamp) "$stem (local $localStamp, UTC $utcStamp, $((Get-TimeZone).Id))"
    $pdfText = if (Test-Path $letterPdf) { [Text.Encoding]::GetEncoding(28591).GetString([IO.File]::ReadAllBytes($letterPdf)) } else { '' }
    $mediaBox = [regex]::Match($pdfText, '/MediaBox\s*\[[^\]]*\]').Value
    Add-Check 'settings: pdf uses letter paper' ($mediaBox -match '^/MediaBox\s*\[\s*0\s+0\s+612(\.0+)?\s+792(\.0+)?\s*\]$') $mediaBox

    # An unsaved change asks in the app's own dialog; Cancel keeps the editor.
    Invoke-Js "(() => { const t = document.getElementById('guide-title'); t.value = t.value + ' (smoke)'; t.dispatchEvent(new Event('input', { bubbles: true })); t.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
    Start-Sleep -Milliseconds 400
    Invoke-Action 'back-library'
    $asked = Wait-Until { Invoke-Js $dialogOpen } 5 'discard dialog'
    $dialogText = Invoke-Js "document.querySelector('[data-part=confirm-dialog]')?.innerText ?? ''"
    Save-Surface 'confirm-discard'
    Invoke-Action 'confirm-cancel'
    $kept = (Invoke-Js "document.querySelector('main')?.dataset.surface ?? ''") -eq 'editor' -and -not (Invoke-Js $dialogOpen)
    Invoke-Action 'back-library'
    Wait-Until { Invoke-Js $dialogOpen } 5 'discard dialog again' | Out-Null
    Invoke-Action 'confirm-accept'
    Wait-Surface 'library'
    $confirmOk = [bool]$asked -and $kept -and "$dialogText" -match 'verwerfen' -and "$dialogText" -match 'Abbrechen'
    Add-Check 'confirm: in-app dialog, cancel keeps, discard leaves' $confirmOk ("$dialogText" -replace '\s+', ' ')
    Invoke-Action 'open-settings'; Wait-Surface 'settings'
    Open-Advanced
    Invoke-Action 'reset-advanced'
    $resetButtons = Invoke-Js "document.querySelectorAll('[data-action=reset-setting]').length"
    Invoke-Action 'reset-setting'
    $afterReset = Invoke-Tauri 'get_settings'
    $restored = $afterReset.file_name_template -eq '{title}' -and $afterReset.pdf_paper -eq 'a4' -and $resetButtons -eq 1
    Add-Check 'settings: reset restores the defaults' $restored "template $($afterReset.file_name_template), paper $($afterReset.pdf_paper), $resetButtons reset button(s)"

    # Style preset and own accent recolor the page and reset back to the preset.
    $accentOf = "getComputedStyle(document.documentElement).getPropertyValue('--sl-accent').trim().toLowerCase()"
    $presetAccent = Invoke-Js $accentOf
    $otherBrand = Invoke-Js "(() => { const s = document.getElementById('brand-select'); return [...s.options].map(o => o.value).find(v => v !== s.value); })()"
    Set-Choice 'brand-select' $otherBrand
    $otherAccent = Invoke-Js $accentOf
    Set-Field 'accent_color' '#aa3366'
    $ownAccent = Invoke-Js $accentOf
    $styleButtons = Invoke-Js "document.querySelectorAll('[data-action=reset-setting]').length"
    Invoke-Action 'reset-setting'; Invoke-Action 'reset-setting'
    $afterStyle = Invoke-Tauri 'get_settings'
    $backAccent = Invoke-Js $accentOf
    $styled = $otherAccent -ne $presetAccent -and $ownAccent -notin $presetAccent, $otherAccent -and
        $styleButtons -eq 2 -and $afterStyle.accent_color -eq '' -and $backAccent -eq $presetAccent
    Add-Check 'settings: style and accent recolor and reset' $styled "preset $presetAccent, $otherBrand $otherAccent, own $ownAccent, back $backAccent, $styleButtons reset button(s)"
    $font = Invoke-Js "document.fonts.ready.then(() => [...document.fonts].filter(f => f.status === 'loaded').map(f => f.family).join(','))"
    Add-Check 'design: brand font loaded' ("$font" -match 'Atkinson Hyperlegible Next') "$font"

    # Style, own accent and the credit line reach HTML, PDF and Markdown.
    $credit = 'Erstellt mit Hows'
    $titleMark = 'Windows PowerShell'
    Set-Choice 'theme-select' 'light'
    Set-Choice 'brand-select' $otherBrand
    Set-Field 'accent_color' '#aa3366'
    $uiAccent = Invoke-Js $accentOf
    Invoke-Action 'close-settings'; Wait-Surface 'library'
    Invoke-Action 'open-recent'; Wait-Surface 'editor'
    $on = Export-Documents 'credit-on'
    $htmlAccent = [regex]::Match($on.html, ':root\{[^}]*?--sl-accent:(#[0-9A-Fa-f]{6});').Groups[1].Value
    Add-Check 'export: html uses style and own accent' ($htmlAccent -and $htmlAccent -eq $uiAccent -and $htmlAccent -ne $presetAccent) "html $htmlAccent, app $uiAccent, preset $presetAccent"
    $offline = $on.html.Contains('src:url(data:font/woff2;base64,') -and $on.html.Contains('Atkinson Hyperlegible Next') -and
        -not $on.html.Contains('http://') -and -not $on.html.Contains('https://')
    Add-Check 'export: html embeds the brand font offline' $offline "$($on.html.Length) chars"
    $fontNames = @([regex]::Matches($on.pdfRaw, '/FontName /([A-Z]{6}\+[A-Za-z-]+)') | ForEach-Object { $_.Groups[1].Value })
    $embedded = $fontNames.Count -eq 2 -and @($fontNames | Where-Object { $_ -match '\+AtkinsonHyperlegibleNext-(Regular|Bold)$' }).Count -eq 2 -and
        $on.pdfRaw.Contains('/FontFile2') -and $on.pdfRaw.Contains('/Identity-H') -and -not $on.pdfRaw.Contains('/Helvetica')
    Add-Check 'export: pdf embeds the brand font' $embedded ($fontNames -join ', ')
    Add-Check 'export: pdf text readable through ToUnicode' ($on.pdfText.Contains($titleMark)) (($on.pdfText -split "`n" | Select-Object -First 3) -join ' | ')
    $creditOn = $on.html.Contains("<footer class=`"credit`">$credit</footer>") -and $on.pdfText.Contains($credit) -and
        $on.markdown.TrimEnd().EndsWith("*$credit*")
    Add-Check 'export: credit in html, pdf and markdown by default' $creditOn "html $($on.html.Contains($credit)), pdf $($on.pdfText.Contains($credit)), md $($on.markdown.Contains($credit))"

    Invoke-Action 'back-library'; Wait-Surface 'library'
    Invoke-Action 'open-settings'; Wait-Surface 'settings'
    $switched = Invoke-Js "(() => { const el = document.querySelector('[data-setting=export_credit]'); if (!el) return false; if (el.checked) el.click(); return !el.checked; })()"
    Start-Sleep -Milliseconds 600
    $creditButtons = Invoke-Js "document.querySelectorAll('[data-action=reset-setting]').length"
    Save-Surface 'settings-export-credit-off'
    Add-Check 'settings: credit switch off, own reset button' ([bool]$switched -and -not (Invoke-Tauri 'get_settings').export_credit -and $creditButtons -eq 3) "$creditButtons reset button(s)"
    Invoke-Action 'close-settings'; Wait-Surface 'library'
    Invoke-Action 'open-recent'; Wait-Surface 'editor'
    $off = Export-Documents 'credit-off'
    $creditOff = -not $off.html.Contains($credit) -and -not $off.html.Contains('class="credit"') -and
        -not $off.markdown.Contains($credit) -and $off.pdfText.Contains($titleMark) -and -not $off.pdfText.Contains($credit)
    Add-Check 'export: credit switched off everywhere' $creditOff "html $($off.html.Contains($credit)), pdf $($off.pdfText.Contains($credit)), md $($off.markdown.Contains($credit))"

    Invoke-Action 'back-library'; Wait-Surface 'library'
    Invoke-Action 'open-settings'; Wait-Surface 'settings'
    Invoke-Action 'reset-setting'; Invoke-Action 'reset-setting'; Invoke-Action 'reset-setting'
    Set-Choice 'theme-select' 'system'
    $afterExport = Invoke-Tauri 'get_settings'
    $exportReset = $afterExport.export_credit -and $afterExport.accent_color -eq '' -and
        $afterExport.brand -eq (Invoke-Tauri 'get_settings_defaults').defaults.brand -and
        (Invoke-Js "document.querySelectorAll('[data-action=reset-setting]').length") -eq 0
    Add-Check 'settings: reset brings the credit back' $exportReset "credit $($afterExport.export_credit), brand $($afterExport.brand), accent '$($afterExport.accent_color)'"
    Invoke-Action 'close-settings'; Wait-Surface 'library'

    # A rectangle drawn with the mouse reaches HTML and PDF in the color of core/store/marks.json.
    $marks = Get-Content (Join-Path $root 'bundle\marks.json') -Raw -Encoding UTF8 | ConvertFrom-Json
    $markRgb = ConvertTo-Rgb $marks.color
    Invoke-Action 'open-recent'; Wait-Surface 'editor'
    Wait-Until { Invoke-Js "!!document.querySelector('.sl-stage-wrap .shot img')?.naturalWidth" } 15 'step image' | Out-Null
    $unmarked = Export-Documents 'unmarked'
    Invoke-Action 'toggle-annotate'
    Invoke-Js "(() => { const b = document.querySelector('[data-tool=rect]'); if (b) b.click(); return !!b; })()" | Out-Null
    Start-Sleep -Milliseconds 300
    $box = Invoke-Js "(() => { const r = document.querySelector('.sl-stage-wrap .shot img').getBoundingClientRect(); return { x: r.left, y: r.top, w: r.width, h: r.height }; })()"
    Send-Mouse 'mouseMoved' ($box.x + $box.w * 0.3) ($box.y + $box.h * 0.3) 0
    Send-Mouse 'mousePressed' ($box.x + $box.w * 0.3) ($box.y + $box.h * 0.3) 1
    foreach ($share in 0.4, 0.5, 0.6, 0.7) {
        Send-Mouse 'mouseMoved' ($box.x + $box.w * $share) ($box.y + $box.h * $share) 1
    }
    Send-Mouse 'mouseReleased' ($box.x + $box.w * 0.7) ($box.y + $box.h * 0.7) 0
    $drawn = Wait-Until { $o = @((Invoke-Tauri 'get_guide').steps | ForEach-Object { @($_.overlays) } | Where-Object { $_ -and $_.type -eq 'rect' }); if ($o.Count -gt 0) { $o[0] } } 10 'drawn rectangle'
    Save-Surface 'annotate-mark'
    Invoke-Action 'toggle-annotate'
    $marked = Export-Documents 'marked'
    $defaultColor = (Invoke-Tauri 'get_settings_defaults').defaults.annotation_color
    Add-Check 'marks: default color from marks.json' ("$defaultColor" -eq $marks.color -and "$($drawn.color)" -eq $marks.color) "marks.json $($marks.color), default $defaultColor, drawn $($drawn.color)"
    $htmlBefore = Measure-HtmlColor $unmarked.html $markRgb ''
    $htmlAfter = Measure-HtmlColor $marked.html $markRgb (Join-Path $results 'mark-export.png')
    Add-Check 'export: drawn mark in html in the marks.json color' ($htmlBefore -eq 0 -and $htmlAfter -ge 500) "$($marks.color) pixels before $htmlBefore, after $htmlAfter"
    $pdfBefore = Measure-PdfColor $unmarked.pdfRaw $markRgb
    $pdfAfter = Measure-PdfColor $marked.pdfRaw $markRgb
    Add-Check 'export: drawn mark in pdf in the marks.json color' ($pdfBefore -eq 0 -and $pdfAfter -ge 500) "$($marks.color) pixels before $pdfBefore, after $pdfAfter"
    Invoke-Tauri 'save_guide' | Out-Null
    Invoke-Action 'back-library'
    if (Invoke-Js $dialogOpen) { Invoke-Action 'confirm-accept' }
    Wait-Surface 'library'

    # {time} in a title: a recording without input gets the local time of the sandbox.
    Invoke-Tauri 'change_setting' "{change:{field:'title_template',value:'{date} {time}'}}" | Out-Null
    $beforeStop = [DateTimeOffset]::Now
    Invoke-Tauri 'recorder_command' "{name:'start'}" | Out-Null
    Start-Sleep -Milliseconds 800
    Invoke-Tauri 'recorder_command' "{name:'stop'}" | Out-Null
    $timedTitle = Wait-Until { $g = Invoke-Tauri 'get_guide'; if ($g -and "$($g.title)" -match '^\d{4}-\d{2}-\d{2} \d{2}\.\d{2}$') { $g.title } } 15 'title with time'
    $afterStop = [DateTimeOffset]::Now
    $stamps = @($beforeStop, $afterStop | ForEach-Object { $_.ToString('yyyy-MM-dd HH.mm', $invariant) })
    $utcNow = $afterStop.UtcDateTime.ToString('yyyy-MM-dd HH.mm', $invariant)
    Add-Check 'title: template fills {date} {time} in local time' ($timedTitle -in $stamps -and $timedTitle -ne $utcNow) "$timedTitle (local $($stamps -join ' or '), UTC $utcNow)"
    Start-Sleep -Milliseconds 800
    if ((Invoke-Js "document.querySelector('main')?.dataset.surface ?? ''") -ne 'library') { Invoke-Action 'back-library' }
    if (Invoke-Js $dialogOpen) { Invoke-Action 'confirm-accept' }
    Wait-Surface 'library'
    Invoke-Tauri 'change_setting' "{change:{field:'title_template',value:''}}" | Out-Null

    try {
        $languages = Invoke-Tour
        Add-Check 'tour: every surface captured' $true "$($script:tourShots.Count) screenshots, languages $($languages -join ',')"
    } catch {
        Add-Check 'tour: every surface captured' $false "$($_.Exception.Message) after $($script:tourShots -join ',')"
        try { Save-Screenshot 'tour-failure.png' } catch { }
    }

    # Pulls the events still queued on the socket into cdpEvents.
    Invoke-Js 'true' | Out-Null
    $refused = @($script:cdpEvents | ForEach-Object {
            if ($_.method -eq 'Log.entryAdded') { $_.params.entry.text }
            elseif ($_.method -eq 'Runtime.consoleAPICalled') { ($_.params.args | ForEach-Object { $_.value }) -join ' ' }
        } | Where-Object { $_ -match 'Content Security Policy|IPC custom protocol failed' } | Select-Object -Unique)
    Add-Check 'csp: nothing refused, IPC over http://ipc.localhost' ($refused.Count -eq 0) ($refused -join ' | ')
    $probe = Invoke-Js "new Promise(resolve => { const style = document.createElement('style'); addEventListener('securitypolicyviolation', e => { style.remove(); resolve(e.effectiveDirective); }, { once: true }); style.textContent = 'body{}'; document.head.append(style); setTimeout(() => resolve('none'), 2000); })"
    Invoke-Js 'true' | Out-Null
    $logged = @($script:cdpEvents | Where-Object { $_.method -eq 'Log.entryAdded' -and $_.params.entry.text -match 'Content Security Policy' }).Count
    Add-Check 'csp: enforced, inline style refused and logged' ($probe -eq 'style-src-elem' -and $logged -gt 0) "$probe, $logged refusal(s) in the log"
} catch {
    Add-Check 'smoke script' $false $_.Exception.Message
    try { Save-Screenshot 'failure.png' } catch { }
} finally {
    if ($script:socket) { $script:socket.Dispose() }
    if ($app -and -not $app.HasExited) { Stop-Process -Id $app.Id -Force }
    $failed = @($checks | Where-Object { -not $_.ok })
    [pscustomobject]@{
        ok = ($checks.Count -gt 0 -and $failed.Count -eq 0)
        finished = (Get-Date).ToString('o')
        checks = $checks
    } | ConvertTo-Json -Depth 4 | Set-Content -Path (Join-Path $results 'result.json') -Encoding UTF8
    Write-Log 'smoke end'
    if ($inSandbox -and -not (Test-Path (Join-Path $results 'keep-open'))) { shutdown.exe /s /t 5 }
}
