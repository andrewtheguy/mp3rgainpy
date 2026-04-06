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


def _find_module():
    env = os.environ.get("M4AGAIN_PYTHON_MODULE")
    if env:
        return Path(env)

    system = platform.system()
    if system == "Linux":
        extensions = [".so"]
        prefixes = ["libm4again", "m4again"]
    elif system == "Darwin":
        extensions = [".dylib", ".so"]
        prefixes = ["libm4again", "m4again"]
    elif system == "Windows":
        extensions = [".pyd", ".dll"]
        prefixes = ["m4again"]
    else:
        extensions = [".so"]
        prefixes = ["libm4again", "m4again"]

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
            "Could not find m4again shared library. "
            "Run `cargo build --features python --lib` first, "
            "or set M4AGAIN_PYTHON_MODULE to the path."
        )
    spec = importlib.util.spec_from_file_location("m4again", str(path))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


m4again = _load_module()


class PythonBindingsTest(unittest.TestCase):
    def test_gain_step_db_constant(self):
        self.assertEqual(m4again.GAIN_STEP_DB, 1.5)

    def test_is_mp4_file(self):
        self.assertTrue(m4again.is_mp4_file(str(TEST_M4A)))

    def test_is_aac_file(self):
        self.assertTrue(m4again.is_aac_file(str(TEST_M4A)))

    def test_is_mp4_file_negative(self):
        self.assertFalse(m4again.is_mp4_file(str(PROJECT_ROOT / ".gitignore")))

    def test_analyze_returns_gain_analysis(self):
        analysis = m4again.analyze(str(TEST_M4A))
        self.assertIsInstance(analysis.sample_count, int)
        self.assertIsInstance(analysis.channel_count, int)
        self.assertIsInstance(analysis.min_gain, int)
        self.assertIsInstance(analysis.max_gain, int)
        self.assertIsInstance(analysis.sample_rate, int)
        self.assertIsInstance(analysis.parse_warnings, int)
        self.assertGreater(analysis.sample_count, 0)
        self.assertGreater(analysis.channel_count, 0)
        self.assertEqual(analysis.sample_rate, 44100)

    def test_analyze_gain_locations_have_attributes(self):
        analysis = m4again.analyze(str(TEST_M4A))
        locations = analysis.gain_locations
        self.assertGreater(len(locations), 0)
        loc = locations[0]
        self.assertIsInstance(loc.sample_index, int)
        self.assertIsInstance(loc.file_offset, int)
        self.assertIsInstance(loc.sample_byte_offset, int)
        self.assertIsInstance(loc.bit_offset, int)
        self.assertIsInstance(loc.channel, int)
        self.assertIsInstance(loc.original_gain, int)

    def test_apply_gain_and_undo(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_m4a = Path(tmpdir) / "test.m4a"
            shutil.copy2(TEST_M4A, tmp_m4a)

            original = m4again.analyze(str(tmp_m4a))
            original_gains = [
                loc.original_gain for loc in original.gain_locations
            ]

            modified_count = m4again.apply_gain_with_undo(str(tmp_m4a), 2)
            self.assertGreater(modified_count, 0)

            after_apply = m4again.analyze(str(tmp_m4a))
            after_gains = [
                loc.original_gain for loc in after_apply.gain_locations
            ]
            for orig, after in zip(original_gains, after_gains):
                self.assertEqual(after, orig + 2)

            undo_count = m4again.undo_gain(str(tmp_m4a))
            self.assertGreater(undo_count, 0)

            restored = m4again.analyze(str(tmp_m4a))
            restored_gains = [
                loc.original_gain for loc in restored.gain_locations
            ]
            self.assertEqual(original_gains, restored_gains)

    def test_apply_gain_zero_is_noop(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_m4a = Path(tmpdir) / "test.m4a"
            shutil.copy2(TEST_M4A, tmp_m4a)
            count = m4again.apply_gain(str(tmp_m4a), 0)
            self.assertEqual(count, 0)

    def test_nonexistent_file_raises(self):
        with self.assertRaises(RuntimeError):
            m4again.analyze("/nonexistent/path/file.m4a")

    def test_non_m4a_file_raises(self):
        with self.assertRaises(RuntimeError):
            m4again.analyze(str(PROJECT_ROOT / ".gitignore"))


if __name__ == "__main__":
    unittest.main()
