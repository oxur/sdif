# Bundled SDIF Source

This directory should contain the IRCAM SDIF C library source code.

## Obtaining the Source

### Option 1: Download from IRCAM

1. Visit the IRCAM Forges: https://github.com/IRCAM-WAM/SDIF
2. Download the source archive
3. Extract it into this directory

### Option 2: Clone from GitHub

```bash
cd sdif-sys
git clone https://github.com/IRCAM-WAM/SDIF.git sdif
```

## Expected Structure

After setup, this directory should contain:

```
sdif/
├── include/
│   ├── sdif.h
│   ├── SdifFile.h
│   ├── SdifFrame.h
│   ├── SdifMatrix.h
│   └── ... (other headers)
├── src/
│   ├── SdifFile.c
│   ├── SdifFrame.c
│   ├── SdifMatrix.c
│   └── ... (other source files)
└── README.md (this file)
```

## Alternative: System Installation

Instead of bundling, you can install SDIF system-wide:

### macOS (Homebrew)
```bash
brew install sdif
```

### Linux (Debian/Ubuntu)
```bash
sudo apt-get install libsdif-dev
```

### From Source
```bash
cd sdif
./configure --prefix=/usr/local
make
sudo make install
```

When using a system installation, build without the `bundled` feature:
```bash
cargo build --no-default-features
```
