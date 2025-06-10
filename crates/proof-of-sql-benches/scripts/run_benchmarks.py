#!/usr/bin/env python3
import os
import sys
import subprocess
import datetime
import argparse
from pathlib import Path
import shutil
import time
from typing import List, Dict, Tuple, Optional

# Parameters
BLITZAR_HANDLE = "blitzar_handle_nu_16.bin"
PUBIC_PARAMETERS = "public_parameters_nu_16.bin"

class BenchmarkConfig:
    def __init__(self, queries: List[str], table_sizes: List[int], 
                 run_dynamic_dory: bool, flags: str):
        self.queries = queries
        self.table_sizes = table_sizes
        self.run_dynamic_dory = run_dynamic_dory
        self.flags = flags

def get_benchmark_config(mode: str) -> BenchmarkConfig:
    """Get benchmark configuration based on mode."""
    if mode == 'd':
        return BenchmarkConfig(
            queries=["filter", "arithmetic", "group-by", "join"],
            table_sizes=[10000, 100000, 1000000, 10000000, 100000000],
            run_dynamic_dory=True,
            flags=""
        )
    elif mode == 'm':
        return BenchmarkConfig(
            queries=["filter", "complex-filter", "group-by", "join"],
            table_sizes=[
                10000, 20000, 30000, 40000, 50000, 60000, 70000, 80000, 90000, 100000,
                110000, 120000, 130000, 140000, 150000, 160000, 170000, 180000, 190000,
                200000, 400000, 600000, 800000, 1000000, 3000000, 6000000, 10000000
            ],
            run_dynamic_dory=False,
            flags="-r 0 -i 10"
        )
    elif mode == 'a':
        return BenchmarkConfig(
            queries=["all"],
            table_sizes=[10000, 100000, 1000000, 10000000, 100000000],
            run_dynamic_dory=True,
            flags=""
        )
    else:
        print(f"Unknown mode: {mode}")
        sys.exit(1)

def setup_environment() -> Tuple[Path, Path, Path]:
    """Set up the environment and return important paths."""
    script_path = Path(os.path.dirname(os.path.realpath(__file__)))
    project_root = script_path.parent.parent.parent
    data_dir = project_root / "crates" / "proof-of-sql-benches" / "data"
    
    # Create data directory if it doesn't exist
    data_dir.mkdir(parents=True, exist_ok=True)
    
    # Set up CSV output path as results_<timestamp>.csv
    timestamp = datetime.datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
    csv_path = data_dir / f"results_{timestamp}.csv"
    os.environ["CSV_PATH"] = str(csv_path)
    print(f"Saving results at: {csv_path}")
    
    return project_root, data_dir, csv_path

def run_command(cmd: List[str], check: bool = True) -> bool:
    """Run a shell command and handle errors appropriately."""
    try:
        subprocess.run(cmd, check=check)
        return True
    except subprocess.CalledProcessError:
        print(f"ERROR: Command failed: {' '.join(cmd)}")
        return False

def download_dory_files(data_dir: Path) -> bool:
    """Download and prepare Dynamic Dory files."""
    download_success = True
    blitzar_handle_path = data_dir / BLITZAR_HANDLE
    public_params_path = data_dir / PUBIC_PARAMETERS
    
    # Check if required files exist, download if not
    if not blitzar_handle_path.exists() or not public_params_path.exists():
        print("Downloading required parameter files...")
        os.chdir(data_dir)
        
        # Download Blitzar handle parts
        for part in ['aa', 'ab', 'ac', 'ad']:
            url = f"https://github.com/spaceandtimelabs/sxt-proof-of-sql/releases/download/dory-prover-params-nu-16/blitzar_handle_nu_16.bin.part.{part}"
            if not run_command(["wget", "-q", "--show-progress", url]):
                download_success = False
                break
        
        # Download public parameters if Blitzar handle parts download succeeded
        if download_success:
            url = "https://github.com/spaceandtimelabs/sxt-proof-of-sql/releases/download/dory-prover-params-nu-16/public_parameters_nu_16.bin"
            if not run_command(["wget", "-q", "--show-progress", url]):
                download_success = False
        
        # Combine Blitzar handle parts if downloads succeeded
        if download_success:
            print("Combining parts into blitzar_handle_nu_16.bin...")
            try:
                with open(blitzar_handle_path, 'wb') as outfile:
                    for part in ['aa', 'ab', 'ac', 'ad']:
                        part_file = data_dir / f"blitzar_handle_nu_16.bin.part.{part}"
                        with open(part_file, 'rb') as infile:
                            shutil.copyfileobj(infile, outfile)
            except Exception as e:
                print(f"ERROR: Failed to combine file parts: {e}")
                download_success = False
            
            # Clean up part files if combination was successful
            if download_success:
                for part in ['aa', 'ab', 'ac', 'ad']:
                    part_file = data_dir / f"blitzar_handle_nu_16.bin.part.{part}"
                    try:
                        os.remove(part_file)
                    except Exception:
                        print(f"Warning: Could not remove part file {part_file}")
        
        if download_success:
            print("Download complete.")
        else:
            print("Download failed. Cannot run Dynamic Dory benchmarks.")
    
    return download_success and blitzar_handle_path.exists() and public_params_path.exists()

def run_hyper_kzg_benchmarks(project_root: Path, config: BenchmarkConfig):
    """Run Hyper-KZG benchmarks."""
    print("Running Hyper-KZG benchmarks...")
    os.chdir(project_root)
    run_command(["cargo", "clean"])
    run_command(["cargo", "update"])
    
    for table_size in config.table_sizes:
        for query in config.queries:
            cmd = ["cargo", "run", "--release", "--bin", "proof-of-sql-benches", 
                  "--", "-s", "hyper-kzg", "-t", str(table_size), "-q", query]
            if config.flags:
                cmd.extend(config.flags.split())
            run_command(cmd)

def run_dynamic_dory_benchmarks(project_root: Path, data_dir: Path, config: BenchmarkConfig):
    """Run Dynamic Dory benchmarks if possible."""
    if not config.run_dynamic_dory:
        print("Skipping Dynamic Dory benchmarks (not requested)")
        return
        
    print("Running Dynamic Dory benchmarks...")
    
    # Download necessary files
    if not download_dory_files(data_dir):
        return
    
    # Set environment variables for Dynamic Dory
    os.environ["BLITZAR_HANDLE_PATH"] = str(data_dir / BLITZAR_HANDLE)
    os.environ["DORY_PUBLIC_PARAMS_PATH"] = str(data_dir / PUBIC_PARAMETERS)
    
    # Filter table sizes for Dynamic Dory (10,000,000 max)
    dory_table_sizes = [size for size in config.table_sizes if size <= 10000000]
    
    # Run the Dynamic Dory benchmarks
    os.chdir(project_root)

    run_command(["cargo", "clean"])
    run_command(["cargo", "update"])

    for table_size in dory_table_sizes:
        for query in config.queries:
            cmd = ["cargo", "run", "--release", "--bin", "proof-of-sql-benches", 
                  "--", "-s", "dynamic-dory", "-t", str(table_size), "-q", query]
            if config.flags:
                cmd.extend(config.flags.split())
            run_command(cmd)

def main():
    # Begin benchmark timer
    start_time = time.time()

    # Parse command-line arguments
    parser = argparse.ArgumentParser(description='Run proof-of-sql-benches')
    parser.add_argument('mode', nargs='?', choices=['d', 'daily', 'm', 'marketing', 'a', 'all'], 
                        default='a', help='Benchmark mode to run')
    args = parser.parse_args()
    
    # Get mode and benchmark configuration
    benchmark_mode = args.mode[0]
    config = get_benchmark_config(benchmark_mode)
    
    # Setup environment 
    project_root, data_dir, csv_path = setup_environment()
    
    # Run benchmarks
    run_hyper_kzg_benchmarks(project_root, config)
    run_dynamic_dory_benchmarks(project_root, data_dir, config)

    # End benchmark timer
    end_time = time.time()
    execution_time = end_time - start_time
    hours, remainder = divmod(execution_time, 3600)
    minutes, seconds = divmod(remainder, 60)
    
    print(f"All benchmarks completed. Results saved to: {csv_path}")
    print(f"Total execution time: {int(hours):02d}:{int(minutes):02d}:{seconds:.2f}")

if __name__ == "__main__":
    main()
