#!/bin/bash
# Check for command-line argument
BENCHMARK_MODE=${1:-"a"}  # Default to "a" if no argument provided

# Define the benchmark parameters
case $BENCHMARK_MODE in
  d|daily)
    echo "Running daily benchmark suite"
    TABLE_SIZES=(10000 100000 1000000 10000000 100000000)
    QUERIES=("filter" "arithmetic" "group-by" "join")
    RUN_DYNAMIC_DORY=true
    FLAGS=""
    ;;
  m|marketing)
    echo "Running marketing benchmark suite"
    TABLE_SIZES=(
      10000 20000 30000 40000 50000 60000 70000 80000 90000 100000
      110000 120000 130000 140000 150000 160000 170000 180000 190000
      200000 400000 600000 800000 1000000 3000000 6000000 10000000
    )
    QUERIES=("filter" "complex-filter" "group-by" "join")
    RUN_DYNAMIC_DORY=false
    FLAGS="-r 0 -i 10"
    ;;
  a|all)
    echo "Running full benchmark suite"
    TABLE_SIZES=(10000 100000 1000000 10000000 100000000)
    QUERIES=("all")
    RUN_DYNAMIC_DORY=true
    FLAGS=""
    ;;
  *)
    echo "Usage: ./run_benchmarks.sh [mode]"
    echo "  Modes:"
    echo "    d, daily     - Quick benchmarks for daily testing"
    echo "    m, marketing - Benchmarks for reports/presentations"
    echo "    a, all       - Full benchmarks"
    exit 1
    ;;
esac

# Get the absolute path to the project root
PROJECT_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
DATA_DIR="$PROJECT_ROOT/crates/proof-of-sql-benches/data"

# Create a "data" directory if it doesn't already exist
mkdir -p "$DATA_DIR"

# Get the current timestamp in the format "YYYY-MM-DD_HH-MM-SS"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")

# Export the CSV_PATH environment variable
export CSV_PATH="$PROJECT_ROOT/crates/proof-of-sql-benches/data/results_${TIMESTAMP}.csv"
echo "Saving results at: ${CSV_PATH}"

# Run the benchmarks
cd "$PROJECT_ROOT"
echo "Running Hyper-KZG benchmarks..."
cargo clean && cargo update
for TABLE_SIZE in "${TABLE_SIZES[@]}"; do
  for QUERY in "${QUERIES[@]}"; do
    cargo run --release --bin proof-of-sql-benches -- -s "hyper-kzg" -t "$TABLE_SIZE" -q "$QUERY" $FLAGS
  done
done

# Run Dynamic Dory benchmarks if requested
if $RUN_DYNAMIC_DORY; then
  echo "Running Dynamic Dory benchmarks..."
  
  # Check if blitzar_handle_nu_16.bin and public_params_nu_16.bin exist
  if [[ ! -f "$DATA_DIR/blitzar_handle_nu_16.bin" ]] || \
     [[ ! -f "$DATA_DIR/public_params_nu_16.bin" ]]; then
     echo "Downloading required parameter files..."
     cd "$DATA_DIR"
     
     for part in aa ab ac ad; do
       echo "Downloading part $part of blitzar_handle_nu_16.bin..."
       wget -q --show-progress https://github.com/spaceandtimelabs/sxt-proof-of-sql/releases/download/dory-prover-params-nu-16/blitzar_handle_nu_16.bin.part.$part
     done
     
     echo "Downloading public_parameters_nu_16.bin..."
     wget -q --show-progress https://github.com/spaceandtimelabs/sxt-proof-of-sql/releases/download/dory-prover-params-nu-16/public_parameters_nu_16.bin
     
     echo "Combining parts into blitzar_handle_nu_16.bin..."
     cat blitzar_handle_nu_16.bin.part.* > blitzar_handle_nu_16.bin
     rm blitzar_handle_nu_16.bin.part.*
     
     # Fix the filename if needed
     if [[ ! -f "$DATA_DIR/public_params_nu_16.bin" ]]; then
       mv public_parameters_nu_16.bin public_params_nu_16.bin || true
     fi
     
     echo "Download complete."
  fi

  # Set these environment variables outside the if block to ensure they're available
  export BLITZAR_HANDLE_PATH="$DATA_DIR/blitzar_handle_nu_16.bin"
  export DORY_PUBLIC_PARAMS_PATH="$DATA_DIR/public_params_nu_16.bin"

  # Note: Dynamic Dory with nu-16 cannot handle table sizes larger than ~10 million
  DORY_TABLE_SIZES=()
  for size in "${TABLE_SIZES[@]}"; do
    if [ "$size" -le 10000000 ]; then
      DORY_TABLE_SIZES+=($size)
    fi
  done

  # Run the Dynamic Dory benchmarks
  cd "$PROJECT_ROOT"
  for TABLE_SIZE in "${DORY_TABLE_SIZES[@]}"; do
    for QUERY in "${QUERIES[@]}"; do
      cargo run --release --bin proof-of-sql-benches -- -s "dynamic-dory" -t "$TABLE_SIZE" -q "$QUERY" $FLAGS
    done
  done
fi

echo "All benchmarks completed. Results saved to: $CSV_PATH"
