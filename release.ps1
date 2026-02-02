param (
    [Parameter(Mandatory = $false)]
    [string]$Version,

    [Parameter(Mandatory = $false)]
    [string]$Notes
)

$ErrorActionPreference = "Stop"

# 1. Get Current Version from package.json
if (-not (Test-Path "package.json")) {
    Write-Error "package.json not found!"
    exit 1
}

$pkgJson = Get-Content "package.json" -Raw | ConvertFrom-Json
$currentVersion = $pkgJson.version
Write-Host "Current Version: $currentVersion"

# 2. Determine New Version
if ([string]::IsNullOrWhiteSpace($Version)) {
    $vParts = $currentVersion.Split('.')
    if ($vParts.Count -eq 3) {
        $newPatch = [int]$vParts[2] + 1
        if ($newPatch -ge 10) {
            $vParts[1] = [int]$vParts[1] + 1
            $vParts[2] = 0
        }
        else {
            $vParts[2] = $newPatch
        }
        $Version = "$($vParts[0]).$($vParts[1]).$($vParts[2])"
    }
    else {
        Write-Error "Cannot auto-increment version format: $currentVersion. Please specify -Version manually."
        exit 1
    }
}
Write-Host "Target Version: $Version"

# 3. Update Files
function Update-FileContent {
    param ($Path, $Regex, $Replacement)
    $content = Get-Content $Path -Raw -Encoding UTF8
    $newContent = $content -replace $Regex, $Replacement
    $utf8NoBom = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($Path, $newContent, $utf8NoBom)
}

Write-Host "Updating configuration files..."
Update-FileContent "package.json" "`"version`": `"$currentVersion`"" "`"version`": `"$Version`""
Update-FileContent "src-tauri/tauri.conf.json" "`"version`": `"$currentVersion`"" "`"version`": `"$Version`""
Update-FileContent "src-tauri/Cargo.toml" "version = `"$currentVersion`"" "version = `"$Version`""
Update-FileContent "src-tauri/installer/SecureVault_USB_Installer.nsi" "!define PRODUCT_VERSION `"$currentVersion`"" "!define PRODUCT_VERSION `"$Version`""

Write-Host "Files updated to $Version"

# 3.5. Cleanup temporary/error log files before build
Write-Host "Cleaning up temporary files..." -ForegroundColor Gray
$cleanupPatterns = @(
    # 로그 파일
    "src-tauri/*.log",
    "*.log",
    
    # 임시 체크/에러 파일
    "check_*.txt",
    "check_log*.txt",
    "errors*.txt",
    
    # 디버그 스크립트/파일
    "extract_errors*.py",
    "filter_warnings.py",
    "fix_bom.ps1",
    "debug_page.html",
    
    # 백업 파일
    "*.backup",
    "src/*.backup",
    "src/**/*.backup",
    
    # 임시 릴리즈 노트
    "release_notes_v*.md",
    
    # Rust 더미 파일
    "src-tauri/check_key_script_dummy.rs"
)
foreach ($pattern in $cleanupPatterns) {
    $files = Get-ChildItem -Path $pattern -ErrorAction SilentlyContinue
    foreach ($file in $files) {
        Remove-Item $file.FullName -Force -ErrorAction SilentlyContinue
        Write-Host "  Removed: $($file.Name)" -ForegroundColor DarkGray
    }
}

# 4. Build
Write-Host "Running build process..."
./build.ps1

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed."
    exit 1
}

# 5. GitHub Release
$installerPath = "src-tauri\installer\SecureVault_USB_Installer.exe"

if (-not (Test-Path $installerPath)) {
    Write-Error "Installer not found at $installerPath"
    exit 1
}

if ([string]::IsNullOrWhiteSpace($Notes)) {
    Write-Host "Generating release notes from git log..." -ForegroundColor Gray
    
    $lastTag = git describe --tags --abbrev=0 2>$null
    if ($lastTag) {
        $commitList = git log "$lastTag..HEAD" --pretty=format:"- %s" --no-merges 2>$null
        if ($commitList) {
            $Notes = "## v$Version Update`r`n`r`nChanges:`r`n`r`n$commitList"
        }
        else {
            $Notes = "## v$Version Update`r`n`r`nSecureVault v$Version Release"
        }
    }
    else {
        $commitList = git log -10 --pretty=format:"- %s" --no-merges 2>$null
        if ($commitList) {
            $Notes = "## v$Version Update`r`n`r`nRecent changes:`r`n`r`n$commitList"
        }
        else {
            $Notes = "## v$Version Update`r`n`r`nSecureVault v$Version Release"
        }
    }
    
    Write-Host "Generated release notes:" -ForegroundColor Green
    Write-Host $Notes -ForegroundColor Cyan
}

Write-Host "Creating GitHub Release v$Version..."
if (Get-Command gh -ErrorAction SilentlyContinue) {
    gh release create "v$Version" "$installerPath" --title "SecureVault v$Version" --notes "$Notes"
    Write-Host "Release v$Version completed successfully!"
}
else {
    Write-Warning "GitHub CLI (gh) not found. Skipping release creation."
    Write-Host "Installer is ready at: $installerPath"
}
