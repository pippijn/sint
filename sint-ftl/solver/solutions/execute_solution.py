import sys
import os
from typing import List, Tuple, Dict, Any, Optional

# Add ai directory to sys.path to import game types and bindings
sys.path.append(os.path.join(os.path.dirname(__file__), "..", "..", "ai"))

from game_types import GameState, GameAction, ItemType, Action, GamePhase
from bindings_wrapper import SintBindings, SolverBindings

# --- Helper Classes ---

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
        act = self._parse_command(cmd)
        
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
            pass
        else:
            raise ValueError(f"Unknown command: {cmd}")
            
        return GameAction.model_validate(d)

class LoggingPlayer(Player):
    def __init__(self, name: str, id: str, rounds_log: List[List[Tuple[str, GameAction]]]):
        super().__init__(name, id)
        self.rounds_log = rounds_log

    def action(self, cmd: str, cost: Optional[int] = None) -> None:
        super().action(cmd, cost)
        if self.rounds_log:
            self.rounds_log[-1].append(self.actions[-1])

    def pass_turn(self) -> None:
        old_action_count = len(self.actions)
        if self.ap > 0:
            self.actions.append((self.id, GameAction.model_validate({"type": "Pass"})))
        self.ap = 0
        if len(self.actions) > old_action_count:
             new_act = self.actions[-1]
             if new_act[1].root.type == "Pass":
                  if self.rounds_log:
                       self.rounds_log[-1].append(new_act)

class RoundScope:
    def __init__(self, players: List[LoggingPlayer], rounds_log: List[List[Tuple[str, GameAction]]], ap_override: Optional[int] = None, sugar_rush: bool = False):
        self.players = players
        self.rounds_log = rounds_log
        self.ap = 2 if ap_override is None else ap_override
        self.sugar_rush = sugar_rush

    def __enter__(self) -> 'RoundScope':
        self.rounds_log.append([])
        for p in self.players:
            p.reset_ap(self.ap)
            p._sugar_rush_active = self.sugar_rush
        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> bool:
        if exc_type is not None:
             return False
        for p in self.players:
            p.pass_turn()
        return True

# --- Runner Logic ---

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
              state = SintBindings.apply_action(state, next_unready, Action.model_validate(act.model_dump(by_alias=True)))
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
    from solution_rounds import rounds_list
    seed = 2236
    player_ids = ["P1", "P2", "P3", "P4", "P5", "P6"]
    state = SintBindings.new_game(player_ids, seed)
    
    rounds_log: List[List[Tuple[str, GameAction]]] = []
    lp_players = [LoggingPlayer(f"P{i}", f"P{i}", rounds_log) for i in range(1, 7)]
    
    print("--- STARTING GAME ---")
    
    for i, r_func in enumerate(rounds_list):
        r_func(lp_players, rounds_log)
        state = run_rounds(state, rounds_log[i:i+1])
        print(f"--- AFTER ROUND {i+1} ---")
        print_state(state)
        if state.phase == GamePhase.Victory:
             print("🎉 VICTORY!")
             break

if __name__ == "__main__":
    main()
