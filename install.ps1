#Requires -Version 5.1
<#
.SYNOPSIS
    Install prebuilt onrecord.exe for Windows (x86_64-pc-windows-msvc) with SHA-256 verification.
.DESCRIPTION
    Resolves the requested or latest GitHub Release tag once, downloads both the Windows
    .zip archive and SHA256SUMS, verifies the archive SHA-256 digest before extraction,
    and atomically stages onrecord.exe into the target bin directory.
.PARAMETER Version
    Release tag such as v0.1.0 (default: latest GitHub release, or $ONRECORD_VERSION).
.PARAMETER BinDir
    Destination directory for onrecord.exe (default: $HOME\.local\bin, or $ONRECORD_INSTALL_DIR).
#>
[CmdletBinding()]
param(
    [string]$Version = "",
    [string]$BinDir = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoSlug = if ($env:ONRECORD_REPO) { $env:ONRECORD_REPO } else { "david-engelmann/onrecord" }
if (-not $Version) {
    $Version = if ($env:ONRECORD_VERSION) { $env:ONRECORD_VERSION } else { "latest" }
}
if (-not $BinDir) {
    $BinDir = if ($env:ONRECORD_INSTALL_DIR) { $env:ONRECORD_INSTALL_DIR } else { Join-Path $HOME ".local\bin" }
}

$Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
if ($Arch -ne "X64") {
    throw "onrecord-install: prebuilt Windows archives currently target X64 (detected $Arch). Install from source with: cargo install --git https://github.com/$RepoSlug.git --locked"
}

$Target = "x86_64-pc-windows-msvc"
$Archive = "onrecord-$Target.zip"

if ($env:ONRECORD_BASE_URL) {
    $BaseUrl = $env:ONRECORD_BASE_URL.TrimEnd("/")
} else {
    if ($Version -eq "latest") {
        $ApiUrl = "https://api.github.com/repos/$RepoSlug/releases/latest"
        $Release = Invoke-RestMethod -Uri $ApiUrl -Headers @{ "User-Agent" = "onrecord-install-ps1" }
        if (-not $Release.tag_name) {
            throw "onrecord-install: could not resolve latest tag from $ApiUrl"
        }
        $Tag = [string]$Release.tag_name
    } elseif ($Version.StartsWith("v")) {
        $Tag = $Version
    } else {
        $Tag = "v$Version"
    }
    $BaseUrl = "https://github.com/$RepoSlug/releases/download/$Tag"
}

$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("onrecord-install-" + [System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

function Copy-OrDownload([string]$SourceUri, [string]$Destination) {
    if ($SourceUri.StartsWith("file://")) {
        $LocalPath = ([System.Uri]$SourceUri).LocalPath
        Copy-Item -LiteralPath $LocalPath -Destination $Destination -Force
    } elseif (Test-Path -LiteralPath $SourceUri) {
        Copy-Item -LiteralPath $SourceUri -Destination $Destination -Force
    } else {
        Invoke-WebRequest -Uri $SourceUri -OutFile $Destination -UseBasicParsing
    }
}

try {
    $ArchivePath = Join-Path $TmpDir $Archive
    $SumsPath = Join-Path $TmpDir "SHA256SUMS"

    Write-Host "Downloading $BaseUrl/$Archive"
    Copy-OrDownload "$BaseUrl/$Archive" $ArchivePath
    Copy-OrDownload "$BaseUrl/SHA256SUMS" $SumsPath

    $ExpectedSha = $null
    foreach ($Line in (Get-Content -LiteralPath $SumsPath)) {
        $Trimmed = $Line.Trim()
        if (-not $Trimmed) { continue }
        $Parts = $Trimmed -split "\s+", 2
        if ($Parts.Count -ge 2) {
            $Name = $Parts[1].TrimStart("*").replace("./", "")
            if ($Name -eq $Archive) {
                $ExpectedSha = $Parts[0].ToLowerInvariant()
                break
            }
        }
    }
    if (-not $ExpectedSha) {
        throw "onrecord-install: archive $Archive not listed in SHA256SUMS"
    }

    $ActualSha = (Get-FileHash -LiteralPath $ArchivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($ActualSha -ne $ExpectedSha) {
        throw "onrecord-install: SHA-256 mismatch for $Archive (expected $ExpectedSha, got $ActualSha)"
    }

    $UnpackDir = Join-Path $TmpDir "unpack"
    Expand-Archive -LiteralPath $ArchivePath -DestinationPath $UnpackDir -Force

    $CandidateRoot = Join-Path $UnpackDir "onrecord.exe"
    $CandidateNested = Join-Path (Join-Path $UnpackDir "onrecord-$Target") "onrecord.exe"
    if (Test-Path -LiteralPath $CandidateRoot) {
        $BinSrc = $CandidateRoot
    } elseif (Test-Path -LiteralPath $CandidateNested) {
        $BinSrc = $CandidateNested
    } else {
        throw "onrecord-install: onrecord.exe not found inside $Archive"
    }

    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
    $StagedBin = Join-Path $BinDir ".onrecord.tmp.$PID.exe"
    $FinalBin = Join-Path $BinDir "onrecord.exe"
    Copy-Item -LiteralPath $BinSrc -Destination $StagedBin -Force
    Move-Item -LiteralPath $StagedBin -Destination $FinalBin -Force

    Write-Host "Verified SHA-256 ($ActualSha) and installed onrecord.exe to $FinalBin"
    $PathEntries = ($env:PATH -split ";") | ForEach-Object { $_.TrimEnd("\") }
    if ($PathEntries -notcontains $BinDir.TrimEnd("\")) {
        Write-Host "Note: add '$BinDir' to your user PATH to invoke 'onrecord' directly."
    }
} finally {
    if (Test-Path -LiteralPath $TmpDir) {
        Remove-Item -LiteralPath $TmpDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
