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
        
        # Load Rules & Strategy
        rules_path = os.path.join(repo_root, 'docs', 'rules.md')
        strategy_path = os.path.join(repo_root, 'docs', 'strategy.md')
        
        with open(rules_path, 'r') as f:
            self.rules_text = f.read()
            
        with open(strategy_path, 'r') as f:
            self.strategy_text = f.read()

        task_path = os.path.join(os.path.dirname(__file__), 'prompt_task.md')
        with open(task_path, 'r') as f:
            self.task_text = f.read()
            
        self.system_instr = f"""You are the expert coordinator for 'Sinterklaas FTL'.
Your goal is to guide the crew (P1-P6) to victory by issuing optimal commands.

=== GAME RULES ===
{self.rules_text}

=== STRATEGY GUIDE ===
{self.strategy_text}
"""

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
            current_score = result.get("score", 0.0)
            
            if state['phase'] in ["GameOver", "Victory"]:
                self._print_results(result, rounds_log)
                break
            
            print(f"Current Score: {current_score:.1f}")

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

            if state['turn_count'] > max_rounds:
                print("Max rounds reached.")
                break

            print(f"\n=== ROUND {state['turn_count']} (Phase: {state['phase']}) ===")

            # C. TACTICAL PLANNING
            # 1. Strategy Pass
            strategy_text = self._get_strategy(state)
            print(f"\n[STRATEGY]\n{strategy_text}\n")
            
            # 2. Action Pass (Draft/Repair Loop)
            actions = self._get_actions(state, strategy_text)
            
            if not actions:
                print(f"!! No valid plan found after multiple attempts. Aborting simulation.")
                break
            
            # 3. Append Block
            # We add a new block to the log. This block represents the actions for *this* round.
            # Note: We do NOT need to "pass remaining AP" explicitly in the log if we use `verify_solution`
            # because `verify_solution`'s block logic auto-passes/readies at the end of a block.
            # HOWEVER, `verify_solution` validates AP consumption.
            # So we MUST ensure the block consumes all AP.
            
            # We simulate locally to determine if AP is left, and append Pass actions if needed.
            complete_block = self._apply_plan_locally(state, actions)
            
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

    def _apply_plan_locally(self, start_state, user_actions):
        """
        Applies user actions locally. 
        Returns the list of valid actions (skipping any that throw errors, though they should be pre-validated).
        """
        curr = start_state
        final_actions = []
        
        # Apply user actions
        for pid, act in user_actions:
            try:
                curr = sint_core.apply_action_with_id(curr, pid, act, None)
                final_actions.append((pid, act))
            except Exception as e:
                print(f"Skipping invalid action during execution: {pid} {act} - {e}")
        
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
                contents=prompt,
                config=types.GenerateContentConfig(
                    system_instruction=self.system_instr
                )
            )
            return resp.text
        except Exception as e:
            print(f"Strategy Gen Error: {e}")
            return "Survive and kill the boss."

    def _get_submit_plan_tool(self):
        return {
            "name": "submit_plan",
            "description": "Submit the tactical plan for all players for the current round.",
            "parameters": {
                "type": "OBJECT",
                "properties": {
                    "actions": self._get_action_list_schema()
                },
                "required": ["actions"]
            }
        }

    def _get_actions(self, state: Dict[str, Any], strategy: str) -> List[Tuple[str, Dict[str, Any]]]:
        max_retries = 5
        
        # Build Initial Prompt (System + User)
        # We don't rely on self.system_instr in the config for this chat-like loop to keep it clean,
        # or we can. Let's use history manually.
        
        prompt = self._construct_action_prompt(state, strategy, attempt=0)
        
        history = [
            types.Content(
                role="user",
                parts=[types.Part(text=prompt)]
            )
        ]
        
        tool = types.Tool(function_declarations=[self._get_submit_plan_tool()])
        
        for attempt in range(max_retries):
            if self.debug:
                print(f"\n--- GENERATION (Attempt {attempt+1}) ---")
            
            try:
                resp = self.client.models.generate_content(
                    model=self.model_name,
                    contents=history,
                    config=types.GenerateContentConfig(
                        system_instruction=self.system_instr,
                        tools=[tool],
                        tool_config=types.ToolConfig(
                            function_calling_config=types.FunctionCallingConfig(
                                mode=types.FunctionCallingConfigMode.ANY
                            )
                        )
                    )
                )
                
                # Check for function call
                fc = None
                if resp.candidates and resp.candidates[0].content.parts:
                    for part in resp.candidates[0].content.parts:
                        if part.function_call:
                            fc = part.function_call
                            break
                
                if not fc:
                    # Model might have just chatted (CoT). Add to history and continue.
                    if resp.text:
                        if self.debug: print(f"Model thought: {resp.text}")
                        history.append(resp.candidates[0].content)
                        # We need to nudge it to call the function if it hasn't?
                        # But Mode=ANY should force it.
                        # If it output text, it might be CoT. The next turn should call function.
                        # Let's verify if we need to loop again for the tool call?
                        # With ANY, it *should* call. If it outputs text *and* call, we get both parts.
                        # If just text, we continue loop.
                        continue
                    else:
                        raise ValueError("Empty response")

                # We have a function call
                if self.debug: print(f"Tool Call: {fc.name}")
                
                if fc.name == "submit_plan":
                    # Fix: Ensure args is a standard dict
                    # Using recursive helper if needed, but fc.args should be dict-like
                    # Ideally verify type.
                    
                    raw_actions = fc.args.get("actions", [])
                    # genai might return 'Map'/'List' types? Convert to native.
                    # Assuming it behaves like dict/list for now or verify.
                    
                    # Convert to native python types just in case (e.g. from Proto/Map)
                    def to_native(o):
                        if hasattr(o, "items"): return {k: to_native(v) for k, v in o.items()}
                        if isinstance(o, list): return [to_native(x) for x in o]
                        if isinstance(o, float) and o.is_integer(): return int(o)
                        return o
                    
                    plan_json = to_native(raw_actions)

                    if self.debug:
                        print(f"Tool Actions: {json.dumps(plan_json, indent=2)}")

                    # Validate
                    valid_plan, error_msg = self._simulate_plan(state, plan_json)
                    
                    if valid_plan:
                        return self._parse_plan_json(plan_json)
                    else:
                        print(f"  > Attempt {attempt+1} Invalid: {error_msg}")
                        print(f"    Failed Plan: {json.dumps(plan_json)}") 
                        
                        # Feed error back
                        history.append(resp.candidates[0].content)
                        
                        # Construct proper FunctionResponse
                        history.append(types.Content(
                            parts=[types.Part(
                                function_response=types.FunctionResponse(
                                    name="submit_plan",
                                    response={
                                        "result": "INVALID_PLAN", 
                                        "error": error_msg,
                                        "failed_plan_preview": json.dumps(plan_json)
                                    }
                                )
                            )]
                        ))
                else:
                    print(f"Unknown tool: {fc.name}")

            except Exception as e:
                print(f"  > Attempt {attempt+1} Failed: {e}")
                # Append error to history just in case
                history.append(types.Content(role="user", parts=[types.Part(text=f"System Error: {e}")]))
                
        return []

    def _simulate_plan(self, start_state: Dict[str, Any], plan: List[Dict[str, Any]]) -> Tuple[bool, Optional[str]]:
        """Returns (True, None) if valid, (False, ErrorMsg) if invalid."""
        sim_state = start_state
        for i, step in enumerate(plan):
            pid = step['player_id']
            act = step['action']
            try:
                core_act = self._convert_to_core_action(act)
                
                # Soft-fix for Pass with 0 AP
                if core_act['type'] == 'Pass':
                    p_curr = sim_state['players'][pid]
                    if p_curr['ap'] == 0:
                        continue # Just skip it

                sim_state = sint_core.apply_action_with_id(sim_state, pid, core_act, None)
            except Exception as e:
                # Gather context for feedback
                p_data = sim_state.get('players', {}).get(pid, {})
                rid = p_data.get('room_id')
                # Room IDs in map are strings or ints depending on serialization, but usually strings in JSON
                # The core might return ints or strings. Let's try both or just check usage.
                # In _state_summary it iterates state['map']['rooms'].items().
                rooms = sim_state.get('map', {}).get('rooms', {})
                room = rooms.get(str(rid)) or rooms.get(rid) or {}
                
                room_items = room.get('items', [])
                p_inv = p_data.get('inventory', [])
                
                context = f"Player {pid} is in Room {rid}. Room Items: {room_items}. Player Inv: {p_inv}."
                
                # specific advice
                advice = ""
                err_str = str(e)
                if "Inventory is full" in err_str:
                    advice = " >> ADVICE: INVENTORY FULL! SKIP THIS PICKUP. You already have the item. Just Move or Throw."
                elif "not in room" in err_str:
                     # Find where it is
                     req_item = act.get('payload', {}).get('item_type') or act.get('item_type')
                     found_rid = None
                     if req_item:
                         for r_id, r_data in sim_state.get('map', {}).get('rooms', {}).items():
                             if req_item in r_data.get('items', []):
                                 found_rid = r_id
                                 break
                     
                     if found_rid is not None:
                         advice = f" >> ADVICE: Item '{req_item}' is NOT here. It is in Room {found_rid}. Go there first."
                     else:
                         advice = " >> ADVICE: Check 'Room Items' in CURRENT STATE. You cannot PickUp what isn't there. Did someone else take it?"
                elif "requires Kitchen" in err_str and "Move" in [s['action'].get('type') for s in plan if s['player_id'] == pid]:
                     advice = " >> ADVICE: You Moved before Baking! The order matters. Bake THEN Move."
                elif "Target not in range" in err_str:
                     advice = " >> ADVICE: You are not adjacent to the target. Move to Hallway (Room 0) first? Most rooms connect via Hallway."

                return False, f"Step {i+1} ({pid} {act.get('type')}) Failed: {err_str}\nContext: {context}{advice}"
        
        # Post-simulation check: Did everyone use their AP?
        unused_ap_msg = []
        for pid, p in sim_state['players'].items():
            if p['ap'] > 0 and not p['is_ready']:
                # Check if they passed in this plan? 
                # Actually if they passed, AP becomes 0 in 'apply_action'.
                # So if AP > 0, they did NOT pass and did not use all AP.
                unused_ap_msg.append(f"{pid} has {p['ap']} AP left.")
        
        if unused_ap_msg:
             return False, "Plan incomplete! You left unused AP. " + ", ".join(unused_ap_msg) + " -> Use Move/Interact/PickUp or explicitly action='Pass' if done."

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
            
        # Validation helpers for complex variants
        if atype == "Throw":
            if "item_index" not in payload:
                # Fallback: if item_type provided, try to hint (but we can't resolve index here easily without state)
                # For now, just raise explicit error
                raise ValueError(f"Throw requires 'item_index' (integer). Got: {payload}")
            # Ensure index is int
            try: payload["item_index"] = int(payload["item_index"])
            except: raise ValueError("Throw 'item_index' must be integer")

        if atype == "Drop":
            if "item_index" not in payload:
                raise ValueError(f"Drop requires 'item_index' (integer).")

        if atype == "PickUp":
             if "item_type" not in payload:
                 raise ValueError("PickUp requires 'item_type' (string).")

        return {"type": atype, "payload": payload}

    def _parse_plan_json(self, plan: List[Dict[str, Any]]) -> List[Tuple[str, Dict[str, Any]]]:
        out = []
        for step in plan:
            out.append((step['player_id'], self._convert_to_core_action(step['action'])))
        return out

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
                            "type": {"type": "string", "enum": ["Move", "Bake", "Shoot", "Extinguish", "Repair", "Throw", "PickUp", "Drop", "Pass", "RaiseShields", "EvasiveManeuvers", "Interact", "Lookout", "Revive", "FirstAid"]},
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
        CURRENT STATE:
        {summary}

        TASK:
        Analyze the situation. We are in Round {state['turn_count']}.
        Determine the high-level strategy for this round.
        Refer to the STRATEGY GUIDE for optimal plays (e.g. 'Perfect Start').
        Should we be Aggressive (attack boss), Defensive (heal/extinguish), or Efficient (stockpile)?
        Assign a role/goal to each player (P1-P6).
        
        CRITICAL: Verify hazards! If the Strategy says "Extinguish Fire" but the CURRENT STATE (above) shows no fire, DO NOT instructions to extinguish.
        
        OUTPUT:
        Plain text strategy description.
        """

    def _construct_action_prompt(self, state: Dict[str, Any], strategy: str, attempt: int, error_msg: Optional[str] = None) -> str:
        summary = self._state_summary(state)
        parts = [
            "STRATEGY:",
            strategy,
            "",
            "CURRENT STATE:",
            summary,
            "",
            self.task_text
        ]
        return "\n".join(parts)

    def _state_summary(self, state: Dict[str, Any]) -> str:
        hull = state['hull_integrity']
        boss = f"{state['enemy']['name']} ({state['enemy']['hp']} HP)"
        
        # Helper for BFS distance
        def get_dist(start: int, target: int, rooms: Dict) -> int:
            if start == target: return 0
            queue = [(start, 0)]
            visited = {start}
            while queue:
                curr, d = queue.pop(0)
                if curr == target: return d
                for n in rooms.get(curr, {}).get('neighbors', []):
                    if n not in visited:
                        visited.add(n)
                        queue.append((n, d+1))
            return 999 # Unreachable

        # Key Locations
        key_rooms = {
            "Kitchen": 5,
            "Cannons": 6, 
            "Bridge": 7,
            "Engine": 4,
            "Hallway": 0
        }

        players = []
        map_rooms = {r['id']: r for r in state['map']['rooms'].values()}
        
        for pid, p in state['players'].items():
            rid = p['room_id']
            # Calculate costs to key rooms
            costs = []
            for name, tid in key_rooms.items():
                d = get_dist(rid, tid, map_rooms)
                if d > 0:
                    costs.append(f"{name}({d} AP)")
            
            cost_str = ", ".join(costs)
            
            # Inventory Warning
            inv_limit = 5 if "Wheelbarrow" in p['inventory'] else 1
            inv_status = f"Inv {p['inventory']}"
            if len(p['inventory']) >= inv_limit:
                inv_status += " [FULL! Cannot PickUp]"
            
            # Neighbors for quick reference
            current_neighbors = map_rooms.get(rid, {}).get('neighbors', [])
            
            players.append(f"{pid}: Room {rid} (Adj: {current_neighbors}), AP {p['ap']}, HP {p['hp']}, {inv_status} [Path Costs: {cost_str}]")
            
        map_info = []
        for rid, r in state['map']['rooms'].items():
            info = f"Room {rid} ({r['name']}): Neighbors={r['neighbors']}"
            if r['hazards']:
                info += f", Hazards={r['hazards']}"
            if r['items']:
                info += f", Items={r['items']}"
            map_info.append(info)
        
        situations = []
        if state.get('active_situations'):
            situations.append("ACTIVE SITUATIONS (Global Effects):")
            for card in state['active_situations']:
                situations.append(f"- {card['title']}: {card['description']}")
                 
        return f"Hull: {hull}\nBoss: {boss}\nPlayers:\n" + "\n".join(players) + "\nMap:\n" + "\n".join(map_info) + "\n" + "\n".join(situations)
    
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
