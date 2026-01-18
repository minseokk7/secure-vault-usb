$ErrorActionPreference = "Stop"

Write-Host "Starting SecureVault Build Process..." -ForegroundColor Cyan

# 1. Build Tauri App
Write-Host "Building Tauri Application..." -ForegroundColor Yellow
# npm run tauri:build # Uncomment if you want to rebuild every time. For now, we assume it's built or we skip if 'fast' is passed
if (-not (Test-Path "src-tauri\target\release\secure-vault-tauri.exe")) {
    npm run tauri:build
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Tauri build failed!"
        exit 1
    }
}
else {
    Write-Host "Tauri executable already exists. Skipping build to save time. (Delete 'src-tauri\target\release\secure-vault-tauri.exe' to force rebuild)" -ForegroundColor Gray
}

# 2. Prepare for Installer Build
$sourceExe = "src-tauri\target\release\secure-vault-tauri.exe"
$destDir = "src-tauri\installer\release"
$destExe = "$destDir\SecureVault.exe"

if (-not (Test-Path $sourceExe)) {
    Write-Error "Source executable not found at: $sourceExe"
    exit 1
}

if (-not (Test-Path $destDir)) {
    New-Item -ItemType Directory -Force -Path $destDir | Out-Null
}

Copy-Item -Path $sourceExe -Destination $destExe -Force
Write-Host "Copied executable to installer directory." -ForegroundColor Green

# 3. Build NSIS Installer
Write-Host "Building NSIS Installer..." -ForegroundColor Yellow
$nsisScript = "src-tauri\installer\SecureVault_USB_Installer.nsi"
$localAppData = [Environment]::GetFolderPath("LocalApplicationData")
$tauriMakensis = "$localAppData\tauri\NSIS\makensis.exe"

$makensisPath = ""

if (Test-Path $tauriMakensis) {
    $makensisPath = $tauriMakensis
}
elseif (Get-Command makensis -ErrorAction SilentlyContinue) {
    $makensisPath = "makensis"
}
else {
    # Try common paths
    $possiblePaths = @(
        "C:\Program Files (x86)\NSIS\makensis.exe",
        "C:\Program Files\NSIS\makensis.exe"
    )
    
    foreach ($path in $possiblePaths) {
        if (Test-Path $path) {
            $makensisPath = $path
            break
        }
    }
}

if ($makensisPath -ne "") {
    Write-Host "Using makensis at: $makensisPath" -ForegroundColor Gray
    & $makensisPath $nsisScript
}
else {
    Write-Error "makensis.exe not found. Please ensure NSIS is installed or Tauri has downloaded it."
    exit 1
}

Write-Host "Build Complete!" -ForegroundColor Cyan
