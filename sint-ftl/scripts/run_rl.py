#!/usr/bin/env python3
import os
import sys
import argparse

# Ensure we can import common
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
import common

def main() -> None:
    parser = argparse.ArgumentParser(description="Run RL Training/Inference")
    parser.add_argument("--rebuild", action="store_true", help="Force rebuild of bindings")
    parser.add_argument("--train", action="store_true", help="Train a new model")
    parser.add_argument("--eval", action="store_true", help="Evaluate an existing model")
    parser.add_argument("--model", type=str, default="solver/rl/models/ppo_sint", help="Path to model")
    parser.add_argument("--steps", type=int, default=100000, help="Number of training steps")
    
    args = parser.parse_args()

    common.check_venv()
    common.install_requirements()
    common.build_bindings(force=args.rebuild)

    # Ensure the models directory exists
    os.makedirs("solver/rl/models", exist_ok=True)

    if args.train:
        print(f"\n🧠 Starting RL Training for {args.steps} steps...")
        common.run_cmd(f"{common.PYTHON_EXEC} solver/rl/train.py --steps {args.steps} --output {args.model}")
    elif args.eval:
        print(f"\n🎮 Evaluating RL Model: {args.model}")
        common.run_cmd(f"{common.PYTHON_EXEC} solver/rl/evaluate.py --model {args.model}")
    else:
        parser.print_help()

if __name__ == "__main__":
    main()

