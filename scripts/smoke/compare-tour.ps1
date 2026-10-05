<#
.SYNOPSIS
  Puts the before and after screenshot tours side by side.

.DESCRIPTION
  Pairs the PNG files of two tour folders by file name and writes a local
  HTML page with one row per surface, plus one side-by-side PNG per pair.
  Each pair gets the share of changed pixels, sampled on a grid, so the
  biggest changes can be found without opening every image.
  Files that exist only in the after tour are listed as new surfaces.
  The output stays outside the repository.

.PARAMETER Before
  Tour folder of the before state. Default: the baseline saved in PR 0.

.PARAMETER After
  Tour folder of the after state. Default: the newest smoke run.

.PARAMETER Out
  Output folder. Default: %LOCALAPPDATA%\hows-smoke\compare-<timestamp>.

.EXAMPLE
  pwsh scripts/smoke/compare-tour.ps1
#>
[CmdletBinding()]
param(
    [string]$Before,
    [string]$After,
    [string]$Out,
    [int]$SampleStep = 4
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$root = Join-Path $env:LOCALAPPDATA 'hows-smoke'
if (-not $Before) { $Before = Join-Path $root 'baseline-before' }
if (-not $After) {
    $latest = Get-ChildItem $root -Directory |
        Where-Object { $_.Name -match '^\d{8}-\d{6}$' -and (Test-Path (Join-Path $_.FullName 'results\tour')) } |
        Sort-Object Name | Select-Object -Last 1
    if (-not $latest) { throw "No smoke run with results\tour under $root" }
    $After = Join-Path $latest.FullName 'results\tour'
}
if (-not $Out) { $Out = Join-Path $root ('compare-' + (Get-Date -Format 'yyyyMMdd-HHmmss')) }
foreach ($folder in $Before, $After) {
    if (-not (Test-Path $folder)) { throw "Missing tour folder: $folder" }
}
New-Item -ItemType Directory -Force -Path (Join-Path $Out 'pairs') | Out-Null

function Get-ChangedShare([Drawing.Bitmap]$a, [Drawing.Bitmap]$b, [int]$step) {
    if ($a.Width -ne $b.Width -or $a.Height -ne $b.Height) { return $null }
    $changed = 0
    $total = 0
    for ($x = 0; $x -lt $a.Width; $x += $step) {
        for ($y = 0; $y -lt $a.Height; $y += $step) {
            $total++
            if ($a.GetPixel($x, $y).ToArgb() -ne $b.GetPixel($x, $y).ToArgb()) { $changed++ }
        }
    }
    return [math]::Round(100 * $changed / $total, 1)
}

function Save-SideBySide([Drawing.Bitmap]$a, [Drawing.Bitmap]$b, [string]$path) {
    $gap = 16
    $canvas = [Drawing.Bitmap]::new($a.Width + $gap + $b.Width, [math]::Max($a.Height, $b.Height))
    $graphics = [Drawing.Graphics]::FromImage($canvas)
    try {
        $graphics.Clear([Drawing.Color]::White)
        $graphics.DrawImage($a, 0, 0, $a.Width, $a.Height)
        $graphics.DrawImage($b, $a.Width + $gap, 0, $b.Width, $b.Height)
        $canvas.Save($path, [Drawing.Imaging.ImageFormat]::Png)
    } finally {
        $graphics.Dispose()
        $canvas.Dispose()
    }
}

$beforeNames = Get-ChildItem $Before -Filter *.png | ForEach-Object Name
$afterNames = Get-ChildItem $After -Filter *.png | ForEach-Object Name
$rows = foreach ($name in ($beforeNames | Sort-Object)) {
    if ($afterNames -notcontains $name) {
        [pscustomobject]@{ Name = $name; Changed = $null; Note = 'missing after' }
        continue
    }
    $a = [Drawing.Bitmap]::new((Join-Path $Before $name))
    $b = [Drawing.Bitmap]::new((Join-Path $After $name))
    try {
        $changed = Get-ChangedShare $a $b $SampleStep
        $note = if ($null -eq $changed) { "size $($a.Width)x$($a.Height) to $($b.Width)x$($b.Height)" } else { '' }
        Save-SideBySide $a $b (Join-Path $Out "pairs\$name")
    } finally {
        $a.Dispose()
        $b.Dispose()
    }
    [pscustomobject]@{ Name = $name; Changed = $changed; Note = $note }
}
$newOnly = $afterNames | Where-Object { $beforeNames -notcontains $_ } | Sort-Object

function ConvertTo-FileUrl([string]$path) { ([Uri](Resolve-Path $path).Path).AbsoluteUri }

$html = [Text.StringBuilder]::new()
[void]$html.AppendLine('<!doctype html><html lang="en"><meta charset="utf-8"><title>Hows tour before and after</title>')
[void]$html.AppendLine('<style>body{font:14px system-ui,sans-serif;margin:24px;background:#f4f4f2}figure{margin:0 0 32px}figcaption{font-weight:600;margin:0 0 8px}.pair{display:flex;gap:12px;align-items:flex-start}.pair img{max-width:49%;border:1px solid #ccc}.label{color:#555;font-weight:400}</style>')
[void]$html.AppendLine("<h1>Tour before and after</h1><p>Before: $([Net.WebUtility]::HtmlEncode($Before))<br>After: $([Net.WebUtility]::HtmlEncode($After))</p>")
foreach ($row in $rows) {
    $label = if ($null -ne $row.Changed) { "$($row.Changed)% of sampled pixels changed" } else { $row.Note }
    [void]$html.AppendLine("<figure><figcaption>$($row.Name) <span class=`"label`">$label</span></figcaption><div class=`"pair`">")
    [void]$html.AppendLine("<img alt=`"before`" src=`"$(ConvertTo-FileUrl (Join-Path $Before $row.Name))`">")
    if ($row.Note -ne 'missing after') {
        [void]$html.AppendLine("<img alt=`"after`" src=`"$(ConvertTo-FileUrl (Join-Path $After $row.Name))`">")
    }
    [void]$html.AppendLine('</div></figure>')
}
[void]$html.AppendLine("<h2>New surfaces without a before image ($(@($newOnly).Count))</h2><p>$(($newOnly | ForEach-Object { [Net.WebUtility]::HtmlEncode($_) }) -join ', ')</p></html>")
$index = Join-Path $Out 'index.html'
[IO.File]::WriteAllText($index, $html.ToString())

$rows | Sort-Object { if ($null -eq $_.Changed) { 101 } else { $_.Changed } } -Descending |
    Format-Table Name, Changed, Note -AutoSize | Out-String | Write-Output
Write-Output "Pairs: $(@($rows).Count), new surfaces: $(@($newOnly).Count)"
Write-Output "Page: $index"
