# RuPing Advanced Installer
# Supports custom installation directory, PATH management, and command aliases

param(
    [string]$InstallPath = "",
    [switch]$Uninstall,
    [switch]$Silent,
    [switch]$NoPath
)

# Check administrator privileges
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Error "This installer requires administrator privileges. Please run PowerShell as administrator."
    exit 1
}

# Function to show folder browser dialog
function Show-FolderBrowserDialog {
    param([string]$Description = "Select Installation Directory")
    
    Add-Type -AssemblyName System.Windows.Forms
    $folderBrowser = New-Object System.Windows.Forms.FolderBrowserDialog
    $folderBrowser.Description = $Description
    $folderBrowser.RootFolder = [System.Environment+SpecialFolder]::MyComputer
    $folderBrowser.SelectedPath = "$env:ProgramFiles\RuPing"
    
    if ($folderBrowser.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
        return $folderBrowser.SelectedPath
    }
    return $null
}

# Function to add to PATH
function Add-ToPath {
    param([string]$PathToAdd)
    
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "Machine")
    if ($currentPath -notlike "*$PathToAdd*") {
        $newPath = "$currentPath;$PathToAdd"
        [Environment]::SetEnvironmentVariable("PATH", $newPath, "Machine")
        Write-Host "Added $PathToAdd to system PATH" -ForegroundColor Green
        return $true
    } else {
        Write-Host "$PathToAdd is already in system PATH" -ForegroundColor Yellow
        return $false
    }
}

# Function to remove from PATH
function Remove-FromPath {
    param([string]$PathToRemove)
    
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "Machine")
    $newPath = $currentPath -replace [regex]::Escape(";$PathToRemove"), ""
    $newPath = $newPath -replace [regex]::Escape("$PathToRemove;"), ""
    $newPath = $newPath -replace [regex]::Escape("$PathToRemove"), ""
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "Machine")
    Write-Host "Removed $PathToRemove from system PATH" -ForegroundColor Green
}

# Function to create command aliases
function Create-CommandAliases {
    param([string]$InstallDir)
    
    $aliases = @("rustping", "rping", "ruping")
    
    foreach ($alias in $aliases) {
        $aliasPath = Join-Path $InstallDir "$alias.cmd"
        $content = @"
@echo off
"%~dp0ruping.exe" %*
"@
        Set-Content -Path $aliasPath -Value $content -Encoding ASCII
        Write-Host "Created alias: $alias" -ForegroundColor Green
    }
}

# Function to remove command aliases
function Remove-CommandAliases {
    param([string]$InstallDir)
    
    $aliases = @("rustping", "rping", "ruping")
    
    foreach ($alias in $aliases) {
        $aliasPath = Join-Path $InstallDir "$alias.cmd"
        if (Test-Path $aliasPath) {
            Remove-Item $aliasPath -Force
            Write-Host "Removed alias: $alias" -ForegroundColor Green
        }
    }
}

# Uninstall function
if ($Uninstall) {
    Write-Host "RuPing Uninstaller" -ForegroundColor Red
    Write-Host "==================" -ForegroundColor Red
    Write-Host ""
    
    # Find installation directory from registry or common locations
    $possiblePaths = @(
        "$env:ProgramFiles\RuPing",
        "$env:LOCALAPPDATA\RuPing",
        "C:\RuPing"
    )
    
    $installDir = $null
    foreach ($path in $possiblePaths) {
        if (Test-Path "$path\ruping.exe") {
            $installDir = $path
            break
        }
    }
    
    if (-not $installDir) {
        Write-Host "RuPing installation not found in common locations." -ForegroundColor Yellow
        if (-not $Silent) {
            $customPath = Read-Host "Please enter the installation directory (or press Enter to cancel)"
            if ($customPath -and (Test-Path "$customPath\ruping.exe")) {
                $installDir = $customPath
            }
        }
    }
    
    if (-not $installDir) {
        Write-Host "Cannot locate RuPing installation. Uninstall cancelled." -ForegroundColor Red
        exit 1
    }
    
    Write-Host "Found RuPing installation at: $installDir" -ForegroundColor Yellow
    
    if (-not $Silent) {
        $confirm = Read-Host "Are you sure you want to uninstall RuPing? (y/N)"
        if ($confirm -ne "y" -and $confirm -ne "Y") {
            Write-Host "Uninstall cancelled." -ForegroundColor Yellow
            exit 0
        }
    }
    
    # Remove from PATH
    Remove-FromPath $installDir
    
    # Remove command aliases
    Remove-CommandAliases $installDir
    
    # Remove installation directory
    try {
        Remove-Item -Path $installDir -Recurse -Force
        Write-Host "Successfully removed installation directory: $installDir" -ForegroundColor Green
    } catch {
        Write-Host "Warning: Could not remove installation directory: $($_.Exception.Message)" -ForegroundColor Yellow
    }
    
    # Remove start menu shortcut
    $startMenuPath = "$env:ProgramData\Microsoft\Windows\Start Menu\Programs\RuPing.lnk"
    if (Test-Path $startMenuPath) {
        Remove-Item $startMenuPath -Force
        Write-Host "Removed start menu shortcut" -ForegroundColor Green
    }
    
    Write-Host ""
    Write-Host "RuPing has been successfully uninstalled!" -ForegroundColor Green
    Write-Host "Please restart your command prompt to update PATH changes." -ForegroundColor Yellow
    
    if (-not $Silent) {
        Read-Host "Press Enter to exit"
    }
    exit 0
}

# Installation process
Write-Host "RuPing Advanced Installer" -ForegroundColor Green
Write-Host "=========================" -ForegroundColor Green
Write-Host ""

# Check if ruping.exe exists
$exePath = "target\release\ruping.exe"
if (-not (Test-Path $exePath)) {
    Write-Host "Building RuPing..." -ForegroundColor Yellow
    
    # Check if Rust is installed
    try {
        $rustVersion = cargo --version
        Write-Host "Detected Rust: $rustVersion" -ForegroundColor Green
    } catch {
        Write-Error "Rust not found. Please install Rust from https://rustup.rs/"
        exit 1
    }
    
    # Build the project
    Write-Host "Building RuPing (Release mode)..." -ForegroundColor Yellow
    cargo build --release
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Build failed. Please check the error messages above."
        exit 1
    }
    
    Write-Host "Build completed successfully." -ForegroundColor Green
}

# Determine installation directory
if (-not $InstallPath) {
    if ($Silent) {
        $InstallPath = "$env:ProgramFiles\RuPing"
    } else {
        Write-Host "Choose installation directory:" -ForegroundColor Cyan
        Write-Host "1. Program Files (Recommended): $env:ProgramFiles\RuPing" -ForegroundColor White
        Write-Host "2. Local AppData: $env:LOCALAPPDATA\RuPing" -ForegroundColor White
        Write-Host "3. Custom directory" -ForegroundColor White
        Write-Host ""
        
        do {
            $choice = Read-Host "Enter your choice (1-3)"
            switch ($choice) {
                "1" { $InstallPath = "$env:ProgramFiles\RuPing"; break }
                "2" { $InstallPath = "$env:LOCALAPPDATA\RuPing"; break }
                "3" { 
                    $InstallPath = Show-FolderBrowserDialog
                    if (-not $InstallPath) {
                        Write-Host "Installation cancelled." -ForegroundColor Yellow
                        exit 0
                    }
                    $InstallPath = Join-Path $InstallPath "RuPing"
                    break 
                }
                default { Write-Host "Invalid choice. Please enter 1, 2, or 3." -ForegroundColor Red }
            }
        } while (-not $InstallPath)
    }
}

Write-Host "Installing RuPing to: $InstallPath" -ForegroundColor Green

# Create installation directory
if (-not (Test-Path $InstallPath)) {
    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
    Write-Host "Created installation directory" -ForegroundColor Green
}

# Copy executable
Copy-Item -Path $exePath -Destination "$InstallPath\ruping.exe" -Force
Write-Host "Copied ruping.exe to installation directory" -ForegroundColor Green

# Create command aliases
Create-CommandAliases $InstallPath

# Add to PATH
if (-not $NoPath) {
    $pathAdded = Add-ToPath $InstallPath
}

# Create start menu shortcut
$startMenuPath = "$env:ProgramData\Microsoft\Windows\Start Menu\Programs"
$shortcutPath = "$startMenuPath\RuPing.lnk"

$WshShell = New-Object -comObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($shortcutPath)
$Shortcut.TargetPath = "cmd.exe"
$Shortcut.Arguments = "/k echo RuPing - Rust Ping Implementation && echo Usage: ruping [options] target && echo Example: ruping 8.8.8.8 && echo Help: ruping --help"
$Shortcut.WorkingDirectory = "$env:USERPROFILE"
$Shortcut.Description = "RuPing - Rust Ping Implementation"
$Shortcut.Save()

Write-Host "Created start menu shortcut" -ForegroundColor Green

Write-Host ""
Write-Host "Installation completed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Available commands:" -ForegroundColor Cyan
Write-Host "  ruping 8.8.8.8              # Basic ping" -ForegroundColor White
Write-Host "  rustping google.com -n 5    # Using rustping alias" -ForegroundColor White
Write-Host "  rping 192.168.1.1 -t        # Using rping alias" -ForegroundColor White
Write-Host "  ruping --help                # Show help" -ForegroundColor White
Write-Host ""
Write-Host "Installation details:" -ForegroundColor Yellow
Write-Host "  Location: $InstallPath" -ForegroundColor White
Write-Host "  PATH updated: $(if(-not $NoPath){'Yes'}else{'No'})" -ForegroundColor White
Write-Host "  Aliases created: rustping, rping, ruping" -ForegroundColor White
Write-Host ""
Write-Host "Important notes:" -ForegroundColor Yellow
Write-Host "- RuPing requires administrator privileges to run" -ForegroundColor White
Write-Host "- Please restart your command prompt to use the new commands" -ForegroundColor White
Write-Host "- Use 'ruping --help' to see all available options" -ForegroundColor White
Write-Host ""
Write-Host "To uninstall:" -ForegroundColor Cyan
Write-Host "  .\installer.ps1 -Uninstall" -ForegroundColor White

if (-not $Silent) {
    Read-Host "Press Enter to exit"
}
