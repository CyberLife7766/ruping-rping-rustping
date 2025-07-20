@echo off
echo RuPing Installer
echo ================
echo.

REM Check for administrator privileges
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo ERROR: Administrator privileges required.
    echo Please run this script as administrator.
    pause
    exit /b 1
)

REM Check if Python is available
python --version >nul 2>&1
if %errorLevel% neq 0 (
    echo ERROR: Python not found.
    echo Please install Python from https://python.org
    pause
    exit /b 1
)

echo Starting Python installer...
echo.

REM Run the Python installer
python installer\installer.py %*

if %errorLevel% neq 0 (
    echo.
    echo Installation failed. Please check the error messages above.
    pause
    exit /b 1
)

echo.
pause
