import os
import sys
import json
import time
import asyncio
from typing import Dict, Any, List, Optional, Tuple

# Add repository root to path to import from 'ai'
repo_root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.append(repo_root)

import sint_core
import sint_solver
from ai.tools import load_game_tools
from google import genai
from google.genai import types

class GeminiSolver:
    def __init__(self, seed: int = 12345, model_name: str = "gemini-2.5-flash-lite", debug: bool = False):
        self.seed = seed
        self.model_name = model_name
        self.debug = debug
        self.client = genai.Client(
            api_key=os.environ.get("GEMINI_API_KEY"),
            http_options={'api_version': 'v1alpha'}
        )
        self.tools, self.tool_map = load_game_tools()
        
        # Load Rules
        rules_path = os.path.join(repo_root, 'docs', 'rules.md')
        with open(rules_path, 'r') as f:
            self.rules_text = f.read()

    def run_full_game(self, max_rounds: int = 30):
        checkpoint_file = f"checkpoint_seed_{self.seed}.json"
        
        # 1. Initialize
        if os.path.exists(checkpoint_file):
            print(f"Loading checkpoint from {checkpoint_file}...")
            rounds_log = self._load_checkpoint(checkpoint_file)
            print(f"Resumed with {len(rounds_log)} rounds recorded.")
        else:
            rounds_log = []
            print(f"Starting new game with Seed {self.seed}")

        # Main Loop: Replay -> Prompt -> Act
        while True:
            # A. Replay to get current valid state
            player_ids = [f"P{i+1}" for i in range(6)]
            initial_state = sint_core.new_game(player_ids, self.seed)
            
            # This replays all recorded rounds and returns the final state (Execution/EnemyAction/Morning...)
            result = sint_solver.verify_solution(initial_state, rounds_log)
            state = result["final_state"]
            
            if state['phase'] in ["GameOver", "Victory"]:
                self._print_results(result, rounds_log)
                break
                
            if state['turn_count'] > max_rounds:
                print("Max rounds reached.")
                break

            # B. Auto-Advance Local State to TacticalPlanning
            # The replay stops at the end of the last provided round block.
            # We need to fast-forward through MorningReport/Telegraph to get to the new Planning phase.
            state = self._fast_forward_to_planning(state)
            
            if state['phase'] in ["GameOver", "Victory"]:
                # Game ended during fast-forward (e.g. death by fire in Morning)
                # We record an empty block if needed, but usually we just stop.
                print(f"Game ended during fast-forward: {state['phase']}")
                break

            print(f"\n=== ROUND {state['turn_count']} (Phase: {state['phase']}) ===")

            # C. TACTICAL PLANNING
            # 1. Strategy Pass
            strategy_text = self._get_strategy(state)
            print(f"\n[STRATEGY]\n{strategy_text}\n")
            
            # 2. Action Pass (Draft/Repair Loop)
            actions = self._get_actions(state, strategy_text)
            
            if not actions:
                print("!! No valid plan found. Forcing PASS for all players.")
                actions = self._get_pass_batch(state)
            
            # 3. Append Block
            # We add a new block to the log. This block represents the actions for *this* round.
            # Note: We do NOT need to "pass remaining AP" explicitly in the log if we use `verify_solution`
            # because `verify_solution`'s block logic auto-passes/readies at the end of a block.
            # HOWEVER, `verify_solution` validates AP consumption.
            # So we MUST ensure the block consumes all AP.
            
            # We simulate locally to determine if AP is left, and append Pass actions if needed.
            complete_block = self._ensure_block_completeness(state, actions)
            
            print(f"\n[COMMITTING] {len(complete_block)} actions...")
            rounds_log.append(complete_block)

            # Checkpoint
            self._save_checkpoint(rounds_log, checkpoint_file)

    def _fast_forward_to_planning(self, state):
        """
        Advances the state locally until it reaches TacticalPlanning, GameOver, or Victory.
        This allows us to prompt the LLM with the correct starting context for the turn.
        """
        curr = state
        loop_limit = 0
        while curr['phase'] not in ["TacticalPlanning", "GameOver", "Victory"] and loop_limit < 100:
            loop_limit += 1
            # Naive auto-advance: Ready everyone.
            # Logic: If not in Planning, the game waits for Ready votes to proceed (Morning->Telegraph->Planning)
            ready_batch = []
            for pid in curr['players']:
                ready_batch.append((pid, {"type": "VoteReady", "payload": {"ready": True}}))
            
            try:
                curr = self._apply_batch_local_only(curr, ready_batch)
            except Exception as e:
                print(f"Fast-forward error: {e}")
                break
        return curr

    def _ensure_block_completeness(self, start_state, user_actions):
        """
        Applies user actions locally. If any player has AP left, appends Pass actions.
        Returns the full list of actions for the block.
        """
        curr = start_state
        final_actions = []
        
        # Apply user actions
        for pid, act in user_actions:
            try:
                curr = sint_core.apply_action_with_id(curr, pid, act, None)
                final_actions.append((pid, act))
            except Exception as e:
                print(f"Skipping invalid action during block compilation: {pid} {act} - {e}")
        
        # Check remaining AP
        pass_actions = []
        for pid, p in curr['players'].items():
            if p['ap'] > 0 and not p['is_ready']:
                pass_actions.append((pid, {"type": "Pass"}))
        
        final_actions.extend(pass_actions)
        return final_actions

    def _apply_batch_local_only(self, state, actions):
        """Applies actions locally without returning a log. Used for phase transitions."""
        curr = state
        for pid, act in actions:
            # We don't catch exceptions here to fail fast on logic bugs
            curr = sint_core.apply_action_with_id(curr, pid, act, None)
        return curr

    def _save_checkpoint(self, rounds_log, filename):
        with open(filename, 'w') as f:
            f.write("{\n  \"rounds_log\": [\n")
            for i, round_block in enumerate(rounds_log):
                f.write("    [\n")
                for j, (pid, act) in enumerate(round_block):
                    act_json = json.dumps(act)
                    # Create the [PID, Action] line
                    line = f'      ["{pid}", {act_json}]'
                    if j < len(round_block) - 1:
                        f.write(line + ",\n")
                    else:
                        f.write(line + "\n")
                if i < len(rounds_log) - 1:
                    f.write("    ],\n")
                else:
                    f.write("    ]\n")
            f.write("  ]\n}\n")
        print(f"Saved checkpoint to {filename}")

    def _load_checkpoint(self, filename):
        with open(filename, 'r') as f:
            data = json.load(f)
        return data.get("rounds_log", [])

    def _get_strategy(self, state: Dict[str, Any]) -> str:
        prompt = self._construct_strategy_prompt(state)
        try:
            resp = self.client.models.generate_content(
                model=self.model_name,
                contents=prompt
            )
            return resp.text
        except Exception as e:
            print(f"Strategy Gen Error: {e}")
            return "Survive and kill the boss."

    def _get_actions(self, state: Dict[str, Any], strategy: str) -> List[Tuple[str, Dict[str, Any]]]:
        max_retries = 3
        last_error = None
        
        for attempt in range(max_retries):
            prompt = self._construct_action_prompt(state, strategy, attempt=attempt, error_msg=last_error)
            
            try:
                resp = self.client.models.generate_content(
                    model=self.model_name,
                    contents=prompt,
                    config=types.GenerateContentConfig(
                        response_mime_type="application/json",
                        response_schema=self._get_action_list_schema()
                    )
                )
                
                plan_json = json.loads(resp.text)
                # Validate Logic (Rust)
                valid_plan, error_msg = self._simulate_plan(state, plan_json)
                
                if valid_plan:
                    return self._parse_plan_json(plan_json)
                else:
                    print(f"  > Attempt {attempt+1} Invalid: {error_msg}")
                    last_error = error_msg
                    
            except Exception as e:
                print(f"  > Attempt {attempt+1} Failed: {e}")
                last_error = str(e)
                
        return []

    def _simulate_plan(self, start_state: Dict[str, Any], plan: List[Dict[str, Any]]) -> Tuple[bool, Optional[str]]:
        """Returns (True, None) if valid, (False, ErrorMsg) if invalid."""
        sim_state = start_state
        for i, step in enumerate(plan):
            pid = step['player_id']
            act = step['action']
            try:
                core_act = self._convert_to_core_action(act)
                sim_state = sint_core.apply_action_with_id(sim_state, pid, core_act, None)
            except Exception as e:
                return False, f"Step {i+1} ({pid} {act['type']}): {e}"
        return True, None

    def _convert_to_core_action(self, simplified_act: Dict[str, Any]) -> Dict[str, Any]:
        """Converts LLM simplified format to Rust Core format."""
        atype = simplified_act.get("type")
        payload = {}
        for k, v in simplified_act.items():
            if k != "type":
                payload[k] = v
        
        unit_variants = {
            "Bake", "Shoot", "RaiseShields", "EvasiveManeuvers", 
            "Interact", "Extinguish", "Repair", "Lookout", "Pass"
        }

        if atype in unit_variants:
            return {"type": atype}
        
        return {"type": atype, "payload": payload}

    def _parse_plan_json(self, plan: List[Dict[str, Any]]) -> List[Tuple[str, Dict[str, Any]]]:
        out = []
        for step in plan:
            out.append((step['player_id'], self._convert_to_core_action(step['action'])))
        return out

    def _get_vote_ready_batch(self, state) -> List[Tuple[str, Dict[str, Any]]]:
        actions = []
        for pid in state['players']:
            actions.append((pid, {"type": "VoteReady", "payload": {"ready": True}}))
        return actions

    def _get_pass_batch(self, state) -> List[Tuple[str, Dict[str, Any]]]:
        actions = []
        for pid, p in state['players'].items():
            if p['ap'] > 0:
                actions.append((pid, {"type": "Pass"}))
        return actions

    def _get_action_list_schema(self):
        return {
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "player_id": {"type": "string", "enum": ["P1", "P2", "P3", "P4", "P5", "P6"]},
                    "action": {
                        "type": "object",
                        "properties": {
                            "type": {"type": "string", "enum": ["Move", "Bake", "Shoot", "Extinguish", "Repair", "Throw", "PickUp", "Drop", "Pass", "VoteReady", "RaiseShields", "EvasiveManeuvers", "Interact", "Lookout", "Revive", "FirstAid"]},
                            "to_room": {"type": "integer"},
                            "target_player": {"type": "string"},
                            "item_index": {"type": "integer"},
                            "item_type": {"type": "string", "enum": ["Peppernut", "Extinguisher", "Wheelbarrow"]}
                        },
                        "required": ["type"]
                    }
                },
                "required": ["player_id", "action"]
            }
        }

    def _construct_strategy_prompt(self, state: Dict[str, Any]) -> str:
        summary = self._state_summary(state)
        return f"""
        CONTEXT:
        {self.rules_text[:2000]}... (Truncated rules)

        CURRENT STATE:
        {summary}

        TASK:
        Analyze the situation. We are in Round {state['turn_count']}.
        Determine the high-level strategy for this round.
        Should we be Aggressive (attack boss), Defensive (heal/extinguish), or Efficient (stockpile)?
        Assign a role/goal to each player (P1-P6).
        
        OUTPUT:
        Plain text strategy description.
        """

    def _construct_action_prompt(self, state: Dict[str, Any], strategy: str, attempt: int, error_msg: Optional[str] = None) -> str:
        summary = self._state_summary(state)
        base = f"""
        STRATEGY:
        {strategy}

        CURRENT STATE:
        {summary}

        TASK:
        Generate a valid sequence of actions for the players to execute this strategy.
        Players have 2 AP. Moving costs 1 AP. Interacting costs 1 AP.
        
        CONSTRAINTS:
        - Output strict JSON array.
        - Validate dependencies (e.g. P1 must bake before P2 can pickup).
        - Use "Move", "Interact", "Bake", "Shoot", "Extinguish", "Repair", "Throw", "PickUp", "Drop", "Pass", "EvasiveManeuvers", "RaiseShields".
        
        ATTEMPT: {attempt+1}
        """
        if error_msg:
            base += f"\n\nPREVIOUS ERROR (Please Fix):\n{error_msg}\n"
            
        return base

    def _state_summary(self, state: Dict[str, Any]) -> str:
        hull = state['hull_integrity']
        boss = f"{state['enemy']['name']} ({state['enemy']['hp']} HP)"
        
        players = []
        for pid, p in state['players'].items():
            players.append(f"{pid}: Room {p['room_id']}, AP {p['ap']}, HP {p['hp']}, Inv {p['inventory']}")
            
        hazards = []
        for rid, r in state['map']['rooms'].items():
             if r['hazards']:
                 hazards.append(f"Room {rid}: {r['hazards']}")
        
        situations = []
        if state.get('active_situations'):
            situations.append("ACTIVE SITUATIONS (Global Effects):")
            for card in state['active_situations']:
                situations.append(f"- {card['title']}: {card['description']}")
                 
        return f"Hull: {hull}\nBoss: {boss}\nPlayers:\n" + "\n".join(players) + "\nHazards:\n" + "\n".join(hazards) + "\n" + "\n".join(situations)
    
    def _print_results(self, result, rounds_log):
        is_success = result["success"]
        print(f"\n=== GAME FINISHED ===")
        print(f"Success: {is_success}")
        print(f"Final Score: {result.get('score', 0)}")
        
        if not is_success:
            print(result.get("failure_summary", "No failure summary"))
        
        # Re-verify to get trajectory text (since verification result struct only holds state/history)
        # Or we could have exposed trajectory string in result. 
        # But 'result.history' is available.
        # We can re-call get_trajectory_log using initial state + history.
        
        # Use sint_solver to print trajectory
        player_ids = [f"P{i+1}" for i in range(6)]
        initial_state = sint_core.new_game(player_ids, self.seed)
        
        print("\n--- TRAJECTORY LOG ---")
        try:
            logs = sint_solver.get_trajectory_log(initial_state, result['history'])
            for l in logs:
                print(l, end='')
        except Exception as e:
            print(f"Error printing trajectory: {e}")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--seed", type=int, default=12345)
    parser.add_argument("--max-rounds", type=int, default=30)
    parser.add_argument("--debug", action="store_true")
    args = parser.parse_args()

    solver = GeminiSolver(seed=args.seed, debug=args.debug)
    solver.run_full_game(max_rounds=args.max_rounds)
