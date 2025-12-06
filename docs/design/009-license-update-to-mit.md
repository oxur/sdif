# License Update: Change to MIT License

## Overview

**Objective:** Update all three crates in the rust-sdif workspace to use the MIT license instead of Apache-2.0.

**Rationale:** MIT is fully compatible with LGPL-2.0 (the license of the bundled SDIF C library), eliminating any license compatibility concerns. MIT is also simpler and more permissive, making the project easier for others to use.

---

## Task 1: Update Cargo.toml Files

Update the `license` field in all three crate manifests.

### 1.1 Workspace Cargo.toml

**File:** `Cargo.toml` (workspace root)

Change:
```toml
[workspace.package]
license = "Apache-2.0"
```

To:
```toml
[workspace.package]
license = "MIT"
```

### 1.2 sdif-sys/Cargo.toml

**File:** `sdif-sys/Cargo.toml`

Ensure the license field is either:
- Inherited from workspace: `license.workspace = true`
- Or explicitly set to: `license = "MIT"`

### 1.3 sdif-rs/Cargo.toml

**File:** `sdif-rs/Cargo.toml`

Ensure the license field is either:
- Inherited from workspace: `license.workspace = true`
- Or explicitly set to: `license = "MIT"`

### 1.4 mat2sdif/Cargo.toml

**File:** `mat2sdif/Cargo.toml`

Ensure the license field is either:
- Inherited from workspace: `license.workspace = true`
- Or explicitly set to: `license = "MIT"`

---

## Task 2: Create/Update LICENSE Files

Create a single LICENSE file at the workspace root. Individual crates do not need their own LICENSE files if they reference the workspace root, but having one in each crate is conventional for crates.io publishing.

### 2.1 Workspace Root LICENSE

**File:** `LICENSE`

```
MIT License

Copyright (c) 2024 [Author Name or Organization]

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

**Note:** Replace `[Author Name or Organization]` with the actual copyright holder name.

### 2.2 Individual Crate LICENSE Files (Recommended)

For clean crates.io publishing, copy the same LICENSE file to each crate directory:

- `sdif-sys/LICENSE` (copy of root LICENSE)
- `sdif-rs/LICENSE` (copy of root LICENSE)
- `mat2sdif/LICENSE` (copy of root LICENSE)

Alternatively, you can use symbolic links on Unix systems:
```bash
cd sdif-sys && ln -s ../LICENSE LICENSE
cd ../sdif-rs && ln -s ../LICENSE LICENSE
cd ../mat2sdif && ln -s ../LICENSE LICENSE
```

However, **copying is preferred** for crates.io compatibility, as symlinks may not be followed during packaging.

---

## Task 3: Update Workspace README

**File:** `README.md` (workspace root)

Add or update the License section near the bottom of the README. Replace any existing license section with:

```markdown
## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### Bundled SDIF C Library

The optional bundled SDIF C library (`sdif-sys/sdif/`) is developed by IRCAM and licensed under **LGPL-2.0**. See `sdif-sys/sdif/COPYING` for the full license text.

When using the `bundled` feature (which statically links the SDIF library), the LGPL-2.0 requirements apply to the SDIF library portion:

- You must provide a way for users to relink with a modified SDIF library
- This is satisfied by providing your source code, or object files sufficient for relinking
- You must include the LGPL-2.0 license text and attribution

When using a system-installed SDIF library (dynamic linking), your application is considered a "work that uses the library" and is not subject to LGPL copyleft requirements.

**SDIF Library Source:** https://sourceforge.net/projects/sdif/
```

---

## Task 4: Update SPDX Headers in Source Files (Optional but Recommended)

Add SPDX license identifiers to the top of each Rust source file. This is a best practice for license clarity.

### Format

Add this comment as the first line of each `.rs` file:

```rust
// SPDX-License-Identifier: MIT
```

### Files to Update

**sdif-sys:**
- `sdif-sys/src/lib.rs`
- `sdif-sys/build.rs`

**sdif-rs:**
- `sdif-rs/src/lib.rs`
- `sdif-rs/src/error.rs`
- `sdif-rs/src/init.rs`
- `sdif-rs/src/signature.rs`
- `sdif-rs/src/data_type.rs`
- `sdif-rs/src/file.rs`
- `sdif-rs/src/frame.rs`
- `sdif-rs/src/matrix.rs`
- `sdif-rs/src/builder.rs`
- `sdif-rs/src/writer.rs`
- `sdif-rs/src/frame_builder.rs`
- `sdif-rs/src/mat/mod.rs` (if exists)
- `sdif-rs/src/mat/file.rs` (if exists)
- `sdif-rs/src/mat/data.rs` (if exists)
- `sdif-rs/src/mat/time.rs` (if exists)
- `sdif-rs/src/mat/convert.rs` (if exists)
- `sdif-rs/src/mat/complex.rs` (if exists)

**mat2sdif:**
- `mat2sdif/src/main.rs`
- `mat2sdif/src/cli.rs`
- `mat2sdif/src/commands/mod.rs`
- `mat2sdif/src/commands/list.rs`
- `mat2sdif/src/commands/convert.rs`
- `mat2sdif/src/commands/validate.rs`
- `mat2sdif/src/max_compat.rs`
- `mat2sdif/src/output.rs`

**Examples:**
- `sdif-rs/examples/read_sdif.rs`
- `sdif-rs/examples/write_sdif.rs`
- `sdif-rs/examples/inspect_sdif.rs`
- `sdif-rs/examples/mat_convert.rs`
- `sdif-rs/examples/streaming.rs`

### Automation Script

To add SPDX headers to all Rust files that don't already have them:

```bash
#!/bin/bash
# add-spdx-headers.sh

HEADER="// SPDX-License-Identifier: MIT"

find . -name "*.rs" -type f | while read file; do
    # Skip files that already have SPDX header
    if ! grep -q "SPDX-License-Identifier" "$file"; then
        # Create temp file with header + original content
        echo "$HEADER" | cat - "$file" > temp && mv temp "$file"
        echo "Added SPDX header to: $file"
    fi
done
```

---

## Task 5: Remove Any Apache-2.0 References

Search for and remove any remaining Apache-2.0 references:

```bash
# Find any remaining Apache references
grep -r "Apache" --include="*.toml" --include="*.md" --include="*.rs" .
grep -r "apache" --include="*.toml" --include="*.md" --include="*.rs" .
grep -r "ASL" --include="*.toml" --include="*.md" --include="*.rs" .
```

Common places to check:
- `Cargo.toml` files (license field)
- `README.md` files (license badges, license section)
- Source file headers
- `CONTRIBUTING.md`
- Any documentation in `docs/`

---

## Task 6: Update License Badge in README (If Present)

If the README has a license badge, update it:

**Old (Apache-2.0):**
```markdown
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
```

**New (MIT):**
```markdown
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
```

---

## Verification Checklist

After completing all tasks, verify:

- [ ] `Cargo.toml` (workspace) has `license = "MIT"`
- [ ] `sdif-sys/Cargo.toml` has MIT license (inherited or explicit)
- [ ] `sdif-rs/Cargo.toml` has MIT license (inherited or explicit)
- [ ] `mat2sdif/Cargo.toml` has MIT license (inherited or explicit)
- [ ] `LICENSE` file exists at workspace root with MIT text
- [ ] `sdif-sys/LICENSE` exists (copy of root LICENSE)
- [ ] `sdif-rs/LICENSE` exists (copy of root LICENSE)
- [ ] `mat2sdif/LICENSE` exists (copy of root LICENSE)
- [ ] `README.md` has updated License section with SDIF/LGPL notice
- [ ] No remaining Apache-2.0 references in the codebase
- [ ] `cargo package --list` shows LICENSE in each crate (test with `cargo package -p sdif-sys --list`)

### Test Commands

```bash
# Verify Cargo.toml license fields
grep -A1 "\[package\]" */Cargo.toml | grep license
grep "license" Cargo.toml

# Verify LICENSE files exist
ls -la LICENSE */LICENSE

# Search for any remaining Apache references
grep -ri "apache" --include="*.toml" --include="*.md" .

# Test packaging (dry run)
cargo package -p sdif-sys --allow-dirty
cargo package -p sdif-rs --allow-dirty
cargo package -p mat2sdif --allow-dirty
```

---

## Summary of Changes

| File | Change |
|------|--------|
| `Cargo.toml` | `license = "MIT"` |
| `sdif-sys/Cargo.toml` | `license.workspace = true` or `license = "MIT"` |
| `sdif-rs/Cargo.toml` | `license.workspace = true` or `license = "MIT"` |
| `mat2sdif/Cargo.toml` | `license.workspace = true` or `license = "MIT"` |
| `LICENSE` | Create with MIT text |
| `sdif-sys/LICENSE` | Copy of root LICENSE |
| `sdif-rs/LICENSE` | Copy of root LICENSE |
| `mat2sdif/LICENSE` | Copy of root LICENSE |
| `README.md` | Update License section |
| `*.rs` files | Add SPDX header (optional) |

---

## Notes

1. **Copyright holder:** Replace `[Author Name or Organization]` in the LICENSE file with the actual name.

2. **Year:** The copyright year should be the year of first publication. Use `2024` or the current year.

3. **SDIF library license:** The LGPL-2.0 license for the bundled SDIF C library remains unchanged. The `sdif-sys/sdif/COPYING` file contains that license and should NOT be modified.

4. **Dual licensing:** If you want to offer dual licensing (MIT OR Apache-2.0), use `license = "MIT OR Apache-2.0"` in Cargo.toml and include both LICENSE-MIT and LICENSE-APACHE files. This is common but adds complexity.
