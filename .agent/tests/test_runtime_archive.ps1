# Exercise real Git archive transport, without a native Windows executable.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'runtime-test-source.ps1')
$Scratch = Join-Path ([IO.Path]::GetTempPath()) "ccvl-archive-guard-$([Guid]::NewGuid().ToString('N'))"
$Saved = @{}
foreach ($Name in @('SOURCE_ARCHIVE', 'SOURCE_SHA256', 'CI_COMMIT_SHA')) {
    $Saved[$Name] = [Environment]::GetEnvironmentVariable($Name)
}
New-Item -ItemType Directory -Path $Scratch | Out-Null
try {
    $Repo = Join-Path $Scratch 'source repo'
    $Empty = Join-Path $Scratch 'without git metadata'
    $Hooks = Join-Path $Scratch 'empty hooks'
    New-Item -ItemType Directory -Path $Repo, $Empty, $Hooks | Out-Null
    git -c init.templateDir= init -q $Repo
    if ($LASTEXITCODE -ne 0) { throw 'Fixture Git initialization failed' }
    $Payload = Join-Path $Repo 'payload with spaces.bin'
    [IO.File]::WriteAllBytes($Payload, [byte[]](0..255))
    git -C $Repo add -- 'payload with spaces.bin'
    if ($LASTEXITCODE -ne 0) { throw 'Fixture Git staging failed' }
    git -C $Repo -c "core.hooksPath=$Hooks" -c commit.gpgSign=false -c user.name=Fixture -c user.email=fixture@example.invalid commit -qm fixture
    if ($LASTEXITCODE -ne 0) { throw 'Fixture Git commit failed' }
    $Commit = (& git -C $Repo rev-parse HEAD | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Fixture Git identity failed' }
    $Source = Join-Path $Scratch 'verified source.tar'
    git -C $Repo archive $Commit -o $Source
    if ($LASTEXITCODE -ne 0) { throw 'Fixture Git archive failed' }
    $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $Source).Hash.ToLowerInvariant()
    $Output = Join-Path $Scratch 'runtime source.tar'
    $env:SOURCE_ARCHIVE = $Source
    $env:SOURCE_SHA256 = $Hash
    $env:CI_COMMIT_SHA = $Commit
    Copy-RuntimeTestArchive -RepoRoot $Empty -Archive $Output
    if ((Get-FileHash -Algorithm SHA256 -LiteralPath $Output).Hash.ToLowerInvariant() -cne $Hash) {
        throw 'Verified transport changed archive bytes'
    }
    tar -xf $Output -C $Empty
    if ($LASTEXITCODE -ne 0) { throw 'Verified archive extraction failed' }
    if ((Get-FileHash -LiteralPath (Join-Path $Empty 'payload with spaces.bin')).Hash -cne
        (Get-FileHash -LiteralPath $Payload).Hash) { throw 'Binary payload changed during transport' }
    (Get-Item -LiteralPath $Source).IsReadOnly = $true
    Copy-RuntimeTestArchive -RepoRoot $Empty -Archive $Output
    if (-not (Get-Item -LiteralPath $Source).IsReadOnly -or
        (Get-FileHash -LiteralPath $Source).Hash.ToLowerInvariant() -cne $Hash -or
        (Get-Item -LiteralPath $Output).IsReadOnly) { throw 'Read-only transport was not preserved safely' }
    (Get-Item -LiteralPath $Source).IsReadOnly = $false

    $Changed = Join-Path $Scratch 'changed archive.tar'
    Copy-Item -LiteralPath $Source -Destination $Changed
    $Stream = [IO.File]::OpenWrite($Changed)
    try { $Stream.WriteByte(0) } finally { $Stream.Dispose() }
    $Plain = Join-Path $Scratch 'archive without commit.tar'
    tar -cf $Plain -C $Repo 'payload with spaces.bin'
    if ($LASTEXITCODE -ne 0) { throw 'Plain archive fixture failed' }
    $PlainHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $Plain).Hash.ToLowerInvariant()
    $Cases = @(
        @{ Source = $Source; Hash = ''; Commit = $Commit; Error = 'complete verified' },
        @{ Source = ''; Hash = $Hash; Commit = $Commit; Error = 'complete verified' },
        @{ Source = $Source; Hash = $Hash; Commit = ''; Error = 'complete verified' },
        @{ Source = $Source; Hash = 'invalid'; Commit = $Commit; Error = 'complete verified' },
        @{ Source = $Changed; Hash = $Hash; Commit = $Commit; Error = 'does not match' },
        @{ Source = $Source; Hash = $Hash; Commit = ('0' * 40); Error = 'matching Git commit' },
        @{ Source = $Plain; Hash = $PlainHash; Commit = $Commit; Error = 'matching Git commit' }
    )
    foreach ($Case in $Cases) {
        $env:SOURCE_ARCHIVE = $Case.Source
        $env:SOURCE_SHA256 = $Case.Hash
        $env:CI_COMMIT_SHA = $Case.Commit
        $Rejected = $false
        try { Copy-RuntimeTestArchive -RepoRoot $Empty -Archive $Output }
        catch { $Rejected = $_.Exception.Message.Contains($Case.Error) }
        if (-not $Rejected) { throw "Invalid archive identity was not rejected: $($Case.Error)" }
    }
    $env:SOURCE_ARCHIVE = ''
    $env:SOURCE_SHA256 = ''
    $env:CI_COMMIT_SHA = $Commit
    Copy-RuntimeTestArchive -RepoRoot $Repo -Archive $Output
    if ((Get-FileHash -Algorithm SHA256 -LiteralPath $Output).Hash.ToLowerInvariant() -cne $Hash) {
        throw 'Legacy checkout archive changed'
    }
}
finally {
    foreach ($Name in $Saved.Keys) { [Environment]::SetEnvironmentVariable($Name, $Saved[$Name]) }
    Remove-Item -LiteralPath $Scratch -Recurse -Force
}
Write-Output 'Runtime archive guards passed: verified/read-only transport, checkout fallback and seven invalid identities.'
