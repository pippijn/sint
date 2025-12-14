import sys
import os
import json
from typing import List, Tuple, Dict, Any, Optional

# Add ai directory to sys.path to import game types and bindings
sys.path.append(os.path.join(os.path.dirname(__file__), "..", "..", "ai"))

from game_types import GameState, GameAction, ItemType
from bindings_wrapper import SintBindings, SolverBindings

# Mock Player class that accumulates actions instead of printing them
class Player:
    def __init__(self, name: str, id: str):
        self.name = name
        self.id = id
        self.max_ap = 2
        self.ap = 2
        self.actions: List[Tuple[str, GameAction]] = []

    def reset_ap(self, ap: int = 2) -> None:
        self.ap = ap
        self.max_ap = ap

    def action(self, cmd: str, cost: int = 1) -> None:
        self.ap -= cost
        # Parse command to GameAction
        act = self._parse_command(cmd)
        self.actions.append((self.id, act))

    def pass_turn(self) -> None:
        if self.ap > 0:
            self.actions.append((self.id, GameAction.model_validate({"type": "Pass"})))
        self.ap = 0

    def _parse_command(self, cmd: str) -> GameAction:
        parts = cmd.split()
        c = parts[0]
        
        d: Dict[str, Any] = {"type": c}
        
        if c == "Move":
            d["payload"] = {"to_room": int(parts[1])}
        elif c == "PickUp":
            item = parts[1] if len(parts) > 1 else "Peppernut"
            d["payload"] = {"item_type": item}
        elif c == "Drop":
            d["payload"] = {"item_index": int(parts[1])}
        elif c == "VoteReady":
            d["payload"] = {"ready": True}
        elif c == "Throw":
            d["payload"] = {"target_player": parts[1], "item_index": int(parts[2])}
        elif c == "Revive" or c == "FirstAid":
            d["payload"] = {"target_player": parts[1]}
        elif c == "Undo":
            d["payload"] = {"action_id": parts[1]}
        elif c in ["Bake", "Shoot", "RaiseShields", "EvasiveManeuvers", "Interact", "Extinguish", "Repair", "Pass", "Lookout"]:
            pass # No payload
        else:
            raise ValueError(f"Unknown command: {cmd}")
            
        return GameAction.model_validate(d)

# Initialize Players
p1 = Player("P1", "P1")
p2 = Player("P2", "P2")
p3 = Player("P3", "P3")
p4 = Player("P4", "P4")
p5 = Player("P5", "P5")
p6 = Player("P6", "P6")
players = [p1, p2, p3, p4, p5, p6]

# Global log to capture actions in round blocks
rounds_log: List[List[Tuple[str, GameAction]]] = []

class LoggingPlayer(Player):
    def action(self, cmd: str, cost: int = 1) -> None:
        super().action(cmd, cost)
        # Append to the current round's log
        if rounds_log:
            rounds_log[-1].append(self.actions[-1])

    def pass_turn(self) -> None:
        super().pass_turn()
        if self.actions and self.actions[-1][1].root.type == "Pass":
             if rounds_log:
                 rounds_log[-1].append(self.actions[-1])

# Re-init with logging
p1 = LoggingPlayer("P1", "P1")
p2 = LoggingPlayer("P2", "P2")
p3 = LoggingPlayer("P3", "P3")
p4 = LoggingPlayer("P4", "P4")
p5 = LoggingPlayer("P5", "P5")
p6 = LoggingPlayer("P6", "P6")
players = [p1, p2, p3, p4, p5, p6]


class RoundScope:
    def __init__(self, ap_override: Optional[int] = None):
        self.ap = 2 if ap_override is None else ap_override

    def __enter__(self) -> 'RoundScope':
        rounds_log.append([])
        for p in players:
            p.reset_ap(self.ap)
        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> bool:
        if exc_type is not None:
             return False # Propagate exceptions
        for p in players:
            p.pass_turn()
        return True


# --- Rounds ---

def r1():
    with RoundScope(3):
        p1.action("Move 0"); p1.action("Move 5") # P1 to Kitchen (5)
        p5.action("Move 0"); p5.action("Move 6") # P5 to Cannons (6)
        p6.action("Move 0"); p6.action("Move 6") # P6 to Cannons (6)
        p2.action("Move 0"); p2.action("Move 3") # P2 to Cargo (3)
        p3.action("Move 0"); p3.action("Move 7") # P3 to Bridge (7)
        p4.action("Move 0"); p4.action("Move 4"); p4.action("PickUp Extinguisher") # P4 to Engine (4) + Extinguisher

def r2():
    with RoundScope(3):
        p4.action("Extinguish", 1)
        p4.action("Interact", 1)
        p4.action("Move 0", 0)
        p3.action("EvasiveManeuvers", 2)
        p1.action("Bake", 1)
        p1.action("PickUp", 1)
        p2.action("Move 0", 0)
        p2.action("Move 6", 0) 

def r3():
    with RoundScope():
        p4.ap -= 1
        p4.action("Move 0", 0)
        p3.action("Move 4", 0) 
        p3.action("Move 7", 0) 
        p2.action("Move 0", 0)
        p1.action("Interact", 1)
        p1.action("Throw P2 0", 1)
        p2.action("Throw P5 0", 1)
        p5.action("Shoot", 1)
        p3.action("EvasiveManeuvers", 2)

def r4():
    with RoundScope():
        p4.action("Move 7", 1) 
        p4.action("Interact", 1)
        p3.action("EvasiveManeuvers", 2)
        p2.action("Move 3", 1) 
        p1.action("Bake", 1)

def r5():
    with RoundScope():
        p2.action("Interact", 1)
        p3.action("EvasiveManeuvers", 2)
        p1.action("PickUp", 1)
        p1.action("Move 0", 1)
        p1.action("Drop 0", 0)

def r6():
    with RoundScope():
        p3.action("EvasiveManeuvers", 2)
        p1.action("PickUp", 1)
        p1.action("Throw P5 0", 1)
        p5.action("Shoot", 1)
        p2.action("Move 6", 2) 

def r7():
    with RoundScope():
        p1.action("Move 0", 1); p1.action("Move 5", 1) 
        p5.action("Move 0", 1); p5.action("Move 6", 1) 
        p6.action("Move 0", 1); p6.action("Move 6", 1) 
        p3.action("Move 0", 1); p3.action("Move 7", 1) 
        p2.action("Move 0", 1)
        p4.action("Move 0", 1); p4.action("Move 4", 1) 

def r8():
    with RoundScope():
        p4.action("Interact", 2) 
        p2.action("Move 4", 1) 
        p3.action("EvasiveManeuvers", 2)
        p5.action("PickUp", 1)
        p6.action("PickUp", 1)
        p5.action("Shoot", 1)
        p6.action("Shoot", 1)
        p1.action("PickUp", 1)

def r9():
    with RoundScope():
        p1.action("Move 0", 1)
        p1.action("Throw P6 0", 1)
        p6.action("Shoot", 1)
        p3.action("EvasiveManeuvers", 2)
        p2.action("Move 0", 1)
        p2.action("Move 3", 1) 

def r10():
    with RoundScope():
        p3.action("EvasiveManeuvers", 2)
        p4.action("Extinguish", 1)
        p4.action("Move 0", 1)
        p2.action("Interact", 1)
        p1.action("Move 5", 1) 
        p1.action("Bake", 1)

def r11():
    with RoundScope():
        p4.action("Move 4", 1) 
        p4.action("Interact", 1)
        p3.action("EvasiveManeuvers", 2)
        p1.action("Bake", 1)
        p1.action("PickUp", 1)
        p2.action("Move 0", 1)
        p2.action("Move 5", 1) 

def r12():
    with RoundScope():
        p1.action("Interact", 1)
        p1.action("Move 0", 1)
        p2.action("PickUp", 1)
        p2.action("Move 0", 1)
        p3.action("EvasiveManeuvers", 2)
        p4.action("Move 0", 1)
        p4.action("Move 1", 1) 

def r13():
    with RoundScope():
        p4.action("Interact", 1)
        p1.action("Move 6", 1) 
        p1.action("Throw P5 0", 1)
        p2.action("Move 6", 1) 
        p2.action("Throw P6 0", 1)
        p5.action("Shoot", 1)
        p6.action("Shoot", 1)
        p3.action("EvasiveManeuvers", 2)

def r14():
    with RoundScope():
        p4.action("Move 0", 1)
        p3.action("EvasiveManeuvers", 2)
        p4.action("Interact", 1)
        p1.action("Move 0", 1)
        p1.action("Move 5", 1) 
        p2.action("Move 0", 1)
        p2.action("Move 5", 1) 

def r15():
    with RoundScope():
        p1.action("Bake", 1)
        p1.action("PickUp", 1)
        p2.action("EvasiveManeuvers", 2)
        p3.action("Move 5", 1)
        p3.action("PickUp", 1)
        p4.action("Move 0", 1)
        p4.action("Move 5", 1) 

def r16():
    with RoundScope():
        p2.action("EvasiveManeuvers", 2)
        p1.action("Move 0", 1)
        p1.action("Throw P5 0", 1)
        p3.action("Move 0", 1)
        p3.action("Throw P6 0", 1)
        p4.action("PickUp", 1)
        p4.action("Move 0", 1)
        p6.action("Move 0", 1)
        p6.action("Move 6", 1) 
        p5.action("Shoot", 1)

def r17():
    with RoundScope():
        p2.action("EvasiveManeuvers", 2)
        p4.action("Move 6", 1) 
        p4.action("Throw P5 1", 1) 
        p5.action("Shoot", 1)
        p6.action("Shoot", 1)
        p1.action("Move 6", 1) 
        p3.action("Move 5", 1) 
        p3.action("Bake", 1) 

def r18():
    with RoundScope():
        p1.action("Move 0", 1)
        p1.action("Move 5", 1)
        p3.action("PickUp", 1)
        p3.action("Move 0", 1)
        p4.action("Move 0", 1)
        p4.action("Move 5", 1) 
        p6.action("Move 0", 1)
        p6.action("Move 5", 1) 
        p2.action("EvasiveManeuvers", 2)

def r19():
    with RoundScope():
        p2.action("EvasiveManeuvers", 2)
        p3.action("Throw P5 0", 1)
        p5.action("Shoot", 1)
        p1.action("PickUp", 1)
        p1.action("Move 0", 1)
        p4.action("PickUp", 1)
        p4.action("Move 0", 1)
        p6.action("PickUp", 1)
        p6.action("Move 0", 1)
        p3.action("Move 5", 1) 

def r20():
    with RoundScope():
        p2.action("EvasiveManeuvers", 2)
        p1.action("Throw P5 0", 1)
        p5.action("Shoot", 1)
        p6.action("Move 6", 1)
        p6.action("Shoot", 1)
        p4.action("Move 6", 1) 
        p3.action("PickUp", 1) 
        p3.action("Move 0", 1) 

def r21():
    with RoundScope():
        p1.action("Interact", 1)
        p3.action("Move 5", 1)
        p3.action("Bake", 1)
        p4.action("Move 0", 1)
        p4.action("Move 5", 1)
        p2.action("EvasiveManeuvers", 2)

def r22():
    with RoundScope():
        p2.action("EvasiveManeuvers", 2)
        p3.action("Move 0", 1)
        p3.action("Throw P5 0", 1)
        p4.action("Move 0", 1)
        p4.action("Throw P6 0", 1)
        p5.action("Shoot", 1)
        p6.action("Shoot", 1)

def r23():
    with RoundScope():
        p6.action("Interact", 1)
        p6.action("Interact", 1)
        p5.action("Interact", 1)
        p2.action("Move 0", 1)

def main() -> None:
    seed = 2236
    
    # Run Rounds to record actions
    r1(); r2(); r3(); r4(); r5(); r6(); r7(); r8(); r9(); r10()
    r11(); r12(); r13(); r14(); r15(); r16(); r17(); r18(); r19(); r20()
    r21(); r22(); r23()
    
    # Execute
    print(f"Executing {len(rounds_log)} rounds...")
    
    player_ids = ["P1", "P2", "P3", "P4", "P5", "P6"]
    initial_state = SintBindings.new_game(player_ids, seed)
    
    result = SolverBindings.verify_solution(initial_state, rounds_log)

    is_success = result["success"]
    is_error = result.get("error") is not None

    if is_success:
        print("✅ SUCCESS!")
    else:
        print("❌ FAILURE!")

    # Validate result state into model for type safety
    final_state = GameState.model_validate(result['final_state'])
    print(f"Final Hull: {final_state.hull_integrity}")
    print(f"Final Boss HP: {final_state.enemy.hp}")
    print(f"Score: {result['score']}")

    if not is_success:
        print(result.get("failure_summary"))

    # Show trajectory if it's a valid game run (Success or Defeat), but not on Execution Error
    if not is_error:
        print("\n--- TRAJECTORY LOG ---")
        history_raw = result['history']
        history: List[Tuple[str, GameAction]] = []
        for pid, act_dict in history_raw:
            history.append((pid, GameAction.model_validate(act_dict)))
            
        logs = SolverBindings.get_trajectory_log(initial_state, history)
        for l in logs:
            print(l, end='')

    if not is_success:
        sys.exit(1)

if __name__ == "__main__":
    main()