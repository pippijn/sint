#!/usr/bin/env python3
import os
import sys
import argparse

# Ensure we can import common
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
import common

def main():
    if not os.environ.get("GEMINI_API_KEY"):
         print("⚠️  GEMINI_API_KEY not set. Solver requires it.")

    parser = argparse.ArgumentParser(description="Run Gemini Solver")
    parser.add_argument("--rebuild", action="store_true", help="Force rebuild of bindings")
    parser.add_argument("--debug", action="store_true", help="Enable debug logging")
    parser.add_argument("--seed", type=int, default=12345, help="RNG Seed")
    parser.add_argument("--max-rounds", type=int, default=30, help="Max rounds to simulate")
    args = parser.parse_args()

    common.check_venv()
    common.build_bindings(force=args.rebuild)

    print(f"\n🧠 Starting Gemini Solver (Seed: {args.seed})...")
    
    # Run the solver script
    # We pass the args via command line to the script? 
    # Or import and run? Importing is better if the script is designed for it, 
    # but `gemini_solver.py` currently has `if __name__ == "__main__": solver.run_full_game()`.
    # It doesn't parse args.
    # I should update `gemini_solver.py` to parse args OR just pass them manually here if I modify it.
    
    # For now, let's just run it as a subprocess to keep environment clean, 
    # but since `gemini_solver.py` doesn't take args yet, I'll modify it first 
    # to accept args, OR I can just edit `gemini_solver.py` to use argparse.
    
    # Better: Update gemini_solver.py to use argparse, then call it here.
    cmd = f"{common.PYTHON_EXEC} solver/solutions/gemini_solver.py --seed {args.seed} --max-rounds {args.max_rounds}"
    if args.debug:
        cmd += " --debug"
        
    common.run_cmd(cmd)

if __name__ == "__main__":
    main()
