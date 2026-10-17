# Runs the checks CI runs: formatting, clippy, tests and a release build.
# With -Linux it runs them again in WSL, building into the Linux home so the
# Windows target directory is left alone. Stops at the first failure.
#
#   .\scripts\check.ps1
#   .\scripts\check.ps1 -Linux

param(
    [switch]$Linux
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    $steps = @(
        'cargo fmt --all -- --check',
        'cargo clippy --workspace --all-targets --locked -- -D warnings',
        'cargo test --workspace --locked',
        'cargo build --release --workspace --locked'
    )
    foreach ($step in $steps) {
        Write-Host "== $step"
        # Merged in cmd, so progress on stderr is never taken for an error
        cmd /c "$step 2>&1"
        if ($LASTEXITCODE -ne 0) { throw "failed: $step" }
    }

    if ($Linux) {
        $wslRoot = (wsl.exe wslpath -a ($root -replace '\\', '/')).Trim()
        $linuxSteps = ($steps | Where-Object { $_ -notlike 'cargo fmt*' }) -join ' && '
        $command = "cd '$wslRoot' && export CARGO_TARGET_DIR=`$HOME/lambda-slicer-target && ($linuxSteps) 2>&1"
        Write-Host "== Linux (WSL): $linuxSteps"
        wsl.exe -e bash -lc $command
        if ($LASTEXITCODE -ne 0) { throw 'failed: Linux checks' }
    }

    Write-Host 'All checks passed'
}
finally {
    Pop-Location
}
