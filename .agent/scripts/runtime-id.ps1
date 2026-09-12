# Keep the identity algorithm aligned with runtime_source.rs and runtime-id.sh.
function Get-SourceFingerprint {
    if ($env:CCVL_BOOTSTRAP_TESTING -eq "1") {
        if (Test-Path Env:CCVL_BOOTSTRAP_TEST_FINGERPRINT) {
            return $env:CCVL_BOOTSTRAP_TEST_FINGERPRINT
        }
        return "test-fingerprint"
    }
    $Files = @(
        Get-Item -ErrorAction Stop -LiteralPath (Join-Path $RepoRoot "Cargo.toml")
        Get-Item -ErrorAction Stop -LiteralPath (Join-Path $RepoRoot ".agent/core/Cargo.toml")
        Get-Item -ErrorAction Stop -LiteralPath (Join-Path $RepoRoot "Cargo.lock")
        Get-Item -ErrorAction Stop -LiteralPath (Join-Path $RepoRoot "rust-toolchain.toml")
        Get-Item -ErrorAction Stop -LiteralPath (Join-Path $RepoRoot ".agent/build.rs")
        Get-ChildItem -ErrorAction Stop -LiteralPath @((Join-Path $RepoRoot ".agent\src"), (Join-Path $RepoRoot ".agent\core\src")) -Recurse -File -Filter "*.rs" |
            Sort-Object FullName
    )
    $Lines = foreach ($File in $Files) {
        $Relative = $File.FullName.Substring($RepoRoot.Length + 1).Replace("\", "/")
        $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $File.FullName).Hash.ToLowerInvariant()
        "$Relative $Hash"
    }
    $Bytes = [Text.UTF8Encoding]::new($false).GetBytes(($Lines -join "`n") + "`n")
    $Hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return ([BitConverter]::ToString($Hasher.ComputeHash($Bytes))).Replace("-", "").ToLowerInvariant()
    }
    finally {
        $Hasher.Dispose()
    }
}
