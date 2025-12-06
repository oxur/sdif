use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Determine linking strategy
    let use_bundled = env::var("CARGO_FEATURE_BUNDLED").is_ok()
        || env::var("CARGO_FEATURE_STATIC").is_ok();

    // Check if we're in a docs.rs build
    let is_docs_rs = env::var("DOCS_RS").is_ok();

    if is_docs_rs {
        // For docs.rs, generate stub bindings without the library
        println!("cargo:warning=Building on docs.rs - generating stub bindings");
        println!("cargo:rustc-cfg=sdif_stub_bindings");
        generate_stub_bindings(&out_dir);
        return;
    }

    let (include_path, lib_path) = if use_bundled {
        match try_build_bundled(&out_dir) {
            Some(paths) => paths,
            None => {
                println!("cargo:warning=SDIF library not available - generating stub bindings");
                println!("cargo:warning=The crate will compile but functions will not be available at runtime");
                println!("cargo:rustc-cfg=sdif_stub_bindings");
                generate_stub_bindings(&out_dir);
                return;
            }
        }
    } else {
        match try_pkg_config() {
            Some(paths) => paths,
            None => {
                println!("cargo:warning=pkg-config failed to find SDIF library");
                match try_build_bundled(&out_dir) {
                    Some(paths) => {
                        println!("cargo:warning=Falling back to bundled build");
                        paths
                    }
                    None => {
                        println!("cargo:warning=SDIF library not available - generating stub bindings");
                        println!("cargo:warning=The crate will compile but functions will not be available at runtime");
                        println!("cargo:rustc-cfg=sdif_stub_bindings");
                        generate_stub_bindings(&out_dir);
                        return;
                    }
                }
            }
        }
    };

    // Generate bindings
    generate_bindings(&include_path, &out_dir);

    // Output linking directives
    if let Some(lib_path) = lib_path {
        println!("cargo:rustc-link-search=native={}", lib_path.display());
    }

    if use_bundled || env::var("CARGO_FEATURE_STATIC").is_ok() {
        println!("cargo:rustc-link-lib=static=sdif");
    } else {
        println!("cargo:rustc-link-lib=sdif");
    }
}

/// Try to find SDIF using pkg-config
fn try_pkg_config() -> Option<(PathBuf, Option<PathBuf>)> {
    // Try pkg-config first
    match pkg_config::Config::new()
        .atleast_version("3.0")
        .probe("sdif")
    {
        Ok(lib) => {
            let include_path = lib.include_paths
                .first()
                .cloned()
                .unwrap_or_else(|| PathBuf::from("/usr/include"));

            let lib_path = lib.link_paths.first().cloned();

            println!("cargo:info=Found SDIF via pkg-config");
            Some((include_path, lib_path))
        }
        Err(e) => {
            println!("cargo:warning=pkg-config error: {}", e);
            None
        }
    }
}

/// Try to build SDIF from bundled source
fn try_build_bundled(out_dir: &PathBuf) -> Option<(PathBuf, Option<PathBuf>)> {
    println!("cargo:info=Attempting to build SDIF from bundled source");

    let sdif_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("sdif");

    // Check if bundled source exists
    if !sdif_dir.exists() {
        println!("cargo:warning=Bundled SDIF source not found at {:?}", sdif_dir);
        return None;
    }

    // Collect C source files
    // Note: SDIF library has source files in sdif/sdif/ directory, not sdif/src/
    let src_dir = sdif_dir.join("sdif");
    let include_dir = sdif_dir.join("include");

    if !src_dir.exists() || !include_dir.exists() {
        println!("cargo:warning=Bundled SDIF source incomplete (missing sdif or include)");
        println!("cargo:warning=  src_dir: {:?}", src_dir);
        println!("cargo:warning=  include_dir: {:?}", include_dir);
        return None;
    }

    let c_files: Vec<_> = match std::fs::read_dir(&src_dir) {
        Ok(entries) => entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension().map(|e| e == "c").unwrap_or(false) {
                    Some(path)
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => {
            println!("cargo:warning=Failed to read bundled SDIF src directory");
            return None;
        }
    };

    if c_files.is_empty() {
        println!("cargo:warning=No C source files found in bundled SDIF");
        return None;
    }

    // Build the static library
    let mut build = cc::Build::new();
    build
        .files(&c_files)
        .include(&include_dir)
        .include(&src_dir)  // Include sdif/sdif/ for local headers like host_architecture.h
        .warnings(false)  // SDIF code may have warnings we can't fix
        .opt_level(2)
        .define("HAVE_STDINT_H", "1");  // Modern C compilers have stdint.h

    // Platform-specific settings
    if cfg!(target_os = "windows") {
        build.define("WIN32", None);
    }

    // Endianness settings - SDIF library needs these for modern architectures
    // ARM64, x86_64, and most modern architectures are little-endian
    if cfg!(target_endian = "little") {
        build.define("HOST_ENDIAN_LITTLE", "1");
    } else {
        build.define("HOST_ENDIAN_BIG", "1");
        build.define("WORDS_BIGENDIAN", "1");
    }

    // Set SDIFTYPES path if needed
    if let Ok(types_path) = env::var("SDIFTYPES") {
        build.define("SDIFTYPES_FILE", Some(types_path.as_str()));
    }

    build.compile("sdif");

    // Mark source files for rebuild tracking
    for file in &c_files {
        println!("cargo:rerun-if-changed={}", file.display());
    }

    Some((include_dir, Some(out_dir.clone())))
}

/// Generate Rust bindings using bindgen
fn generate_bindings(include_path: &PathBuf, out_dir: &PathBuf) {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include_path.display()))

        // Allowlist SDIF types and functions
        .allowlist_function("Sdif.*")
        .allowlist_function("_Sdif.*")
        .allowlist_type("Sdif.*")
        .allowlist_type("_Sdif.*")
        .allowlist_type("eSdif.*")
        .allowlist_var("Sdif.*")
        .allowlist_var("eSdif.*")

        // File mode enums
        .allowlist_type("SdifFileModeE")
        .allowlist_var("eReadFile")
        .allowlist_var("eWriteFile")
        .allowlist_var("eUnknownFileMode")
        .allowlist_var("ePredefinedTypes")
        .allowlist_var("eModeMask")

        // Data type enums
        .allowlist_type("SdifDataTypeE")
        .allowlist_var("eFloat4")
        .allowlist_var("eFloat8")
        .allowlist_var("eInt1")
        .allowlist_var("eInt2")
        .allowlist_var("eInt4")
        .allowlist_var("eUInt1")
        .allowlist_var("eUInt2")
        .allowlist_var("eUInt4")
        .allowlist_var("eText")

        // Create type aliases for compatibility
        .type_alias("SdifFileModeET")
        .type_alias("SdifDataTypeET")

        // Derive useful traits where possible
        .derive_debug(true)
        .derive_default(true)
        .derive_copy(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_partialeq(true)

        // Other options
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate_comments(true)
        .layout_tests(true)

        .generate()
        .expect("Failed to generate bindings");

    let bindings_path = out_dir.join("bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("Failed to write bindings");

    println!("cargo:info=Generated bindings at {:?}", bindings_path);
}

/// Generate stub bindings when SDIF library is not available
/// This allows the crate to compile for publishing, but the functions won't be usable
fn generate_stub_bindings(out_dir: &PathBuf) {
    let stub_bindings = r#"
// Stub bindings generated because SDIF library was not available at build time.
// To use this crate, you must:
// 1. Install the SDIF library system-wide, OR
// 2. Download the SDIF source and place it in the sdif/ directory, then rebuild with --features bundled
//
// See the README.md for detailed instructions.

use std::os::raw::{c_char, c_int, c_void, c_double, c_float};

// Opaque types (matching real SDIF library structure)
#[repr(C)]
pub struct SdifFileT {
    _private: [u8; 0],
}

// Type aliases
pub type SdifSignature = u32;
pub type SdifFloat8 = c_double;
pub type SdifFloat4 = c_float;

// File mode enum
pub type SdifFileModeET = u32;
pub const SdifFileModeET_eReadFile: u32 = 1;
pub const SdifFileModeET_eWriteFile: u32 = 2;
pub const SdifFileModeET_ePredefinedTypes: u32 = 4;
pub const SdifFileModeET_eModeMask: u32 = 7;

// Data type enum
pub type SdifDataTypeET = u32;
pub const SdifDataTypeET_eFloat4: u32 = 0x0004;
pub const SdifDataTypeET_eFloat8: u32 = 0x0008;
pub const SdifDataTypeET_eInt1: u32 = 0x0001;
pub const SdifDataTypeET_eInt2: u32 = 0x0002;
pub const SdifDataTypeET_eInt4: u32 = 0x0004;
pub const SdifDataTypeET_eUInt1: u32 = 0x0101;
pub const SdifDataTypeET_eUInt2: u32 = 0x0102;
pub const SdifDataTypeET_eUInt4: u32 = 0x0104;
pub const SdifDataTypeET_eText: u32 = 0x0301;

// Stub function declarations - these will link but panic at runtime
extern "C" {
    pub fn SdifGenInit(name: *const c_char) -> c_int;
    pub fn SdifGenKill();
    pub fn SdifFOpen(name: *const c_char, mode: SdifFileModeET) -> *mut SdifFileT;
    pub fn SdifFClose(file: *mut SdifFileT) -> c_int;
    pub fn SdifFReadGeneralHeader(file: *mut SdifFileT) -> usize;
    pub fn SdifFReadAllASCIIChunks(file: *mut SdifFileT) -> isize;
    pub fn SdifSignatureConst(a: c_char, b: c_char, c: c_char, d: c_char) -> SdifSignature;
    pub fn SdifSizeofDataType(data_type: SdifDataTypeET) -> usize;

    // Frame reading functions
    pub fn SdifFReadFrameHeader(file: *mut SdifFileT) -> isize;
    pub fn SdifFSkipFrameData(file: *mut SdifFileT) -> isize;
    pub fn SdifFCurrTime(file: *mut SdifFileT) -> c_double;
    pub fn SdifFCurrFrameSignature(file: *mut SdifFileT) -> SdifSignature;
    pub fn SdifFCurrNbMatrix(file: *mut SdifFileT) -> u32;
    pub fn SdifFGetSignature(file: *mut SdifFileT) -> u32;

    // Matrix reading functions
    pub fn SdifFReadMatrixHeader(file: *mut SdifFileT) -> isize;
    pub fn SdifFSkipMatrixData(file: *mut SdifFileT) -> isize;
    pub fn SdifFCurrMatrixSignature(file: *mut SdifFileT) -> SdifSignature;
    pub fn SdifFCurrNbRow(file: *mut SdifFileT) -> u32;
    pub fn SdifFCurrNbCol(file: *mut SdifFileT) -> u32;
    pub fn SdifFCurrDataType(file: *mut SdifFileT) -> SdifDataTypeET;
    pub fn SdifFReadOneRow(file: *mut SdifFileT) -> isize;
    pub fn SdifFCurrOneRowData(file: *mut SdifFileT) -> *mut c_void;
    pub fn SdifFReadMatrixData(file: *mut SdifFileT) -> isize;

    // Writing functions - General
    pub fn SdifFWriteGeneralHeader(file: *mut SdifFileT) -> usize;
    pub fn SdifFWriteAllASCIIChunks(file: *mut SdifFileT) -> isize;

    // Writing functions - Frame and Matrix (simple)
    pub fn SdifFWriteFrameAndOneMatrix(
        file: *mut SdifFileT,
        frame_sig: SdifSignature,
        stream_id: u32,
        time: c_double,
        matrix_sig: SdifSignature,
        data_type: SdifDataTypeET,
        nb_row: u32,
        nb_col: u32,
        data: *mut c_void,
    ) -> usize;

    // Writing functions - Frame (for multi-matrix frames)
    pub fn SdifFSetCurrFrameHeader(
        file: *mut SdifFileT,
        signature: SdifSignature,
        size: u32,
        nb_matrix: u32,
        stream_id: u32,
        time: c_double,
    );
    pub fn SdifFWriteFrameHeader(file: *mut SdifFileT) -> usize;

    // Writing functions - Matrix
    pub fn SdifFSetCurrMatrixHeader(
        file: *mut SdifFileT,
        signature: SdifSignature,
        data_type: SdifDataTypeET,
        nb_row: u32,
        nb_col: u32,
    );
    pub fn SdifFWriteMatrixHeader(file: *mut SdifFileT) -> usize;
    pub fn SdifFWriteMatrixData(file: *mut SdifFileT, data: *mut c_void) -> usize;
    pub fn SdifFWritePadding(file: *mut SdifFileT, padding_size: u32) -> usize;

    // Signature conversion functions
    pub fn SdifStringToSignature(str_: *const c_char) -> SdifSignature;
    pub fn SdifSignatureToString(sig: SdifSignature) -> *const c_char;

    // NVT functions
    pub fn SdifFNameValueList(file: *mut SdifFileT) -> *mut c_void;  // Returns SdifNameValuesLT*
    pub fn SdifNameValuesLNewTable(nvt_list: *mut c_void, stream_id: u32) -> *mut c_void;
    pub fn SdifNameValuesLPutCurrNVT(
        nvt_list: *mut c_void,
        name: *const c_char,
        value: *const c_char,
    );

    // Matrix type definition functions
    pub fn SdifFGetMatrixTypesTable(file: *mut SdifFileT) -> *mut c_void;  // Returns SdifHashTableT*
    pub fn SdifCreateMatrixType(
        signature: SdifSignature,
        predefined: *mut c_void,
    ) -> *mut c_void;  // Returns SdifMatrixTypeT*
    pub fn SdifMatrixTypeInsertTailColumnDef(
        mtype: *mut c_void,
        column_name: *const c_char,
    );
    pub fn SdifPutMatrixType(
        table: *mut c_void,
        mtype: *mut c_void,
    );

    // Frame type definition functions
    pub fn SdifFGetFrameTypesTable(file: *mut SdifFileT) -> *mut c_void;  // Returns SdifHashTableT*
    pub fn SdifCreateFrameType(
        signature: SdifSignature,
        predefined: *mut c_void,
    ) -> *mut c_void;  // Returns SdifFrameTypeT*
    pub fn SdifFrameTypePutComponent(
        ftype: *mut c_void,
        component_sig: SdifSignature,
        component_name: *const c_char,
    );
    pub fn SdifPutFrameType(
        table: *mut c_void,
        ftype: *mut c_void,
    );
}

#[cfg(test)]
mod stub_warning {
    #[test]
    #[ignore]
    fn warn_about_stubs() {
        panic!("This is a stub build of sdif-sys. The SDIF library must be installed to run tests.");
    }
}
"#;

    let bindings_path = out_dir.join("bindings.rs");
    std::fs::write(&bindings_path, stub_bindings)
        .expect("Failed to write stub bindings");

    println!("cargo:info=Generated stub bindings at {:?}", bindings_path);
}
