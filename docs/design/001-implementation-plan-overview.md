# Rust SDIF Library - Implementation Plan Overview

## Project Structure

The project consists of three main components:
1. **sdif-sys** - Raw FFI bindings to IRCAM SDIF C library
2. **sdif-rs** - Safe, idiomatic Rust wrapper
3. **mat2sdif** - CLI tool for converting MATLAB/Octave files to SDIF

## Development Phases

### Phase 1: FFI Foundation (sdif-sys)
**Duration:** 1-2 days  
**Dependencies:** None  
**Deliverables:**
- `sdif-sys` crate with bindgen-generated bindings
- `build.rs` supporting both system libraries and bundled compilation
- Basic smoke tests verifying C library initialization

**Key Activities:**
- Configure Cargo.toml with `links = "sdif"` and build dependencies
- Implement build.rs with pkg-config fallback to bundled source
- Generate bindings from SDIF C headers with appropriate allowlists
- Test basic C function calls (SdifGenInit, SdifGenKill)

**Success Criteria:**
- Bindings compile on Linux, macOS, Windows
- Can initialize and shutdown SDIF library without crashes
- All core types (SdifFileT, SdifSignature, etc.) are exposed

---

### Phase 2: Safe Reading API (sdif-rs core)
**Duration:** 3-4 days  
**Dependencies:** Phase 1 complete  
**Deliverables:**
- Core error handling types
- SdifFile reader with RAII
- Frame and Matrix iterators
- Global initialization management

**Key Activities:**
- Define SdifError enum and Result alias
- Wrap SdifFileT with Drop implementation
- Implement Frame and Matrix borrowing from SdifFile
- Create iterator-based high-level API
- Add signature conversion utilities
- Write integration tests with test SDIF files

**Success Criteria:**
- Can open and read SDIF files safely
- Iterator API works for frames and matrices
- All memory is properly managed (no leaks)
- Test suite passes with sample SDIF files

---

### Phase 3: Writing API with Builder Pattern
**Duration:** 3-4 days  
**Dependencies:** Phase 2 complete  
**Deliverables:**
- SdifFileBuilder with typestate pattern
- SdifWriter for active file writing
- FrameBuilder for multi-matrix frames
- Write operation tests

**Key Activities:**
- Implement typestate builder (Unconfigured → HeaderWritten states)
- Add NVT metadata, matrix type, and frame type registration
- Create FrameBuilder with consuming API
- Support both single-matrix and multi-matrix frames
- Add data type conversion (f32/f64)
- Test write-then-read roundtrips

**Success Criteria:**
- Can create valid SDIF files from scratch
- Builder API prevents invalid state transitions at compile time
- Written files can be read back with matching data
- Supports both f32 and f64 matrix data

---

### Phase 4: MAT File Integration
**Duration:** 2-3 days  
**Dependencies:** Phase 3 complete  
**Deliverables:**
- MatData struct for parsing MAT files
- Time vector detection heuristics
- MAT-to-SDIF conversion helpers
- Configuration struct for conversion options

**Key Activities:**
- Integrate matfile crate for Level 5 MAT parsing
- Implement MatData wrapper with shape/type extraction
- Add time vector detection (1D arrays with ascending values)
- Create conversion utilities mapping MAT arrays to SDIF frames
- Handle complex numbers (split or magnitude/phase)
- Test with real MATLAB/Octave files

**Success Criteria:**
- Can parse numeric arrays from MAT files
- Correctly identifies time vectors
- Converts MAT matrices to SDIF-compatible format
- Handles transposition (column-major to row-major)

---

### Phase 5: mat2sdif CLI Tool
**Duration:** 2-3 days  
**Dependencies:** Phases 3 & 4 complete  
**Deliverables:**
- Command-line binary with clap
- Variable listing mode
- Max compatibility enforcement
- End-to-end conversion pipeline

**Key Activities:**
- Define CLI interface with clap derive macros
- Implement --list mode for MAT variable inspection
- Add Max-specific validation (partial limits, zero amplitude)
- Create conversion pipeline with error handling
- Add dry-run mode for validation
- Write usage documentation and examples

**Success Criteria:**
- Can convert MAT files to SDIF from command line
- Produces Max-compatible SDIF files
- Clear error messages for unsupported features
- Works with typical audio analysis MAT files

---

### Phase 6: Documentation and Polish
**Duration:** 2-3 days  
**Dependencies:** Phases 1-5 complete  
**Deliverables:**
- Complete rustdoc documentation
- Examples directory with working code
- Integration test suite
- CI/CD configuration
- README with quick start guide

**Key Activities:**
- Add rustdoc to all public APIs with examples
- Create example programs (read, write, convert)
- Write integration tests for full workflows
- Set up GitHub Actions for cross-platform CI
- Document Max compatibility requirements
- Create contributor guidelines

**Success Criteria:**
- All public APIs have documentation with examples
- Examples compile and run successfully
- CI passes on Linux, macOS, Windows
- README provides clear getting-started path
- Test coverage >80% of public APIs

---

## Timeline Summary

| Phase | Duration | Cumulative |
|-------|----------|------------|
| 1. FFI Foundation | 1-2 days | 1-2 days |
| 2. Reading API | 3-4 days | 4-6 days |
| 3. Writing API | 3-4 days | 7-10 days |
| 4. MAT Integration | 2-3 days | 9-13 days |
| 5. CLI Tool | 2-3 days | 11-16 days |
| 6. Documentation | 2-3 days | 13-19 days |

**Total Estimated Duration:** 2.5-4 weeks (assuming full-time development)

## Risk Mitigation

**Risk:** SDIF C library build issues on Windows  
**Mitigation:** Test bundled compilation early, provide pre-built binaries if needed

**Risk:** MAT file format variations not handled by matfile crate  
**Mitigation:** Test with diverse MAT files early, document supported subset

**Risk:** Max compatibility issues discovered late  
**Mitigation:** Create Max test patches in Phase 3, validate SDIF outputs continuously

**Risk:** Performance issues with large files  
**Mitigation:** Use streaming iteration patterns, benchmark with realistic datasets

## Testing Strategy

Each phase includes specific tests:
- **Unit tests:** Test individual functions and methods
- **Integration tests:** Test complete workflows across module boundaries
- **Roundtrip tests:** Write data, read back, verify equality
- **Compatibility tests:** Verify outputs work with Max/MSP
- **Performance tests:** Benchmark with large files (added in Phase 6)

## Deployment

**Library (sdif-rs):**
- Publish to crates.io after Phase 6
- Semantic versioning starting at 0.1.0
- Tag GitHub releases

**Binary (mat2sdif):**
- Cross-compile for Linux, macOS, Windows
- Distribute via GitHub releases
- Consider cargo-binstall support

**Documentation:**
- Auto-deploy rustdoc to docs.rs
- Host examples on GitHub
- Create Max/MSP integration guide
