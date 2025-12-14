#!/usr/bin/env python3
import sys
import os

# Add ai directory to sys.path
sys.path.append(os.path.join(os.path.dirname(__file__), "..", "..", "ai"))

from bindings_wrapper import SintBindings, SolverBindings
from game_types import GamePhase

def main() -> None:
    from solution_rounds import rounds
    seed = 2236
    player_ids = ["P1", "P2", "P3", "P4", "P5", "P6"]
    
    print("--- EXECUTING SOLUTION (SEED BASED) ---")
    
    result = SolverBindings.verify_solution(player_ids, seed, rounds)
    
    if not result["success"]:
        print("❌ FAILURE!")
        print(result.get("failure_summary"))
        if result.get('history'):
                # We still need a state for trajectory logging if we want to see it
                from bindings_wrapper import SintBindings
                state = SintBindings.new_game(player_ids, seed)
                history = result['history']
                logs = SolverBindings.get_trajectory_log(state, history)
                for l in logs:
                    print(l, end='')
        sys.exit(1)
            
    print("🎉 VICTORY!")

if __name__ == "__main__":
    main()