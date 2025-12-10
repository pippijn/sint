#!/usr/bin/env python3
import os
import sys
import argparse

# Ensure we can import common
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
import common

def main():
    if not os.environ.get("GEMINI_API_KEY"):
         print("⚠️  GEMINI_API_KEY not set. AI Agent requires it.")

    parser = argparse.ArgumentParser(description="Run AI Agent")
    parser.add_argument("--rebuild", action="store_true", help="Force rebuild of bindings")
    # Capture unknown args to pass to agent
    args, agent_args = parser.parse_known_args()

    common.check_venv()
    common.build_bindings(force=args.rebuild)

    # Reconstruct agent args string
    agent_args_str = " ".join(agent_args)
    if not agent_args_str:
         agent_args_str = "--player AI_Bot --room Room_A"

    try:
        print(f"\n🤖 Starting AI Agent with args: {agent_args_str}")
        common.run_cmd(f"{common.PYTHON_EXEC} ai/agent.py {agent_args_str}")
    except KeyboardInterrupt:
        print("\n🛑 Stopping...")

if __name__ == "__main__":
    main()