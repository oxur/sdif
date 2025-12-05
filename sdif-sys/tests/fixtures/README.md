# Test Fixtures

This directory contains SDIF test files for integration testing.

## Required Files

For full test coverage, add the following files:

1. `simple.sdif` - A minimal SDIF file with:
   - One 1TRC frame
   - One matrix with a few rows
   - Basic NVT entries

2. `multiframe.sdif` - An SDIF file with:
   - Multiple frames at different times
   - Multiple matrices per frame

3. `all_types.sdif` - An SDIF file demonstrating:
   - 1TRC, 1HRM, 1FQ0 frame types
   - Float32 and Float64 data
   - Complex NVT data

## Creating Test Files

Test files can be created using:
- SPEAR (spectral analysis application)
- AudioSculpt
- pysdif3 (Python SDIF library)
- The sdif-rs write API (once implemented)

## Example: Creating with pysdif3

```python
import pysdif3

with pysdif3.SdifFile('simple.sdif', 'w') as f:
    f.add_NVT({'creator': 'test', 'date': '2024-01-01'})
    f.add_frame_type('1TRC', '1TRC SinusoidalTracks')
    f.add_matrix_type('1TRC', 'Index, Frequency, Amplitude, Phase')

    f.new_frame('1TRC', 0.0, 1)
    f.add_matrix('1TRC', [[1, 440.0, 0.5, 0.0], [2, 880.0, 0.3, 0.0]])
```
