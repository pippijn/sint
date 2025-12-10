#!/usr/bin/env python3
import os
import sys
import argparse

# Ensure we can import common
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
import common

def main():
    parser = argparse.ArgumentParser(description="Run Solution Verifier")
    parser.add_argument("--rebuild", action="store_true", help="Force rebuild of bindings")
    args = parser.parse_args()

    common.check_venv()
    common.build_bindings(force=args.rebuild)

    print("\n📜 Executing Solution Script...")
    common.run_cmd(f"{common.PYTHON_EXEC} solver/solutions/execute_solution.py")

if __name__ == "__main__":
    main()