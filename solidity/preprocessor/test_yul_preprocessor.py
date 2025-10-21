#!/usr/bin/env python3
"""
Tests for YulPreprocessor demonstrating preprocessing capabilities.
"""

from pathlib import Path
import pytest
from yul_preprocessor import YulPreprocessor, YulFunction


class TestYulPreprocessor:
    """Test suite for YulPreprocessor."""

    def setup_method(self):
        """Set up test paths."""
        self.test_files_dir = Path(__file__).parent / "test_files"

    def test_basic_import_preprocessing(self):
        """Test basic import statement preprocessing."""
        test_dir = self.test_files_dir / "basic_import"
        target_file = test_dir / "main.presl"

        # Process the target file
        preprocessor = YulPreprocessor(root_dir=test_dir)
        result = preprocessor.process_file(target_file)

        # Verify the function was imported
        assert "function add5(x) -> result" in result
        assert "result := add(x, 5)" in result
        assert "// import add5 from utils.presl" not in result

    def test_multiple_imports(self):
        """Test importing multiple functions."""
        test_dir = self.test_files_dir / "multiple_imports"
        target_file = test_dir / "calculator.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result = preprocessor.process_file(target_file)

        # Verify both functions were imported
        assert "function multiply(a, b) -> result" in result
        assert "function divide(a, b) -> result" in result
        assert result.count("function multiply") == 1
        assert result.count("function divide") == 1

    def test_relative_path_import(self):
        """Test importing with relative paths."""
        test_dir = self.test_files_dir / "relative_path_import"
        main_file = test_dir / "main_with_relative_import.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result = preprocessor.process_file(main_file)

        assert "function double(x) -> result" in result
        assert "result := mul(x, 2)" in result

    def test_function_deduplication(self):
        """Test that duplicate function imports are deduplicated."""
        test_dir = self.test_files_dir / "function_deduplication"
        target_file = test_dir / "main_dedup.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result = preprocessor.process_file(target_file)

        # Should only have one definition
        assert result.count("function square(x) -> result") == 1

    def test_circular_minimal_allowed(self):
        """Test that circular dependencies are now allowed and handled correctly."""
        test_dir = self.test_files_dir / "circular_regular"
        file_a = test_dir / "a.presl"
        file_b = test_dir / "b.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)

        # Process both files - should not raise an error
        result_a = preprocessor.process_file(file_a)
        result_b = preprocessor.process_file(file_b)

        # Both files should have both funcA and funcB in their assembly blocks
        assert "function funcA() -> result" in result_a
        assert "function funcB() -> result" in result_b
        assert "function funcA() -> result" in result_b
        assert "function funcB() -> result" in result_a

        # Verify both files have access to all functions in the cycle
        # Note: functions may appear multiple times (original + imported)
        assert result_a.count("function funcA") >= 1
        assert result_a.count("function funcB") >= 1
        assert result_b.count("function funcA") >= 1
        assert result_b.count("function funcB") >= 1

    def test_missing_function_error(self):
        """Test error when importing non-existent function."""
        test_dir = self.test_files_dir / "missing_function"
        target_file = test_dir / "target.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)

        with pytest.raises(ValueError, match="Function 'nonExistentFunc' not found"):
            preprocessor.process_file(target_file)

    def test_complex_function_signature(self):
        """Test importing functions with complex signatures."""
        test_dir = self.test_files_dir / "complex_function"
        target_file = test_dir / "main_complex.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result = preprocessor.process_file(target_file)

        assert "function computeMultiple(a, b, c) -> x, y, z" in result
        assert "x := add(a, b)" in result
        assert "y := mul(b, c)" in result
        assert "z := sub(c, a)" in result

    def test_multiple_assembly_blocks(self):
        """Test file with multiple assembly blocks."""
        test_dir = self.test_files_dir / "multiple_assembly_blocks"
        target_file = test_dir / "main_multi.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result = preprocessor.process_file(target_file)

        assert "function func1() -> result" in result
        assert "function func2() -> result" in result

    def test_extract_yul_functions(self):
        """Test YulFunction extraction from assembly blocks."""
        preprocessor = YulPreprocessor()

        assembly_code = """
            function add(a, b) -> result {
                result := add(a, b)
            }

            function multiply(x, y) -> z {
                z := mul(x, y)
            }

            let value := add(5, 10)
        """

        functions = preprocessor.extract_yul_functions(assembly_code)

        assert len(functions) == 2
        assert "add" in functions
        assert "multiply" in functions
        assert functions["add"].name == "add"
        assert functions["multiply"].name == "multiply"

    def test_find_assembly_blocks(self):
        """Test finding assembly blocks in Solidity code."""
        preprocessor = YulPreprocessor()

        code = """
        contract Test {
            function first() external {
                assembly {
                    let x := 1
                }
            }

            function second() external {
                assembly {
                    let y := 2
                }
            }
        }
        """

        blocks = preprocessor.find_assembly_blocks(code)

        assert len(blocks) == 2
        assert "let x := 1" in blocks[0][2]
        assert "let y := 2" in blocks[1][2]

    def test_caching(self):
        """Test that processed files are cached."""
        test_dir = self.test_files_dir / "caching"
        source_file = test_dir / "cached.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)

        # Process twice
        result1 = preprocessor.process_file(source_file)
        result2 = preprocessor.process_file(source_file)

        # Should be cached
        assert source_file in preprocessor.processed_cache
        assert result1 == result2

    def test_preprocess_file_output(self):
        """Test preprocessing with file output."""
        test_dir = self.test_files_dir / "preprocess_output"
        input_file = test_dir / "input.presl"
        output_file = test_dir / "output" / "output.post.sol"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        preprocessor.preprocess_file(
            str(input_file), str(output_file), format_output=False
        )

        assert output_file.exists()
        content = output_file.read_text()
        assert "function testFunc() -> result" in content

        # Clean up output file
        output_file.unlink()

    def test_multiple_imports_per_line(self):
        """Test importing multiple functions in a single import statement."""
        test_dir = self.test_files_dir / "multiple_imports_per_line"
        target_file = test_dir / "main.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result = preprocessor.process_file(target_file)

        # Verify requested functions were imported
        assert "function add(a, b) -> result" in result
        assert "function multiply(a, b) -> result" in result
        assert result.count("function add(a, b) -> result") == 1
        assert result.count("function multiply(a, b) -> result") == 1
        # Note: subtract may also be included as it's from the same file
        # This is correct behavior - importing from a file gets its complete definition

    def test_self_import(self):
        """Test importing functions from a different assembly block in the same file."""
        test_dir = self.test_files_dir / "self_import"
        target_file = test_dir / "single_self_import.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result = preprocessor.process_file(target_file)

        # The function should appear twice: once in the original block, once imported
        assert result.count("function utilFunc(x) -> result") == 2
        assert "result := add(x, 42)" in result
        # Verify it's usable in the second block
        assert "let value := utilFunc(10)" in result

        # IMPORTANT: When importing from self (different assembly block in same file),
        # the imported function SHOULD have coverage exclusion markers
        assert "function exclude_coverage_start_utilFunc() {}" in result
        assert "function exclude_coverage_stop_utilFunc() {}" in result

    def test_self_import_multiple(self):
        """Test importing multiple functions from a different assembly block in the same file."""
        test_dir = self.test_files_dir / "self_import"
        target_file = test_dir / "self_referencing.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result = preprocessor.process_file(target_file)

        # Each function should appear twice: once in the original block, once imported
        assert result.count("function helper(x) -> result") == 2
        assert result.count("function anotherHelper(y) -> result") == 2
        assert "result := mul(x, 2)" in result
        assert "result := add(y, 10)" in result
        # Verify they're usable in the second block
        assert "let doubled := helper(5)" in result
        assert "let increased := anotherHelper(doubled)" in result

        # IMPORTANT: When importing from self (different assembly block in same file),
        # the imported functions SHOULD have coverage exclusion markers
        assert "function exclude_coverage_start_helper() {}" in result
        assert "function exclude_coverage_stop_helper() {}" in result
        assert "function exclude_coverage_start_anotherHelper() {}" in result
        assert "function exclude_coverage_stop_anotherHelper() {}" in result

    def test_circular_with_external_import(self):
        """Test that C can import from a circular group A-B."""
        test_dir = self.test_files_dir / "circular_regular"
        file_c = test_dir / "c.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result_c = preprocessor.process_file(file_c)

        # C should have funcA, funcB, and funcC
        assert "function funcA() -> result" in result_c
        assert "function funcB() -> result" in result_c
        assert "function funcC() -> result" in result_c

    def test_nested_circular_dependencies(self):
        """Test nested circular dependencies: C -> B0..B1 -> A0..A1 -> utils.

        With the new dependency tracking, only functions that are actually used (called)
        are imported. Since funcB0/funcB1 don't call funcA0/funcA1, those won't be
        transitively included in C's imports."""
        test_dir = self.test_files_dir / "nested_circular"

        # Process all files
        preprocessor = YulPreprocessor(root_dir=test_dir)

        result_c = preprocessor.process_file(test_dir / "c.presl")
        result_b0 = preprocessor.process_file(test_dir / "b0.presl")
        result_b1 = preprocessor.process_file(test_dir / "b1.presl")
        result_a0 = preprocessor.process_file(test_dir / "a0.presl")
        result_a1 = preprocessor.process_file(test_dir / "a1.presl")

        # A0 and A1 form a cycle, so they should both have funcA0 and funcA1
        assert "function funcA0() -> result" in result_a0
        assert "function funcA1() -> result" in result_a0
        assert "function funcA0() -> result" in result_a1
        assert "function funcA1() -> result" in result_a1

        # A0 and A1 should also have utils functions they imported
        assert "function utilAdd" in result_a0
        assert "function utilMul" in result_a1

        # B0 and B1 form a cycle, so they should both have funcB0 and funcB1
        assert "function funcB0() -> result" in result_b0
        assert "function funcB1() -> result" in result_b0
        assert "function funcB0() -> result" in result_b1
        assert "function funcB1() -> result" in result_b1

        # B0 and B1 should also have A0 and A1 functions (transitively imported)
        assert "function funcA0() -> result" in result_b0
        assert "function funcA1() -> result" in result_b1

        # C should have functions from B cycle
        # When importing from a cycle, all functions from that cycle (and their
        # external dependencies) are included, not just the directly called ones
        assert "function funcC() -> result" in result_c
        assert "function funcB0() -> result" in result_c
        assert "function funcB1() -> result" in result_c
        # funcA0 and funcA1 should be in C because they're external dependencies
        # of the {b0, b1} cycle
        assert "function funcA0() -> result" in result_c
        assert "function funcA1() -> result" in result_c
        # util functions should also be in C as they're external dependencies
        # of the {a0, a1} cycle
        assert "function utilAdd" in result_c
        assert "function utilMul" in result_c

    def test_unused_functions_excluded(self):
        """Test that unrelated functions are NOT imported when not dependencies.

        When importing 'baz' from a library that also has 'foo' and 'bar'
        (where foo calls bar but neither is related to baz), only baz should
        be imported, not foo or bar.
        """
        test_dir = self.test_files_dir / "unused_functions"
        main_file = test_dir / "main.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result = preprocessor.process_file(main_file)

        # baz should be imported (explicitly requested)
        assert "function baz() -> result" in result
        assert "result := 300" in result

        # mainFunc should be in the result (it's defined locally)
        assert "function mainFunc() -> result" in result

        # foo and bar should NOT be imported (not dependencies of baz)
        assert "function foo() -> result" not in result
        assert "function bar() -> result" not in result

        # unrelated should also NOT be imported
        assert "function unrelated() -> result" not in result

    def test_t_presl_file_processing(self):
        """Test that .t.presl files replace .presl with .post.sol in imports."""
        test_dir = self.test_files_dir / "t_presol_test"
        input_file = test_dir / "Example.t.presl"
        output_file = test_dir / "Example.t.post.sol"

        preprocessor = YulPreprocessor(root_dir=test_dir)

        # Process the .t.presl file using regular process_file
        result = preprocessor.process_file(input_file)
        output_file.write_text(result)

        # Verify .presl was replaced with .post.sol
        assert 'from "./Utils.post.sol"' in result
        assert 'from "./Helper.post.sol"' in result
        assert ".presl" not in result

        # Clean up
        output_file.unlink()

    def test_multiline_function_definition(self):
        """Test importing functions with multiline signatures."""
        test_dir = self.test_files_dir / "multiline_function"
        target_file = test_dir / "main.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result = preprocessor.process_file(target_file)

        # Verify the multiline function was imported correctly
        assert "function multiline_with_many_params" in result
        assert "foo, bar, baz, qux, quux, corge, grault" in result
        assert "result_a, result_b" in result
        assert "function another_multiline_func" in result
        assert "x := add(a, b)" in result
        assert "y := mul(b, c)" in result

    def test_extract_multiline_functions(self):
        """Test extraction of functions with multiline signatures."""
        preprocessor = YulPreprocessor()

        assembly_code = """
            function multiline_with_many_params(
                foo, bar, baz, qux, quux, corge, grault
            ) -> result_a, result_b {
                result_a := add(foo, bar)
                result_b := mulmod(baz, qux, add(quux, add(corge, grault)))
            }

            function simple_func(x) -> result {
                result := add(x, 1)
            }
        """

        functions = preprocessor.extract_yul_functions(assembly_code)

        assert len(functions) == 2
        assert "multiline_with_many_params" in functions
        assert "simple_func" in functions

        # Check that the multiline signature was properly extracted
        multiline_func = functions["multiline_with_many_params"]
        assert "foo" in multiline_func.signature
        assert "bar" in multiline_func.signature
        assert "grault" in multiline_func.signature
        assert "result_a, result_b" in multiline_func.signature

    def test_solidity_imports_converted_in_presl_output(self):
        """Test that Solidity imports with .presl extension are converted to .post.sol in .post.sol output."""
        test_dir = self.test_files_dir / "sol_import_conversion"
        test_file = test_dir / "TestContract.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result = preprocessor.process_file(test_file)

        # Verify no .presl imports remain in Solidity import statements
        import re

        presl_imports = re.findall(
            r'import\s+(?:.*?\s+from\s+)?["\'][^"\']*?\.presl["\']', result
        )

        assert (
            len(presl_imports) == 0
        ), f"Found .presl imports in .post.sol output: {presl_imports}"
        # Verify they were converted to .post.sol
        assert 'import "./SomeLib.post.sol"' in result
        assert 'import {Util} from "./Utils.post.sol"' in result

    def test_solidity_imports_converted_in_t_presl_output(self):
        """Test that .t.presl files have both styles of Solidity imports with .presl converted to .post.sol."""
        test_dir = self.test_files_dir / "t_presl_sol_imports"
        test_file = test_dir / "ExampleTest.t.presl"
        output_file = test_dir / "ExampleTest.t.post.sol"

        preprocessor = YulPreprocessor(root_dir=test_dir)

        try:
            # Process using regular process_file
            result = preprocessor.process_file(test_file)
            output_file.write_text(result)

            # Verify no .presl remains in output
            import re

            presl_imports = re.findall(
                r'import\s+(?:.*?\s+from\s+)?["\'][^"\']*?\.presl["\']', result
            )
            assert (
                len(presl_imports) == 0
            ), f"Found .presl imports in .t.post.sol output: {presl_imports}"

            # Verify both import styles were converted to .post.sol
            assert ".post.sol" in result, "Output should have .post.sol imports"
            assert 'import "./SomeLib.post.sol"' in result
            assert 'from "./Utils.post.sol"' in result
            assert 'from "./Helper.post.sol"' in result
        finally:
            # Clean up
            if output_file.exists():
                output_file.unlink()

    def test_self_import_with_external_deps(self):
        """Test that self imports correctly include external dependencies of the imported function.

        This is a regression test for a bug where importing a function 'from self' would
        include the function but not its external dependencies (functions that were imported
        from other files within that function's assembly block).
        """
        test_dir = self.test_files_dir / "self_import_with_external_deps"
        target_file = test_dir / "main.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result = preprocessor.process_file(target_file)

        # In the first() function's assembly block, compute() calls add_one() and double_value()
        # which are imported from helper.presl
        first_block_start = result.find("function first()")
        second_block_start = result.find("function second()")
        first_block = result[first_block_start:second_block_start]

        # First block should have all three functions
        assert "function compute(x) -> result" in first_block
        assert "function add_one(x) -> result" in first_block
        assert "function double_value(x) -> result" in first_block

        # In the second() function's assembly block, we import compute from self
        # This should transitively import add_one and double_value as well
        second_block = result[second_block_start:]

        # Second block should also have all three functions (the bug was that it only had compute)
        assert (
            "function compute(x) -> result" in second_block
        ), "compute() should be imported from self"
        assert (
            "function add_one(x) -> result" in second_block
        ), "add_one() should be transitively imported as a dependency of compute()"
        assert (
            "function double_value(x) -> result" in second_block
        ), "double_value() should be transitively imported as a dependency of compute()"

        # Verify compute actually calls these functions
        compute_in_second = second_block[
            second_block.find("function compute") : second_block.find(
                "function compute"
            )
            + 200
        ]
        assert "add_one" in compute_in_second
        assert "double_value" in compute_in_second

    def test_no_presl_references_in_post_sol_imports(self):
        """
        Strict test: ensure NO .presl references appear in import statements of any .post.sol output.
        This test processes multiple files and validates all outputs.
        """
        import re

        test_cases = [
            {
                "dir": "strict_no_presl_imports/case1",
                "input": "MultipleImports.presl",
                "use_t_presl": False,
            },
            {
                "dir": "strict_no_presl_imports/case2",
                "input": "TestWithComments.t.presl",
                "use_t_presl": True,
            },
        ]

        for test_case in test_cases:
            test_dir = self.test_files_dir / test_case["dir"]
            input_file = test_dir / test_case["input"]

            try:
                preprocessor = YulPreprocessor(root_dir=test_dir)

                # Use regular process_file for both .presl and .t.presl files
                result = preprocessor.process_file(input_file)

                if test_case["use_t_presl"]:
                    output_file = test_dir / test_case["input"].replace(
                        ".t.presl", ".t.post.sol"
                    )
                else:
                    output_file = input_file.with_suffix(".post.sol")

                output_file.write_text(result)

                # Read the output
                output_content = output_file.read_text()

                # STRICT CHECK: Find any .presl references in import statements
                presl_in_imports = re.findall(
                    r'import\s+(?:.*?\s+from\s+)?["\'][^"\']*?\.presl["\']',
                    output_content,
                )

                assert len(presl_in_imports) == 0, (
                    f"FAILED: Found .presl in import statements in {output_file.name}:\n"
                    f"  Test case: {test_case['dir']}\n"
                    f"  Matches: {presl_in_imports}\n"
                    f"  All .presl references MUST be converted to .post.sol in imports"
                )

                # Verify .post.sol replacements exist
                assert ".post.sol" in output_content, (
                    f"FAILED: No .post.sol references found in {output_file.name}. "
                    f"Expected .presl imports to be converted."
                )

            finally:
                # Clean up output files
                if output_file.exists():
                    output_file.unlink()

    def test_slither_comments_preserved(self):
        """Test that Slither exemption comments are preserved with imported functions."""
        test_dir = self.test_files_dir / "slither_comments"
        main_file = test_dir / "main.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result = preprocessor.process_file(main_file)

        # Verify both functions were imported
        assert "function complexFunction(a, b, c) -> result" in result
        assert "function simpleFunction(x) -> y" in result

        # Verify Slither comments are preserved
        # Should have slither-disable-start before complexFunction
        assert "// slither-disable-start cyclomatic-complexity" in result
        # Should have slither-disable-end after complexFunction
        assert "// slither-disable-end cyclomatic-complexity" in result
        # Should have slither-disable-next-line before simpleFunction
        assert "// slither-disable-next-line write-after-write" in result

        # Verify coverage exclusion markers are added
        assert "function exclude_coverage_start_complexFunction() {}" in result
        assert "function exclude_coverage_stop_complexFunction() {}" in result
        assert "function exclude_coverage_start_simpleFunction() {}" in result
        assert "function exclude_coverage_stop_simpleFunction() {}" in result

        # Verify the ordering is correct (disable-start comes before the function)
        start_idx = result.find("// slither-disable-start cyclomatic-complexity")
        cov_start_idx = result.find("function exclude_coverage_start_complexFunction")
        func_idx = result.find("function complexFunction")
        cov_stop_idx = result.find("function exclude_coverage_stop_complexFunction")
        end_idx = result.find("// slither-disable-end cyclomatic-complexity")
        assert (
            start_idx < cov_start_idx < func_idx < cov_stop_idx < end_idx
        ), "Slither comments and coverage markers should properly wrap the function"

        # Verify no duplicate disable-end comments
        assert result.count("// slither-disable-end cyclomatic-complexity") == 1

    def test_coverage_exclusion_for_external_imports(self):
        """Test that functions imported from external files get coverage exclusion markers."""
        test_dir = self.test_files_dir / "basic_import"
        target_file = test_dir / "main.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result = preprocessor.process_file(target_file)

        # Verify the imported function has coverage exclusion markers
        assert "function exclude_coverage_start_add5() {}" in result
        assert "function exclude_coverage_stop_add5() {}" in result

        # Verify the markers wrap the function
        start_idx = result.find("function exclude_coverage_start_add5")
        func_idx = result.find("function add5(x) -> result")
        stop_idx = result.find("function exclude_coverage_stop_add5")
        assert (
            start_idx < func_idx < stop_idx
        ), "Coverage markers should wrap the imported function"

    def test_coverage_exclusion_circular_dependencies(self):
        """Test that functions in their own file are NOT coverage-excluded in circular dependencies."""
        test_dir = self.test_files_dir / "circular_regular"
        file_a = test_dir / "a.presl"
        file_b = test_dir / "b.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result_a = preprocessor.process_file(file_a)
        result_b = preprocessor.process_file(file_b)

        # In a.post.sol, funcA is defined locally so it should NOT have coverage exclusion
        # but funcB is imported so it SHOULD have coverage exclusion

        # funcB imported into a.post.sol should have coverage exclusion
        assert "function exclude_coverage_start_funcB() {}" in result_a
        assert "function exclude_coverage_stop_funcB() {}" in result_a

        # funcA imported into b.post.sol should have coverage exclusion
        assert "function exclude_coverage_start_funcA() {}" in result_b
        assert "function exclude_coverage_stop_funcA() {}" in result_b

        # IMPORTANT: funcA in a.post.sol should NOT have coverage exclusion
        # because it's defined in a.presl (the source file for a.post.sol)
        assert "function exclude_coverage_start_funcA() {}" not in result_a
        assert "function exclude_coverage_stop_funcA() {}" not in result_a

        # Similarly, funcB in b.post.sol should NOT have coverage exclusion
        # because it's defined in b.presl (the source file for b.post.sol)
        assert "function exclude_coverage_start_funcB() {}" not in result_b
        assert "function exclude_coverage_stop_funcB() {}" not in result_b

    def test_coverage_exclusion_with_dependencies(self):
        """Test that transitive dependencies also get coverage exclusion markers."""
        test_dir = self.test_files_dir / "multiple_imports"
        target_file = test_dir / "calculator.presl"

        preprocessor = YulPreprocessor(root_dir=test_dir)
        result = preprocessor.process_file(target_file)

        # Both imported functions should have coverage exclusion markers
        assert "function exclude_coverage_start_multiply() {}" in result
        assert "function exclude_coverage_stop_multiply() {}" in result
        assert "function exclude_coverage_start_divide() {}" in result
        assert "function exclude_coverage_stop_divide() {}" in result


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
