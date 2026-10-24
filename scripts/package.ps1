# Builds the release binaries and writes the portable Windows package,
# dist\lambda-slicer-<version>-windows-x86_64.zip, with a SHA-256 file beside
# it. The zip holds the app, the CLI, a short README, the licenses and the
# third-party notices. Earlier output of the same version in dist is replaced.
#
#   .\scripts\package.ps1

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    # Merged in cmd, so progress on stderr is never taken for an error
    cmd /c 'cargo build --release --locked -p lambda-slicer-gui -p lambda-slicer-cli 2>&1'
    if ($LASTEXITCODE -ne 0) { throw 'release build failed' }

    $version = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"' |
        Select-Object -First 1).Matches[0].Groups[1].Value
    $name = "lambda-slicer-$version-windows-x86_64"
    $stage = Join-Path 'dist' $name
    if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
    New-Item -ItemType Directory -Force $stage | Out-Null

    Copy-Item 'target\release\lambda-slicer-gui.exe', 'target\release\lambda-slicer.exe' $stage
    Copy-Item 'packaging\README.txt', 'LICENSE-MIT', 'LICENSE-APACHE' $stage
    python scripts\notices.py x86_64-pc-windows-msvc (Join-Path $stage 'THIRD-PARTY-NOTICES.txt')
    if ($LASTEXITCODE -ne 0) { throw 'notices failed' }

    $commit = (git rev-parse --short HEAD).Trim()
    $dirty = if (git status --porcelain) { ' with uncommitted changes' } else { '' }
    $built = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd HH:mm') + ' UTC'
    @(
        "Lambda Slicer $version",
        "Commit $commit$dirty",
        "Built $built for x86_64-pc-windows-msvc"
    ) | Set-Content -Encoding utf8 (Join-Path $stage 'VERSION.txt')

    $zip = Join-Path 'dist' "$name.zip"
    if (Test-Path $zip) { Remove-Item -Force $zip }
    Compress-Archive -Path $stage -DestinationPath $zip
    $hash = (Get-FileHash -Algorithm SHA256 $zip).Hash.ToLower()
    # One line ending in LF, as sha256sum -c on Linux reads it. Set-Content
    # would end it in CRLF on Windows, which makes the name not match
    Set-Content -NoNewline -Encoding ascii -Path "$zip.sha256" -Value "$hash  $name.zip`n"

    Write-Host "$zip"
    Write-Host "sha256 $hash"
}
finally {
    Pop-Location
}
