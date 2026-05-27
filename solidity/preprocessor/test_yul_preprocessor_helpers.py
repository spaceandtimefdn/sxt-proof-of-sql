#!/usr/bin/env python3
"""Additional helper coverage tests for the Yul preprocessor."""

from pathlib import Path
import subprocess
import sys

import pytest
import yul_preprocessor
from yul_preprocessor import YulFunction, YulPreprocessor


def test_yul_function_identity_uses_signature():
    """Test YulFunction equality, hashing, and repr helpers."""
    first = YulFunction(
        name="add",
        signature="function add(a, b) -> result",
        body="result := add(a, b)",
        full_text="function add(a, b) -> result { result := add(a, b) }",
    )
    same_signature = YulFunction(
        name="addRenamed",
        signature="function add(a, b) -> result",
        body="result := add(a, b)",
        full_text="function addRenamed(a, b) -> result { result := add(a, b) }",
    )
    different_signature = YulFunction(
        name="add",
        signature="function add(a, b, c) -> result",
        body="result := add(add(a, b), c)",
        full_text="function add(a, b, c) -> result { result := add(add(a, b), c) }",
    )

    assert first == same_signature
    assert first != different_signature
    assert first != "not a YulFunction"
    assert len({first, same_signature, different_signature}) == 2
    assert repr(first) == "YulFunction(add, sig=function add(a, b) -> result)"


def test_find_yul_function_calls_only_returns_known_functions():
    """Test function-call discovery ignores Yul built-ins and unknown names."""
    preprocessor = YulPreprocessor()
    all_functions = {
        "helper": YulFunction(
            name="helper",
            signature="function helper(x) -> result",
            body="result := add(x, 1)",
            full_text="function helper(x) -> result { result := add(x, 1) }",
        ),
        "wrap": YulFunction(
            name="wrap",
            signature="function wrap(x) -> result",
            body="result := helper(x)",
            full_text="function wrap(x) -> result { result := helper(x) }",
        ),
    }

    called = preprocessor.find_yul_function_calls(
        "let x := add(1, 2)\nlet y := helper(x)\nlet z := missing(y)",
        all_functions,
    )

    assert called == {"helper"}


def test_get_function_dependencies_returns_recursive_closure():
    """Test recursive helper dependencies are collected exactly once."""
    preprocessor = YulPreprocessor()
    all_functions = {
        "entry": YulFunction(
            name="entry",
            signature="function entry(x) -> result",
            body="result := mid(x)",
            full_text="function entry(x) -> result { result := mid(x) }",
        ),
        "mid": YulFunction(
            name="mid",
            signature="function mid(x) -> result",
            body="result := leaf(x)",
            full_text="function mid(x) -> result { result := leaf(x) }",
        ),
        "leaf": YulFunction(
            name="leaf",
            signature="function leaf(x) -> result",
            body="result := add(x, 1)",
            full_text="function leaf(x) -> result { result := add(x, 1) }",
        ),
        "unused": YulFunction(
            name="unused",
            signature="function unused(x) -> result",
            body="result := mul(x, 2)",
            full_text="function unused(x) -> result { result := mul(x, 2) }",
        ),
    }

    dependencies = preprocessor.get_function_dependencies("entry", all_functions)

    assert set(dependencies) == {"entry", "mid", "leaf"}
    assert preprocessor.get_function_dependencies("missing", all_functions) == {}


def test_get_function_dependencies_handles_recursive_calls():
    """Test recursive dependency graphs do not loop forever."""
    preprocessor = YulPreprocessor()
    all_functions = {
        "first": YulFunction(
            name="first",
            signature="function first(x) -> result",
            body="result := second(x)",
            full_text="function first(x) -> result { result := second(x) }",
        ),
        "second": YulFunction(
            name="second",
            signature="function second(x) -> result",
            body="result := first(x)",
            full_text="function second(x) -> result { result := first(x) }",
        ),
    }

    dependencies = preprocessor.get_function_dependencies("first", all_functions)

    assert set(dependencies) == {"first", "second"}


def test_find_assembly_blocks_ignores_unclosed_blocks():
    """Test an unmatched assembly brace stops scanning without a partial block."""
    preprocessor = YulPreprocessor()

    blocks = preprocessor.find_assembly_blocks(
        """
        contract Broken {
            function run() external {
                assembly {
                    let x := 1
        """
    )

    assert blocks == []


def test_resolve_import_path_handles_root_relative_and_file_relative_imports(
    tmp_path,
):
    """Test import paths resolve from either the root dir or current file."""
    current_file = tmp_path / "contracts" / "main.presl"
    current_file.parent.mkdir()
    preprocessor = YulPreprocessor(root_dir=tmp_path)

    assert preprocessor.resolve_import_path("lib/helper.presl", current_file) == (
        current_file.parent / "lib" / "helper.presl"
    ).resolve()
    assert preprocessor.resolve_import_path("/lib/helper.presl", current_file) == (
        tmp_path / "lib" / "helper.presl"
    )


def test_collect_external_dependencies_skips_self_missing_and_unresolved_imports(
    monkeypatch, tmp_path
):
    """Test cycle dependency collection ignores paths it cannot resolve yet."""
    cycle_file = tmp_path / "cycle.presl"
    missing_cycle_file = tmp_path / "missing.presl"
    cycle_file.write_text(
        """
contract Cycle {
    function run() external {
        assembly {
            // import local from self
            // import missing from missing.presl
            function local() -> result {
                result := 1
            }
        }
    }
}
""",
        encoding="utf-8",
    )
    preprocessor = YulPreprocessor(root_dir=tmp_path)

    monkeypatch.setattr(
        preprocessor,
        "resolve_import",
        lambda *args, **kwargs: (_ for _ in ()).throw(ValueError("not yet")),
    )

    dependencies = preprocessor.collect_external_dependencies_for_cycle(
        {cycle_file, missing_cycle_file},
        [cycle_file],
    )

    assert dependencies == {}


def test_collect_all_functions_in_cycle_detects_signature_conflicts(tmp_path):
    """Test cycle collection reports duplicate names with different signatures."""
    first_file = tmp_path / "first.presl"
    second_file = tmp_path / "second.presl"
    first_file.write_text(
        """
contract First {
    function run() external {
        assembly {
            function shared(a) -> result {
                result := a
            }
        }
    }
}
""",
        encoding="utf-8",
    )
    second_file.write_text(
        """
contract Second {
    function run() external {
        assembly {
            function shared(a, b) -> result {
                result := add(a, b)
            }
        }
    }
}
""",
        encoding="utf-8",
    )
    preprocessor = YulPreprocessor(root_dir=tmp_path)

    with pytest.raises(ValueError, match="Function signature conflict"):
        preprocessor.collect_all_functions_in_cycle(
            {first_file, second_file},
            [first_file, second_file],
        )


def test_resolve_import_reports_missing_self_and_target_errors(tmp_path):
    """Test direct import error paths produce useful exceptions."""
    current_file = tmp_path / "current.presl"
    current_file.write_text(
        """
contract Current {
    function run() external {
        assembly {
            // import missing_external from missing.presl
            function local() -> result {
                result := 1
            }
        }
    }
}
""",
        encoding="utf-8",
    )
    target_file = tmp_path / "target.sol"
    target_file.write_text(
        """
contract Target {
    function run() external {
        assembly {
            function present() -> result {
                result := 1
            }
        }
    }
}
""",
        encoding="utf-8",
    )
    preprocessor = YulPreprocessor(root_dir=tmp_path)

    with pytest.raises(FileNotFoundError, match="Current file not found"):
        preprocessor.resolve_import("local", "self", tmp_path / "missing.presl", [])

    with pytest.raises(ValueError, match="Function 'missing' not found"):
        preprocessor.resolve_import("missing", "self", current_file, [])

    with pytest.raises(FileNotFoundError, match="Import target not found"):
        preprocessor.resolve_import("missing", "missing.sol", current_file, [])

    current_cycle = {current_file}
    preprocessor.cycle_groups[frozenset(current_cycle)] = {}
    with pytest.raises(
        ValueError,
        match="Function 'missing' not found in circular dependency group",
    ):
        preprocessor.resolve_import(
            "missing",
            "current.presl",
            current_file,
            [],
            cycle_group=current_cycle,
        )

    target_cycle = {target_file}
    preprocessor.cycle_groups[frozenset(target_cycle)] = {}
    with pytest.raises(ValueError, match="Function 'missing' not found in cycle"):
        preprocessor.resolve_import("missing", "target.sol", current_file, [])


def test_should_skip_file_detects_does_not_compile_marker(tmp_path):
    """Test skip marker detection in the first few lines."""
    preprocessor = YulPreprocessor()
    skipped = tmp_path / "skip.presl"
    included = tmp_path / "include.presl"

    skipped.write_text(
        "\n//   does - not - compile\ncontract Skip {}\n",
        encoding="utf-8",
    )
    included.write_text("contract Include {}\n", encoding="utf-8")

    assert preprocessor.should_skip_file(skipped)
    assert not preprocessor.should_skip_file(included)
    assert not preprocessor.should_skip_file(tmp_path / "missing.presl")


def test_format_with_forge_success(monkeypatch, tmp_path):
    """Test forge formatter success path."""
    preprocessor = YulPreprocessor()
    target = tmp_path / "output.sol"
    target.write_text("contract Test {}\n", encoding="utf-8")

    def fake_run(args, capture_output, text, timeout):
        assert args == ["forge", "fmt", str(target)]
        assert capture_output
        assert text
        assert timeout == 30

        class Result:
            returncode = 0
            stderr = ""

        return Result()

    monkeypatch.setattr(yul_preprocessor.subprocess, "run", fake_run)

    assert preprocessor.format_with_forge(target)


def test_format_with_forge_failure_paths(monkeypatch, tmp_path, capsys):
    """Test formatter warning paths for failure, timeout, and exceptions."""
    preprocessor = YulPreprocessor()
    target = tmp_path / "output.sol"
    target.write_text("contract Test {}\n", encoding="utf-8")

    class FailedResult:
        returncode = 1
        stderr = "bad format"

    monkeypatch.setattr(
        yul_preprocessor.subprocess,
        "run",
        lambda *args, **kwargs: FailedResult(),
    )
    assert not preprocessor.format_with_forge(target)
    assert "bad format" in capsys.readouterr().err

    monkeypatch.setattr(
        yul_preprocessor.subprocess,
        "run",
        lambda *args, **kwargs: (_ for _ in ()).throw(FileNotFoundError()),
    )
    assert not preprocessor.format_with_forge(target)
    assert "forge not found" in capsys.readouterr().err

    monkeypatch.setattr(
        yul_preprocessor.subprocess,
        "run",
        lambda *args, **kwargs: (_ for _ in ()).throw(
            subprocess.TimeoutExpired("forge", 30)
        ),
    )
    assert not preprocessor.format_with_forge(target)
    assert "timed out" in capsys.readouterr().err

    monkeypatch.setattr(
        yul_preprocessor.subprocess,
        "run",
        lambda *args, **kwargs: (_ for _ in ()).throw(RuntimeError("boom")),
    )
    assert not preprocessor.format_with_forge(target)
    assert "boom" in capsys.readouterr().err


def test_preprocess_file_writes_default_and_explicit_outputs(
    monkeypatch, tmp_path, capsys
):
    """Test single-file preprocessing output paths and optional formatting."""
    input_file = tmp_path / "input.presl"
    explicit_output = tmp_path / "custom.post.sol"
    input_file.write_text(
        """
contract Test {
    function run() external {
        assembly {
            function local() -> result {
                result := 1
            }
        }
    }
}
""",
        encoding="utf-8",
    )

    preprocessor = YulPreprocessor(root_dir=tmp_path)
    preprocessor.preprocess_file(str(input_file), format_output=False)

    default_output = tmp_path / "input.post.sol"
    assert default_output.exists()
    assert "function local() -> result" in default_output.read_text(encoding="utf-8")

    formatted_paths = []
    monkeypatch.setattr(
        YulPreprocessor,
        "format_with_forge",
        lambda _self, path: formatted_paths.append(path) or True,
    )
    preprocessor.preprocess_file(str(input_file), str(explicit_output))

    assert explicit_output.exists()
    assert formatted_paths == [explicit_output]
    assert "Formatted with forge fmt" in capsys.readouterr().out


def test_preprocess_file_defaults_non_presl_output_suffix(tmp_path):
    """Test non-.presl inputs still default to a .post.sol output path."""
    input_file = tmp_path / "input.sol"
    input_file.write_text("contract Plain {}\n", encoding="utf-8")

    preprocessor = YulPreprocessor(root_dir=tmp_path)
    preprocessor.preprocess_file(str(input_file), format_output=False)

    assert (tmp_path / "input.post.sol").read_text(encoding="utf-8") == (
        "contract Plain {}\n"
    )


def test_preprocess_file_reraises_processing_errors(tmp_path, capsys):
    """Test single-file preprocessing reports and reraises invalid imports."""
    bad_file = tmp_path / "bad.presl"
    bad_file.write_text(
        """
contract Bad {
    function run() external {
        assembly {
            // import missing from missing.presl
        }
    }
}
""",
        encoding="utf-8",
    )

    preprocessor = YulPreprocessor(root_dir=tmp_path)

    with pytest.raises(FileNotFoundError):
        preprocessor.preprocess_file(str(bad_file), format_output=False)
    assert "Error processing" in capsys.readouterr().err


def test_preprocess_directory_handles_success_skip_failure_and_formatting(
    monkeypatch, tmp_path, capsys
):
    """Test directory preprocessing across success, skipped, and failed files."""
    good_file = tmp_path / "good.presl"
    skip_file = tmp_path / "skip.t.presl"
    bad_file = tmp_path / "bad.presl"
    good_file.write_text(
        """
contract Good {
    function run() external {
        assembly {
            function local() -> result {
                result := 1
            }
        }
    }
}
""",
        encoding="utf-8",
    )
    skip_file.write_text(
        "// does-not-compile\ncontract Skip {}\n",
        encoding="utf-8",
    )
    bad_file.write_text(
        """
contract Bad {
    function run() external {
        assembly {
            // import missing from missing.presl
        }
    }
}
""",
        encoding="utf-8",
    )

    formatted_paths = []
    monkeypatch.setattr(
        YulPreprocessor,
        "format_with_forge",
        lambda _self, path: formatted_paths.append(path) and False,
    )

    preprocessor = YulPreprocessor(root_dir=tmp_path)
    exit_code = preprocessor.preprocess_directory(str(tmp_path))

    assert exit_code == 1
    assert (tmp_path / "good.post.sol").exists()
    assert not (tmp_path / "skip.t.post.sol").exists()
    assert formatted_paths == [tmp_path]
    output = capsys.readouterr()
    assert "marked as non-compiling" in output.out
    assert "Error:" in output.err
    assert "formatting failed" in output.err


def test_preprocess_directory_reports_successful_formatting(
    monkeypatch, tmp_path, capsys
):
    """Test directory preprocessing reports a successful forge format pass."""
    good_file = tmp_path / "good.presl"
    good_file.write_text("contract Good {}\n", encoding="utf-8")

    monkeypatch.setattr(YulPreprocessor, "format_with_forge", lambda _self, _path: True)

    preprocessor = YulPreprocessor(root_dir=tmp_path)
    assert preprocessor.preprocess_directory(str(tmp_path)) == 0
    assert "Formatted all files in directory" in capsys.readouterr().out


def test_main_usage_and_directory_validation(monkeypatch, tmp_path, capsys):
    """Test command-line entrypoint argument handling."""
    monkeypatch.setattr(sys, "argv", ["yul_preprocessor.py"])
    with pytest.raises(SystemExit) as missing_args:
        yul_preprocessor.main()
    assert missing_args.value.code == 1
    assert "Usage:" in capsys.readouterr().out

    not_a_directory = tmp_path / "file.presl"
    not_a_directory.write_text("contract File {}\n", encoding="utf-8")
    monkeypatch.setattr(sys, "argv", ["yul_preprocessor.py", str(not_a_directory)])
    with pytest.raises(SystemExit) as invalid_directory:
        yul_preprocessor.main()
    assert invalid_directory.value.code == 1
    assert "is not a directory" in capsys.readouterr().out

    monkeypatch.setattr(
        YulPreprocessor,
        "preprocess_directory",
        lambda _self, directory: 0,
    )
    monkeypatch.setattr(sys, "argv", ["yul_preprocessor.py", str(tmp_path)])
    with pytest.raises(SystemExit) as valid_directory:
        yul_preprocessor.main()
    assert valid_directory.value.code == 0
