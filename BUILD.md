# UnEgg - Alpine Linux Build

Decompressor for EGG archive format (ESTsoft), compiled for Alpine Linux with musl libc.

## Quick Start

### Build on Alpine Linux (static binary)
```sh
chmod +x build-alpine.sh
./build-alpine.sh
```

### Build on any Linux (dynamic)
```sh
make
```

### Build static on any system with musl-gcc
```sh
make CC=musl-gcc CXX=musl-g++ LDFLAGS="-static"
```

### Docker build (produces static musl binary)
```sh
docker build -t unegg-alpine .
docker cp $(docker create unegg-alpine):/unegg ./unegg
```

### Install
```sh
make install                  # installs to /usr/local/bin
make install PREFIX=/usr      # custom prefix
```

## Usage

```
unegg [commands] [archive filename] [destination path]

Commands:
  -h          Display help
  -l          List files in archive
  -x          Extract all files
  -vX         Verbose level X
  -r          No progress display (for redirection)
  -dD         Display options (n=name, p=packed, u=unpacked, etc.)
```

## Supported Formats
- **EGG** - ESTsoft EGG archive format
- **ALZ** - ESTsoft ALZ archive format

## Supported Compression Algorithms
- Store (no compression)
- Deflate (zlib)
- BZip2
- LZMA
- AZO (ESTsoft proprietary)

## Supported Encryption
- AES-256
- Zip-compatible encryption

## Source Structure
```
Source/unegg/
├── UnEgg.cpp              # Main entry point
├── CommandLine.cpp/h      # CLI argument parsing
├── EventHandler.cpp/h     # Progress/event handling
└── nest/                  # Core library
    ├── algorithm/         # Compression codecs
    │   ├── azo/           # AZO decoder
    │   ├── BZipCoder.*    # BZip2 wrapper
    │   ├── DeflateCoder.* # Deflate/zlib wrapper
    │   └── LZMACoder.*    # LZMA wrapper
    ├── encryption/        # Decryption (AES, Zip)
    ├── format/            # Archive format parsers
    │   ├── egg/           # EGG format
    │   └── alz/           # ALZ format
    ├── stream/            # I/O streams
    └── third-party/       # Bundled C libraries
        ├── algorithm/     # bzip2, lzma, zlib
        └── encryption/    # AES, SHA1, HMAC
```

## Patches Applied
1. Added `#include <unistd.h>` in `UnEgg.cpp` for `getcwd()` (musl compatibility)
2. Added `-D_7ZIP_ST` flag for LZMA single-thread mode (avoids missing `LzFindMt.h`)

## License
See `Source/unegg/doc/license.txt` and `Module/doc/license.txt`

## Credits
Original project: https://github.com/dterracino/UnEgg
Author: ESTsoft Corp.
