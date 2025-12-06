# Test Fixtures for sdif-rs

This directory should contain SDIF test files for integration testing.

## Required Files

1. **simple.sdif** - Minimal SDIF file with:
   - At least one 1TRC frame
   - At least one matrix with a few rows
   - Basic NVT entries (creator, date)

2. **multiframe.sdif** - SDIF file with:
   - Multiple frames at different times
   - Multiple matrices per frame

3. **all_types.sdif** - SDIF file demonstrating:
   - Various frame types (1TRC, 1HRM, 1FQ0)
   - Both Float32 and Float64 data

## Creating Test Files

### Using Python (pysdif3)

```python
import pysdif3
import numpy as np

# Create simple.sdif
with pysdif3.SdifFile('simple.sdif', 'w') as f:
    # Add metadata
    f.add_NVT({'creator': 'sdif-rs-test', 'date': '2024-01-01'})

    # Define types
    f.add_frame_type('1TRC', '1TRC SinusoidalTracks')
    f.add_matrix_type('1TRC', 'Index, Frequency, Amplitude, Phase')

    # Write a frame
    data = np.array([
        [1, 440.0, 0.5, 0.0],
        [2, 880.0, 0.3, 1.57],
        [3, 1320.0, 0.2, 3.14],
    ])

    f.new_frame('1TRC', 0.0, 1)
    f.add_matrix('1TRC', data)

    f.new_frame('1TRC', 0.1, 1)
    f.add_matrix('1TRC', data * 0.9)

print("Created simple.sdif")
```

### Using SPEAR

1. Open an audio file in SPEAR
2. Perform analysis
3. Export as SDIF

## Running Tests with Fixtures

Once fixtures are in place:

```bash
# Run all tests including those requiring fixtures
cargo test -- --include-ignored

# Run only fixture-dependent tests
cargo test --test integration -- --include-ignored
```
