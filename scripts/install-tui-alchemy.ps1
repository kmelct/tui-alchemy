param(
    [string]$Version = $env:TUI_ALCHEMY_VERSION,
    [string]$AssetBaseUrl = $env:TUI_ALCHEMY_ASSET_BASE_URL,
    [string]$BinaryBaseUrl = $env:TUI_ALCHEMY_BINARY_BASE_URL,
    [string]$BinaryUrl = $env:TUI_ALCHEMY_BINARY_URL,
    [string]$InstallDir = $env:TUI_ALCHEMY_INSTALL_DIR
)

$ErrorActionPreference = "Stop"
$AppName = "tui-alchemy"
if ([string]::IsNullOrWhiteSpace($Version)) { $Version = "0.2.0" }
if ([string]::IsNullOrWhiteSpace($AssetBaseUrl)) { $AssetBaseUrl = "https://pub-ec563771aa2c4e0f942506be4f1593ce.r2.dev" }
if ([string]::IsNullOrWhiteSpace($BinaryBaseUrl)) { $BinaryBaseUrl = "$AssetBaseUrl/downloads" }
$AutoYes = $env:TUI_ALCHEMY_YES


function Test-Command([string]$Name) {
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}
function Prompt-YesNo([string]$Prompt) {
    if ($AutoYes -eq "1" -or $AutoYes -eq "true") {
        Write-Host "$Prompt yes"
        return $true
    }
    if (-not [Environment]::UserInteractive) { return $false }
    $answer = Read-Host "$Prompt [y/N]"
    return $answer -eq "y" -or $answer -eq "Y" -or $answer -eq "yes" -or $answer -eq "YES"
}

function Install-MissingDependency([string]$Label, [string]$WingetId) {
    if (-not (Prompt-YesNo "$AppName needs $Label to install the prebuilt binary. Install it now?")) {
        return $false
    }
    if (-not (Test-Command "winget")) { return $false }
    winget install --id $WingetId --exact --accept-package-agreements --accept-source-agreements
    return $LASTEXITCODE -eq 0
}


function Get-HostTriple {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
    switch ($arch) {
        "x64" { return "x86_64-pc-windows-msvc" }
        "arm64" { return "aarch64-pc-windows-msvc" }
        default { return $null }
    }
}

function Get-InstallDir {
    if (-not [string]::IsNullOrWhiteSpace($InstallDir)) { return $InstallDir }
    return Join-Path $env:LOCALAPPDATA "Programs\tui-alchemy\bin"
}

function Install-FromBinary {
    $triple = Get-HostTriple
    if ([string]::IsNullOrWhiteSpace($triple)) { return $false }
    if (-not (Test-Command "tar")) {
        if (-not (Install-MissingDependency "tar" "GnuWin32.Tar")) { return $false }
        if (-not (Test-Command "tar")) { return $false }
    }

    $archiveName = "$AppName-$Version-$triple.tar.gz"
    $url = $BinaryUrl
    if ([string]::IsNullOrWhiteSpace($url)) { $url = "$BinaryBaseUrl/$archiveName" }

    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $tmpDir | Out-Null
    try {
        $archive = Join-Path $tmpDir $archiveName
        Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $archive
        & tar -xzf $archive -C $tmpDir
        if ($LASTEXITCODE -ne 0) { return $false }

        $binary = Join-Path $tmpDir "$AppName.exe"
        if (-not (Test-Path $binary)) { return $false }

        $destinationDir = Get-InstallDir
        New-Item -ItemType Directory -Path $destinationDir -Force | Out-Null
        $destination = Join-Path $destinationDir "$AppName.exe"
        Copy-Item -Force $binary $destination
        Write-Host "$AppName $Version installed successfully. Run: $destination"
        if (($env:Path -split ';') -notcontains $destinationDir) {
            Write-Host "Add this directory to PATH for shorter launches: $destinationDir" -ForegroundColor Yellow
        }
        return $true
    }
    catch {
        return $false
    }
    finally {
        Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
    }
}

function Install-FromCargoPackage {
    if (-not (Test-Command "cargo")) { return $false }
    cargo install $AppName --version $Version --locked --force
    return $LASTEXITCODE -eq 0
}

if (Install-FromBinary) { exit 0 }
if (Install-FromCargoPackage) {
    Write-Host "$AppName $Version installed from crates.io. Run: $AppName"
    exit 0
}

throw "Could not install a prebuilt $AppName binary for this platform, and Cargo is unavailable for crates.io fallback."
