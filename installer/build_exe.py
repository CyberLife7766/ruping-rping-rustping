#!/usr/bin/env python3
"""
Build script to create standalone exe files for RuPing installer and uninstaller
Requires PyInstaller: pip install pyinstaller
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path

def check_pyinstaller():
    """Check if PyInstaller is available"""
    try:
        import PyInstaller
        return True
    except ImportError:
        return False

def install_pyinstaller():
    """Install PyInstaller"""
    print("Installing PyInstaller...")
    try:
        subprocess.run([sys.executable, "-m", "pip", "install", "pyinstaller"], check=True)
        return True
    except subprocess.CalledProcessError:
        return False

def prepare_installer_resources():
    """Prepare resources for installer"""
    # Find ruping.exe
    ruping_exe_path = None
    possible_paths = [
        Path("../target/release/ruping.exe"),
        Path("target/release/ruping.exe"),
        Path("ruping.exe")
    ]

    for path in possible_paths:
        if path.exists():
            ruping_exe_path = path.resolve()
            break

    if not ruping_exe_path:
        print("ERROR: ruping.exe not found. Please build it first with 'cargo build --release'")
        return False

    print(f"Found ruping.exe at: {ruping_exe_path}")

    # Copy ruping.exe to installer directory for embedding
    shutil.copy2(ruping_exe_path, "ruping.exe")
    print("Copied ruping.exe to installer directory")

    return True

def build_exe(script_path, output_name, icon_path=None, add_data=None):
    """Build exe using PyInstaller"""
    cmd = [
        sys.executable, "-m", "PyInstaller",
        "--onefile",
        "--console",
        "--name", output_name,
        "--distpath", "../release",  # Output to release directory
        "--workpath", "build",
        "--specpath", "build"
    ]

    if icon_path and Path(icon_path).exists():
        cmd.extend(["--icon", icon_path])

    # Add data files if specified
    if add_data:
        for src, dst in add_data:
            # Use correct PyInstaller syntax: source;destination
            cmd.extend(["--add-data", f"{src};{dst}"])
            print(f"Adding data: {src} -> {dst}")

    cmd.append(str(script_path))

    print(f"Building {output_name}.exe...")
    print(f"Command: {' '.join(cmd)}")

    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print(f"Successfully built {output_name}.exe")
        return True
    except subprocess.CalledProcessError as e:
        print(f"Failed to build {output_name}.exe:")
        print(e.stdout)
        print(e.stderr)
        return False

def main():
    print("RuPing Installer/Uninstaller EXE Builder")
    print("=========================================")
    print()

    # Check PyInstaller
    if not check_pyinstaller():
        print("PyInstaller not found. Installing...")
        if not install_pyinstaller():
            print("ERROR: Failed to install PyInstaller")
            return False

    # Change to installer directory
    installer_dir = Path(__file__).parent
    os.chdir(installer_dir)

    # Prepare resources
    if not prepare_installer_resources():
        return False

    # Clean previous builds
    for dir_name in ["build"]:
        if Path(dir_name).exists():
            shutil.rmtree(dir_name)
            print(f"Cleaned {dir_name} directory")

    # Ensure release directory exists
    release_dir = Path("../release")
    release_dir.mkdir(exist_ok=True)

    # Build installer with embedded resources
    success = True

    # First build the standalone uninstaller
    print("Building standalone uninstaller...")
    if not build_exe("uninstaller.py", "ruping-uninstaller"):
        print("ERROR: Failed to build uninstaller")
        return False

    # Verify files exist before adding them
    ruping_exe_path = Path("ruping.exe")
    uninstaller_exe_path = Path("../release/ruping-uninstaller.exe")

    if not ruping_exe_path.exists():
        print(f"ERROR: {ruping_exe_path} not found")
        return False

    if not uninstaller_exe_path.exists():
        print(f"ERROR: {uninstaller_exe_path} not found")
        return False

    print(f"Found ruping.exe: {ruping_exe_path.resolve()}")
    print(f"Found ruping-uninstaller.exe: {uninstaller_exe_path.resolve()}")

    # Copy uninstaller.exe to installer directory for embedding
    local_uninstaller_path = Path("ruping-uninstaller.exe")
    shutil.copy2(uninstaller_exe_path, local_uninstaller_path)
    print(f"Copied uninstaller to: {local_uninstaller_path.resolve()}")

    installer_data = [
        (str(ruping_exe_path.resolve()), "."),
        (str(local_uninstaller_path.resolve()), ".")
    ]

    if not build_exe("installer.py", "ruping-installer", add_data=installer_data):
        success = False

    # Clean up temporary files
    temp_files = ["ruping.exe", "ruping-uninstaller.exe"]
    for temp_file in temp_files:
        if Path(temp_file).exists():
            Path(temp_file).unlink()
            print(f"Cleaned temporary file: {temp_file}")

    if success:
        print()
        print("Build completed successfully!")
        print()
        print("Generated files in release directory:")
        release_dir = Path("../release")
        if release_dir.exists():
            for exe_file in release_dir.glob("*.exe"):
                print(f"  {exe_file}")

        print()
        print("Usage:")
        print("  ruping-installer.exe --help")
        print("  ruping-installer.exe --install-path \"C:\\MyTools\\RuPing\"")
        print("  ruping-uninstaller.exe --help")
        print("  ruping-uninstaller.exe --silent")
        print()
        print("The installer includes:")
        print("  - ruping.exe (main program)")
        print("  - ruping-uninstaller.exe (compiled uninstaller)")
        print("  - Command aliases (ruping.cmd, rustping.cmd, rping.cmd)")
        print("  - PATH management")
    else:
        print()
        print("Build failed. Please check the error messages above.")

    return success

if __name__ == "__main__":
    try:
        success = main()
        input("\nPress Enter to exit...")
        sys.exit(0 if success else 1)
    except KeyboardInterrupt:
        print("\nBuild cancelled by user.")
        sys.exit(1)
    except Exception as e:
        print(f"ERROR: Build failed: {e}")
        sys.exit(1)
