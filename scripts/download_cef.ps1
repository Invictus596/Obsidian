<#
.SYNOPSIS
    Downloads and extracts standard Windows 64-bit CEF binaries from the
    Spotify CEF binary tracker into a `cef_binary/` folder at the repo root.

.DESCRIPTION
    Obsidian requires pre-compiled Chromium Embedded Framework (CEF) binaries
    to build Phase 1. This script automates fetching the latest stable
    Windows x64 CEF distribution published by Spotify's CEF mirror and
    extracting it into ./cef_binary.

    Source: https://cef-builds.spotifycdn.com/index.json
    (Spotify CEF binary tracker mirrors Chromium's official CEF builds.)

.NOTES
    Requires PowerShell 5.1+ and internet access.
#>

[CmdletBinding()]
param(
    # Destination folder for the extracted CEF binaries.
    [string]$OutputDir = "cef_binary",

    # Spotify CEF build index JSON.
    [string]$IndexUrl = "https://cef-builds.spotifycdn.com/index.json",

    # Optional override for the direct zip URL.
    [string]$ZipUrl = $null
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Write-Host "=== Obsidian CEF Downloader ===" -ForegroundColor Cyan

# ---------------------------------------------------------------------------
# Resolve the latest Windows x64 CEF zip URL from index.json.
# ---------------------------------------------------------------------------
if (-not $ZipUrl) {
    Write-Host "Querying Spotify CEF build index for latest Windows x64 build..." -ForegroundColor Gray

    try {
        $json = Invoke-RestMethod -Uri $IndexUrl -TimeoutSec 60 -ContentType "application/json"
    } catch {
        Write-Error "Failed to fetch CEF build index: $_"
        exit 1
    }

    # The JSON is keyed by platform tab (windows64, macosx64, linux64, etc.).
    # Each entry has a 'versions' array sorted newest first.
    $windows64Builds = $json.windows64
    if (-not $windows64Builds -or -not $windows64Builds.versions) {
        Write-Error "No Windows x64 builds found in CEF index."
        exit 1
    }

    $versions = $windows64Builds.versions
    $zipName = $null

    # Iterate versions newest-first, looking for a non-beta client file.
    # Prefer .zip, fall back to .tar.bz2.
    foreach ($version in $versions) {
        $files = $version.files
        if (-not $files) { continue }

        # Skip beta builds.
        $isBeta = ($files[0].name -like "*_beta*")
        if ($isBeta) { continue }

        # Find the 'client' file type, preferring .zip over .tar.bz2.
        $zipFile = $files | Where-Object { $_.type -eq 'client' -and $_.name -like "*.zip" } | Select-Object -First 1
        $tarFile = $files | Where-Object { $_.type -eq 'client' -and $_.name -like "*.tar.bz2" } | Select-Object -First 1

        if ($zipFile) {
            $zipName = $zipFile.name
            $archiveType = "zip"
        } elseif ($tarFile) {
            $zipName = $tarFile.name
            $archiveType = "tar.bz2"
        } else {
            continue
        }

        Write-Host "Found CEF build: $($version.cef_version) / Chromium $($version.chromium_version)" -ForegroundColor Green
        Write-Host "File: $zipName ($archiveType)" -ForegroundColor Gray
        break
    }

    if (-not $zipName) {
        Write-Error "Could not locate a Windows x64 client CEF zip in the build index."
        exit 1
    }

    $ZipUrl = "https://cef-builds.spotifycdn.com/$zipName"
    $script:ArchiveType = $archiveType
} else {
    Write-Host "Using provided CEF zip URL: $ZipUrl" -ForegroundColor Gray
    $script:ArchiveType = "zip"
}

# ---------------------------------------------------------------------------
# Download the archive.
# ---------------------------------------------------------------------------
$zipPath = Join-Path $env:TEMP "obsidian_cef_binary.zip"

if (-not (Test-Path $zipPath)) {
    Write-Host "Downloading CEF binaries from $ZipUrl ..." -ForegroundColor Cyan
    try {
        Invoke-WebRequest -Uri $ZipUrl -OutFile $zipPath -TimeoutSec 300
    } catch {
        Write-Error "Failed to download CEF binaries: $_"
        exit 1
    }
    Write-Host "Downloaded to $zipPath" -ForegroundColor Green
} else {
    Write-Host "Using cached archive at $zipPath" -ForegroundColor Gray
}

# ---------------------------------------------------------------------------
# Extract into the output directory.
# ---------------------------------------------------------------------------
$outputPath = Resolve-Path -Path $OutputDir -ErrorAction SilentlyContinue
if (-not $outputPath) {
    Write-Host "Creating output directory: $OutputDir" -ForegroundColor Gray
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
    $outputPath = Resolve-Path -Path $OutputDir
}

Write-Host "Extracting CEF binaries into $($outputPath.Path) ..." -ForegroundColor Cyan

Write-Host "Extracting CEF binaries into $($outputPath.Path) ..." -ForegroundColor Cyan

# Determine archive type from file extension.
$isTarBz2 = $zipName -like "*.tar.bz2"

if ($isTarBz2) {
    # Windows 10/11 includes tar.exe for .tar.bz2 extraction.
    try {
        & tar -xjf $zipPath -C $outputPath.Path
    } catch {
        Write-Error "Failed to extract CEF archive (tar.bz2): $_"
        exit 1
    }
} else {
    # Use Expand-Archive (built into PowerShell 5.0+) for .zip.
    try {
        Expand-Archive -Path $zipPath -DestinationPath $outputPath.Path -Force
    } catch {
        Write-Error "Failed to extract CEF archive: $_"
        exit 1
    }
}

# Spotify CEF zips/tars often contain a top-level folder (e.g. cef_binary_.../).
# If only one folder was extracted, promote its contents up one level so that
# cef_binary/Release and cef_binary/Include sit directly under $OutputDir.
$childItems = @(Get-ChildItem -Path $outputPath.Path)
if ($childItems.Count -eq 1 -and $childItems[0].PSIsContainer) {
    $inner = $childItems[0].FullName
    Write-Host "Flattening nested folder: $($childItems[0].Name)" -ForegroundColor Gray
    Get-ChildItem -Path $inner | Move-Item -Destination $outputPath.Path -Force
    Remove-Item -Path $inner -Recurse -Force
}

# ---------------------------------------------------------------------------
# Sanity check: confirm expected layout.
# ---------------------------------------------------------------------------
$releaseDir = Join-Path $outputPath.Path "Release"
if (Test-Path -Path $releaseDir) {
    Write-Host "CEF Release folder found at $releaseDir" -ForegroundColor Green
} else {
    Write-Warning "Expected Release folder not found at $releaseDir. Verify the archive layout."
}

$includeDir = Join-Path $outputPath.Path "Include"
if (Test-Path -Path $includeDir) {
    Write-Host "CEF Include folder found at $includeDir" -ForegroundColor Green
} else {
    Write-Warning "Expected Include folder not found at $includeDir. Verify the archive layout."
}

Write-Host "" -ForegroundColor Gray
Write-Host "=== CEF binaries ready ===" -ForegroundColor Cyan
Write-Host "Next steps:" -ForegroundColor Gray
Write-Host "  1. Set CEF_ROOT=$($outputPath.Path) (optional, defaults to cef_binary/)" -ForegroundColor Gray
Write-Host "  2. cargo build --release --features obsidian-core/cef-bindings" -ForegroundColor Gray
