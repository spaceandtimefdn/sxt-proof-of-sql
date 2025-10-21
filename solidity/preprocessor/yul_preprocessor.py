#!/usr/bin/env python3
"""
Yul Import Preprocessor

Preprocesses .presl files to .post.sol files by resolving // import statements
within assembly blocks. Recursively processes dependencies and handles function
deduplication.

Usage:
    python3 yul_preprocessor.py <directory>

Import Syntax:
    // import <function_name> from <file_path>
    // import <func1>, <func2>, ... from <file_path>
    // import <function_name> from self

Example:
    // import __computeFold from ../base/MathUtil.presl
    // import __err from Errors.sol
    // import add, multiply from math.presl
    // import helperFunc from self
"""

import os
import re
import sys
import subprocess
from pathlib import Path
from typing import Dict, Set, List, Tuple, Optional


class YulFunction:
    """Represents a parsed Yul function."""

    def __init__(
        self,
        name: str,
        signature: str,
        body: str,
        full_text: str,
        pre_comments: str = "",
        post_comments: str = "",
        source_file: Optional[Path] = None,
    ):
        self.name = name
        self.signature = signature  # function name(...) -> ...
        self.body = body
        self.full_text = full_text  # Complete function including definition
        self.pre_comments = (
            pre_comments  # Comments before function (e.g., slither-disable-start)
        )
        self.post_comments = (
            post_comments  # Comments after function (e.g., slither-disable-end)
        )
        self.source_file = source_file  # The .presl file where this function is defined

    def __eq__(self, other):
        if not isinstance(other, YulFunction):
            return False
        return self.signature == other.signature

    def __hash__(self):
        return hash(self.signature)

    def __repr__(self):
        return f"YulFunction({self.name}, sig={self.signature})"


class YulPreprocessor:
    """Preprocesses Solidity files with Yul import statements."""

    def __init__(self, root_dir: Optional[str] = None):
        self.root_dir = Path(root_dir) if root_dir else Path.cwd()
        self.processed_cache: Dict[Path, str] = {}  # Cache of processed files
        self.cycle_groups: Dict[frozenset, Dict[str, YulFunction]] = (
            {}
        )  # Cache for circular dependency groups

        # Regex patterns
        # Updated to support multiple imports: import foo, bar, baz from file.sol
        self.import_pattern = re.compile(r"//\s*import\s+([\w\s,]+)\s+from\s+([^\s]+)")
        self.assembly_start = re.compile(r"assembly\s*\{")
        # Pattern for complete function signature (used for matching full signatures)
        self.function_pattern = re.compile(
            r"function\s+(\w+)\s*\([^)]*\)(?:\s*->\s*[^{]*)?", re.DOTALL
        )
        # Simple pattern to extract just the function name (for multiline signatures)
        self.function_name_pattern = re.compile(r"function\s+(\w+)")

    def find_assembly_blocks(self, content: str) -> List[Tuple[int, int, str]]:
        """
        Find all assembly blocks in the content.
        Returns list of (start_pos, end_pos, block_content) tuples.
        """
        blocks = []
        pos = 0

        while True:
            match = self.assembly_start.search(content, pos)
            if not match:
                break

            # Find the opening brace
            start = match.end()
            brace_count = 1
            i = start

            while i < len(content) and brace_count > 0:
                if content[i] == "{":
                    brace_count += 1
                elif content[i] == "}":
                    brace_count -= 1
                i += 1

            if brace_count == 0:
                # Found matching close brace
                block_content = content[start : i - 1]
                blocks.append((match.start(), i, block_content))
                pos = i
            else:
                break

        return blocks

    def extract_yul_functions(
        self, assembly_content: str, source_file: Optional[Path] = None
    ) -> Dict[str, YulFunction]:
        """
        Extract all Yul function definitions from an assembly block.
        Returns dict mapping function name to YulFunction object.
        Handles multiline function signatures.
        Captures Slither exemption comments before and after functions.
        """
        functions = {}

        # Find all function definitions
        lines = assembly_content.split("\n")
        i = 0

        while i < len(lines):
            line = lines[i]

            # Check if line starts with "function" keyword
            if line.strip().startswith("function"):
                # Look back for Slither disable comments before the function
                # Only capture slither-disable-start or slither-disable-next-line
                # Do NOT capture slither-disable-end as that belongs to the previous function
                pre_comment_lines = []
                j = i - 1
                while j >= 0:
                    prev_line = lines[j].strip()
                    # Check for slither-disable-start or slither-disable-next-line (not disable-end)
                    if "slither-disable" in prev_line and prev_line.startswith("//"):
                        if "slither-disable-end" not in prev_line:
                            # This is a disable-start or disable-next-line, include it
                            pre_comment_lines.insert(0, lines[j])
                            j -= 1
                        else:
                            # This is a disable-end from a previous function, stop here
                            break
                    elif prev_line == "":
                        # Allow empty lines but don't add them
                        j -= 1
                    else:
                        # Stop when we hit non-comment/non-empty content
                        break

                # Extract function name using simple pattern
                func_match = self.function_name_pattern.search(line)

                if func_match:
                    func_name = func_match.group(1)
                else:
                    # If we can't find function name on the first line, skip
                    i += 1
                    continue

                # Extract the signature (everything up to the opening brace)
                sig_start = i
                sig_lines = [line]

                # Find opening brace of function body (may span multiple lines)
                while i < len(lines) and "{" not in lines[i]:
                    i += 1
                    if i < len(lines):
                        sig_lines.append(lines[i])

                if i >= len(lines):
                    break

                # Join signature lines and normalize whitespace for matching
                signature_text = " ".join(sig_lines)
                # Extract signature part (everything before the opening brace)
                brace_pos = signature_text.find("{")
                if brace_pos != -1:
                    signature = signature_text[:brace_pos].strip()
                else:
                    # Shouldn't happen but handle gracefully
                    i += 1
                    continue

                # Now extract the function body
                brace_count = lines[i].count("{") - lines[i].count("}")
                func_lines = sig_lines[:]
                i += 1

                while i < len(lines) and brace_count > 0:
                    func_lines.append(lines[i])
                    brace_count += lines[i].count("{") - lines[i].count("}")
                    i += 1

                # Look ahead for Slither disable-end comments after the function
                post_comment_lines = []
                temp_i = i

                # Check if there was a slither-disable-start in pre-comments
                has_disable_start = any(
                    "slither-disable-start" in line for line in pre_comment_lines
                )

                # If there's a slither-disable-start, search more aggressively for the matching end
                max_lookahead = 20 if has_disable_start else 5

                while temp_i < len(lines) and (temp_i - i) < max_lookahead:
                    next_line = lines[temp_i].strip()
                    # Check for slither-disable-end
                    if "slither-disable-end" in next_line and next_line.startswith(
                        "//"
                    ):
                        post_comment_lines.append(lines[temp_i])
                        i = temp_i + 1
                        break
                    elif next_line == "" or has_disable_start:
                        # Allow empty lines, or any content if we're searching for a matching end
                        temp_i += 1
                    else:
                        # Stop when we hit any other content (function or non-comment)
                        # unless we're searching for a matching slither-disable-end
                        break

                full_text = "\n".join(func_lines)
                body = "\n".join(func_lines[len(sig_lines) :])
                pre_comments = "\n".join(pre_comment_lines) if pre_comment_lines else ""
                post_comments = (
                    "\n".join(post_comment_lines) if post_comment_lines else ""
                )

                functions[func_name] = YulFunction(
                    name=func_name,
                    signature=signature,
                    body=body,
                    full_text=full_text,
                    pre_comments=pre_comments,
                    post_comments=post_comments,
                    source_file=source_file,
                )
            else:
                i += 1

        return functions

    def find_yul_function_calls(
        self, body: str, all_functions: Dict[str, YulFunction]
    ) -> Set[str]:
        """
        Find all Yul function calls in a function body.
        Returns set of function names that are called.
        """
        called_functions = set()

        # Pattern to match function calls: function_name(
        # This looks for word boundaries followed by an opening parenthesis
        call_pattern = re.compile(r"\b(\w+)\s*\(")

        for match in call_pattern.finditer(body):
            func_name = match.group(1)
            # Only include if it's actually a defined function (not a built-in or keyword)
            if func_name in all_functions:
                called_functions.add(func_name)

        return called_functions

    def get_function_dependencies(
        self, func_name: str, all_functions: Dict[str, YulFunction]
    ) -> Dict[str, YulFunction]:
        """
        Get a function and all its recursive dependencies.
        Returns dict of function name to YulFunction object.
        """
        if func_name not in all_functions:
            return {}

        result = {}
        to_process = [func_name]
        processed = set()

        while to_process:
            current = to_process.pop()
            if current in processed:
                continue

            processed.add(current)
            result[current] = all_functions[current]

            # Find all functions called by this function
            called = self.find_yul_function_calls(
                all_functions[current].body, all_functions
            )
            for called_func in called:
                if called_func not in processed:
                    to_process.append(called_func)

        return result

    def resolve_import_path(self, import_path: str, current_file: Path) -> Path:
        """Resolve an import path relative to the current file."""
        if import_path.startswith("/"):
            # Absolute path from root
            resolved = self.root_dir / import_path.lstrip("/")
        else:
            # Relative path
            resolved = (current_file.parent / import_path).resolve()

        return resolved

    def collect_external_dependencies_for_cycle(
        self, cycle_files: Set[Path], processing_stack: List[Path]
    ) -> Dict[str, YulFunction]:
        """
        Collect all external dependencies (functions imported from files outside the cycle).
        Returns dict of external functions.
        """
        external_functions = {}

        for file_path in cycle_files:
            if not file_path.exists():
                continue

            content = file_path.read_text(encoding="utf-8")
            assembly_blocks = self.find_assembly_blocks(content)

            for _, _, block_content in assembly_blocks:
                # Find all import statements
                lines = block_content.split("\n")
                for line in lines:
                    import_match = self.import_pattern.search(line)
                    if not import_match:
                        continue

                    func_names_str = import_match.group(1)
                    import_path = import_match.group(2)

                    # Skip 'self' imports
                    if import_path.strip().lower() == "self":
                        continue

                    # Resolve the import path
                    target_file = self.resolve_import_path(import_path, file_path)

                    # Only process if target is outside the cycle
                    if target_file in cycle_files:
                        continue

                    # Parse function names
                    func_names = [name.strip() for name in func_names_str.split(",")]

                    # Process each imported function
                    for func_name in func_names:
                        try:
                            # Recursively resolve this import (it's outside the cycle)
                            imported_funcs = self.resolve_import(
                                func_name,
                                import_path,
                                file_path,
                                processing_stack,
                                "",
                                None,  # No cycle group since we're importing from outside
                            )
                            # Add to external functions
                            for name, func in imported_funcs.items():
                                if name not in external_functions:
                                    external_functions[name] = func
                        except Exception as e:
                            # If we can't resolve, continue - it will be caught later
                            pass

        return external_functions

    def collect_all_functions_in_cycle(
        self, cycle_files: Set[Path], processing_stack: List[Path]
    ) -> Dict[str, YulFunction]:
        """
        Collect all functions from all files in a circular dependency group,
        including their external dependencies.
        Returns a unified dictionary of all functions.
        """
        all_functions = {}

        # First, collect all functions defined in the cycle
        for file_path in cycle_files:
            if not file_path.exists():
                continue

            content = file_path.read_text(encoding="utf-8")
            assembly_blocks = self.find_assembly_blocks(content)

            for _, _, block_content in assembly_blocks:
                functions = self.extract_yul_functions(block_content, file_path)
                for func_name, func in functions.items():
                    if func_name in all_functions:
                        # Check for signature conflicts
                        if all_functions[func_name].signature != func.signature:
                            raise ValueError(
                                f"Function signature conflict in circular dependency group:\n"
                                f"  Function: {func_name}\n"
                                f"  Signature 1: {all_functions[func_name].signature}\n"
                                f"  Signature 2: {func.signature}"
                            )
                    else:
                        all_functions[func_name] = func

        # Now collect external dependencies
        external_deps = self.collect_external_dependencies_for_cycle(
            cycle_files, processing_stack
        )

        # Add external dependencies
        all_functions.update(external_deps)

        return all_functions

    def process_file(
        self,
        file_path: Path,
        processing_stack: Optional[List[Path]] = None,
        cycle_group: Optional[Set[Path]] = None,
    ) -> str:
        """
        Process a .presl file and resolve all imports recursively.
        Returns the processed content.
        Handles circular dependencies by processing dependency cycles as unified groups.
        """
        # Ensure file_path is resolved to an absolute path for consistent comparison
        file_path = file_path.resolve()

        if processing_stack is None:
            processing_stack = []

        # Check for circular dependencies
        if file_path in processing_stack:
            # We found a cycle! Collect all files in the cycle
            cycle_files = set()
            found_start = False
            for path in processing_stack:
                if path == file_path:
                    found_start = True
                if found_start:
                    cycle_files.add(path)
            cycle_files.add(file_path)

            # Check if we already have this cycle group cached
            cycle_key = frozenset(cycle_files)
            if cycle_key not in self.cycle_groups:
                # Collect all functions from all files in the cycle
                self.cycle_groups[cycle_key] = self.collect_all_functions_in_cycle(
                    cycle_files, processing_stack
                )

            # Return cached content if available (we've already processed this as part of the cycle)
            if file_path in self.processed_cache:
                return self.processed_cache[file_path]

            # Mark that we're processing a cycle
            if cycle_group is None:
                cycle_group = cycle_files

            # Continue processing (will be handled specially in process_assembly_block)

        # Check cache
        if file_path in self.processed_cache:
            return self.processed_cache[file_path]

        # Read the file
        if not file_path.exists():
            raise FileNotFoundError(f"File not found: {file_path}")

        content = file_path.read_text(encoding="utf-8")

        # Add to processing stack
        processing_stack.append(file_path)

        # Find all assembly blocks
        assembly_blocks = self.find_assembly_blocks(content)

        # Process each assembly block in reverse order (to maintain positions)
        for start_pos, end_pos, block_content in reversed(assembly_blocks):
            processed_block = self.process_assembly_block(
                block_content, file_path, processing_stack, cycle_group
            )

            # Replace the block in the content
            before = content[:start_pos]
            after = content[end_pos:]
            content = before + "assembly {\n" + processed_block + "\n    }" + after

        # Remove from processing stack (only if it's actually in there)
        if file_path in processing_stack:
            processing_stack.remove(file_path)

        # Replace .presl with .post.sol in Solidity import statements
        # Handle both formats: import "path.presl"; and import {X} from "path.presl";
        content = re.sub(
            r'(import\s+(?:.*?\s+from\s+)?["\'])([^"\']*?)\.presl(["\'])',
            r"\1\2.post.sol\3",
            content,
        )

        # Cache the result
        self.processed_cache[file_path] = content

        return content

    def process_assembly_block(
        self,
        block_content: str,
        current_file: Path,
        processing_stack: List[Path],
        cycle_group: Optional[Set[Path]] = None,
    ) -> str:
        """
        Process a single assembly block, resolving all imports.
        Handles circular dependencies by using the unified function set for the cycle group.
        """
        lines = block_content.split("\n")
        result_lines = []
        imported_functions: Dict[str, YulFunction] = {}

        # Check if current file is part of a cycle group
        current_cycle_key = None
        if cycle_group and current_file in cycle_group:
            current_cycle_key = frozenset(cycle_group)

        # Extract local functions from this block to detect what needs to be deduplicated
        # These are functions defined in THIS specific assembly block
        local_functions = self.extract_yul_functions(block_content, current_file)
        local_function_names = set(local_functions.keys())

        i = 0
        while i < len(lines):
            line = lines[i]

            # Check for import statement
            import_match = self.import_pattern.search(line)

            if import_match:
                func_names_str = import_match.group(1)
                import_path = import_match.group(2)

                # Parse multiple function names (comma-separated)
                func_names = [name.strip() for name in func_names_str.split(",")]

                # Resolve imports for all requested functions and collect their dependencies
                imported_funcs = {}
                for func_name in func_names:
                    funcs_with_deps = self.resolve_import(
                        func_name,
                        import_path,
                        current_file,
                        processing_stack,
                        block_content,
                        cycle_group,
                    )
                    # Merge with deduplication
                    for name, func in funcs_with_deps.items():
                        if name not in imported_funcs:
                            imported_funcs[name] = func

                # Add to imported functions with deduplication
                for name, func in imported_funcs.items():
                    if name in imported_functions:
                        # Check if signatures match
                        if imported_functions[name].signature != func.signature:
                            raise ValueError(
                                f"Function signature mismatch for '{name}':\n"
                                f"  Existing: {imported_functions[name].signature}\n"
                                f"  New:      {func.signature}\n"
                                f"  In file:  {current_file}"
                            )
                        # Otherwise, keep the first one (deduplicate)
                    else:
                        imported_functions[name] = func

                # Replace import line with imported function(s)
                # We'll add all imported functions at the end
                i += 1
            else:
                result_lines.append(line)
                i += 1

        # If this file is part of a cycle group, include ALL functions from the cycle
        if current_cycle_key and current_cycle_key in self.cycle_groups:
            cycle_functions = self.cycle_groups[current_cycle_key]
            # Add all cycle functions (they're already deduplicated)
            for func_name, func in cycle_functions.items():
                if func_name not in imported_functions:
                    imported_functions[func_name] = func

        # Remove local function definitions from result_lines if they're in imported_functions
        # This prevents duplication when functions are part of a cycle group
        if local_functions and imported_functions:
            # Need to rebuild result_lines without function definitions that are imported
            filtered_lines = []
            skip_until = -1

            for idx, line in enumerate(result_lines):
                if idx < skip_until:
                    continue

                # Check if this line starts a function definition
                func_match = self.function_pattern.search(line)
                if func_match and line.strip().startswith("function"):
                    func_name = func_match.group(1)

                    # If this function is in imported_functions, skip its entire definition
                    if func_name in imported_functions and func_name in local_functions:
                        # Find the end of this function
                        brace_count = line.count("{") - line.count("}")
                        skip_until = idx + 1

                        while skip_until < len(result_lines) and brace_count > 0:
                            brace_count += result_lines[skip_until].count(
                                "{"
                            ) - result_lines[skip_until].count("}")
                            skip_until += 1

                        continue

                filtered_lines.append(line)

            result_lines = filtered_lines

        # Prepend all imported functions at the beginning of the block
        if imported_functions:
            func_lines = []
            for func in imported_functions.values():
                # A function should NOT have coverage exclusion if it's defined in THIS assembly block
                # It SHOULD have coverage exclusion if it's imported from:
                # 1. A different file (func.source_file != current_file)
                # 2. The same file but a different assembly block (func.name not in local_function_names)
                is_truly_local = func.name in local_function_names

                # Add coverage exclusion start marker FIRST for non-local functions
                if not is_truly_local:
                    func_lines.append(
                        f"            function exclude_coverage_start_{func.name}() {{}} // solhint-disable-line no-empty-blocks"
                    )
                # Include pre-comments (e.g., slither-disable-start) after coverage start
                if func.pre_comments:
                    func_lines.append(func.pre_comments)
                # Add the function itself
                func_lines.append(func.full_text)
                # Include post-comments (e.g., slither-disable-end) after function
                if func.post_comments:
                    func_lines.append(func.post_comments)
                # Add coverage exclusion stop marker LAST for non-local functions
                if not is_truly_local:
                    func_lines.append(
                        f"            function exclude_coverage_stop_{func.name}() {{}} // solhint-disable-line no-empty-blocks"
                    )

            # Add imported functions before other content
            return "\n".join(func_lines) + "\n" + "\n".join(result_lines)
        else:
            return "\n".join(result_lines)

    def resolve_import(
        self,
        func_name: str,
        import_path: str,
        current_file: Path,
        processing_stack: List[Path],
        current_block_content: str = "",
        cycle_group: Optional[Set[Path]] = None,
    ) -> Dict[str, YulFunction]:
        """
        Resolve a single import statement.
        Returns dict of function name to YulFunction (only includes requested function and its dependencies).
        Supports 'self' keyword to import from the current file's own assembly block.
        Handles circular dependencies by using cached cycle group functions.
        """
        # Handle 'self' imports - search all assembly blocks in the current file
        if import_path.strip().lower() == "self":
            # Read the entire current file to search all assembly blocks
            if not current_file.exists():
                raise FileNotFoundError(f"Current file not found: {current_file}")

            file_content = current_file.read_text(encoding="utf-8")
            assembly_blocks = self.find_assembly_blocks(file_content)

            # Extract all functions from all assembly blocks in the current file
            all_functions = {}
            external_deps = {}  # Track functions imported from external files

            for _, _, block_content in assembly_blocks:
                functions = self.extract_yul_functions(block_content, current_file)
                all_functions.update(functions)

                # Also resolve any imports within this block to get external dependencies
                lines = block_content.split("\n")
                for line in lines:
                    import_match = self.import_pattern.search(line)
                    if not import_match:
                        continue

                    func_names_str = import_match.group(1)
                    ext_import_path = import_match.group(2)

                    # Skip recursive self imports
                    if ext_import_path.strip().lower() == "self":
                        continue

                    # Parse function names
                    ext_func_names = [
                        name.strip() for name in func_names_str.split(",")
                    ]

                    # Resolve each external import
                    for ext_func_name in ext_func_names:
                        try:
                            ext_funcs = self.resolve_import(
                                ext_func_name,
                                ext_import_path,
                                current_file,
                                processing_stack,
                                "",
                                cycle_group,
                            )
                            external_deps.update(ext_funcs)
                        except Exception:
                            # If we can't resolve, it might be resolved later
                            pass

            # Combine local functions with external dependencies for dependency resolution
            all_functions_with_deps = {**all_functions, **external_deps}

            # Check if the requested function exists
            if func_name not in all_functions:
                available = ", ".join(sorted(all_functions.keys()))
                raise ValueError(
                    f"Function '{func_name}' not found in any assembly block in {current_file.name}\n"
                    f"Available functions: {available}"
                )

            # Return only the requested function and its dependencies
            return self.get_function_dependencies(func_name, all_functions_with_deps)

        # Resolve the file path
        target_file = self.resolve_import_path(import_path, current_file)

        # Check if target is in our current cycle group
        if cycle_group and target_file in cycle_group:
            # This is a circular dependency - use the cached cycle functions
            cycle_key = frozenset(cycle_group)
            if cycle_key in self.cycle_groups:
                all_functions = self.cycle_groups[cycle_key]
                if func_name not in all_functions:
                    available = ", ".join(sorted(all_functions.keys()))
                    raise ValueError(
                        f"Function '{func_name}' not found in circular dependency group\n"
                        f"Available functions: {available}"
                    )
                # Return only the requested function and its dependencies
                return self.get_function_dependencies(func_name, all_functions)

        # If it's a .presl file, process it first
        if target_file.suffix == ".presl":
            processed_content = self.process_file(
                target_file, processing_stack, cycle_group
            )
        else:
            # Regular .sol file, just read it
            if not target_file.exists():
                raise FileNotFoundError(f"Import target not found: {target_file}")
            processed_content = target_file.read_text(encoding="utf-8")

        # Check if target file is part of any cached cycle group
        target_cycle_functions = None
        for cycle_key, cycle_functions in self.cycle_groups.items():
            if target_file in cycle_key:
                target_cycle_functions = cycle_functions
                break

        # If target is part of a cycle, return all functions from that cycle
        if target_cycle_functions is not None:
            if func_name not in target_cycle_functions:
                available = ", ".join(sorted(target_cycle_functions.keys()))
                raise ValueError(
                    f"Function '{func_name}' not found in cycle group\n"
                    f"Available functions: {available}"
                )
            # Return ALL functions from the cycle (they're all potentially interdependent)
            return target_cycle_functions.copy()

        # Find all assembly blocks in the target file
        assembly_blocks = self.find_assembly_blocks(processed_content)

        # Extract all functions from all assembly blocks
        all_functions = {}
        for _, _, block_content in assembly_blocks:
            functions = self.extract_yul_functions(block_content, target_file)
            all_functions.update(functions)

        # Check if the requested function exists
        if func_name not in all_functions:
            available = ", ".join(sorted(all_functions.keys()))
            raise ValueError(
                f"Function '{func_name}' not found in {target_file}\n"
                f"Available functions: {available}"
            )

        # Return only the requested function and its recursive dependencies
        return self.get_function_dependencies(func_name, all_functions)

    def format_with_forge(self, path: Path) -> bool:
        """
        Format .sol files using forge fmt.
        If path is a directory, formats all files in the directory.
        If path is a file, formats just that file.
        Returns True if successful, False otherwise.
        """
        try:
            result = subprocess.run(
                ["forge", "fmt", str(path)],
                capture_output=True,
                text=True,
                timeout=30,
            )
            if result.returncode == 0:
                return True
            else:
                target = "directory" if path.is_dir() else path.name
                print(
                    f"⚠ Warning: forge fmt failed for {target}: {result.stderr}",
                    file=sys.stderr,
                )
                return False
        except FileNotFoundError:
            print(
                "⚠ Warning: forge not found in PATH, skipping formatting",
                file=sys.stderr,
            )
            return False
        except subprocess.TimeoutExpired:
            target = "directory" if path.is_dir() else path.name
            print(f"⚠ Warning: forge fmt timed out for {target}", file=sys.stderr)
            return False
        except Exception as e:
            target = "directory" if path.is_dir() else path.name
            print(f"⚠ Warning: forge fmt error for {target}: {e}", file=sys.stderr)
            return False

    def preprocess_file(
        self,
        input_path: str,
        output_path: Optional[str] = None,
        format_output: bool = True,
    ):
        """
        Preprocess a single .presl file to .post.sol.
        Optionally formats the output with forge fmt.
        """
        input_file = Path(input_path).resolve()

        if output_path:
            output_file = Path(output_path)
        else:
            # Generate output filename
            if input_file.suffix == ".presl":
                output_file = input_file.with_suffix(".post.sol")
            else:
                output_file = input_file.with_suffix(".post.sol")

        print(f"Processing: {input_file}")
        print(f"Output to:  {output_file}")

        try:
            processed_content = self.process_file(input_file)
            output_file.parent.mkdir(parents=True, exist_ok=True)
            output_file.write_text(processed_content, encoding="utf-8")
            print(f"✓ Successfully preprocessed {input_file.name}")

            # Format with forge fmt
            if format_output:
                if self.format_with_forge(output_file):
                    print(f"✓ Formatted with forge fmt")
        except Exception as e:
            print(f"✗ Error processing {input_file}: {e}", file=sys.stderr)
            raise

    def should_skip_file(self, file_path: Path) -> bool:
        """
        Check if a file should be skipped due to having a "// does-not-compile" marker.
        The marker must be in the top few lines with loose spacing requirements.
        """
        try:
            with open(file_path, "r", encoding="utf-8") as f:
                # Check first 10 lines for the marker
                for _ in range(10):
                    line = f.readline()
                    if not line:
                        break
                    # Normalize whitespace and check for the marker
                    normalized = re.sub(r"\s+", "", line.strip().lower())
                    if (
                        "does-not-compile" in normalized
                        or "doesnotcompile" in normalized
                    ):
                        return True
            return False
        except Exception:
            return False

    def preprocess_directory(self, directory: str, format_output: bool = True) -> int:
        """
        Preprocess all .presl files (including .t.presl) in a directory.
        Optionally formats the outputs with forge fmt.
        Returns 0 if all files are preprocessed successfully (except those with "// does-not-compile" marker),
        returns 1 if any file fails to preprocess (excluding marked files).
        """
        dir_path = Path(directory)
        # Find all .presl files (which includes .t.presl since they end in .presl)
        presl_files = sorted(dir_path.rglob("*.presl"))

        print(f"Found {len(presl_files)} .presl files in {directory}")

        output_files = []
        failed = False

        # Process all .presl files (including .t.presl)
        for pre_file in presl_files:
            # Check if file should be skipped
            if self.should_skip_file(pre_file):
                print(
                    f"\n⊘ Skipping: {pre_file.relative_to(dir_path)} (marked as non-compiling)"
                )
                continue

            # Determine output filename: .presl -> .post.sol (works for both .presl and .t.presl)
            output_file = pre_file.with_suffix(".post.sol")

            try:
                print(f"\nProcessing: {pre_file.relative_to(dir_path)}")
                processed_content = self.process_file(pre_file)
                output_file.write_text(processed_content, encoding="utf-8")
                print(f"✓ Output: {output_file.relative_to(dir_path)}")
                output_files.append(output_file)
            except Exception as e:
                print(f"✗ Error: {e}", file=sys.stderr)
                failed = True

        # Format all output files with forge fmt by formatting the entire directory
        if format_output and output_files:
            print(f"\nFormatting {len(output_files)} output files with forge fmt...")
            if self.format_with_forge(dir_path):
                print(f"✓ Formatted all files in directory")
            else:
                print(f"⚠ Warning: formatting failed", file=sys.stderr)

        return 1 if failed else 0


def main():
    if len(sys.argv) < 2:
        print("Usage:")
        print("  python3 yul_preprocessor.py <directory>")
        sys.exit(1)

    directory = sys.argv[1]

    # Verify the argument is a directory
    dir_path = Path(directory)
    if not dir_path.is_dir():
        print(f"Error: '{directory}' is not a directory")
        sys.exit(1)

    preprocessor = YulPreprocessor(root_dir=directory)
    exit_code = preprocessor.preprocess_directory(directory)
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
