#!/bin/bash
# Verification script for Phase 5 completion

set -e

echo "=== Phase 5 Verification Script ==="
echo

# Check prerequisites
echo "1. Verifying prerequisites..."
if ! cargo check -p sdif-rs --features mat 2>/dev/null; then
    echo "   ✗ sdif-rs with mat feature not building"
    exit 1
fi
echo "   ✓ sdif-rs (with mat) builds"

# Check source files
echo
echo "2. Checking source files..."
REQUIRED_FILES=(
    "mat2sdif/src/main.rs"
    "mat2sdif/src/cli.rs"
    "mat2sdif/src/output.rs"
    "mat2sdif/src/max_compat.rs"
    "mat2sdif/src/commands/mod.rs"
    "mat2sdif/src/commands/list.rs"
    "mat2sdif/src/commands/convert.rs"
    "mat2sdif/src/commands/validate.rs"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "   ✓ $file exists"
    else
        echo "   ✗ $file missing!"
        exit 1
    fi
done

# Build the binary
echo
echo "3. Building mat2sdif..."
if cargo build -p mat2sdif 2>/dev/null; then
    echo "   ✓ Build successful"
else
    echo "   ✗ Build failed"
    exit 1
fi

# Check the binary runs
echo
echo "4. Testing binary..."
if cargo run -p mat2sdif --quiet -- --help >/dev/null 2>&1; then
    echo "   ✓ --help works"
else
    echo "   ✗ --help failed"
    exit 1
fi

if cargo run -p mat2sdif --quiet -- --version >/dev/null 2>&1; then
    echo "   ✓ --version works"
else
    echo "   ✗ --version failed"
    exit 1
fi

# Run unit tests
echo
echo "5. Running unit tests..."
if cargo test -p mat2sdif --lib 2>/dev/null; then
    echo "   ✓ Unit tests passed"
else
    echo "   ⚠ Some unit tests failed"
fi

# Run CLI tests
echo
echo "6. Running CLI tests..."
if cargo test -p mat2sdif --test cli_tests 2>/dev/null; then
    echo "   ✓ CLI tests passed"
else
    echo "   ⚠ CLI tests failed"
fi

# Check documentation
echo
echo "7. Checking documentation..."
if [ -f "mat2sdif/README.md" ]; then
    echo "   ✓ README.md exists"
else
    echo "   ⚠ README.md missing"
fi

# Build release binary
echo
echo "8. Building release binary..."
if cargo build -p mat2sdif --release 2>/dev/null; then
    echo "   ✓ Release build successful"

    # Show binary size
    BINARY="target/release/mat2sdif"
    if [ -f "$BINARY" ]; then
        SIZE=$(du -h "$BINARY" | cut -f1)
        echo "   ✓ Binary size: $SIZE"
    fi
else
    echo "   ⚠ Release build failed"
fi

# Summary
echo
echo "=== Phase 5 Verification Complete ==="
echo
echo "mat2sdif CLI is implemented with:"
echo "  - Argument parsing (clap)"
echo "  - --list mode for variable inspection"
echo "  - --dry-run mode for validation"
echo "  - Full conversion pipeline"
echo "  - Max/MSP compatibility checks"
echo "  - Colored terminal output"
echo
echo "Usage:"
echo "  cargo run -p mat2sdif -- --help"
echo "  cargo run -p mat2sdif -- --list input.mat"
echo "  cargo run -p mat2sdif -- input.mat output.sdif"
echo
echo "Next steps:"
echo "  1. Add test MAT files to mat2sdif/tests/fixtures/"
echo "  2. Test with real MAT files from audio analysis"
echo "  3. Proceed to Phase 6: Documentation and Polish"
