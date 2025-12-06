# Obtaining the SDIF C Library Source

The SDIF C library source code is needed for the `bundled` feature to work. This document explains how to obtain and set up the source.

## Download Location

The SDIF library is hosted on SourceForge:

**Project Page:** https://sourceforge.net/projects/sdif/

**Latest Version:** SDIF-3.11.7 (2020-10-06)

**Direct Download Links:**
- SDIF-3.11.7: https://sourceforge.net/projects/sdif/files/sdif/SDIF-3.11.7/
- SDIF-3.11.4: https://sourceforge.net/projects/sdif/files/sdif/SDIF-3.11.4/SDIF-3.11.4-src.zip/download

## License

The SDIF library is licensed under **LGPL v2.0** (GNU Lesser General Public License version 2.0).

This is compatible with MIT-licensed Rust code as long as:
- The SDIF library is dynamically linked, OR
- If statically linked, the combined work's source is available

For the `bundled` feature, we compile and statically link, so users should be aware of LGPL requirements.

## Setup Instructions for Claude Code

### Option 1: Download and Extract (Recommended)

```bash
# Navigate to sdif-sys directory
cd sdif-sys

# Download the source archive
curl -L "https://sourceforge.net/projects/sdif/files/sdif/SDIF-3.11.7/SDIF-3.11.7-src.zip/download" -o SDIF-3.11.7-src.zip

# Or use wget
wget "https://sourceforge.net/projects/sdif/files/sdif/SDIF-3.11.7/SDIF-3.11.7-src.zip/download" -O SDIF-3.11.7-src.zip

# Extract
unzip SDIF-3.11.7-src.zip

# Rename to 'sdif' for consistency
mv SDIF-3.11.7-src sdif

# Clean up
rm SDIF-3.11.7-src.zip
```

### Option 2: SVN Checkout (Latest Development)

The SourceForge project also has an SVN repository:

```bash
cd sdif-sys
svn checkout https://svn.code.sf.net/p/sdif/code/trunk sdif
```

## Expected Directory Structure

After setup, the structure should be:

```
sdif-sys/
├── Cargo.toml
├── build.rs
├── src/
│   └── lib.rs
└── sdif/                      # ← SDIF C source here
    ├── COPYING                # LGPL license
    ├── README
    ├── configure              # Autotools configure script
    ├── Makefile.in
    ├── include/
    │   └── sdif.h             # Main header file
    ├── sdif/                   # Source files
    │   ├── SdifFile.c
    │   ├── SdifFrame.c
    │   ├── SdifMatrix.c
    │   ├── SdifGenInit.c
    │   └── ... (many more .c files)
    └── ...
```

## Key Source Files for build.rs

The build.rs needs to compile these essential files (at minimum):

```rust
// In build.rs
let source_files = [
    "sdif/sdif/SdifFile.c",
    "sdif/sdif/SdifFrame.c", 
    "sdif/sdif/SdifMatrix.c",
    "sdif/sdif/SdifGenInit.c",
    "sdif/sdif/SdifSignatureTab.c",
    "sdif/sdif/SdifNameValue.c",
    "sdif/sdif/SdifStreamID.c",
    "sdif/sdif/SdifHash.c",
    "sdif/sdif/SdifErrMess.c",
    "sdif/sdif/SdifFGet.c",
    "sdif/sdif/SdifFPut.c",
    "sdif/sdif/SdifFRead.c",
    "sdif/sdif/SdifFWrite.c",
    "sdif/sdif/SdifSelect.c",
    "sdif/sdif/SdifTest.c",
    "sdif/sdif/SdifString.c",
    "sdif/sdif/SdifList.c",
    // ... check the actual source for complete list
];
```

## Header File for Bindgen

The main header to use with bindgen is:

```c
// wrapper.h
#include "sdif/include/sdif.h"
```

## Build Configuration

When compiling with cc crate:

```rust
cc::Build::new()
    .files(&source_files)
    .include("sdif/include")
    .define("HAVE_STDINT_H", "1")
    // Platform-specific defines may be needed
    .compile("sdif");
```

## Verification

After setup, verify the source is present:

```bash
# Check for main header
ls sdif-sys/sdif/include/sdif.h

# Check for source files
ls sdif-sys/sdif/sdif/*.c | head -5

# Check license
head -20 sdif-sys/sdif/COPYING
```

## Troubleshooting

### "sdif.h not found"
- Ensure the `sdif/` directory is directly inside `sdif-sys/`
- Check that `include/sdif.h` exists inside the `sdif/` directory

### Compilation errors
- The SDIF library was designed for Unix systems
- Windows may need additional configuration
- Check for missing dependencies (usually just libc)

### Download issues
- SourceForge sometimes has slow mirrors
- Try the direct SVN checkout as an alternative
- The zip file is only ~1.2 MB

## Alternative: System Installation

If you prefer to install SDIF system-wide (detected via pkg-config):

```bash
# Extract and build
tar xzf SDIF-3.11.7-src.tar.gz
cd SDIF-3.11.7-src
./configure --prefix=/usr/local
make
sudo make install

# Verify pkg-config finds it
pkg-config --libs --cflags sdif
```

Then the Rust build will use the system library instead of bundled.
