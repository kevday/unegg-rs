#!/usr/bin/env python3
"""
Nautilus extension for UnEgg - adds right-click context menu
for .egg and .alz archives.
"""
import os
import subprocess
import gi

gi.require_version('Nautilus', '4.0')
from gi.repository import Nautilus, GObject, Gio, GLib

# Supported MIME types
MIME_TYPES = [
    'application/x-egg-archive',
    'application/x-alz-archive',
]

# Also match by extension as fallback
EXTENSIONS = ['.egg', '.alz', '.EGG', '.ALZ']


def is_archive(file_info):
    """Check if file is an EGG/ALZ archive."""
    mime = file_info.get_mime_type()
    if mime in MIME_TYPES:
        return True
    name = file_info.get_name()
    return any(name.endswith(ext) for ext in EXTENSIONS)


class UnEggExtension(GObject.GObject, Nautilus.MenuProvider):
    def __init__(self):
        super().__init__()

    def _get_file_path(self, file_info):
        """Get the full filesystem path."""
        location = file_info.get_location()
        if location:
            return location.get_path()
        return None

    def _run_extract(self, archive_path, dest_dir, password=None):
        """Run extraction in a terminal."""
        # Build the command
        cmd_parts = [
            'unegg-extract',
            archive_path,
        ]
        if dest_dir:
            cmd_parts.append(dest_dir)
        if password:
            cmd_parts.append(password)

        # Build terminal command
        cmd_str = ' '.join(f'"{p}"' for p in cmd_parts)

        # Try different terminal emulators
        terminals = [
            ['gnome-terminal', '--'],
            ['xterm', '-e'],
            ['konsole', '-e'],
            ['xfce4-terminal', '-e'],
            ['mate-terminal', '-e'],
            ['lxterminal', '-e'],
            ['alacritty', '-e'],
            ['kitty'],
        ]

        for term in terminals:
            try:
                full_cmd = term + cmd_parts
                subprocess.Popen(full_cmd,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                    start_new_session=True)
                return
            except FileNotFoundError:
                continue

        # Fallback: run without terminal (output goes nowhere useful)
        subprocess.Popen(cmd_parts,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True)

    def _extract_here(self, menu, files):
        """Extract to same directory."""
        for f in files:
            path = self._get_file_path(f)
            if path:
                dest = os.path.dirname(path)
                self._run_extract(path, dest)

    def _extract_to(self, menu, files):
        """Extract to a subdirectory named after the archive."""
        for f in files:
            path = self._get_file_path(f)
            if path:
                parent = os.path.dirname(path)
                name = os.path.splitext(os.path.basename(path))[0]
                dest = os.path.join(parent, name)
                os.makedirs(dest, exist_ok=True)
                self._run_extract(path, dest)

    def _extract_with_password(self, menu, files):
        """Extract with password prompt."""
        for f in files:
            path = self._get_file_path(f)
            if path:
                # Try to get password via zenity/yad, fallback to terminal
                password = self._ask_password(path)
                if password:
                    dest = os.path.dirname(path)
                    self._run_extract(path, dest, password)

    def _crack_password(self, menu, files):
        """Launch password recovery tool."""
        for f in files:
            path = self._get_file_path(f)
            if path:
                try:
                    subprocess.Popen(
                        ['unegg-crack-gui', path],
                        stdout=subprocess.DEVNULL,
                        stderr=subprocess.DEVNULL,
                        start_new_session=True
                    )
                except FileNotFoundError:
                    pass

    def _ask_password(self, archive_path):
        """Ask for password using GUI dialog, return password or None."""
        name = os.path.basename(archive_path)

        # Try zenity first
        try:
            result = subprocess.run([
                'zenity', '--password',
                '--title=UnEgg - Password Required',
                f'--text=Archive "{name}" is encrypted.\nEnter password:',
            ], capture_output=True, text=True, timeout=300)
            if result.returncode == 0:
                return result.stdout.strip()
            return None
        except (FileNotFoundError, subprocess.TimeoutExpired):
            pass

        # Try yad
        try:
            result = subprocess.run([
                'yad', '--form', '--field=Password:H',
                f'--title=UnEgg - Password for {name}',
                '--text=This archive is encrypted. Enter password:',
            ], capture_output=True, text=True, timeout=300)
            if result.returncode == 0:
                return result.stdout.strip().rstrip('|')
            return None
        except (FileNotFoundError, subprocess.TimeoutExpired):
            pass

        # Try kdialog
        try:
            result = subprocess.run([
                'kdialog', '--password',
                f'Enter password for "{name}":',
            ], capture_output=True, text=True, timeout=300)
            if result.returncode == 0:
                return result.stdout.strip()
            return None
        except (FileNotFoundError, subprocess.TimeoutExpired):
            pass

        # Fallback: let the terminal handle it
        return None

    def get_file_items(self, *args):
        """Called by Nautilus to get menu items for selected files."""
        # Handle both Nautilus 4.0 and 3.0 API
        if len(args) >= 2:
            files = args[1]
        else:
            files = args[0] if args else []

        if not files:
            return []

        # Check if any selected file is an archive
        archives = [f for f in files if is_archive(f)]
        if not archives:
            return []

        items = []

        # "Extract Here"
        item = Nautilus.MenuItem(
            name='UnEgg::extract_here',
            label='Extrair aqui',
            tip='Extract archive to current directory',
        )
        item.connect('activate', self._extract_here, archives)
        items.append(item)

        # "Extract to folder"
        item = Nautilus.MenuItem(
            name='UnEgg::extract_to',
            label='Extrair para pasta...',
            tip='Extract archive to a named subfolder',
        )
        item.connect('activate', self._extract_to, archives)
        items.append(item)

        # "Extract with password"
        item = Nautilus.MenuItem(
            name='UnEgg::extract_password',
            label='Extrair com senha...',
            tip='Extract encrypted archive with password',
        )
        item.connect('activate', self._extract_with_password, archives)
        items.append(item)

        # "Crack password" (for encrypted archives)
        item = Nautilus.MenuItem(
            name='UnEgg::crack',
            label='Recuperar senha...',            
            tip='Brute force or wordlist password recovery',
        )
        item.connect('activate', self._crack_password, archives)
        items.append(item)

        return items
