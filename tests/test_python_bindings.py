import importlib.util
import os
import platform
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
TESTDATA_DIR = PROJECT_ROOT / "testdata"
TEST_M4A = TESTDATA_DIR / "test.m4a"
TEST_MP3 = TESTDATA_DIR / "test.mp3"


def _find_module():
    env = os.environ.get("MP3RGAINPY_PYTHON_MODULE")
    if env:
        return Path(env)

    system = platform.system()
    if system == "Linux":
        extensions = [".so"]
        prefixes = ["libmp3rgainpy", "mp3rgainpy"]
    elif system == "Darwin":
        extensions = [".dylib", ".so"]
        prefixes = ["libmp3rgainpy", "mp3rgainpy"]
    elif system == "Windows":
        extensions = [".pyd", ".dll"]
        prefixes = ["mp3rgainpy"]
    else:
        extensions = [".so"]
        prefixes = ["libmp3rgainpy", "mp3rgainpy"]

    for build_type in ["debug", "release"]:
        for prefix in prefixes:
            for ext in extensions:
                candidate = PROJECT_ROOT / "target" / build_type / f"{prefix}{ext}"
                if candidate.exists():
                    return candidate

    return None


def _load_module():
    path = _find_module()
    if path is None:
        raise RuntimeError(
            "Could not find mp3rgainpy shared library. "
            "Run `cargo build --features python --lib` first, "
            "or set MP3RGAINPY_PYTHON_MODULE to the path."
        )
    spec = importlib.util.spec_from_file_location("mp3rgainpy", str(path))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


mp3rgainpy = _load_module()


class AacBindingsTest(unittest.TestCase):
    def test_gain_step_db_constant(self):
        self.assertEqual(mp3rgainpy.GAIN_STEP_DB, 1.5)

    def test_is_mp4_file(self):
        self.assertTrue(mp3rgainpy.is_mp4_file(str(TEST_M4A)))

    def test_is_aac_file(self):
        self.assertTrue(mp3rgainpy.is_aac_file(str(TEST_M4A)))

    def test_is_mp4_file_negative(self):
        self.assertFalse(mp3rgainpy.is_mp4_file(str(PROJECT_ROOT / ".gitignore")))

    def test_aac_analyze_returns_gain_analysis(self):
        analysis = mp3rgainpy.aac_analyze(str(TEST_M4A))
        self.assertIsInstance(analysis.sample_count, int)
        self.assertIsInstance(analysis.channel_count, int)
        self.assertIsInstance(analysis.min_gain, int)
        self.assertIsInstance(analysis.max_gain, int)
        self.assertIsInstance(analysis.sample_rate, int)
        self.assertIsInstance(analysis.parse_warnings, int)
        self.assertGreater(analysis.sample_count, 0)
        self.assertGreater(analysis.channel_count, 0)
        self.assertEqual(analysis.sample_rate, 44100)

    def test_aac_analyze_gain_locations_have_attributes(self):
        analysis = mp3rgainpy.aac_analyze(str(TEST_M4A))
        locations = analysis.gain_locations
        self.assertGreater(len(locations), 0)
        loc = locations[0]
        self.assertIsInstance(loc.sample_index, int)
        self.assertIsInstance(loc.file_offset, int)
        self.assertIsInstance(loc.sample_byte_offset, int)
        self.assertIsInstance(loc.bit_offset, int)
        self.assertIsInstance(loc.channel, int)
        self.assertIsInstance(loc.original_gain, int)

    def test_aac_apply_gain_and_undo(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_m4a = Path(tmpdir) / "test.m4a"
            shutil.copy2(TEST_M4A, tmp_m4a)

            original = mp3rgainpy.aac_analyze(str(tmp_m4a))
            original_gains = [
                loc.original_gain for loc in original.gain_locations
            ]

            modified_count = mp3rgainpy.aac_apply_gain_with_undo(str(tmp_m4a), 2)
            self.assertGreater(modified_count, 0)

            after_apply = mp3rgainpy.aac_analyze(str(tmp_m4a))
            after_gains = [
                loc.original_gain for loc in after_apply.gain_locations
            ]
            for orig, after in zip(original_gains, after_gains):
                self.assertEqual(after, orig + 2)

            undo_count = mp3rgainpy.aac_undo_gain(str(tmp_m4a))
            self.assertGreater(undo_count, 0)

            restored = mp3rgainpy.aac_analyze(str(tmp_m4a))
            restored_gains = [
                loc.original_gain for loc in restored.gain_locations
            ]
            self.assertEqual(original_gains, restored_gains)

    def test_aac_apply_gain_zero_is_noop(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_m4a = Path(tmpdir) / "test.m4a"
            shutil.copy2(TEST_M4A, tmp_m4a)
            count = mp3rgainpy.aac_apply_gain(str(tmp_m4a), 0)
            self.assertEqual(count, 0)

    def test_aac_analyze_nonexistent_file_raises(self):
        with self.assertRaises(RuntimeError):
            mp3rgainpy.aac_analyze("/nonexistent/path/file.m4a")

    def test_aac_analyze_non_m4a_file_raises(self):
        with self.assertRaises(RuntimeError):
            mp3rgainpy.aac_analyze(str(PROJECT_ROOT / ".gitignore"))


class Mp3BindingsTest(unittest.TestCase):
    def test_mp3_analyze_returns_analysis(self):
        analysis = mp3rgainpy.mp3_analyze(str(TEST_MP3))
        self.assertIsInstance(analysis.frame_count, int)
        self.assertIsInstance(analysis.mpeg_version, str)
        self.assertIsInstance(analysis.channel_mode, str)
        self.assertIsInstance(analysis.channel_count, int)
        self.assertIsInstance(analysis.min_gain, int)
        self.assertIsInstance(analysis.max_gain, int)
        self.assertIsInstance(analysis.avg_gain, float)
        self.assertIsInstance(analysis.headroom_steps, int)
        self.assertIsInstance(analysis.headroom_db, float)
        self.assertGreater(analysis.frame_count, 0)
        self.assertEqual(analysis.mpeg_version, "MPEG1")
        self.assertEqual(analysis.channel_count, 2)
        self.assertLessEqual(analysis.min_gain, analysis.max_gain)

    def test_mp3_apply_gain_zero_is_noop(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_mp3 = Path(tmpdir) / "test.mp3"
            shutil.copy2(TEST_MP3, tmp_mp3)
            count = mp3rgainpy.mp3_apply_gain(str(tmp_mp3), 0)
            self.assertEqual(count, 0)

    def test_mp3_apply_gain_and_undo(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_mp3 = Path(tmpdir) / "test.mp3"
            shutil.copy2(TEST_MP3, tmp_mp3)

            original = mp3rgainpy.mp3_analyze(str(tmp_mp3))
            orig_min, orig_max = original.min_gain, original.max_gain

            modified = mp3rgainpy.mp3_apply_gain_with_undo(str(tmp_mp3), 2)
            self.assertGreater(modified, 0)

            after = mp3rgainpy.mp3_analyze(str(tmp_mp3))
            self.assertEqual(after.min_gain, orig_min + 2)
            self.assertEqual(after.max_gain, orig_max + 2)

            undone = mp3rgainpy.mp3_undo_gain(str(tmp_mp3))
            self.assertGreater(undone, 0)

            restored = mp3rgainpy.mp3_analyze(str(tmp_mp3))
            self.assertEqual(restored.min_gain, orig_min)
            self.assertEqual(restored.max_gain, orig_max)

    def test_mp3_analyze_non_mp3_file_raises(self):
        with self.assertRaises(RuntimeError):
            mp3rgainpy.mp3_analyze(str(PROJECT_ROOT / ".gitignore"))


if __name__ == "__main__":
    unittest.main()
