#!/usr/bin/env python3
"""
RuPing Standalone Installer
A self-contained installer that includes all necessary files
"""

import os
import sys
import shutil
import argparse
import json
import subprocess
import winreg
from pathlib import Path
import ctypes
import base64
import tempfile

# Embedded files (will be populated by build script)
EMBEDDED_FILES = {
    'ruping.exe': '',
    'uninstaller.py': '',
    'ruping.cmd': '@echo off\n"%~dp0ruping.exe" %*\n',
    'rustping.cmd': '@echo off\n"%~dp0ruping.exe" %*\n',
    'rping.cmd': '@echo off\n"%~dp0ruping.exe" %*\n'
}

class RuPingStandaloneInstaller:
    def __init__(self):
        self.app_name = "RuPing"
        self.exe_name = "ruping.exe"
        self.aliases = ["ruping", "rustping", "rping"]
        self.default_install_path = Path(os.environ.get('PROGRAMFILES', 'C:\\Program Files')) / self.app_name
        self.install_info_file = "install_info.json"
        
    def is_admin(self):
        """Check if running with administrator privileges"""
        try:
            return ctypes.windll.shell32.IsUserAnAdmin()
        except:
            return False
    
    def require_admin(self):
        """Ensure administrator privileges"""
        if not self.is_admin():
            print("ERROR: Administrator privileges required.")
            print("Please run this installer as administrator.")
            sys.exit(1)
    
    def list_bundle_contents(self, debug=False):
        """List all files in PyInstaller bundle for debugging"""
        if debug and hasattr(sys, '_MEIPASS'):
            bundle_dir = Path(sys._MEIPASS)
            print(f"PyInstaller bundle directory: {bundle_dir}")
            if bundle_dir.exists():
                print("Bundle contents:")
                for item in bundle_dir.rglob("*"):
                    if item.is_file():
                        size_mb = item.stat().st_size / (1024 * 1024)
                        print(f"  {item.relative_to(bundle_dir)} ({size_mb:.2f} MB)")
            else:
                print("Bundle directory does not exist!")
        elif debug:
            print("Not running from PyInstaller bundle")

    def extract_embedded_file(self, filename, target_path, debug=False):
        """Extract embedded file to target path"""
        if debug:
            print(f"\n=== Extracting {filename} ===")
            print(f"Target path: {target_path}")
            # Debug: List bundle contents first
            self.list_bundle_contents(debug=True)

        # First try to extract from PyInstaller bundle
        if hasattr(sys, '_MEIPASS'):
            bundle_dir = Path(sys._MEIPASS)
            bundled_path = bundle_dir / filename

            if bundled_path.exists():
                try:
                    if debug:
                        print(f"Found bundled file! Size: {bundled_path.stat().st_size} bytes")
                    # Ensure target directory exists
                    Path(target_path).parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(bundled_path, target_path)

                    # Verify the copy
                    if Path(target_path).exists():
                        if debug:
                            copied_size = Path(target_path).stat().st_size
                            print(f"✅ Successfully copied {filename} from bundle ({copied_size} bytes)")
                        return True
                    else:
                        if debug:
                            print(f"❌ Copy failed: target file does not exist")
                        return False

                except Exception as e:
                    if debug:
                        print(f"❌ Failed to copy {filename} from bundle: {e}")
                        import traceback
                        traceback.print_exc()
                    return False
            else:
                if debug:
                    print(f"❌ Bundled file not found at {bundled_path}")

        # Fallback to embedded content (for standalone script)
        if debug:
            print(f"Trying fallback methods for {filename}...")

        if filename not in EMBEDDED_FILES:
            # Try to find file locally
            local_paths = [
                Path(filename),
                Path("../target/release") / filename if filename.endswith('.exe') else Path(filename),
                Path(__file__).parent / filename,
                Path(__file__).parent.parent / "target" / "release" / filename if filename.endswith('.exe') else Path(filename)
            ]

            for local_path in local_paths:
                if local_path.exists():
                    try:
                        if debug:
                            print(f"Found local file! Size: {local_path.stat().st_size} bytes")
                        Path(target_path).parent.mkdir(parents=True, exist_ok=True)
                        shutil.copy2(local_path, target_path)
                        if debug:
                            print(f"✅ Successfully copied {filename} from {local_path}")
                        return True
                    except Exception as e:
                        if debug:
                            print(f"❌ Failed to copy {filename} from {local_path}: {e}")
                        continue

            if debug:
                print(f"❌ Could not find {filename} in any local path")
            return False

        # Try embedded content
        content = EMBEDDED_FILES[filename]
        if not content:
            if debug:
                print(f"❌ No content for {filename} in EMBEDDED_FILES")
            return False

        try:
            if debug:
                print(f"Extracting {filename} from embedded content...")
            Path(target_path).parent.mkdir(parents=True, exist_ok=True)

            if filename.endswith('.exe'):
                # Binary file - decode from base64
                with open(target_path, 'wb') as f:
                    f.write(base64.b64decode(content))
            else:
                # Text file
                with open(target_path, 'w', encoding='utf-8') as f:
                    f.write(content)
            if debug:
                print(f"✅ Successfully extracted {filename} from embedded content")
            return True
        except Exception as e:
            if debug:
                print(f"❌ Failed to extract {filename}: {e}")
                import traceback
                traceback.print_exc()
            return False
    
    def add_to_path(self, path_to_add):
        """Add directory to system PATH"""
        try:
            with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, 
                              r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
                              0, winreg.KEY_ALL_ACCESS) as key:
                current_path, _ = winreg.QueryValueEx(key, "PATH")
                
                if str(path_to_add) not in current_path:
                    new_path = f"{current_path};{path_to_add}"
                    winreg.SetValueEx(key, "PATH", 0, winreg.REG_EXPAND_SZ, new_path)
                    
                    # Broadcast environment change
                    HWND_BROADCAST = 0xFFFF
                    WM_SETTINGCHANGE = 0x001A
                    ctypes.windll.user32.SendMessageW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, "Environment")
                    
                    print(f"Added {path_to_add} to system PATH")
                    return True
                else:
                    print(f"{path_to_add} is already in system PATH")
                    return True
        except Exception as e:
            print(f"Failed to add to PATH: {e}")
            return False
    
    def create_start_menu_shortcut(self, install_dir):
        """Create start menu shortcut"""
        try:
            start_menu = Path(os.environ['PROGRAMDATA']) / "Microsoft" / "Windows" / "Start Menu" / "Programs"
            shortcut_path = start_menu / "RuPing.lnk"
            
            # Create a simple .url file instead of .lnk for simplicity
            url_content = f"""[InternetShortcut]
URL=file:///{install_dir / self.exe_name}
IconFile={install_dir / self.exe_name}
IconIndex=0
"""
            with open(start_menu / "RuPing.url", 'w') as f:
                f.write(url_content)
            
            print("Created start menu shortcut")
            return True
        except Exception as e:
            print(f"Warning: Failed to create start menu shortcut: {e}")
            return False
    
    def save_install_info(self, install_dir):
        """Save installation information"""
        info = {
            "install_path": str(install_dir),
            "version": "0.1.0",
            "aliases": self.aliases,
            "installed_files": [
                self.exe_name,
                "ruping-uninstaller.exe",
                self.install_info_file
            ] + [f"{alias}.cmd" for alias in self.aliases]
        }
        
        info_path = install_dir / self.install_info_file
        try:
            with open(info_path, 'w') as f:
                json.dump(info, f, indent=2)
            print(f"Saved installation info to {info_path}")
        except Exception as e:
            print(f"Warning: Failed to save installation info: {e}")
    
    def install(self, install_path=None, no_path=False, silent=False):
        """Install RuPing"""
        self.require_admin()
        
        if not silent:
            print("RuPing Standalone Installer")
            print("===========================")
            print()
        
        # Determine installation directory
        if install_path:
            install_dir = Path(install_path)
        else:
            install_dir = self.default_install_path
        
        if not silent:
            print(f"Installing to: {install_dir}")
        
        # Create installation directory
        try:
            install_dir.mkdir(parents=True, exist_ok=True)
        except Exception as e:
            print(f"ERROR: Failed to create installation directory: {e}")
            return False
        
        # Extract all embedded files
        files_to_extract = [
            (self.exe_name, install_dir / self.exe_name),
            ("ruping-uninstaller.exe", install_dir / "ruping-uninstaller.exe")
        ]

        # Add alias files
        for alias in self.aliases:
            files_to_extract.append((f"{alias}.cmd", install_dir / f"{alias}.cmd"))

        # Enable debug mode for detailed output (can be disabled for production)
        debug_mode = not silent

        for filename, target_path in files_to_extract:
            if not self.extract_embedded_file(filename, target_path, debug=debug_mode):
                print(f"ERROR: Failed to extract {filename}")
                return False
            if not silent:
                print(f"Extracted: {filename}")
        
        # Add to PATH
        if not no_path:
            self.add_to_path(install_dir)
        
        # Create start menu shortcut
        self.create_start_menu_shortcut(install_dir)
        
        # Save installation info
        self.save_install_info(install_dir)
        
        if not silent:
            print()
            print("Installation completed successfully!")
            print()
            print("Available commands:")
            for alias in self.aliases:
                print(f"  {alias} 8.8.8.8              # Basic ping")
            print()
            print("Important notes:")
            print("- RuPing requires administrator privileges to run")
            print("- Please restart your command prompt to use the new commands")
            print("- Use 'ruping --help' to see all available options")
            print()
            print("To uninstall:")
            print(f"  \"{install_dir / 'ruping-uninstaller.exe'}\"")
        
        return True

def main():
    parser = argparse.ArgumentParser(description="RuPing Standalone Installer")
    parser.add_argument("--install-path", help="Custom installation directory")
    parser.add_argument("--no-path", action="store_true", help="Don't add to system PATH")
    parser.add_argument("--silent", action="store_true", help="Silent installation")
    
    args = parser.parse_args()
    
    installer = RuPingStandaloneInstaller()
    
    try:
        success = installer.install(
            install_path=args.install_path,
            no_path=args.no_path,
            silent=args.silent
        )
        
        if not args.silent:
            input("\nPress Enter to exit...")
        
        sys.exit(0 if success else 1)
        
    except KeyboardInterrupt:
        print("\nInstallation cancelled by user.")
        sys.exit(1)
    except Exception as e:
        print(f"ERROR: Installation failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
