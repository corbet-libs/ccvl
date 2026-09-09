$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$Bootstrap = Join-Path $RepoRoot ".agent\scripts\bootstrap.ps1"
$Scratch = Join-Path ([IO.Path]::GetTempPath()) "ccvl-bootstrap-$([Guid]::NewGuid().ToString('N'))"
$ProbeBin = Join-Path $Scratch "bin"
New-Item -ItemType Directory -Path $ProbeBin -Force | Out-Null
$OldPath = $env:PATH
$OldCache = $env:CCVL_BOOTSTRAP_CACHE_ROOT
$OldForce = $env:CCVL_BOOTSTRAP_FORCE_LOCAL

function Assert-Plan([string]$Expected) {
    $Output = (& $Bootstrap plan --from-source | Out-String)
    if (-not $Output.Contains($Expected)) { throw "Expected '$Expected' in bootstrap plan: $Output" }
    if ($ErrorActionPreference -ne "Stop") { throw "A bootstrap probe changed the caller's error preference" }
    if (Test-Path -LiteralPath $env:CCVL_BOOTSTRAP_CACHE_ROOT) {
        throw "A bootstrap plan changed its cache"
    }
}

try {
    $env:PATH = $ProbeBin
    $env:CCVL_BOOTSTRAP_CACHE_ROOT = Join-Path $Scratch "cache"
    $env:CCVL_BOOTSTRAP_FORCE_LOCAL = "0"
    [IO.File]::WriteAllText((Join-Path $ProbeBin "cargo.cmd"), "@echo cargo 1.97.0 (fixture)`r`n")
    foreach ($Version in @("1.94.0", "1.97.0", "1.100.0", "1.93.9", "1.94.0-nightly", "invalid")) {
        [IO.File]::WriteAllText((Join-Path $ProbeBin "rustc.cmd"), "@echo rustc $Version (fixture)`r`n")
        if ($Version -in @("1.94.0", "1.97.0", "1.100.0")) {
            Assert-Plan "Rust toolchain: system $Version"
        }
        else { Assert-Plan "Rust toolchain: install stable " }
    }
    # Exercise the selector through the actual bootstrap, without Rust or network access.
    foreach ($Selector in @("stable", "1.97.0-test-host")) {
        $Rustup = @"
@echo off
if "%*" == "toolchain list" goto active
if "%*" == "run $Selector rustc --version" goto rustc
if "%*" == "run $Selector cargo --version" goto cargo
echo error: toolchain '%2' is not installed 1>&2
exit /b 1
:active
echo 1.97.0-test-host (default)
exit /b 0
:rustc
echo rustc 1.97.0 (fixture)
exit /b 0
:cargo
echo cargo 1.97.0 (fixture)
exit /b 0
"@
        [IO.File]::WriteAllText((Join-Path $ProbeBin "rustup.cmd"), $Rustup)
        Assert-Plan "Rust toolchain: system 1.97.0"
    }
}
finally {
    $env:PATH = $OldPath
    $env:CCVL_BOOTSTRAP_CACHE_ROOT = $OldCache
    $env:CCVL_BOOTSTRAP_FORCE_LOCAL = $OldForce
    [IO.Directory]::Delete($Scratch, $true)
}
Write-Output "Windows bootstrap reuses suitable stable compilers and rejects unsupported versions."
