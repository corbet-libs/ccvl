# Source transport for the Windows runtime test; never requires a Git checkout
# when the caller supplies the complete verified archive identity.
function Copy-RuntimeTestArchive {
    param([string]$RepoRoot, [string]$Archive)
    if (-not $env:SOURCE_ARCHIVE -and -not $env:SOURCE_SHA256) {
        git -C $RepoRoot archive HEAD -o $Archive
        if ($LASTEXITCODE -ne 0) { throw "Archive creation failed" }
        return
    }
    if (-not $env:SOURCE_ARCHIVE -or $env:SOURCE_SHA256 -cnotmatch '^[0-9a-f]{64}$' -or
        $env:CI_COMMIT_SHA -cnotmatch '^[0-9a-f]{40}$') {
        throw "Runtime tests require the complete verified source archive identity."
    }
    # Verify and extract the same owned snapshot, even if the transport changes.
    Copy-Item -LiteralPath $env:SOURCE_ARCHIVE -Destination $Archive
    (Get-Item -LiteralPath $Archive).IsReadOnly = $false
    $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $Archive).Hash.ToLowerInvariant()
    if ($Hash -cne $env:SOURCE_SHA256) {
        throw "Runtime test source archive does not match the submitted source."
    }
    # git-get-tar-commit-id reads exactly the first 1024 bytes. Do not send tar
    # through PowerShell's text pipeline, which changes bytes on Windows 5.1.
    $Header = New-Object byte[] 1024
    $InputFile = [IO.File]::OpenRead($Archive)
    try {
        $Offset = 0
        while ($Offset -lt $Header.Length) {
            $Count = $InputFile.Read($Header, $Offset, $Header.Length - $Offset)
            if ($Count -eq 0) { throw "Runtime test archive has no matching Git commit header." }
            $Offset += $Count
        }
    }
    finally { $InputFile.Dispose() }
    $Process = New-Object Diagnostics.Process
    $Process.StartInfo.FileName = (Get-Command git -CommandType Application -ErrorAction Stop).Source
    $Process.StartInfo.Arguments = 'get-tar-commit-id'
    $Process.StartInfo.UseShellExecute = $false
    $Process.StartInfo.CreateNoWindow = $true
    $Process.StartInfo.RedirectStandardInput = $true
    $Process.StartInfo.RedirectStandardOutput = $true
    $Process.StartInfo.RedirectStandardError = $true
    try {
        if (-not $Process.Start()) { throw "Could not inspect the runtime test archive commit." }
        $Process.StandardInput.BaseStream.Write($Header, 0, $Header.Length)
        $Process.StandardInput.Close()
        $ArchiveCommit = $Process.StandardOutput.ReadToEnd().Trim()
        $null = $Process.StandardError.ReadToEnd()
        $Process.WaitForExit()
        if ($Process.ExitCode -ne 0 -or $ArchiveCommit -cne $env:CI_COMMIT_SHA) {
            throw "Runtime test archive has no matching Git commit header."
        }
    }
    finally { $Process.Dispose() }
}
