# m4againpy

Python bindings for AAC/M4A gain adjustment, powered by the [mp3rgain](https://crates.io/crates/mp3rgain) Rust crate via PyO3.

## Installation

Install from the GitHub Pages package index:

```bash
pip install m4againpy --extra-index-url https://andrewtheguy.github.io/m4againpy/simple/
```

## Usage

```python
import m4again

# Check file type
m4again.is_mp4_file("track.m4a")   # True/False
m4again.is_aac_file("track.m4a")   # True/False

# Analyze gain locations
analysis = m4again.analyze("track.m4a")
print(f"Samples: {analysis.sample_count}")
print(f"Channels: {analysis.channel_count}")
print(f"Sample rate: {analysis.sample_rate}")
print(f"Gain range: {analysis.min_gain} - {analysis.max_gain}")

for loc in analysis.gain_locations:
    print(f"  ch={loc.channel} gain={loc.original_gain} @ offset {loc.file_offset}")

# Apply gain (1 step = 1.5 dB)
m4again.apply_gain("track.m4a", gain_steps=3)       # +4.5 dB
m4again.apply_gain("track.m4a", gain_steps=-2)      # -3.0 dB

# Apply gain with undo support
m4again.apply_gain_with_undo("track.m4a", gain_steps=5)
m4again.undo_gain("track.m4a")  # restores original gains

# Gain step constant
print(m4again.GAIN_STEP_DB)  # 1.5
```

## Gain Steps

Each gain step corresponds to **1.5 dB** (`m4again.GAIN_STEP_DB`). Positive values increase volume, negative values decrease it.

## API

| Function | Description |
|---|---|
| `apply_gain(path, steps)` | Apply gain adjustment |
| `apply_gain_with_undo(path, steps)` | Apply gain with undo metadata |
| `undo_gain(path)` | Restore original gains |
| `analyze(path)` | Analyze gain locations |
| `is_mp4_file(path)` | Check if file is MP4 |
| `is_aac_file(path)` | Check if file is AAC |
