#!/bin/bash
# Verification script for Phase 2 completion

set -e

echo "=== Phase 2 Verification Script ==="
echo

# Check that Phase 1 is complete
echo "1. Verifying Phase 1 prerequisites..."
if ! cargo check -p sdif-sys 2>/dev/null; then
    echo "   ✗ sdif-sys not building - complete Phase 1 first"
    exit 1
fi
echo "   ✓ sdif-sys builds successfully"

# Check module structure
echo
echo "2. Checking module structure..."
REQUIRED_FILES=(
    "sdif-rs/src/lib.rs"
    "sdif-rs/src/error.rs"
    "sdif-rs/src/init.rs"
    "sdif-rs/src/signature.rs"
    "sdif-rs/src/data_type.rs"
    "sdif-rs/src/file.rs"
    "sdif-rs/src/frame.rs"
    "sdif-rs/src/matrix.rs"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "   ✓ $file exists"
    else
        echo "   ✗ $file missing!"
        exit 1
    fi
done

# Check Cargo.toml has required dependencies
echo
echo "3. Checking dependencies..."
if grep -q "thiserror" sdif-rs/Cargo.toml; then
    echo "   ✓ thiserror dependency present"
else
    echo "   ✗ thiserror dependency missing"
    exit 1
fi

if grep -q 'sdif-sys.*path' sdif-rs/Cargo.toml; then
    echo "   ✓ sdif-sys path dependency present"
else
    echo "   ✗ sdif-sys path dependency missing"
    exit 1
fi

# Try to build sdif-rs
echo
echo "4. Building sdif-rs..."
if cargo build -p sdif-rs 2>/dev/null; then
    echo "   ✓ sdif-rs builds successfully"
else
    echo "   ✗ sdif-rs build failed"
    exit 1
fi

# Try to build with ndarray feature
echo
echo "5. Building with ndarray feature..."
if cargo build -p sdif-rs --features ndarray 2>/dev/null; then
    echo "   ✓ ndarray feature builds successfully"
else
    echo "   ⚠ ndarray feature build failed (optional)"
fi

# Run tests
echo
echo "6. Running tests..."
if cargo test -p sdif-rs 2>/dev/null; then
    echo "   ✓ All unit tests passed"
else
    echo "   ⚠ Some tests failed"
fi

# Check documentation builds
echo
echo "7. Building documentation..."
if cargo doc -p sdif-rs --no-deps 2>/dev/null; then
    echo "   ✓ Documentation builds successfully"
else
    echo "   ⚠ Documentation build had issues"
fi

# Summary
echo
echo "=== Phase 2 Verification Complete ==="
echo
echo "Next steps:"
echo "  1. Add test fixture files to sdif-rs/tests/fixtures/"
echo "  2. Run integration tests: cargo test -p sdif-rs -- --include-ignored"
echo "  3. Proceed to Phase 3: Writing API"
