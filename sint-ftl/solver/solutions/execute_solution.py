import sys
import os
import json
from typing import List, Tuple, Dict, Any, Optional

# Add ai directory to sys.path to import game types and bindings
sys.path.append(os.path.join(os.path.dirname(__file__), "..", "..", "ai"))

from game_types import GameState, GameAction, ItemType, Action, GamePhase
from bindings_wrapper import SintBindings, SolverBindings

# Mock Player class that accumulates actions instead of printing them
class Player:
    def __init__(self, name: str, id: str):
        self.name = name
        self.id = id
        self.max_ap = 2
        self.ap = 2
        self._sugar_rush_moves = 0
        self._sugar_rush_active = False
        self.actions: List[Tuple[str, GameAction]] = []

    def reset_ap(self, ap: int = 2) -> None:
        self.ap = ap
        self.max_ap = ap
        self._sugar_rush_moves = 0

    def action(self, cmd: str, cost: Optional[int] = None) -> None:
        # Parse command to GameAction
        act = self._parse_command(cmd)
        
        # Determine cost if not provided
        if cost is None:
             if cmd.startswith("Move") and self._sugar_rush_active and self._sugar_rush_moves < 5:
                  cost = 0
             elif cmd.startswith("Move"):
                  cost = 1
             elif cmd == "RaiseShields" or cmd == "EvasiveManeuvers":
                  cost = 2
             elif cmd == "Pass" or cmd == "VoteReady" or cmd.startswith("Chat") or cmd.startswith("Undo"):
                  cost = 0
             else:
                  cost = 1

        if cmd.startswith("Move"):
             self._sugar_rush_moves += 1

        self.ap -= cost
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

# Global log to capture actions in round blocks
rounds_log: List[List[Tuple[str, GameAction]]] = []

class LoggingPlayer(Player):
    def action(self, cmd: str, cost: Optional[int] = None) -> None:
        super().action(cmd, cost)
        # Append to the current round's log
        if rounds_log:
            rounds_log[-1].append(self.actions[-1])

    def pass_turn(self) -> None:
        old_action_count = len(self.actions)
        if self.ap > 0:
            self.actions.append((self.id, GameAction.model_validate({"type": "Pass"})))
        self.ap = 0
        if len(self.actions) > old_action_count:
             new_act = self.actions[-1]
             if new_act[1].root.type == "Pass":
                  if rounds_log:
                       rounds_log[-1].append(new_act)

# Initialize Players
p1 = LoggingPlayer("P1", "P1")
p2 = LoggingPlayer("P2", "P2")
p3 = LoggingPlayer("P3", "P3")
p4 = LoggingPlayer("P4", "P4")
p5 = LoggingPlayer("P5", "P5")
p6 = LoggingPlayer("P6", "P6")
players = [p1, p2, p3, p4, p5, p6]


class RoundScope:
    def __init__(self, ap_override: Optional[int] = None, sugar_rush: bool = False):
        self.ap = 2 if ap_override is None else ap_override
        self.sugar_rush = sugar_rush

    def __enter__(self) -> 'RoundScope':
        rounds_log.append([])
        for p in players:
            p.reset_ap(self.ap)
            p._sugar_rush_active = self.sugar_rush
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
        p1.action("Move 0"); p1.action("Move 5") # Kitchen
        p5.action("Move 0"); p5.action("Move 6") # Cannons
        p6.action("Move 0"); p6.action("Move 6") # Cannons
        p2.action("Move 0"); p2.action("Move 3") # Cargo
        p3.action("Move 0"); p3.action("Move 7") # Bridge
        p4.action("Move 0"); p4.action("Move 4"); p4.action("PickUp Extinguisher") # Engine

def r2():
    with RoundScope(3, sugar_rush=True):
        p4.action("Move 0", 0); p4.action("Move 6", 0); p4.action("Extinguish")
        p1.action("Bake"); p1.action("PickUp Peppernut"); p1.action("Move 0", 0)
        p2.action("PickUp Wheelbarrow"); p2.action("Move 0", 0); p2.action("Move 9", 0)
        p3.action("RaiseShields")
        p5.action("Move 0", 0)
        p6.action("Move 0", 0)

def r3():
    with RoundScope(3, sugar_rush=True):
        p1.action("Move 5", 0); p1.action("Interact", 1) # Solve Sugar Rush
        p1.action("Throw P5 0", 1)
        p5.action("Move 6", 1); p5.action("Shoot", 1)
        p2.action("PickUp Peppernut", 1); p2.action("PickUp Peppernut", 1); p2.action("Move 0", 1)
        p3.action("RaiseShields", 2)
        p4.action("Move 0", 1)
        p6.action("Move 6", 1)

def r4():
    with RoundScope(2):
        p4.action("Extinguish"); p4.action("Move 4")
        p2.action("Move 6"); p2.action("Throw P5 1") # P2 inv: [WB, Nut1, Nut2] -> throws index 1
        p5.action("Shoot")
        p3.action("RaiseShields")
        p1.action("Bake"); p1.action("PickUp Peppernut")

def r5():
    with RoundScope(2):
        p4.ap = 1 # Overheating
        p4.action("Extinguish")
        p2.action("Throw P5 1")
        p1.action("Move 0"); p1.action("Throw P2 0")
        p2.action("Throw P6 1")
        p5.action("Shoot")
        p6.action("Shoot")
        p3.action("RaiseShields")

def r6():
    with RoundScope(2):
        p4.ap = 1 # Overheating
        p4.action("Interact") # Solve Overheating
        p1.action("Move 9"); p1.action("Interact") # Solve Mice Plague
        p2.action("Move 0"); p2.action("Move 9")
        p3.action("RaiseShields")

def r7():
    with RoundScope(2):
        p2.action("PickUp Peppernut"); p2.action("PickUp Peppernut")
        p1.action("Move 0")
        p4.action("Move 0")
        p3.action("RaiseShields")

def r8():
    with RoundScope(2):
        p2.action("Move 0"); p2.action("Move 6")
        p4.action("Move 6")
        p1.action("Move 6")
        p2.action("Throw P5 1"); p2.action("Throw P6 1")
        p5.action("Shoot"); p6.action("Shoot")
        p3.action("RaiseShields")

def run_rounds(initial_state: GameState, rounds: List[List[Tuple[str, GameAction]]]) -> GameState:
    result = SolverBindings.verify_solution(initial_state, rounds)
    is_incomplete = not result["success"] and result.get("failed_action") is None and result.get("error") is None
    
    if not result["success"] and not is_incomplete:
        print("❌ FAILURE!")
        print(result.get("failure_summary"))
        if result.get('history'):
            history = [(pid, GameAction.model_validate(act_dict)) for pid, act_dict in result['history']]
            logs = SolverBindings.get_trajectory_log(initial_state, history)
            for l in logs:
                print(l, end='')
        sys.exit(1)
    
    state = GameState.model_validate(result['final_state'])
    
    while state.phase != GamePhase.TacticalPlanning and state.phase != GamePhase.GameOver and state.phase != GamePhase.Victory:
         next_unready = None
         for p in state.players.root.values():
              if not p.is_ready:
                   next_unready = p.id
                   break
         if next_unready:
              act = GameAction.model_validate({"type": "VoteReady", "payload": {"ready": True}})
              state = SintBindings.apply_action(state, next_unready, Action.model_validate(act.model_dump()))
         else:
              break
    return state

def print_state(state: GameState):
    print(f"Hull: {state.hull_integrity} | Enemy HP: {state.enemy.hp} | Phase: {state.phase} | Turn: {state.turn_count}")
    if state.enemy.next_attack:
        print(f"Enemy Telegraph: {state.enemy.next_attack.effect} at Room {state.enemy.next_attack.target_room}")
    for pid, p in state.players.root.items():
        inv = [i.value for i in p.inventory]
        print(f"  {pid}: Room {p.room_id} | HP {p.hp} | AP {p.ap} | Inv {inv}")
    for rid, r in state.map.rooms.root.items():
        if r.hazards or r.items:
            items = [i.value for i in r.items]
            print(f"  Room {rid} ({r.name.value}): Hazards {r.hazards} | Items {items}")
    if state.active_situations:
        print(f"Active Situations: {[s.title for s in state.active_situations]}")

def main() -> None:
    seed = 2236
    player_ids = ["P1", "P2", "P3", "P4", "P5", "P6"]
    state = SintBindings.new_game(player_ids, seed)
    
    print("--- STARTING GAME ---")
    
    for i, r_func in enumerate([r1, r2, r3, r4, r5, r6, r7, r8]):
        r_func()
        state = run_rounds(state, rounds_log[i:i+1])
        print(f"--- AFTER ROUND {i+1} ---")
        print_state(state)
        if state.phase == GamePhase.Victory:
             print("🎉 VICTORY!")
             break

if __name__ == "__main__":
    main()