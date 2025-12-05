# sdif-sys

[![Crates.io](https://img.shields.io/crates/v/sdif-sys.svg)](https://crates.io/crates/sdif-sys)
[![Documentation](https://docs.rs/sdif-sys/badge.svg)](https://docs.rs/sdif-sys)

Raw FFI bindings to the IRCAM SDIF (Sound Description Interchange Format) C library.

## ⚠️ Important Note

**This crate is published as a placeholder on crates.io.** When downloaded from crates.io without
the SDIF C library installed, it will compile with stub bindings that provide the type definitions
but no actual functionality. The stub functions will link but cannot be called at runtime.

To use this crate, you **must** have the SDIF C library available either:
1. Installed system-wide (recommended), OR
2. As bundled source (see below)

## Overview

This crate provides low-level, unsafe bindings to the SDIF library. For most use cases,
you should use the `sdif-rs` crate which provides a safe, idiomatic Rust API.

## Building

### Using System Library (Recommended)

If you have SDIF installed system-wide:

```bash
cargo build
```

The build script will use `pkg-config` to find the library.

### Using Bundled Source

To compile SDIF from source:

1. Download the SDIF source (see `sdif/README.md`)
2. Build with the bundled feature:

```bash
cargo build --features bundled
```

### Static Linking

For static linking:

```bash
cargo build --features static
```

## Requirements

- Rust 1.70 or later
- clang (for bindgen)
- pkg-config (for system library detection)
- SDIF C library (system or bundled)

## Safety

All functions in this crate are unsafe. The SDIF library:

- Requires initialization before use (`SdifGenInit`)
- Uses global state (not thread-safe)
- Requires specific call sequences for reading/writing

## License

MIT License - see LICENSE file.

## See Also

- [sdif-rs](../sdif-rs) - Safe Rust wrapper
- [SDIF Specification](http://sdif.sourceforge.net/standard/sdif-standard.html)
- [IRCAM SDIF](https://github.com/IRCAM-WAM/SDIF)
