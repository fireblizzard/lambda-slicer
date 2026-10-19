# Times the viewer playing a demo and samples its memory meanwhile
# (docs/viewer-profile.md, "Performance")
#
# viewer-perf.ps1 -Game DIR -Demo FILE [-At SECONDS] [-Seconds 10] [-Size 1920x1080]
#
# Runs the release build's bench for -Seconds from -At, the view drawn
# -Size pixels with vsync off, and prints its report with the process's
# peak private bytes, peak working set and peak dedicated video memory,
# sampled every 100 ms. Build first with
# cargo build --release -p lambda-slicer-gui

param(
    [Parameter(Mandatory = $true)][string]$Game,
    [Parameter(Mandatory = $true)][string]$Demo,
    [double]$At = 0,
    [double]$Seconds = 10,
    [string]$Size = "1920x1080"
)

$ErrorActionPreference = "Stop"
$exe = Join-Path $PSScriptRoot "..\target\release\lambda-slicer-gui.exe"
if (-not (Test-Path $exe)) {
    throw "build the release viewer first, $exe is missing"
}
$report = [System.IO.Path]::GetTempFileName()
$culture = [System.Globalization.CultureInfo]::InvariantCulture
$arguments = @(
    "`"$Demo`"", "--game", "`"$Game`"",
    "--viewer-at", $At.ToString($culture),
    "--viewer-bench", $Seconds.ToString($culture), "`"$report`"",
    "--viewer-size", $Size, "--size", $Size
)
$process = Start-Process -FilePath $exe -ArgumentList $arguments -PassThru

$private = 0
$vram = 0
while (-not $process.HasExited) {
    $process.Refresh()
    if ($process.PrivateMemorySize64 -gt $private) {
        $private = $process.PrivateMemorySize64
    }
    # One instance for each adapter the process uses
    $gpu = Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUProcessMemory |
        Where-Object { $_.Name -like "pid_$($process.Id)_*" } |
        Measure-Object -Property DedicatedUsage -Sum
    if ($gpu.Sum -gt $vram) {
        $vram = $gpu.Sum
    }
    Start-Sleep -Milliseconds 100
}
$peakWorking = $process.PeakWorkingSet64

Get-Content $report
Remove-Item $report
"peak private bytes {0:N0} MiB, peak working set {1:N0} MiB, peak dedicated video memory {2:N0} MiB" -f `
    ($private / 1MB), ($peakWorking / 1MB), ($vram / 1MB)
