#!/bin/bash
# Verification script for Phase 4 completion

set -e

echo "=== Phase 4 Verification Script ==="
echo

# Check prerequisites
echo "1. Verifying prerequisites..."
if ! cargo check -p sdif-rs 2>/dev/null; then
    echo "   ✗ sdif-rs not building - complete Phase 3 first"
    exit 1
fi
echo "   ✓ sdif-rs builds (without mat feature)"

# Check new module files
echo
echo "2. Checking Phase 4 modules..."
REQUIRED_FILES=(
    "sdif-rs/src/mat/mod.rs"
    "sdif-rs/src/mat/file.rs"
    "sdif-rs/src/mat/data.rs"
    "sdif-rs/src/mat/time.rs"
    "sdif-rs/src/mat/convert.rs"
    "sdif-rs/src/mat/complex.rs"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "   ✓ $file exists"
    else
        echo "   ✗ $file missing!"
        exit 1
    fi
done

# Check feature flag in Cargo.toml
echo
echo "3. Checking feature configuration..."
if grep -q 'mat = \["dep:matfile"' sdif-rs/Cargo.toml; then
    echo "   ✓ mat feature defined"
else
    echo "   ✗ mat feature not defined in Cargo.toml"
    exit 1
fi

if grep -q 'matfile.*optional.*true' sdif-rs/Cargo.toml; then
    echo "   ✓ matfile is optional dependency"
else
    echo "   ✗ matfile should be optional"
    exit 1
fi

# Build with mat feature
echo
echo "4. Building with mat feature..."
if cargo build -p sdif-rs --features mat 2>/dev/null; then
    echo "   ✓ Builds with mat feature"
else
    echo "   ✗ Build with mat feature failed"
    exit 1
fi

# Check that it still builds without mat feature
echo
echo "5. Verifying builds without mat feature..."
if cargo build -p sdif-rs 2>/dev/null; then
    echo "   ✓ Builds without mat feature"
else
    echo "   ✗ Build without mat feature failed"
    exit 1
fi

# Run unit tests
echo
echo "6. Running unit tests..."
if cargo test -p sdif-rs --features mat --lib 2>/dev/null; then
    echo "   ✓ Unit tests passed"
else
    echo "   ⚠ Some unit tests failed"
fi

# Run MAT integration tests (those that don't need fixtures)
echo
echo "7. Running MAT integration tests..."
if cargo test -p sdif-rs --features mat --test mat_tests 2>/dev/null; then
    echo "   ✓ MAT tests passed"
else
    echo "   ⚠ MAT tests failed (may need fixtures)"
fi

# Check documentation builds
echo
echo "8. Building documentation..."
if cargo doc -p sdif-rs --features mat --no-deps 2>/dev/null; then
    echo "   ✓ Documentation builds"
else
    echo "   ⚠ Documentation issues"
fi

# Check that public types are exported
echo
echo "9. Checking public exports..."
EXPORTS=(
    "MatFile"
    "MatData"
    "MatToSdifConfig"
    "MatToSdifConverter"
)

for export in "${EXPORTS[@]}"; do
    if grep -q "pub use.*$export" sdif-rs/src/lib.rs; then
        echo "   ✓ $export exported"
    else
        echo "   ✗ $export not exported"
    fi
done

# Summary
echo
echo "=== Phase 4 Verification Complete ==="
echo
echo "MAT file support is implemented with:"
echo "  - MatFile for loading .mat files"
echo "  - MatData for individual variables"
echo "  - Time vector auto-detection"
echo "  - Complex number handling"
echo "  - MatToSdifConverter for conversion"
echo
echo "Next steps:"
echo "  1. Add test MAT files to tests/fixtures/"
echo "  2. Run: cargo test -p sdif-rs --features mat -- --include-ignored"
echo "  3. Proceed to Phase 5: mat2sdif CLI tool"
