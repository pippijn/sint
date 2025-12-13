#!/usr/bin/env python3
import argparse
import sys
import os
import subprocess
from concurrent.futures import ThreadPoolExecutor

# Ensure we can import common
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
import common

def run_single_seed(seed, extra_args, output_override=None):
    binary = os.path.join(common.ROOT_DIR, "target", "release", "solve")
    cmd = [binary, "--seed", str(seed)] + extra_args
    
    # Handle output override
    if output_override:
        cmd.extend(["--output", output_override])

    print(f"🚀 [Seed {seed}] Running...")
    # Flush stdout to ensure order
    sys.stdout.flush()
    
    try:
        # Capture output in memory (stdout + stderr)
        result = subprocess.run(
            cmd, 
            stdout=subprocess.PIPE, 
            stderr=subprocess.STDOUT, 
            text=True, 
            encoding='utf-8', 
            errors='replace'
        )
        ret = result.returncode
        output = result.stdout
    except Exception as e:
        ret = -1
        output = f"System Error executing subprocess: {e}"
    
    return seed, ret, output

def print_result(seed, ret, output):
    print(f"\n--- Output for Seed {seed} ---")
    
    if output:
        lines = output.splitlines()
        if len(lines) > 50:
            print(f"... (showing last 50 of {len(lines)} lines) ...")
            print("\n".join(lines[-50:]))
        else:
            print(output.rstrip()) # rstrip to avoid double newline if output ends with one

    if ret != 0:
        print(f"❌ [Seed {seed}] Failed with exit code {ret}")
    else:
        print(f"✅ [Seed {seed}] Finished")

def main():
    parser = argparse.ArgumentParser(description="Run Solver (Multi-Seed Support)", add_help=False)
    parser.add_argument("--seeds", help="Comma-separated list of seeds (e.g., 12345,67890)", default="12345")
    parser.add_argument("--parallel", "-p", action="store_true", help="Run seeds in parallel")
    parser.add_argument("--output-pattern", help="Pattern for output file (e.g. 'solve_{}.txt'). '{}' is replaced by seed.", default="solve_{}.txt")
    parser.add_argument("--help", action="store_true", help="Show help")
    
    # Parse known args, leave the rest for the solver binary
    args, unknown_args = parser.parse_known_args()

    if args.help:
        parser.print_help()
        print("\n--- Solver Binary Help ---")
        # Ensure binary exists or build it just to show help? 
        # Easier to just run cargo run --help but that might be slow.
        # Let's assume the user knows or we can try running it if built.
        subprocess.call(["cargo", "run", "--release", "--bin", "solve", "--", "--help"])
        return

    # 1. Build
    print("🔨 Building solver binary...")
    common.run_cmd("cargo build --release --bin solve")

    # 2. Parse Seeds
    seeds = [s.strip() for s in args.seeds.split(',') if s.strip()]
    if not seeds:
        print("❌ No seeds provided.")
        sys.exit(1)

    # 3. Handle Arguments
    # Always filter out manual --output flags to avoid conflict/confusion
    new_args = []
    skip_next = False
    for arg in unknown_args:
        if skip_next:
            skip_next = False
            continue
        if arg in ["--output", "-o"]:
            skip_next = True # Skip the value
            continue
        # Handle joined syntax like --output=file.txt ?
        if arg.startswith("--output=") or arg.startswith("-o="):
            continue
        new_args.append(arg)
    filtered_args = new_args

    # 4. Execution
    results = []
    
    if args.parallel and len(seeds) > 1:
        print(f"🚀 Running {len(seeds)} seeds in parallel...")
        with ThreadPoolExecutor(max_workers=min(len(seeds), os.cpu_count() or 4)) as executor:
            futures = []
            for seed in seeds:
                out_file = args.output_pattern.format(seed) if args.output_pattern else None
                futures.append(executor.submit(run_single_seed, seed, filtered_args, out_file))
            
            # Collect results in order (futures list is ordered by seeds)
            for f in futures:
                results.append(f.result())
    else:
        # Sequential
        for seed in seeds:
            out_file = args.output_pattern.format(seed) if args.output_pattern else None
            results.append(run_single_seed(seed, filtered_args, out_file))

    # 5. Report
    failed = False
    for seed, ret, output in results:
        print_result(seed, ret, output)
        if ret != 0:
            failed = True

    if failed:
        sys.exit(1)

if __name__ == "__main__":
    main()
