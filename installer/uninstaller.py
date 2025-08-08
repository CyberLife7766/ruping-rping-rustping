#!/usr/bin/env python3
"""
RuPing Standalone Uninstaller
Removes RuPing installation and cleans up system changes
"""

import os
import sys
import json
import shutil
import winreg
import ctypes
from pathlib import Path

class RuPingStandaloneUninstaller:
    def __init__(self):
        self.app_name = "RuPing"
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
            print("Please run this uninstaller as administrator.")
            sys.exit(1)
    
    def remove_from_path(self, path_to_remove):
        """Remove directory from system PATH"""
        try:
            with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, 
                              r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
                              0, winreg.KEY_ALL_ACCESS) as key:
                current_path, _ = winreg.QueryValueEx(key, "PATH")
                
                # Remove all occurrences
                paths = current_path.split(';')
                new_paths = [p for p in paths if p != str(path_to_remove)]
                new_path = ';'.join(new_paths)
                
                winreg.SetValueEx(key, "PATH", 0, winreg.REG_EXPAND_SZ, new_path)
                
                # Broadcast environment change (avoid hang using timeout)
                HWND_BROADCAST = 0xFFFF
                WM_SETTINGCHANGE = 0x001A
                SMTO_ABORTIFHUNG = 0x0002
                try:
                    result = ctypes.c_ulong()
                    ctypes.windll.user32.SendMessageTimeoutW(
                        HWND_BROADCAST,
                        WM_SETTINGCHANGE,
                        0,
                        ctypes.c_wchar_p("Environment"),
                        SMTO_ABORTIFHUNG,
                        3000,
                        ctypes.byref(result),
                    )
                except Exception as e:
                    print(f"Warning: Failed to broadcast environment change (non-fatal): {e}")
                
                print(f"Removed {path_to_remove} from system PATH")
                return True
        except Exception as e:
            print(f"Failed to remove from PATH: {e}")
            return False
    
    def remove_start_menu_shortcut(self):
        """Remove start menu shortcut"""
        try:
            start_menu = Path(os.environ['PROGRAMDATA']) / "Microsoft" / "Windows" / "Start Menu" / "Programs"
            shortcuts = ["RuPing.lnk", "RuPing.url"]
            
            removed = False
            for shortcut in shortcuts:
                shortcut_path = start_menu / shortcut
                if shortcut_path.exists():
                    shortcut_path.unlink()
                    print(f"Removed start menu shortcut: {shortcut}")
                    removed = True
            
            return removed
        except Exception as e:
            print(f"Warning: Failed to remove start menu shortcut: {e}")
        return False
    
    def find_installation(self):
        """Find RuPing installation"""
        # First, try to find installation info in current directory
        current_dir = Path(__file__).parent

        # If running as exe, get the directory where the exe is located
        if hasattr(sys, '_MEIPASS'):
            # Running as PyInstaller bundle, use the directory where the exe is located
            if len(sys.argv) > 0:
                exe_path = Path(sys.argv[0]).resolve()
                current_dir = exe_path.parent

        info_file = current_dir / self.install_info_file

        if info_file.exists():
            try:
                with open(info_file, 'r') as f:
                    info = json.load(f)
                return Path(info['install_path']), info
            except Exception as e:
                print(f"Warning: Failed to read installation info: {e}")

        # Check if current directory looks like an installation
        if (current_dir / "ruping.exe").exists():
            return current_dir, None

        # Try common installation locations
        possible_paths = [
            Path(os.environ.get('PROGRAMFILES', 'C:\\Program Files')) / self.app_name,
            Path(os.environ.get('LOCALAPPDATA', '')) / self.app_name,
            Path("C:") / self.app_name
        ]

        for path in possible_paths:
            if path.exists() and (path / "ruping.exe").exists():
                return path, None

        return None, None
    
    def uninstall(self, install_path=None, silent=False):
        """Uninstall RuPing"""
        self.require_admin()
        
        if not silent:
            print("RuPing Standalone Uninstaller")
            print("=============================")
            print()
        
        # Find installation
        if install_path:
            install_dir = Path(install_path)
            install_info = None
        else:
            install_dir, install_info = self.find_installation()
        
        if not install_dir or not install_dir.exists():
            print("ERROR: RuPing installation not found.")
            if not silent:
                custom_path = input("Please enter the installation directory (or press Enter to cancel): ").strip()
                if custom_path and Path(custom_path).exists():
                    install_dir = Path(custom_path)
                else:
                    print("Uninstall cancelled.")
                    return False
            else:
                return False
        
        if not silent:
            print(f"Found RuPing installation at: {install_dir}")
            confirm = input("Are you sure you want to uninstall RuPing? (y/N): ").strip().lower()
            if confirm not in ['y', 'yes']:
                print("Uninstall cancelled.")
                return False
        
        # Remove from PATH
        self.remove_from_path(install_dir)
        
        # Remove start menu shortcut
        self.remove_start_menu_shortcut()
        
        # Remove files
        files_to_remove = []
        if install_info and 'installed_files' in install_info:
            files_to_remove = install_info['installed_files']
        else:
            # Default files to remove
            files_to_remove = [
                "ruping.exe",
                "ruping.cmd",
                "rustping.cmd", 
                "rping.cmd",
                "uninstaller.py",
                "install_info.json"
            ]
        
        removed_files = 0
        for filename in files_to_remove:
            file_path = install_dir / filename
            if file_path.exists():
                try:
                    file_path.unlink()
                    if not silent:
                        print(f"Removed: {filename}")
                    removed_files += 1
                except Exception as e:
                    print(f"Warning: Failed to remove {filename}: {e}")
        
        # Try to remove installation directory if empty
        try:
            if not any(install_dir.iterdir()):
                install_dir.rmdir()
                if not silent:
                    print(f"Removed installation directory: {install_dir}")
            else:
                if not silent:
                    print(f"Installation directory not empty, keeping: {install_dir}")
        except Exception as e:
            print(f"Warning: Failed to remove installation directory: {e}")
        
        if not silent:
            print()
            print("RuPing has been successfully uninstalled!")
            print(f"Removed {removed_files} files.")
            print("Please restart your command prompt to update PATH changes.")
        
        return True

def main():
    import argparse
    
    parser = argparse.ArgumentParser(description="RuPing Standalone Uninstaller")
    parser.add_argument("--install-path", help="Installation directory to remove")
    parser.add_argument("--silent", action="store_true", help="Silent uninstallation")
    
    args = parser.parse_args()
    
    uninstaller = RuPingStandaloneUninstaller()
    
    try:
        success = uninstaller.uninstall(
            install_path=args.install_path,
            silent=args.silent
        )
        
        if not args.silent:
            input("\nPress Enter to exit...")
        
        sys.exit(0 if success else 1)
        
    except KeyboardInterrupt:
        print("\nUninstall cancelled by user.")
        sys.exit(1)
    except Exception as e:
        print(f"ERROR: Uninstall failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
