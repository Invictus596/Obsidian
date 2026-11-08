<#
.SYNOPSIS
    Downloads and extracts standard Windows 64-bit CEF binaries from the
    Spotify CEF binary tracker into a `cef_binary/` folder at the repo root.

.DESCRIPTION
    Obsidian requires pre-compiled Chromium Embedded Framework (CEF) binaries
    to build Phase 1. This script automates fetching the latest stable
    Windows x64 CEF distribution published by Spotify's CEF mirror and
    extracting it into ./cef_binary.

    Source: https://cef-builds.spotifycdn.com/cef_binary_*.zip
    (Spotify CEF binary tracker mirrors Chromium's official CEF builds.)

.NOTES
    Requires PowerShell 5.1+ and internet access.
#>

[CmdletBinding()]
param(
    # Destination folder for the extracted CEF binaries.
    [string]$OutputDir = "cef_binary",

    # Spotify CEF tracker index page listing available builds.
    [string]$TrackerUrl = "https://cef-builds.spotifycdn.com/index.html",

    # Optional override for the direct zip URL.
    [string]$ZipUrl = $null
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Write-Host "=== Obsidian CEF Downloader ===" -ForegroundColor Cyan

# ---------------------------------------------------------------------------
# Resolve the latest Windows x64 CEF zip URL.
# ---------------------------------------------------------------------------
if (-not $ZipUrl) {
    Write-Host "Querying Spotify CEF tracker for latest Windows x64 build..." -ForegroundColor Gray

    try {
        $html = Invoke-RestMethod -Uri $TrackerUrl -TimeoutSec 30
    } catch {
        Write-Error "Failed to fetch CEF tracker page: $_"
        exit 1
    }

    # The tracker page lists links like cef_binary_113.17+g0f3a4e2+chromium-113.0.5672.63_windows64_client.zip
    # We look for the most recent windows64 client zip.
    $matches = ($html | Out-String) -match 'href="([^"]*windows64[^"]*\.zip)"'
    if (-not $matches) {
        Write-Error "Could not locate a Windows x64 CEF zip on the tracker page."
        exit 1
    }

    # Take the first match (tracker lists newest first).
    $zipName = $matches[1]
    $ZipUrl = "https://cef-builds.spotifycdn.com/$zipName"

    Write-Host "Found CEF build: $zipName" -ForegroundColor Green
} else {
    Write-Host "Using provided CEF zip URL: $ZipUrl" -ForegroundColor Gray
}

# ---------------------------------------------------------------------------
# Download the archive.
# ---------------------------------------------------------------------------
$zipPath = Join-Path $env:TEMP "obsidian_cef_binary.zip"

if (-not (Test-Path $zipPath)) {
    Write-Host "Downloading CEF binaries from $ZipUrl ..." -ForegroundColor Cyan
    try {
        Invoke-WebRequest -Uri $ZipUrl -OutFile $zipPath -TimeoutSec 120
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

# Use Expand-Archive (built into PowerShell 5.0+).
try {
    Expand-Archive -Path $zipPath -DestinationPath $outputPath.Path -Force
} catch {
    Write-Error "Failed to extract CEF archive: $_"
    exit 1
}

# Spotify CEF zips often contain a top-level folder (e.g. cef_binary_.../).
# If only one folder was extracted, promote its contents up one level so that
# cef_binary/Release and cef_binary/Include sit directly under $OutputDir.
$childItems = Get-ChildItem -Path $outputPath.Path
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
Write-Host "  2. cargo build --release" -ForegroundColor Gray
