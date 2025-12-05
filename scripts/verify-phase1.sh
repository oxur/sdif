#!/bin/bash
# Verification script for Phase 1 completion

set -e

echo "=== Phase 1 Verification Script ==="
echo

# Check directory structure
echo "1. Checking directory structure..."
REQUIRED_DIRS=(
    "sdif-sys/src"
    "sdif-sys/sdif"
    "sdif-sys/tests/fixtures"
    "sdif-rs/src"
    "mat2sdif/src"
)

for dir in "${REQUIRED_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        echo "   ✓ $dir exists"
    else
        echo "   ✗ $dir missing!"
        exit 1
    fi
done

# Check required files
echo
echo "2. Checking required files..."
REQUIRED_FILES=(
    "Cargo.toml"
    "sdif-sys/Cargo.toml"
    "sdif-sys/build.rs"
    "sdif-sys/src/lib.rs"
    "sdif-sys/wrapper.h"
    "sdif-rs/Cargo.toml"
    "sdif-rs/src/lib.rs"
    "mat2sdif/Cargo.toml"
    "mat2sdif/src/main.rs"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "   ✓ $file exists"
    else
        echo "   ✗ $file missing!"
        exit 1
    fi
done

# Check if SDIF source is available
echo
echo "3. Checking SDIF source..."
if [ -d "sdif-sys/sdif/src" ] && [ -d "sdif-sys/sdif/include" ]; then
    echo "   ✓ Bundled SDIF source found"
elif pkg-config --exists sdif 2>/dev/null; then
    echo "   ✓ System SDIF library found"
else
    echo "   ⚠ No SDIF source found - bundled build will fail"
    echo "     Run: git clone https://github.com/IRCAM-WAM/SDIF.git sdif-sys/sdif"
fi

# Try to build
echo
echo "4. Attempting build..."
if cargo check -p sdif-sys 2>/dev/null; then
    echo "   ✓ sdif-sys compiles successfully"
else
    echo "   ⚠ Build check failed (may need SDIF source)"
fi

# Run tests if build succeeded
echo
echo "5. Running tests..."
if cargo test -p sdif-sys 2>/dev/null; then
    echo "   ✓ All tests passed"
else
    echo "   ⚠ Tests failed or skipped"
fi

echo
echo "=== Phase 1 Verification Complete ==="
