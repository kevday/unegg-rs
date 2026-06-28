# UnEgg - Complete Project

EGG/ALZ archive decompressor for Alpine Linux with GNOME integration.

## Quick Start

### Install APK on Alpine 3.24
```sh
apk add --allow-untrusted bin/unegg-0.5.0-r5-x86_64.apk
```

### Build from C source (musl static)
```sh
make -f Makefile.c
```

### Build Rust version
```sh
cd rust-src
cargo build --release
```

### Rebuild APK
```sh
# Requires Alpine chroot or Docker
cd apk-build
# See BUILD.md for full instructions
```

## Project Structure
```
unegg-all/
├── README.md                    # This file
├── BUILD.md                     # Detailed build instructions
├── Makefile.c                   # C Makefile (musl cross-compile)
├── Dockerfile                   # Docker build for Alpine
├── build-alpine.sh              # Build script for Alpine
├── build-apk.sh                 # APK package builder
│
├── rust-src/                    # Pure Rust implementation
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs              # CLI
│       ├── format/              # EGG/ALZ parsers
│       ├── algo/                # Decompression (Deflate/BZip2/LZMA/AZO)
│       └── crypto/              # AES-256 / PKZIP decryption
│
├── packaging/                   # GNOME integration
│   └── nautilus-unegg/
│       ├── unegg_extension.py   # Nautilus context menu extension
│       ├── unegg-extract        # Extract wrapper (with password)
│       ├── unegg-crack          # Password recovery engine
│       └── unegg-crack-gui      # GUI for password recovery
│
├── apk-build/                   # Alpine package build
│   └── APKBUILD                 # Alpine package recipe
│
├── patches/                     # Source patches
│   ├── 0001-add-unistd-for-getcwd.patch
│   └── 0002-add-7zip-st-flag.patch
│
└── bin/                         # Pre-built binaries
    ├── unegg-musl-static        # C version (musl, static, 2MB)
    ├── unegg-rs                 # Rust version (751KB)
    └── unegg-0.5.0-r5-x86_64.apk  # Alpine 3.24 package
```

## Features
- **Decompression**: Store, Deflate, BZip2, LZMA, AZO
- **Encryption**: AES-256, AES-128, PKZIP compatible
- **GNOME Integration**: Right-click context menu, MIME types
- **Password Recovery**: Brute force, wordlist, hash verification
- **Alpine 3.24**: Native APK package with post-install hooks

## Tools
| Tool | Description |
|------|-------------|
| `unegg` | Main decompressor CLI |
| `unegg-extract` | Extract wrapper with password support |
| `unegg-crack` | Password recovery (CLI) |
| `unegg-crack-gui` | Password recovery (GUI) |

## unegg-crack Usage
```sh
# Common passwords
unegg-crack file.egg --common

# Wordlist
unegg-crack file.egg -w rockyou.txt

# Brute force
unegg-crack file.egg -m 1 -M 8 -c "abc123" -t 4

# Verify hash
unegg-crack file.egg --hash "salt_hex:hash_hex"

# Generate hash
unegg-crack --generate-hash "mypassword"
```
