param([Parameter(Mandatory = $true)][string]$Binary)
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'runtime-test-source.ps1')
& (Join-Path $PSScriptRoot 'test_runtime_archive.ps1')
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$Binary = (Resolve-Path $Binary).Path
$TestBinary = $Binary
. (Join-Path $RepoRoot ".agent\scripts\runtime-id.ps1")
$Expected = Get-SourceFingerprint
if ((& $Binary runtime-id | Out-String).Trim() -ne $Expected) {
    throw "PowerShell and native runtime identities disagree"
}
$Scratch = Join-Path ([IO.Path]::GetTempPath()) "ccvl-runtime-$([Guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $Scratch | Out-Null
try {
    $Archive = Join-Path $Scratch "source.tar"
    Copy-RuntimeTestArchive -RepoRoot $RepoRoot -Archive $Archive
    tar -xf $Archive -C $Scratch
    if ($LASTEXITCODE -ne 0) { throw "Archive extraction failed" }
    Remove-Item -LiteralPath $Archive
    # Exercise the actual download installer using the current native binary.
    function Invoke-WebRequest {
        param([switch]$UseBasicParsing, [string]$Uri, [string]$OutFile)
        if ($Uri.EndsWith(".sha256")) {
            $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $TestBinary).Hash.ToLowerInvariant()
            [IO.File]::WriteAllText($OutFile, "$Hash  ccvl.exe`n")
        }
        else { Copy-Item -LiteralPath $TestBinary -Destination $OutFile }
    }
    & (Join-Path $Scratch "ccvl.ps1") setup
    & (Join-Path $Scratch "ccvl.ps1") doctor
    if ($LASTEXITCODE -ne 0) { throw "Matching Windows runtime failed" }
    [IO.File]::AppendAllText((Join-Path $Scratch ".agent\src\main.rs"), "`n// edited runtime`n")
    $Rejected = $false
    try { & (Join-Path $Scratch "ccvl.ps1") doctor }
    catch { $Rejected = $_.Exception.Message.Contains("runtime is stale") }
    if (-not $Rejected) { throw "Windows launcher accepted a stale runtime" }
    $Rejected = $false
    try { & (Join-Path $Scratch ".agent\scripts\bootstrap.ps1") install }
    catch { $Rejected = $_.Exception.Message.Contains("Matching precompiled binary unavailable") }
    if (-not $Rejected) { throw "Windows installer accepted a stale download" }
}
finally {
    [IO.Directory]::Delete($Scratch, $true)
}
Write-Output "Windows installation and stale-runtime rejection passed."
