from pathlib import Path
import os
import shutil
import subprocess
import textwrap
import time


SOLIDITY_DIR = Path(__file__).resolve().parent.parent
CACHE_FILE = ".pre_forge_cache/input_fingerprint.sha256"
GENERATED_FILE = "src/Example.post.sol"


def _write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(textwrap.dedent(content).lstrip(), encoding="utf-8")


def _make_executable(path: Path) -> None:
    path.chmod(path.stat().st_mode | 0o111)


def _create_workspace(tmp_path: Path) -> tuple[Path, dict[str, str]]:
    workspace = tmp_path / "solidity"
    (workspace / "scripts").mkdir(parents=True, exist_ok=True)
    (workspace / "preprocessor").mkdir(parents=True, exist_ok=True)
    shutil.copy(SOLIDITY_DIR / "scripts" / "pre_forge.sh", workspace / "scripts" / "pre_forge.sh")
    shutil.copy(
        SOLIDITY_DIR / "preprocessor" / "yul_preprocessor.py",
        workspace / "preprocessor" / "yul_preprocessor.py",
    )
    _write(
        workspace / "src" / "Example.presl",
        """
        contract Example {
            function value() external pure returns (uint256 out) {
                assembly {
                    function add1(x) -> result {
                        result := add(x, 1)
                    }

                    out := add1(41)
                }
            }
        }
        """,
    )

    fake_bin = tmp_path / "bin"
    forge = fake_bin / "forge"
    _write(
        forge,
        """
        #!/usr/bin/env bash
        set -euo pipefail
        if [ "${1:-}" != "fmt" ]; then
            printf '%s\n' "$*" >> "${FAKE_FORGE_LOG:?}"
        fi
        """,
    )
    _make_executable(forge)

    env = os.environ.copy()
    env["FAKE_FORGE_LOG"] = str(tmp_path / "forge.log")
    env["PATH"] = f"{fake_bin}:{env['PATH']}"
    return workspace, env


def _run_preforge(workspace: Path, env: dict[str, str], *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["bash", "scripts/pre_forge.sh", *args],
        cwd=workspace,
        env=env,
        check=True,
        capture_output=True,
        text=True,
    )


def _forge_log_lines(env: dict[str, str]) -> list[str]:
    log_file = Path(env["FAKE_FORGE_LOG"])
    if not log_file.exists():
        return []
    return [line for line in log_file.read_text(encoding="utf-8").splitlines() if line]


def test_pre_forge_uses_cache_when_inputs_are_unchanged(tmp_path: Path) -> None:
    workspace, env = _create_workspace(tmp_path)

    _run_preforge(workspace, env, "test")
    generated = workspace / GENERATED_FILE
    assert generated.exists()

    initial_mtime = generated.stat().st_mtime_ns
    time.sleep(0.02)

    second_run = _run_preforge(workspace, env, "test")

    assert "cache hit" in second_run.stdout.lower()
    assert generated.stat().st_mtime_ns == initial_mtime
    assert _forge_log_lines(env) == ["test", "test"]


def test_pre_forge_invalidates_cache_when_presl_inputs_change(tmp_path: Path) -> None:
    workspace, env = _create_workspace(tmp_path)
    _run_preforge(workspace, env, "test")

    source_file = workspace / "src" / "Example.presl"
    source_file.write_text(source_file.read_text(encoding="utf-8").replace("add1(41)", "add1(42)"), encoding="utf-8")
    time.sleep(0.02)

    rerun = _run_preforge(workspace, env, "test")
    generated = workspace / GENERATED_FILE

    assert "cache hit" not in rerun.stdout.lower()
    assert "add1(42)" in generated.read_text(encoding="utf-8")
    assert _forge_log_lines(env) == ["test", "test"]


def test_pre_forge_clean_removes_generated_files_without_reprocessing(tmp_path: Path) -> None:
    workspace, env = _create_workspace(tmp_path)
    _run_preforge(workspace, env, "test")

    generated = workspace / GENERATED_FILE
    cache_file = workspace / CACHE_FILE
    assert generated.exists()
    assert cache_file.exists()

    clean_run = _run_preforge(workspace, env, "clean")

    assert "cache hit" not in clean_run.stdout.lower()
    assert not generated.exists()
    assert not cache_file.exists()
    assert _forge_log_lines(env) == ["test", "clean"]
