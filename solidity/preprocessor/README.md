# Yul Preprocessor

A preprocessor for Solidity files that resolves Yul function imports within assembly blocks, enabling better code organization and reusability for inline assembly code.

## Features

- **Import Yul functions** from other `.presl` files into your assembly blocks
- **Multiple imports per line**: Import several functions in a single statement
- **Self imports**: Reference functions from different assembly blocks in the same file
- **Relative path support**: Import from files using relative paths
- **Circular dependencies allowed**: Files can import from each other - circular dependency groups are automatically resolved
- **Transitive dependency resolution**: Importing from a file automatically includes all its dependencies
- **Function deduplication**: Automatically deduplicates identical function imports
- **Caching**: Efficiently processes files with intelligent caching
- **Automatic formatting**: Runs `forge fmt` on output files for clean, consistent formatting

## Installation

No installation required! Just use Python 3.6+:

```bash
python3 yul_preprocessor.py <directory>
```

**Optional**: Install [Foundry](https://book.getfoundry.sh/getting-started/installation) for automatic formatting of output files with `forge fmt`.

## Usage

### Basic Usage

Process all `.presl` and `.t.presl` files in a directory:
```bash
python3 yul_preprocessor.py ./contracts
```

The preprocessor processes both `.presl` and `.t.presl` files (for test files), generating corresponding `.post.sol` and `.t.post.sol` output files respectively.

The preprocessor automatically runs `forge fmt` on all generated `.post.sol` files to ensure clean, consistent formatting. If `forge` is not available in your PATH, the preprocessor will skip formatting with a warning.

### Import Syntax

The preprocessor supports three import patterns:

#### 1. Single Function Import
```solidity
// import <function_name> from <file_path>
```

Example:
```solidity
assembly {
    // import add5 from utils.presl
    let result := add5(10)
}
```

#### 2. Multiple Functions Per Line
```solidity
// import <func1>, <func2>, <func3> from <file_path>
```

Example:
```solidity
assembly {
    // import add, multiply, divide from math.presl
    let sum := add(5, 10)
    let product := multiply(3, 7)
}
```

#### 3. Self Import
```solidity
// import <function_name> from self
```

Import functions from a different assembly block in the same file:
```solidity
contract Example {
    function defineHelpers() external pure {
        assembly {
            function helper(x) -> result {
                result := mul(x, 2)
            }
        }
    }

    function useHelpers() external pure {
        assembly {
            // import helper from self
            let doubled := helper(5)
        }
    }
}
```

### Relative Paths

Import from subdirectories or parent directories:

```solidity
// import compute_fold from ../base/MathUtil.presl
// import err from ./errors/Errors.sol
// import safe_add from lib/SafeMath.presl
```

## Complete Example

### Source File: `utils.presl`
```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Utils {
    function process() external pure returns (uint256) {
        assembly {
            function add5(x) -> result {
                result := add(x, 5)
            }

            function multiply2(x) -> result {
                result := mul(x, 2)
            }
        }
    }
}
```

### Target File: `main.presl`
```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Main {
    function compute() external pure returns (uint256) {
        assembly {
            // import add5, multiply2 from utils.presl
            let a := add5(10)        // a = 15
            let b := multiply2(a)    // b = 30
        }
    }
}
```

### Output File: `main.post.sol`
```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Main {
    function compute() external pure returns (uint256) {
        assembly {
            function add5(x) -> result {
                result := add(x, 5)
            }
            function multiply2(x) -> result {
                result := mul(x, 2)
            }

            let a := add5(10)        // a = 15
            let b := multiply2(a)    // b = 30
        }
    }
}
```

## File Naming Convention

- **Input files**:
  - `*.presl` - Standard files with import statements
  - `*.t.presl` - Test files with import statements
- **Output files**:
  - `*.post.sol` - Processed standard files with imports resolved
  - `*.t.post.sol` - Processed test files with imports resolved

The preprocessor automatically generates `.post.sol` files from `.presl` files and `.t.post.sol` files from `.t.presl` files.

## Circular Dependencies

Circular dependencies are **fully supported**! When files A and B import from each other, they are processed as a unified dependency group.

### How It Works

When a circular dependency is detected (e.g., A imports from B, B imports from A), the preprocessor:

1. Identifies all files in the circular group
2. Collects all Yul functions from all files in the group
3. Makes the complete set of functions available to every file in the group
4. Ensures all assembly blocks in the cycle have identical function sets

### Example

**file_a.presl:**
```solidity
assembly {
    // import funcB from file_b.presl
    function funcA() -> result { result := 1 }
}
```

**file_b.presl:**
```solidity
assembly {
    // import funcA from file_a.presl
    function funcB() -> result { result := 2 }
}
```

After processing, both files will have both `funcA` and `funcB` in their assembly blocks.

### Nested Circular Dependencies

The preprocessor handles complex scenarios where:
- Group C imports from Group B (B0, B1) which are mutually dependent
- Group B imports from Group A (A0, A1) which are mutually dependent
- Group A imports from standalone files

All transitive dependencies are correctly resolved and propagated.

## Error Handling

The preprocessor detects and reports several types of errors:

### Missing Functions
```
ValueError: Function 'nonExistent' not found in utils.presl
Available functions: add5, multiply2
```

### Function Signature Mismatch
```
ValueError: Function signature mismatch for 'add':
  Existing: function add(a, b) -> result
  New:      function add(x) -> result
```

## Architecture

### Key Components

1. **YulFunction**: Represents a parsed Yul function with name, signature, body, and full text
2. **YulPreprocessor**: Main processor class that handles:
   - File parsing and processing
   - Import resolution
   - Function extraction
   - Caching

### Processing Flow

```
Input .presl file
        ↓
Find assembly blocks
        ↓
For each assembly block:
  - Parse import statements
  - Resolve imported functions
  - Process dependencies recursively
  - Deduplicate functions
  - Insert functions at block start
        ↓
Generate .post.sol file
```

## Testing

Run the test suite:

```bash
python3 -m pytest test_yul_preprocessor.py -v
```

Test coverage includes:
- Basic imports
- Multiple imports per line
- Self imports
- Relative path imports
- Function deduplication
- Circular dependency detection
- Missing function errors
- Complex function signatures
- Multiple assembly blocks
- Caching

## Advanced Features

### Function Deduplication

If the same function is imported multiple times, only one copy is included:

```solidity
assembly {
    // import add from math.presl
    // import add from math.presl  // Deduplicated
    let x := add(1, 2)
}
```

### Multiple Assembly Blocks

The preprocessor handles multiple assembly blocks within a single contract:

```solidity
contract Multi {
    function first() external pure {
        assembly {
            // import func1 from lib.presl
        }
    }

    function second() external pure {
        assembly {
            // import func2 from lib.presl
        }
    }
}
```

### Caching

Processed files are cached to improve performance when the same file is imported multiple times in a dependency tree.

## Behavior Notes

- **Transitive Dependencies**: When you import a function from a file, you automatically get all functions from that file's assembly block. This ensures the complete dependency closure is available.
- **Import statements** must be on a single line
- **Import syntax** must follow: `// import <names> from <path>`
- Only **Yul functions** within `assembly {}` blocks are extracted and imported

## Contributing

To add new features or fix bugs:

1. Add test cases in `test_files/` directory
2. Update `test_yul_preprocessor.py` with corresponding tests
3. Implement changes in `yul_preprocessor.py`
4. Run tests to ensure everything passes
