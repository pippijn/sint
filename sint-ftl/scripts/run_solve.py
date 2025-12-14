#!/usr/bin/env python3
import argparse
import sys
import os
import subprocess
import re
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
            # We still need to capture it if we want to parse it for the summary
            # but the user said for single seed don't capture.
            # Let's compromise: if we need to parse it, we capture it.
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
            print(output)
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

def print_summary(results: List[Tuple[str, int, Optional[str]]]) -> None:
    print("\n" + "="*80)
    print(f"{'AGGREGATE PERFORMANCE REPORT':^80}")
    print("="*80)
    print(f"{'Seed':<10} | {'Result':<10} | {'Dmg':<5} | {'Rem':<5} | {'Hull':<5} | {'Rounds':<7} | {'Fitness Score':<15}")
    print("-" * 80)

    total_fitness = 0.0
    victories = 0
    total_hull = 0
    total_damage = 0
    total_rem_hp = 0
    total_rounds = 0
    count = 0

    for seed, ret, output in results:
        if not output:
            continue
        
        count += 1
        # Extraction
        phase_match = re.search(r"Final Phase: (\w+)", output)
        hull_match = re.search(r"Hull: (-?\d+)", output)
        damage_match = re.search(r"Total Damage: (\d+)", output)
        rem_hp_match = re.search(r"Total Enemy HP: (\d+)", output)
        rounds_match = re.search(r"Rounds: (\d+)", output)
        fitness_match = re.search(r"Fitness Score: ([\d.-]+)", output)

        phase = phase_match.group(1) if phase_match else "Unknown"
        hull = int(hull_match.group(1)) if hull_match else 0
        damage = int(damage_match.group(1)) if damage_match else 0
        rem_hp = int(rem_hp_match.group(1)) if rem_hp_match else 0
        rounds = int(rounds_match.group(1)) if rounds_match else 0
        fitness = float(fitness_match.group(1)) if fitness_match else 0.0

        win = phase == "Victory"
        if win:
            victories += 1
        
        result_str = "WIN" if win else "LOSS"
        print(f"{seed:<10} | {result_str:<10} | {damage:<5} | {rem_hp:<5} | {hull:<5} | {rounds:<7} | {fitness:<15.1f}")

        total_fitness += fitness
        total_hull += hull
        total_damage += damage
        total_rem_hp += rem_hp
        total_rounds += rounds

    if count > 0:
        print("-" * 80)
        avg_fitness = total_fitness / count
        avg_hull = total_hull / count
        avg_damage = total_damage / count
        avg_rem_hp = total_rem_hp / count
        avg_rounds = total_rounds / count
        win_rate = (victories / count) * 100

        print(f"{'AVERAGE':<10} | {win_rate:>5.1f}% WIN | {avg_damage:<5.1f} | {avg_rem_hp:<5.1f} | {avg_hull:<5.1f} | {avg_rounds:<7.1f} | {avg_fitness:<15.1f}")
    
    print("="*80 + "\n")

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

    # 6. Summary
    print_summary(results)

    if failed:
        sys.exit(1)

if __name__ == "__main__":
    main()
