use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Determine linking strategy
    let use_bundled = env::var("CARGO_FEATURE_BUNDLED").is_ok()
        || env::var("CARGO_FEATURE_STATIC").is_ok();

    let (include_path, lib_path) = if use_bundled {
        build_bundled(&out_dir)
    } else {
        match try_pkg_config() {
            Some(paths) => paths,
            None => {
                println!("cargo:warning=pkg-config failed to find SDIF library, falling back to bundled");
                build_bundled(&out_dir)
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

/// Build SDIF from bundled source
fn build_bundled(out_dir: &PathBuf) -> (PathBuf, Option<PathBuf>) {
    println!("cargo:info=Building SDIF from bundled source");

    let sdif_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("sdif");

    // Check if bundled source exists
    if !sdif_dir.exists() {
        panic!(
            "Bundled SDIF source not found at {:?}. \n\
             Either install the SDIF library system-wide, \n\
             or download the SDIF source and place it in the sdif/ directory. \n\
             See README.md for instructions.",
            sdif_dir
        );
    }

    // Collect C source files
    // Note: Adjust these paths based on actual SDIF source structure
    let src_dir = sdif_dir.join("src");
    let include_dir = sdif_dir.join("include");

    let c_files: Vec<_> = std::fs::read_dir(&src_dir)
        .expect("Failed to read sdif/src directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().map(|e| e == "c").unwrap_or(false) {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    if c_files.is_empty() {
        panic!("No C source files found in {:?}", src_dir);
    }

    // Build the static library
    let mut build = cc::Build::new();
    build
        .files(&c_files)
        .include(&include_dir)
        .warnings(false)  // SDIF code may have warnings we can't fix
        .opt_level(2);

    // Platform-specific settings
    if cfg!(target_os = "windows") {
        build.define("WIN32", None);
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

    (include_dir, Some(out_dir.clone()))
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
        .allowlist_type("SdifFileModeET")
        .allowlist_var("eReadFile")
        .allowlist_var("eWriteFile")
        .allowlist_var("ePredefinedTypes")
        .allowlist_var("eModeMask")

        // Data type enums
        .allowlist_type("SdifDataTypeET")
        .allowlist_var("eFloat4")
        .allowlist_var("eFloat8")
        .allowlist_var("eInt1")
        .allowlist_var("eInt2")
        .allowlist_var("eInt4")
        .allowlist_var("eUInt1")
        .allowlist_var("eUInt2")
        .allowlist_var("eUInt4")
        .allowlist_var("eText")

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
