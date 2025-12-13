#!/usr/bin/env python3
import argparse
import sys
import os
import subprocess
from concurrent.futures import ThreadPoolExecutor
from typing import List, Tuple, Optional, Any

# Ensure we can import common
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
import common

def run_single_seed(seed: str, extra_args: List[str], output_path: str, capture_output: bool = True) -> Tuple[str, int, Optional[str]]:
    binary = os.path.join(common.ROOT_DIR, "target", "release", "solve")
    cmd = [binary, "--seed", str(seed)] + extra_args
    
    # Explicit output path
    cmd.extend(["--output", output_path])

    print(f"🚀 [Seed {seed}] Running... (Output: {output_path})")
    # Flush stdout to ensure order
    sys.stdout.flush()
    
    output: Optional[str] = None
    ret: int = -1

    try:
        if capture_output:
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
        else:
            # Stream directly to console
            ret = subprocess.call(cmd)
            output = None # Output handled by subprocess
    except Exception as e:
        ret = -1
        output = f"System Error executing subprocess: {e}"
    
    return seed, ret, output

def print_result(seed: str, ret: int, output: Optional[str]) -> None:
    print(f"\n--- Output for Seed {seed} ---")
    
    if output:
        lines = output.splitlines()
        if len(lines) > 50:
            print(f"... (showing last 50 of {len(lines)} lines) ...")
            print("\n".join(lines[-50:]))
        else:
            print(output.rstrip()) # rstrip to avoid double newline if output ends with one
    # If output is None, it was already streamed.

    if ret != 0:
        print(f"❌ [Seed {seed}] Failed with exit code {ret}")
    else:
        print(f"✅ [Seed {seed}] Finished")

def main() -> None:
    parser = argparse.ArgumentParser(description="Run Solver (Multi-Seed Support)", add_help=False)
    parser.add_argument("--seeds", help="Comma-separated list of seeds (e.g., 12345,67890)", default="12345")
    parser.add_argument("--parallel", "-p", action="store_true", help="Run seeds in parallel")
    parser.add_argument("--output-dir", help="Directory for output files.", default="/tmp")
    parser.add_argument("--output-pattern", help="Filename pattern (e.g. 'solve_{}.txt'). '{}' is replaced by seed.", default="solve_{}.txt")
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

    # Special Case: TUI Mode
    if "--tui" in unknown_args:
        print("📺 TUI mode detected. Running directly (Single Seed)...")
        seed = seeds[0]
        binary = os.path.join(common.ROOT_DIR, "target", "release", "solve")
        cmd = [binary, "--seed", str(seed)] + unknown_args
        
        # Run interactively without capture
        try:
            ret = subprocess.call(cmd)
            sys.exit(ret)
        except KeyboardInterrupt:
            sys.exit(130)

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

    # Ensure output dir exists
    if not os.path.exists(args.output_dir):
        try:
            os.makedirs(args.output_dir)
        except OSError as e:
            print(f"❌ Failed to create output directory {args.output_dir}: {e}")
            sys.exit(1)

    # 4. Execution
    results: List[Tuple[str, int, Optional[str]]] = []
    
    if args.parallel and len(seeds) > 1:
        print(f"🚀 Running {len(seeds)} seeds in parallel...")
        with ThreadPoolExecutor(max_workers=min(len(seeds), os.cpu_count() or 4)) as executor:
            futures = []
            for seed in seeds:
                filename = args.output_pattern.format(seed)
                out_path = os.path.join(args.output_dir, filename)
                # Parallel runs MUST capture output to prevent interleaving
                futures.append(executor.submit(run_single_seed, seed, filtered_args, out_path, True))
            
            # Collect results in order (futures list is ordered by seeds)
            for f in futures:
                results.append(f.result())
    else:
        # Sequential
        for seed in seeds:
            filename = args.output_pattern.format(seed)
            out_path = os.path.join(args.output_dir, filename)
            # If running a single seed alone, don't capture (let it stream)
            # If running multiple seeds sequentially, we could stream, but let's capture to be safe/clean?
            # User request: "when a single seed is passed, we don't capture stdout"
            should_capture = len(seeds) > 1
            results.append(run_single_seed(seed, filtered_args, out_path, should_capture))

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
