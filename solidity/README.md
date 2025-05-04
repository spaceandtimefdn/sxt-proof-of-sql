# Development Dependencies Installation
1. `forge`
    ```bash
    curl -L https://foundry.paradigm.xyz | bash
    foundryup
    ```
2. `lcov`/`genhtml`
    ```bash
    sudo apt install lcov
    ```
3. `solhint`
    ```bash
    npm install -g solhint
    ```
4. `slither`
    ```bash
    pipx install slither-analyzer
    ```
5. `aderyn` (Recommended)
    ```bash
    npm install -g @cyfrin/aderyn
    ```

# Build and Test
To generate the solidity library artifact,

```bash
./scripts/preprocess_yul_imports.sh src
```

The final artifact is `./src/verifier/Verifier.t.post.sol`.


To run all tests and lints:

```bash
./scripts/lint-and-test.sh
```