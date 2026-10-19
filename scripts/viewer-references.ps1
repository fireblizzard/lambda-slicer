# Captures the viewer's frames of the reference clips and compares them
# with the game's (docs/viewer-validation.md, "Reference captures")
#
# viewer-references.ps1 -Game DIR -Out FOLDER [-Clips NAME,...] [-Length 6] [-Gamma 2,2,2,2]
#
# For each clip, `ls_12d_kill` for example, in the game's `ag` folder, the
# viewer's frames are saved every 0.05 s at 640 by 480 into FOLDER\<clip>,
# through the clip's -Length in seconds and one more, drawn with the game's
# gamma settings. When the game's frames are there too, recorded with
# `startmovie` under the clip's name without its first underscore
# (`ls12d_kill`), the clip's length from the game's first frame that changes
# is compared and a contact sheet of pairs is written to FOLDER\<clip>.png. Build first with
# cargo build --release -p lambda-slicer-gui --example frame_compare

param(
    [Parameter(Mandatory = $true)][string]$Game,
    [Parameter(Mandatory = $true)][string]$Out,
    [string[]]$Clips = @(
        "ls_12d_kill", "ls_12d_gauss", "ls_12d_delayed", "ls_12d_reload", "ls_12d_egon",
        "ls_12d_ambient", "ls_12d_brush", "ls_12d_monster"
    ),
    [double]$Length = 6,
    [string]$Gamma = "2,2,2,2"
)

$ErrorActionPreference = "Stop"
$release = Join-Path $PSScriptRoot "..\target\release"
$viewer = Join-Path $release "lambda-slicer-gui.exe"
$compare = Join-Path $release "examples\frame_compare.exe"
foreach ($tool in @($viewer, $compare)) {
    if (-not (Test-Path $tool)) {
        throw "build the release viewer and frame_compare first, $tool is missing"
    }
}

$culture = [System.Globalization.CultureInfo]::InvariantCulture
$steps = [int][math]::Floor(($Length + 1) / 0.05)
$times = (0..$steps | ForEach-Object { ($_ * 0.05).ToString("0.00", $culture) }) -join ","
$ag = Join-Path $Game "ag"

foreach ($clip in $Clips) {
    $demo = Join-Path $ag "$clip.dem"
    if (-not (Test-Path $demo)) {
        "$clip`: no $demo"
        continue
    }

    # Saved again from scratch, as the viewer keeps no capture it did not
    # make this run
    $frames = Join-Path $Out $clip
    if (Test-Path $frames) {
        Remove-Item -Recurse -Force $frames
    }
    New-Item -ItemType Directory -Force $frames | Out-Null
    $arguments = @(
        "`"$demo`"", "--game", "`"$Game`"", "--viewer-gamma", $Gamma,
        "--viewer-capture", $times, "`"$frames`"", "--viewer-size", "640x480"
    )
    Start-Process -FilePath $viewer -ArgumentList $arguments -Wait

    $movie = $clip -replace "^ls_", "ls"
    if (-not (Test-Path (Join-Path $ag "${movie}00001.bmp"))) {
        "$clip`: captured, the game's frames ($movie) not recorded yet"
        continue
    }
    "$clip`:"
    $sheet = Join-Path $Out "$clip.png"
    $length = $Length.ToString($culture)
    & $compare (Join-Path $ag $movie) $frames --seconds $length --sheet $sheet
}
