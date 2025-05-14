#!/bin/bash

# Get the absolute path to the project root
PROJECT_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"

# Create a "data" directory if it doesn't already exist
mkdir -p "$PROJECT_ROOT/crates/proof-of-sql-benches/data"

# Get the current timestamp in the format "YYYY-MM-DD_HH-MM-SS"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")

# Export the CSV_PATH environment variable
export CSV_PATH="$PROJECT_ROOT/crates/proof-of-sql-benches/data/results_${TIMESTAMP}.csv"
echo "Saving results at: ${CSV_PATH}"

# Define the schemes and table sizes to iterate over
SCHEMES=("hyper-kzg")
TABLE_SIZES=(
  10000 20000 30000 40000 50000 60000 70000 80000 90000 100000
  110000 120000 130000 140000 150000 160000 170000 180000 190000
  200000 400000 600000 800000 1000000 3000000 6000000 10000000
)

# Define the queries to run
QUERIES=("filter" "complex-filter" "group-by")

# Run the benchmarks
cd "$PROJECT_ROOT"
for SCHEME in "${SCHEMES[@]}"; do
  for QUERY in "${QUERIES[@]}"; do
    for TABLE_SIZE in "${TABLE_SIZES[@]}"; do
      cargo run --release --bin proof-of-sql-benches -- -s "$SCHEME" -t "$TABLE_SIZE" -r 0 -i 10 -q "$QUERY"
    done
  done
done

# Join query
# The data gets doubled - 1/2 of the table size is equivalent to the table size for other queries
for SCHEME in "${SCHEMES[@]}"; do
  for TABLE_SIZE in "${TABLE_SIZES[@]}"; do
    HALF_TABLE_SIZE=$((TABLE_SIZE / 2))
    cargo run --release --bin proof-of-sql-benches -- -s "$SCHEME" -t "$HALF_TABLE_SIZE" -r 0 -i 10 -q join
  done
done
