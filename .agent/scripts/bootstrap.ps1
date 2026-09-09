param(
    [Parameter(Position = 0)]
    [ValidateSet("plan", "install")]
    [string]$Mode = "plan",
    [Parameter(Position = 1)]
    [ValidateSet("--from-source")]
    [string]$BuildOption
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$CacheRoot = if (Test-Path Env:CCVL_BOOTSTRAP_CACHE_ROOT) {
    $env:CCVL_BOOTSTRAP_CACHE_ROOT
}
else {
    Join-Path $RepoRoot ".agent\cache\ccvl"
}
$LocalBin = Join-Path $CacheRoot "bin"
$CargoHome = Join-Path $CacheRoot "cargo"
$RustupHome = Join-Path $CacheRoot "rustup"
$TargetDir = Join-Path $CacheRoot "target"
$Binary = Join-Path $LocalBin "ccvl.exe"
$FromSource = $BuildOption -eq "--from-source" -or $env:CCVL_BOOTSTRAP_FORCE_LOCAL -eq "1"
$ToolchainFile = Get-Content -Raw (Join-Path $RepoRoot "rust-toolchain.toml")
if ($ToolchainFile -notmatch '(?m)^\s*channel\s*=\s*"([^"]+)"') {
    throw "rust-toolchain.toml does not declare a Rust channel"
}
$RustChannel = $Matches[1]
$CargoManifest = Get-Content -Raw (Join-Path $RepoRoot "Cargo.toml")
if ($CargoManifest -notmatch '(?m)^\s*rust-version\s*=\s*"([^"]+)"') {
    throw "Cargo.toml does not declare rust-version"
}
$RustMinimum = [version]$Matches[1]
if ($RustMinimum.Build -lt 0) {
    $RustMinimum = [version]"$RustMinimum.0"
}

function Get-PlatformKey {
    $Architecture = [Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
    switch ($Architecture) {
        "X64" { return "Windows-x86_64" }
        "Arm64" { return "Windows-aarch64" }
        default { throw "Unsupported Windows architecture: $Architecture" }
    }
}

function Get-CommandPath([string]$Name) {
    $Command = Get-Command $Name -CommandType Application -ErrorAction SilentlyContinue
    if ($null -eq $Command) {
        return $null
    }
    return $Command.Source
}

function Test-RustVersion([string]$Output) {
    return $Output -match '^rustc ([0-9]+\.[0-9]+\.[0-9]+)( |$)' -and
        [version]$Matches[1] -ge $RustMinimum
}

function Invoke-OutsideRepository([string]$Executable, [string[]]$Arguments) {
    $PreviousErrorActionPreference = $ErrorActionPreference
    Push-Location ([IO.Path]::GetTempPath())
    try {
        # Windows PowerShell 5.1 treats redirected native stderr as ErrorRecords.
        # A failed probe must return null so another installed tool can be tried.
        $ErrorActionPreference = "Continue"
        $Output = (& $Executable @Arguments 2>&1 | Out-String).Trim()
        if ($LASTEXITCODE -ne 0) {
            return $null
        }
        return $Output
    }
    catch {
        return $null
    }
    finally {
        $ErrorActionPreference = $PreviousErrorActionPreference
        Pop-Location
    }
}

function Invoke-ManagedRustup([string[]]$Arguments) {
    $Rustup = Join-Path $CargoHome "bin\rustup.exe"
    if (-not (Test-Path -LiteralPath $Rustup -PathType Leaf)) {
        return $null
    }
    $OldCargoHome = $env:CARGO_HOME
    $OldRustupHome = $env:RUSTUP_HOME
    try {
        $env:CARGO_HOME = $CargoHome
        $env:RUSTUP_HOME = $RustupHome
        return Invoke-OutsideRepository $Rustup $Arguments
    }
    finally {
        $env:CARGO_HOME = $OldCargoHome
        $env:RUSTUP_HOME = $OldRustupHome
    }
}

function Find-ExistingRustupToolchain([string]$Rustup, [switch]$Managed) {
    $Candidates = @("stable")
    $ListArguments = @("toolchain", "list")
    $Installed = if ($Managed) { Invoke-ManagedRustup $ListArguments }
        else { Invoke-OutsideRepository $Rustup $ListArguments }
    if ($null -ne $Installed) {
        foreach ($Toolchain in ($Installed -split '\r?\n')) {
            if ($Toolchain -match '\([^)]*\bdefault\b[^)]*\)') {
                $Candidates += ($Toolchain -split ' ')[0]
            }
        }
    }
    foreach ($Candidate in $Candidates) {
        $RustcArguments = @("run", $Candidate, "rustc", "--version")
        $CargoArguments = @("run", $Candidate, "cargo", "--version")
        $Rustc = if ($Managed) { Invoke-ManagedRustup $RustcArguments }
            else { Invoke-OutsideRepository $Rustup $RustcArguments }
        if ($null -eq $Rustc -or -not (Test-RustVersion $Rustc)) { continue }
        $Cargo = if ($Managed) { Invoke-ManagedRustup $CargoArguments }
            else { Invoke-OutsideRepository $Rustup $CargoArguments }
        if ($null -ne $Cargo) { return $Candidate }
    }
    return $null
}

function Test-ManagedRust {
    $script:ManagedToolchain = Find-ExistingRustupToolchain "" -Managed
    if ($null -eq $ManagedToolchain) { return $false }
    $Rustc = Invoke-ManagedRustup @("run", $ManagedToolchain, "rustc", "--version")
    $script:ManagedRustVersion = ($Rustc -split ' ')[1]
    return $true
}

. (Join-Path $PSScriptRoot "runtime-id.ps1")

$Platform = Get-PlatformKey
$ReleaseAsset = switch ($Platform) {
    "Windows-x86_64" { "ccvl-windows-x86_64.exe" }
    "Windows-aarch64" { "ccvl-windows-arm64.exe" }
}
$FetchEnabled = $null -ne $ReleaseAsset -and -not $FromSource
$Assets = @(Import-Csv (Join-Path $PSScriptRoot "tool-assets.csv") |
    Where-Object { $_.tool -eq "rustup-init" -and $_.platform -eq $Platform })
if ($Assets.Count -ne 1) {
    throw "Incomplete rustup-init asset table for $Platform"
}
$Asset = $Assets[0]

$Fingerprint = Get-SourceFingerprint
$ReleaseBase = if (Test-Path Env:CCVL_RELEASE_BASE) { $env:CCVL_RELEASE_BASE } else {
    "https://github.com/corbet-labs/ccvl/releases/download/runtime-$Fingerprint"
}
$BinaryState = "install"
if (Test-Path -LiteralPath $Binary -PathType Leaf) {
    $RuntimeId = Invoke-OutsideRepository $Binary @("runtime-id")
    if ($RuntimeId -eq $Fingerprint) { $BinaryState = "ready" }
}

$SystemKind = "none"
$SystemCargo = $null
$SystemRustup = $null
if ($env:CCVL_BOOTSTRAP_FORCE_LOCAL -ne "1") {
    $CandidateRustup = Get-CommandPath "rustup"
    if ($null -ne $CandidateRustup) {
        $SystemToolchain = Find-ExistingRustupToolchain $CandidateRustup
        if ($null -ne $SystemToolchain) {
            $CandidateRustc = Invoke-OutsideRepository $CandidateRustup @("run", $SystemToolchain, "rustc", "--version")
            $SystemKind = "rustup"
            $SystemRustup = $CandidateRustup
            $SystemRustVersion = ($CandidateRustc -split ' ')[1]
        }
    }
    if ($SystemKind -eq "none" -and $null -eq $CandidateRustup) {
        $CandidateRustcPath = Get-CommandPath "rustc"
        $CandidateCargoPath = Get-CommandPath "cargo"
        if ($null -ne $CandidateRustcPath -and $null -ne $CandidateCargoPath) {
            $CandidateRustc = Invoke-OutsideRepository $CandidateRustcPath @("--version")
            if ($null -ne $CandidateRustc -and (Test-RustVersion $CandidateRustc)) {
                $SystemKind = "standalone"
                $SystemCargo = $CandidateCargoPath
                $SystemRustVersion = ($CandidateRustc -split ' ')[1]
            }
        }
    }
}

$ToolchainState = "install"
if (Test-ManagedRust) {
    $ToolchainState = "managed"
}
elseif ($SystemKind -ne "none") {
    $ToolchainState = "system"
}

Write-Output "ccvl bootstrap plan"
Write-Output "  platform: $Platform"
if (-not $FromSource) {
    Write-Output "  Rust toolchain: not required (precompiled runtime)"
}
else {
switch ($ToolchainState) {
    "managed" { Write-Output "  Rust toolchain: managed $ManagedRustVersion" }
    "system" { Write-Output "  Rust toolchain: system $SystemRustVersion" }
    default {
        Write-Output "  Rust toolchain: install $RustChannel with pinned rustup-init $($Asset.version)"
    }
}
}
Write-Output "  ccvl binary: $BinaryState"
if ($BinaryState -eq "install" -and $FetchEnabled) {
    Write-Output "  prebuilt binary: $ReleaseAsset (matching runtime required)"
}
Write-Output "  missing bootstrap commands: none"
Write-Output "  host packages: none"
if ($BinaryState -eq "ready") {
    Write-Output "No ccvl build changes required."
}

if ($Mode -eq "plan") {
    Write-Output "No changes made. Run .\ccvl.cmd setup to execute this plan."
    return
}

$TemporaryRoot = Join-Path ([IO.Path]::GetTempPath()) "ccvl-bootstrap-$([Guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $TemporaryRoot | Out-Null
try {
    $FetchedBinary = $false
    if ($BinaryState -eq "install" -and $FetchEnabled) {
        try {
            $Staged = Join-Path $TemporaryRoot $ReleaseAsset
            Invoke-WebRequest -UseBasicParsing -Uri "$ReleaseBase/$ReleaseAsset" -OutFile $Staged
            Invoke-WebRequest -UseBasicParsing -Uri "$ReleaseBase/$ReleaseAsset.sha256" -OutFile "$Staged.sha256"
            $ExpectedHash = ((Get-Content -Raw -LiteralPath "$Staged.sha256").Trim() -split "\s+")[0]
            if ([string]::IsNullOrEmpty($ExpectedHash)) {
                throw "Empty checksum sidecar for $ReleaseAsset"
            }
            $ActualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $Staged).Hash.ToLowerInvariant()
            if ($ActualHash -ne $ExpectedHash.ToLowerInvariant()) {
                throw "Checksum mismatch for $ReleaseAsset"
            }
            Unblock-File -LiteralPath $Staged
            $RuntimeId = Invoke-OutsideRepository $Staged @("runtime-id")
            if ($RuntimeId -ne $Fingerprint) { throw "Downloaded binary does not match this workspace runtime" }
            New-Item -ItemType Directory -Force -Path $LocalBin | Out-Null
            Copy-Item -LiteralPath $Staged -Destination $Binary -Force
            $FetchedBinary = $true
        }
        catch {
            throw "Matching precompiled binary unavailable. Download a complete release, or use setup --from-source for development. $($_.Exception.Message)"
        }
    }

    if ($BinaryState -eq "install" -and $ToolchainState -eq "install" -and -not $FetchedBinary) {
        New-Item -ItemType Directory -Force -Path $CargoHome, $RustupHome | Out-Null
        $Download = Join-Path $TemporaryRoot $Asset.asset
        Write-Output "Downloading pinned rustup-init $($Asset.version)"
        Invoke-WebRequest -UseBasicParsing -Uri $Asset.url -OutFile $Download
        $ActualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $Download).Hash.ToLowerInvariant()
        if ($ActualHash -ne $Asset.sha256) {
            throw "Checksum mismatch for $($Asset.asset)"
        }
        Unblock-File -LiteralPath $Download
        $OldCargoHome = $env:CARGO_HOME
        $OldRustupHome = $env:RUSTUP_HOME
        try {
            $env:CARGO_HOME = $CargoHome
            $env:RUSTUP_HOME = $RustupHome
            & $Download -y --no-modify-path --profile minimal --default-toolchain $RustChannel
            if ($LASTEXITCODE -ne 0) {
                throw "rustup-init failed with exit code $LASTEXITCODE"
            }
        }
        finally {
            $env:CARGO_HOME = $OldCargoHome
            $env:RUSTUP_HOME = $OldRustupHome
        }
        if (-not (Test-ManagedRust)) {
            throw "Managed Rust $RustChannel is unavailable after installation"
        }
        $ToolchainState = "managed"
    }

    if ($BinaryState -eq "install" -and -not $FetchedBinary) {
        New-Item -ItemType Directory -Force -Path $CacheRoot, $CargoHome, $TargetDir | Out-Null
        $CargoArguments = @(
            "install", "--locked", "--force", "--path", $RepoRoot, "--root", $CacheRoot
        )
        $OldCargoHome = $env:CARGO_HOME
        $OldRustupHome = $env:RUSTUP_HOME
        $OldTargetDir = $env:CARGO_TARGET_DIR
        try {
            $env:CARGO_HOME = $CargoHome
            $env:CARGO_TARGET_DIR = $TargetDir
            Push-Location $TemporaryRoot
            try {
                switch ($ToolchainState) {
                    "managed" {
                        $env:RUSTUP_HOME = $RustupHome
                        $Rustup = Join-Path $CargoHome "bin\rustup.exe"
                        & $Rustup run $ManagedToolchain cargo @CargoArguments
                    }
                    "system" {
                        if ($SystemKind -eq "rustup") {
                            & $SystemRustup run $SystemToolchain cargo @CargoArguments
                        }
                        else {
                            & $SystemCargo @CargoArguments
                        }
                    }
                }
                if ($LASTEXITCODE -ne 0) {
                    throw "cargo install failed with exit code $LASTEXITCODE"
                }
            }
            finally {
                Pop-Location
            }
        }
        finally {
            $env:CARGO_HOME = $OldCargoHome
            $env:RUSTUP_HOME = $OldRustupHome
            $env:CARGO_TARGET_DIR = $OldTargetDir
        }
        if (-not (Test-Path -LiteralPath $Binary -PathType Leaf)) {
            throw "cargo install did not produce $Binary"
        }
        $Fingerprint = Get-SourceFingerprint
        if ((Invoke-OutsideRepository $Binary @("runtime-id")) -ne $Fingerprint) {
            throw "Built runtime identity mismatch"
        }
    }

    Push-Location $RepoRoot
    try {
        & $Binary setup
        if ($LASTEXITCODE -ne 0) {
            throw "ccvl setup verification failed with exit code $LASTEXITCODE"
        }
    }
    finally {
        Pop-Location
    }
}
finally {
    if (Test-Path -LiteralPath $TemporaryRoot) {
        [IO.Directory]::Delete($TemporaryRoot, $true)
    }
}
Write-Output "Setup complete. The repository-local binary is $Binary."
