@echo off
echo Building RuPing Installer EXE
echo =============================
echo.

REM Check if Python is available
python --version >nul 2>&1
if %errorLevel% neq 0 (
    echo ERROR: Python not found.
    echo Please install Python from https://python.org
    pause
    exit /b 1
)

echo Installing requirements...
python -m pip install -r installer\requirements.txt

if %errorLevel% neq 0 (
    echo Warning: Failed to install some requirements
    echo The installer may still work with basic functionality
    echo.
)

echo Building EXE files...
cd installer
python build_exe.py

echo.
echo Build completed! Check installer\dist\ for the EXE files.
pause
