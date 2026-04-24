# mp3rgainpy

Python bindings for MP3 and AAC/M4A gain adjustment, powered by the [mp3rgain](https://crates.io/crates/mp3rgain) Rust crate via PyO3.

## Installation

Install from the GitHub Pages package index:

```bash
pip install mp3rgainpy --extra-index-url https://andrewtheguy.github.io/mp3rgainpy/simple/
```

## Usage

```python
import mp3rgainpy

# --- AAC / M4A ---

mp3rgainpy.is_mp4_file("track.m4a")   # True/False
mp3rgainpy.is_aac_file("track.m4a")   # True/False

analysis = mp3rgainpy.aac_analyze("track.m4a")
print(f"Samples: {analysis.sample_count}")
print(f"Channels: {analysis.channel_count}")
print(f"Sample rate: {analysis.sample_rate}")
print(f"Gain range: {analysis.min_gain} - {analysis.max_gain}")

for loc in analysis.gain_locations:
    print(f"  ch={loc.channel} gain={loc.original_gain} @ offset {loc.file_offset}")

mp3rgainpy.aac_apply_gain("track.m4a", gain_steps=3)       # +4.5 dB
mp3rgainpy.aac_apply_gain("track.m4a", gain_steps=-2)      # -3.0 dB

mp3rgainpy.aac_apply_gain_with_undo("track.m4a", gain_steps=5)
mp3rgainpy.aac_undo_gain("track.m4a")  # restores original gains

# --- MP3 ---

mp3 = mp3rgainpy.mp3_analyze("song.mp3")
print(f"{mp3.mpeg_version} {mp3.channel_mode}, {mp3.frame_count} frames")
print(f"Gain range: {mp3.min_gain} - {mp3.max_gain} (avg {mp3.avg_gain:.1f})")
print(f"Headroom: {mp3.headroom_db:+.1f} dB")

mp3rgainpy.mp3_apply_gain("song.mp3", gain_steps=2)        # +3.0 dB
mp3rgainpy.mp3_apply_gain_with_undo("song.mp3", gain_steps=4)
mp3rgainpy.mp3_undo_gain("song.mp3")   # reverts using APEv2 undo tag

# Gain step constant (shared by both)
print(mp3rgainpy.GAIN_STEP_DB)  # 1.5
```

## Gain Steps

Each gain step corresponds to **1.5 dB** (`mp3rgainpy.GAIN_STEP_DB`). Positive values increase volume, negative values decrease it.

## API

### AAC / M4A

| Function | Description |
|---|---|
| `aac_apply_gain(path, steps)` | Apply gain adjustment |
| `aac_apply_gain_with_undo(path, steps)` | Apply gain with undo metadata |
| `aac_undo_gain(path)` | Restore original gains |
| `aac_analyze(path)` | Analyze gain locations (returns `AacGainAnalysis`) |
| `is_mp4_file(path)` | Check if file is MP4 |
| `is_aac_file(path)` | Check if file is AAC |

### MP3

| Function | Description |
|---|---|
| `mp3_apply_gain(path, steps)` | Apply gain adjustment (saturating) |
| `mp3_apply_gain_with_undo(path, steps)` | Apply gain and record APEv2 undo tag |
| `mp3_undo_gain(path)` | Revert gain using the APEv2 undo tag |
| `mp3_analyze(path)` | Analyze frame gain stats (returns `Mp3Analysis`) |

## Development

The Rust crate lives in `native-helper/`; the Python package lives in
`mp3rgainpy/`. Maturin builds the native extension as
`mp3rgainpy/_native.<abi>.so` directly in the source tree, so iterating on
the Rust code only requires rebuilding the `.so` — no wheel reinstall.

```bash
# One-time setup
python -m venv .venv
source .venv/bin/activate
pip install --upgrade pip maturin

# After each Rust change (rebuilds _native.so in place, skips wheel install)
maturin develop --skip-install --manifest-path native-helper/Cargo.toml

# Run tests
python -m unittest tests.test_python_bindings

# Rust-only tests
cargo test --manifest-path native-helper/Cargo.toml
```

Version bumps happen in `pyproject.toml`; `native-helper/Cargo.toml` stays
pinned at `0.0.0` because the crate is never published independently.
