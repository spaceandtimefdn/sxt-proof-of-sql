import os
import shutil
import stat
import subprocess
import tempfile
import time
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PRE_FORGE_SCRIPT = REPO_ROOT / "solidity" / "scripts" / "pre_forge.sh"
YUL_PREPROCESSOR = REPO_ROOT / "solidity" / "preprocessor" / "yul_preprocessor.py"


def make_solidity_fixture(tmp_path: Path) -> Path:
    solidity_dir = tmp_path / "solidity"
    (solidity_dir / "scripts").mkdir(parents=True)
    (solidity_dir / "preprocessor").mkdir()
    (solidity_dir / "src").mkdir()

    shutil.copy(PRE_FORGE_SCRIPT, solidity_dir / "scripts" / "pre_forge.sh")
    shutil.copy(YUL_PREPROCESSOR, solidity_dir / "preprocessor" / "yul_preprocessor.py")

    (solidity_dir / "src" / "Main.presl").write_text(
        """contract Main {
    function answer() external pure returns (uint256) {
        assembly {
            function helper() -> result {
                result := 42
            }

            let value := helper()
            mstore(0x00, value)
            return(0x00, 0x20)
        }
    }
}
""",
        encoding="utf-8",
    )
    return solidity_dir


def make_fake_forge(tmp_path: Path) -> Path:
    forge_path = tmp_path / "forge"
    forge_path.write_text(
        """#!/usr/bin/env bash
set -euo pipefail
printf '%s\\n' "$*" >> "${FORGE_LOG}"
""",
        encoding="utf-8",
    )
    forge_path.chmod(forge_path.stat().st_mode | stat.S_IXUSR)
    return forge_path


def run_pre_forge(
    solidity_dir: Path, forge_log: Path, *args: str
) -> subprocess.CompletedProcess:
    env = os.environ.copy()
    env["FORGE_LOG"] = str(forge_log)
    env["PATH"] = f"{forge_log.parent}:{env['PATH']}"
    return subprocess.run(
        [str(solidity_dir / "scripts" / "pre_forge.sh"), *args],
        cwd=solidity_dir,
        capture_output=True,
        text=True,
        check=False,
        env=env,
    )


class PreForgeScriptTests(unittest.TestCase):
    def test_pre_forge_skips_regeneration_when_inputs_are_unchanged(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_path = Path(tmpdir)
            solidity_dir = make_solidity_fixture(tmp_path)
            forge_log = tmp_path / "forge.log"
            make_fake_forge(tmp_path)

            first = run_pre_forge(solidity_dir, forge_log, "test", "--summary")
            self.assertEqual(first.returncode, 0)

            output_file = solidity_dir / "src" / "Main.post.sol"
            manifest_file = solidity_dir / ".pre_forge_inputs.sha256"
            self.assertTrue(output_file.exists())
            self.assertTrue(manifest_file.exists())
            first_mtime = output_file.stat().st_mtime_ns

            second = run_pre_forge(solidity_dir, forge_log, "test", "--summary")
            self.assertEqual(second.returncode, 0)
            self.assertIn("Using cached .post.sol files", second.stdout)
            self.assertEqual(output_file.stat().st_mtime_ns, first_mtime)

            self.assertEqual(
                forge_log.read_text(encoding="utf-8").splitlines(),
                ["fmt .", "test --summary", "test --summary"],
            )

    def test_pre_forge_regenerates_outputs_when_inputs_change(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_path = Path(tmpdir)
            solidity_dir = make_solidity_fixture(tmp_path)
            forge_log = tmp_path / "forge.log"
            make_fake_forge(tmp_path)

            first = run_pre_forge(solidity_dir, forge_log, "test", "--summary")
            self.assertEqual(first.returncode, 0)

            input_file = solidity_dir / "src" / "Main.presl"
            output_file = solidity_dir / "src" / "Main.post.sol"
            first_mtime = output_file.stat().st_mtime_ns

            time.sleep(0.01)
            input_file.write_text(
                input_file.read_text(encoding="utf-8").replace("42", "43"),
                encoding="utf-8",
            )

            second = run_pre_forge(solidity_dir, forge_log, "test", "--summary")
            self.assertEqual(second.returncode, 0)
            self.assertGreater(output_file.stat().st_mtime_ns, first_mtime)
            self.assertIn("43", output_file.read_text(encoding="utf-8"))

            self.assertEqual(
                forge_log.read_text(encoding="utf-8").splitlines(),
                ["fmt .", "test --summary", "fmt .", "test --summary"],
            )

    def test_pre_forge_clean_clears_generated_outputs_without_preprocessing(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_path = Path(tmpdir)
            solidity_dir = make_solidity_fixture(tmp_path)
            forge_log = tmp_path / "forge.log"
            make_fake_forge(tmp_path)

            first = run_pre_forge(solidity_dir, forge_log, "test", "--summary")
            self.assertEqual(first.returncode, 0)

            clean = run_pre_forge(solidity_dir, forge_log, "clean")
            self.assertEqual(clean.returncode, 0)
            self.assertFalse((solidity_dir / "src" / "Main.post.sol").exists())
            self.assertFalse((solidity_dir / ".pre_forge_inputs.sha256").exists())

            self.assertEqual(
                forge_log.read_text(encoding="utf-8").splitlines(),
                ["fmt .", "test --summary", "clean"],
            )


if __name__ == "__main__":
    unittest.main()
