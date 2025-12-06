# Test Fixtures for mat2sdif

This directory should contain MAT files for integration testing.

## Required Files

### simple.mat
A basic MAT file with:
- `time` - 1D array of 100 time values (0.0 to 1.0)
- `partials` - 2D array [100, 4] with Index, Freq, Amp, Phase columns

### complex.mat
A MAT file with complex data:
- `time` - 1D array of time values
- `spectrum` - 2D complex array

### f0.mat
A pitch tracking result:
- `time` - 1D array of time values
- `f0` - 2D array [N, 2] with Frequency, Confidence columns

## Creating Test Files

See sdif-rs/tests/fixtures/README.md for instructions on creating
MAT files using MATLAB, Octave, or Python.

### Quick Python Script

```python
import numpy as np
from scipy.io import savemat

# simple.mat
time = np.arange(0, 1.0, 0.01)  # 100 time points
partials = np.zeros((100, 4))
for i in range(100):
    partials[i] = [1, 440 + i*5, 0.5 * np.exp(-i/30), i * 0.1]
savemat('simple.mat', {'time': time, 'partials': partials})

# f0.mat
f0_data = np.column_stack([
    220 + 10 * np.sin(np.linspace(0, 4*np.pi, 100)),  # Frequency
    0.9 + 0.1 * np.random.rand(100)  # Confidence
])
savemat('f0.mat', {'time': time, 'f0': f0_data})
```
