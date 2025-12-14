import asyncio
import json
import uuid
import websockets
import textwrap
import os
from typing import Any, Dict, Optional, List
from google import genai
from google.genai import types as genai_types
import sint_core # type: ignore
from context import MemoryBank
from tools import load_game_tools
from bindings_wrapper import SintBindings
from game_types import GameState, Action, GamePhase, Player, Room, Card, Enemy, ProposedAction, CardId

class GameAgent:
    def __init__(self, player_id: str, room_id: str, server_url: str, max_turns: int = 0, debug: bool = False) -> None:
        self.player_id = player_id
        self.room_id = room_id
        self.server_url = server_url
        self.max_turns = max_turns
        self.debug = debug
        self.state: Optional[GameState] = None
        self.memory = MemoryBank()
        
        self.debounce_task: Optional[asyncio.Task[None]] = None
        self.turns_taken: int = 0
        
        self.tools, self.tool_map = load_game_tools()
        
        self.client = genai.Client(
            api_key=os.environ.get("GEMINI_API_KEY"),
            http_options={'api_version': 'v1alpha'}
        )
        self.model_name = 'gemini-2.5-flash-lite'
        self.system_instr = self._load_system_prompt()

        self.websocket: Optional[Any] = None

    async def run(self) -> None:
        print(f"Agent {self.player_id} connecting to {self.server_url}...")
        try:
            async with websockets.connect(self.server_url) as ws:
                self.websocket = ws
                
                # Join (Network)
                await ws.send(json.dumps({
                    "type": "Join", 
                    "payload": { "room_id": self.room_id, "player_id": self.player_id }
                }))
                
                # Join (Game State)
                await self.send_event(Action.model_validate({"type": "Join", "payload": {"name": self.player_id}}))
                
                # Sync
                await ws.send(json.dumps({ 
                    "type": "SyncRequest", 
                    "payload": { "requestor_id": self.player_id } 
                }))
                
                # Kickstart
                self.schedule_think(delay=0.1)

                async for message in ws:
                    data = json.loads(message)
                    await self.handle_message(data)
        except websockets.exceptions.ConnectionClosed:
            print("WebSocket connection closed.")

    async def send_event(self, action: Action) -> None:
        event = {
            "id": str(uuid.uuid4()),
            "player_id": self.player_id,
            "action": action.model_dump(mode='json')
        }
        msg = { "type": "Event", "payload": { "sequence_id": 0, "data": event } }
        if self.websocket:
            await self.websocket.send(json.dumps(msg))

    def _string_keys_to_int(self, d: Any) -> Any:
        if isinstance(d, dict):
            new_dict = {}
            for k, v in d.items():
                if isinstance(k, str) and k.isdigit():
                    new_key = int(k)
                else:
                    new_key = k
                new_dict[new_key] = self._string_keys_to_int(v)
            return new_dict
        elif isinstance(d, list):
            return [self._string_keys_to_int(x) for x in d]
        return d

    async def handle_message(self, data: Dict[str, Any]) -> None:
        msg_type = data.get("type")
        payload = data.get("payload")
        if payload is None: return

        if msg_type == "Welcome":
            print(f"Joined room: {payload.get('room_id')}")
            
        elif msg_type == "SyncRequest":
            req_id = payload.get("requestor_id")
            if req_id != self.player_id and self.state and self.state.sequence_id > 0:
                print("Responding to SyncRequest...")
                state_str = json.dumps(self.state.model_dump(mode='json'))
                await self.send_event(Action.model_validate({"type": "FullSync", "payload": {"state_json": state_str}}))
            
        elif msg_type == "Event":
            try:
                if not self.state:
                     self.state = SintBindings.new_game([self.player_id], 12345)
                     if self.debug:
                         print(f"DEBUG: Initial state: {self.state.model_dump(mode='json')}")

                event_data = payload.get("data", {})
                pid = event_data.get("player_id")
                action_dict = event_data.get("action", {})
                
                if self.debug:
                    print(f"DEBUG: Processing Action: {action_dict}")
                    print(f"DEBUG: Current State (dump): {self.state.model_dump(mode='json')}")

                action_model = Action.model_validate(action_dict)
                self.state = SintBindings.apply_action(self.state, pid, action_model, None)
                
                # Use model for type-safe access
                # Action (Root) -> Union[GameAction, MetaAction] (Root) -> Variant
                inner_action = action_model.root.root
                act_type = inner_action.type
                
                desc = f"Player {pid} performed {act_type}"
                # Handle specific variants if needed for logging
                from game_types import GameAction1, GameAction15
                if isinstance(inner_action, GameAction1): # Move
                    desc += f" to room {inner_action.payload.to_room}"
                elif isinstance(inner_action, GameAction15): # Chat
                    desc += f": '{inner_action.payload.message}'"
                
                print(f"Event Received: {desc}")
                self.memory.add_log(desc)
                
                # Smart Scheduling
                me = self.state.players.root.get(self.player_id)
                if not me: return
                is_ready = me.is_ready

                if pid == self.player_id:
                    # My action
                    if is_ready:
                        print("I am Ready. Waiting for others/phase change.")
                    elif act_type == "Chat":
                        print("I chatted. Waiting for reply/events.")
                    else:
                        # I did something (Move/PickUp) and I'm not ready yet. Keep planning.
                        self.schedule_think(delay=0.1)
                else:
                    # Others' action
                    if is_ready:
                        # I'm ready, I usually don't care what others do until phase change.
                        pass
                    else:
                        # I'm not ready, their action might change my plan.
                        self.schedule_think(delay=0.5)
                
            except Exception as e:
                print(f"Error applying action: {e}")
                self.memory.add_log(f"ACTION ERROR: {e}")
                self.schedule_think(delay=0.5)

    def schedule_think(self, delay: float) -> None:
        if self.debounce_task:
            self.debounce_task.cancel()
        self.debounce_task = asyncio.create_task(self.think_after_delay(delay))

    async def think_after_delay(self, delay: float) -> None:
        await asyncio.sleep(delay)
        await self.think()

    async def think(self) -> None:
        if not self.state: return
        
        # Check limit logic - AFTER state check, before acting
        if self.max_turns > 0 and self.turns_taken >= self.max_turns:
            print(f"Max turns ({self.max_turns}) reached. Closing.")
            if self.websocket: await self.websocket.close()
            return

        print("AI Thinking...")
        self.turns_taken += 1

        # Summarize
        if self.memory.should_summarize():
            print("Summarizing history...")
            chunk = self.memory.get_chunk_to_summarize()
            summary_prompt = "Summarize these game events into a concise narrative:\n" + "\n".join(chunk)
            resp = await self.client.aio.models.generate_content(
                model=self.model_name,
                contents=summary_prompt
            )
            if resp.text:
                self.memory.commit_summary(resp.text)

        # Context
        state = self.state
        me = state.players.root.get(self.player_id)
        if not me: return
        
        room_id = me.room_id
        # RootModel uses .root, keys are strings
        room = state.map.rooms.root.get(str(room_id))
        
        # History
        memory_text = self.memory.get_full_context_text()
        
        # Chat Log (Source of Truth)
        chat_log = state.chat_log
        recent_chat = chat_log[-10:] # Last 10 messages
        chat_lines = []
        for msg in recent_chat:
            sender = msg.sender
            text = msg.text
            if sender == self.player_id:
                chat_lines.append(f"CHAT: YOU ({sender}): {text}")
            else:
                chat_lines.append(f"CHAT: {sender}: {text}")
        chat_text = "\n".join(chat_lines)
        
        status_desc = f"YOU ARE: {me.name} (ID: {self.player_id})\nSTATUS: HP {me.hp}/3, AP {me.ap}/2. Inventory: {me.inventory}"
        room_desc = f"Room {room_id} ({room.name if room else 'Unknown'}). Items={room.items if room else []}, Hazards={room.hazards if room else []}, ConnectsTo={room.neighbors if room else []}. People: {[p.name for p in state.players.root.values() if p.room_id == room_id]}"
        
        team_status = "TEAM STATUS:\n"
        for pid, p in state.players.root.items():
            ready_mark = "[READY]" if p.is_ready else "[WAITING]"
            team_status += f"- {p.name} (ID: {pid}): Room {p.room_id}, HP {p.hp}/3 {ready_mark}\n"

        queue_desc = "PLANNED ACTIONS:\n"
        queue = state.proposal_queue
        if queue:
            for proposed in queue:
                queue_desc += f"- {proposed.player_id}: {proposed.action} [ID: {proposed.id}]\n"
        else:
            queue_desc += "(None)\n"
        queue_desc += "(Use action_undo(action_id='ID') to cancel your own actions)\n"

        ap = me.ap
        ap_warning = ""
        if ap <= 0:
            ap_warning = "WARNING: YOU HAVE 0 AP REMAINING. If you need to change your plan, you MUST use `action_undo` first. Undo refunds the AP cost of the action, giving you AP back to use again."
        else:
            ap_warning = f"You have {ap} AP remaining. VoteReady executes queued actions and keeps remaining AP. Pass DESTROYS remaining AP."

        phase = state.phase
        
        phase_hint = ""
        if phase == GamePhase.Lobby:
            phase_hint = "HINT: In Lobby, you can ONLY Chat (0 AP), SetName (0 AP), or VoteReady (0 AP). You CANNOT Move or act yet. VoteReady starts the game."
        elif phase == GamePhase.TacticalPlanning:
            phase_hint = "HINT: Propose actions. When your plan is set, VoteReady to execute. ONLY use Pass if you want to forfeit your remaining AP."
        elif phase in [GamePhase.MorningReport, GamePhase.EnemyTelegraph]:
            phase_hint = "HINT: Read the report. You CANNOT Move or Act in this phase. You MUST VoteReady (0 AP) to advance to 'TacticalPlanning' where you can then move/act."

        active_cards = state.active_situations
        
        situation_desc = ""
        latest_event = state.latest_event
        if latest_event:
             situation_desc += f"JUST DRAWN EVENT: {latest_event.title}: {latest_event.description}\n"

        if active_cards:
            situation_desc += "ACTIVE CARDS:\n" + "\n".join([f"- {c.title}: {c.description}" for c in active_cards])

        enemy = state.enemy
        next_attack = enemy.next_attack
        enemy_intent = ""
        if next_attack:
            enemy_intent = f"ENEMY INTENT: The enemy is targeting Room {next_attack.target_room} with {next_attack.effect}!"
        else:
            enemy_intent = "ENEMY INTENT: Unknown (Hidden or not yet revealed)."

        # Map Topology
        map_desc = "SHIP LAYOUT:\n"
        try:
            rooms = state.map.rooms.root.values()
            sorted_rooms = sorted(rooms, key=lambda x: x.id)
            for r in sorted_rooms:
                map_desc += f"- Room {r.id} ({r.name}): Items={r.items}, Hazards={r.hazards}, ConnectsTo={r.neighbors}\n"
        except Exception as e:
            map_desc += f"Error reading map: {e}\n"

        prompt_parts = [
            f"PHASE: {phase}",
            phase_hint,
            ap_warning,
            situation_desc,
            enemy_intent,
            "",
            map_desc,
            "RECENT EVENTS:",
            memory_text,
            "",
            "CHAT HISTORY:",
            chat_text,
            "",
            team_status,
            queue_desc,
            "CURRENT SITUATION:",
            room_desc,
            status_desc,
            f"Hull: {state.hull_integrity}. Turn: {state.turn_count}.",
            ""
        ]
        
        prompt = "\n".join(prompt_parts)
        
        if self.debug:
            print(f"DEBUG: Prompt sent to AI:\n{prompt}")
        
        # Filter Tools
        all_funcs = self.tools.function_declarations or []
        allowed_names = {"action_chat", "action_fullsync", "action_join", "action_setname"} # Base tools

        # 1. Get Valid Actions from Core
        # We pass the raw state dict directly
        valid_actions_raw = SintBindings.get_valid_actions(self.state, self.player_id)
        
        # 2. Map to Tool Names
        for act in valid_actions_raw:
            # Action (Root) -> Union[GameAction, MetaAction] (Root) -> Variant (e.g. GameAction1)
            # All variants have a 'type' field.
            act_type = act.root.root.type
            allowed_names.add(f"action_{act_type.lower()}")
            
        filtered_funcs = [fn for fn in all_funcs if fn.name in allowed_names]
        current_tool_config = genai_types.Tool(function_declarations=filtered_funcs)

        config = genai_types.GenerateContentConfig(
            system_instruction=self.system_instr,
            tools=[current_tool_config]
        )

        try:
            response = await self.client.aio.models.generate_content(
                model=self.model_name,
                contents=[prompt],
                config=config
            )
            
            if response.candidates and response.candidates[0].content and response.candidates[0].content.parts:
                for part in response.candidates[0].content.parts:
                    if fn := part.function_call:
                        if fn.name:
                            print(f"AI decided to: {fn.name}")
                            args = dict(fn.args) if fn.args else {}
                            await self.execute_tool(fn.name, args)
                    if part.text:
                        print(f"AI Thought: {part.text}")
                    
        except Exception as e:
            print(f"AI Generation Error: {e}")
            self.schedule_think(delay=0.5)

        # Check exit condition after turn completion
        if self.max_turns > 0 and self.turns_taken >= self.max_turns:
            print(f"Max turns ({self.max_turns}) reached. Closing.")
            if self.websocket: await self.websocket.close()

    def _load_system_prompt(self) -> str:
        try:
            base_dir = os.path.dirname(os.path.dirname(__file__)) # Up one level to root
            rules_path = os.path.join(base_dir, 'docs', 'rules.md')
            prompt_path = os.path.join(os.path.dirname(__file__), 'system_prompt.txt')
            
            with open(prompt_path, 'r') as f:
                prompt = f.read().format(player_id=self.player_id)
            
            with open(rules_path, 'r') as f:
                rules = f.read()
                
            return f"{prompt}\n\n=== GAME RULES ===\n{rules}"
        except Exception as e:
            print(f"Error loading system prompt: {e}")
            return f"You are {self.player_id}. Cooperate to survive."

    async def execute_tool(self, tool_name: str, args: Dict[str, Any]) -> None:
        if self.debug:
            print(f"DEBUG: Executing tool {tool_name} with args: {args}")

        if tool_name.startswith("action_"):
            # Use mapping if available, otherwise heuristic
            action_type = self.tool_map.get(tool_name)
            if not action_type:
                 action_type = tool_name.replace("action_", "").capitalize()

            clean_args = dict(args)
            if "to_room" in clean_args:
                try: clean_args["to_room"] = int(clean_args["to_room"])
                except: pass

            if "item_index" in clean_args:
                try: clean_args["item_index"] = int(clean_args["item_index"])
                except: pass
            
            # If no args, pass None (for Unit Variants in Rust)
            payload = clean_args if clean_args else None
            
            action_model = Action.model_validate({"type": action_type, "payload": payload})
            await self.send_event(action_model)
            print(f"Sent action: {action_type}")
