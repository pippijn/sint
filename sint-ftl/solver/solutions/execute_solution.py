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
    state = SintBindings.new_game(player_ids, seed)
    
    print("--- STARTING GAME ---")
    
    # We verify the whole solution up to round i
    for i in range(len(rounds)):
        current_rounds = rounds[:i+1]
        result = SolverBindings.verify_solution(state, current_rounds)
        
        if not result["success"] and not (result.get("failed_action") is None and result.get("error") is None):
            print(f"❌ FAILURE in block {i+1}!")
            print(result.get("failure_summary"))
            if result.get('history'):
                 history = result['history']
                 logs = SolverBindings.get_trajectory_log(state, history)
                 for l in logs:
                      print(l, end='')
            sys.exit(1)
            
        final_state = result['final_state']
        print(f"--- AFTER ROUND {i+1} ---")
        print(SolverBindings.format_game_state(final_state))
        
        if final_state['phase'] == GamePhase.Victory.value:
             print("🎉 VICTORY!")
             break

if __name__ == "__main__":
    main()