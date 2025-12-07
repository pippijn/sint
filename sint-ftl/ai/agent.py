import argparse
import asyncio
import json
import os
import sys
import uuid
import websockets
from typing import Any, Dict, List, Optional
import google.generativeai as genai 
from google.generativeai.types import Tool
import sint_core # type: ignore
from context import MemoryBank

# Configure Gemini
API_KEY = os.environ.get("GEMINI_API_KEY")
if not API_KEY:
    print("Error: GEMINI_API_KEY not set")

genai.configure(api_key=API_KEY) # type: ignore[attr-defined]

class GameAgent:
    def __init__(self, player_id: str, room_id: str, server_url: str = "ws://localhost:3000/ws", max_turns: int = 0) -> None:
        self.player_id = player_id
        self.room_id = room_id
        self.server_url = server_url
        self.max_turns = max_turns
        self.state_json: Dict[str, Any] = {}
        self.memory = MemoryBank()
        
        self.debounce_task: Optional[asyncio.Task[None]] = None
        self.last_seq: int = 0
        self.turns_taken: int = 0
        
        self.tools = self._load_tools()
        self.model = genai.GenerativeModel( # type: ignore[attr-defined]
            model_name='gemini-2.5-flash-lite',
            tools=[self.tools],
        )
        self.websocket: Optional[Any] = None

    def _load_tools(self) -> Tool:
        # (Same as before, preserving schema cleaning logic)
        schema_json: str = sint_core.get_schema_json()
        schema = json.loads(schema_json)
        funcs = []
        # get_state
        funcs.append(genai.types.FunctionDeclaration(name="get_state", description="Get state", parameters={"type": "object", "properties": {}}))
        
        if "oneOf" in schema:
            for variant in schema["oneOf"]:
                props = variant.get("properties", {})
                type_field = props.get("type", {})
                action_name = type_field.get("const") or type_field.get("enum", [None])[0]
                if not action_name: continue
                tool_name = f"action_{action_name.lower()}"
                payload_schema = props.get("payload", {"type": "object", "properties": {}})
                
                def clean(s: Dict[str, Any]) -> None:
                    if not isinstance(s, dict): return
                    allowed = {"type", "format", "description", "nullable", "enum", "properties", "required", "items"}
                    for k in list(s.keys()):
                        if k not in allowed: s.pop(k, None)
                    if "properties" in s and isinstance(s["properties"], dict):
                        for v in s["properties"].values(): clean(v)
                    if "items" in s:
                        if isinstance(s["items"], dict): clean(s["items"])
                        elif isinstance(s["items"], list):
                            for i in s["items"]: clean(i)
                    if "required" in s and "properties" in s:
                        s["required"] = [k for k in s["required"] if k in s["properties"]]
                        if not s["required"]: s.pop("required")

                clean(payload_schema)
                funcs.append(genai.types.FunctionDeclaration(name=tool_name, description=f"Propose action: {action_name}", parameters=payload_schema))
        
        return Tool(function_declarations=funcs)

    async def run(self) -> None:
        print(f"Agent {self.player_id} connecting to {self.server_url}...")
        try:
            async with websockets.connect(self.server_url) as ws:
                self.websocket = ws
                
                # Join
                join_msg = { "type": "Join", "payload": { "room_id": self.room_id, "player_id": self.player_id } }
                await ws.send(json.dumps(join_msg))
                
                # Send Join Event
                join_event = {
                    "type": "Event",
                    "payload": {
                        "sequence_id": 0,
                        "data": {
                            "id": str(uuid.uuid4()),
                            "player_id": self.player_id,
                            "action": { "type": "Join", "payload": { "name": self.player_id } }
                        }
                    }
                }
                await ws.send(json.dumps(join_event))
                
                # Initial Think (Kickstart)
                self.schedule_think(delay=2.0)

                async for message in ws:
                    data = json.loads(message)
                    await self.handle_message(data)
        except websockets.exceptions.ConnectionClosed:
            print("WebSocket connection closed.")

    async def handle_message(self, data: Dict[str, Any]) -> None:
        msg_type = data.get("type")
        payload = data.get("payload")
        if payload is None: return

        if msg_type == "Welcome":
            print(f"Joined room: {payload.get('room_id')}")
            
        elif msg_type == "Event":
            try:
                # 1. Update State
                if not self.state_json:
                     self.state_json = sint_core.new_game([self.player_id], 12345)

                event_data = payload.get("data", {})
                pid = event_data.get("player_id")
                action = event_data.get("action", {})
                
                self.state_json = sint_core.apply_action_with_id(self.state_json, pid, action, None)
                seq = self.state_json.get('sequence_id', 0)
                
                # 2. Log Event (Human Readable)
                action_type = action.get("type")
                action_payload = action.get("payload", {})
                
                # Simple formatter
                desc = f"Player {pid} performed {action_type}"
                if action_type == "Move":
                    desc += f" to room {action_payload.get('to_room')}"
                elif action_type == "Chat":
                    desc += f": '{action_payload.get('message')}'"
                
                print(f"Event Received: {desc}")
                self.memory.add_log(desc)
                
                # 3. Schedule Think (Debounce)
                # If it was ME acting, maybe think faster? Or wait?
                # If it was someone else, wait for them to finish.
                self.schedule_think(delay=5.0)
                
            except Exception as e:
                print(f"Error applying action: {e}")

    def schedule_think(self, delay: float) -> None:
        if self.debounce_task:
            self.debounce_task.cancel()
        self.debounce_task = asyncio.create_task(self.think_after_delay(delay))

    async def think_after_delay(self, delay: float) -> None:
        await asyncio.sleep(delay)
        await self.think()

    async def think(self) -> None:
        if not self.state_json: return
        if self.max_turns > 0 and self.turns_taken >= self.max_turns:
            print(f"Max turns ({self.max_turns}) reached. Closing.")
            if self.websocket: await self.websocket.close()
            return

        print("AI Thinking...")
        self.turns_taken += 1

        # 1. Summarize if needed
        if self.memory.should_summarize():
            print("Summarizing history...")
            chunk = self.memory.get_chunk_to_summarize()
            summary_prompt = "Summarize these game events into a concise narrative bullet point:\n" + "\n".join(chunk)
            # Use a separate simpler model call or same model
            resp = await self.model.generate_content_async(summary_prompt)
            if resp.text:
                self.memory.commit_summary(resp.text)
                print(f"Summary committed: {resp.text[:50]}...")

        # 2. Construct Prompt
        state = self.state_json
        me = state['players'].get(self.player_id)
        if not me: return
        
        room_id = me['room_id']
        room = state['map']['rooms'].get(room_id) or state['map']['rooms'].get(str(room_id))
        
        # Context
        history_text = self.memory.get_full_context_text()
        
        # Current Snapshot
        status_desc = f"HP {me['hp']}/3, AP {me['ap']}/2. Inventory: {me['inventory']}"
        room_desc = f"Room {room_id} ({room.get('name')}). Hazards: {room.get('hazards')}. People: {[p['name'] for p in state['players'].values() if p['room_id'] == room_id]}"
        
        system_instruction = f"""
        You are {self.player_id}, a crew member of The Steamboat in 'Operation Peppernut'.
        Goal: Cooperate to survive.
        Roleplay: Logical but characterful.
        AP: You have 2 AP per turn.
        
        Use tools to act. Use 'action_chat' to speak.
        """
        
        full_prompt = f"""
        {history_text}
        
        CURRENT SITUATION:
        {room_desc}
        {status_desc}
        Hull: {state['hull_integrity']}. Turn: {state['turn_count']}.
        
        What is your next move?
        """
        
        print("DEBUG: Prompt sent.")
        # We manually construct the request to avoid implicit history
        # We pass system_instruction as argument to generate_content if supported, or prepend.
        # genai library supports system_instruction at model init. We initialized it without (or with generic).
        # Let's just assume the initialized model has generic instructions, and we pass context in user prompt.
        
        try:
            response = await self.model.generate_content_async(
                contents=[full_prompt],
                # We can't easily swap system instructions per call in this lib version without re-init
                # So we rely on the prompt context.
            )
            
            # Execute Tools
            for part in response.parts:
                if fn := part.function_call:
                    print(f"AI decided to: {fn.name}")
                    await self.execute_tool(fn.name, dict(fn.args))
                if part.text:
                    print(f"AI Thought: {part.text}")
                    
        except Exception as e:
            print(f"AI Generation Error: {e}")

    async def execute_tool(self, tool_name: str, args: Dict[str, Any]) -> None:
        if tool_name == "get_state": return
        
        if tool_name.startswith("action_"):
            action_type = tool_name.replace("action_", "").capitalize()
            clean_args = dict(args)
            if "to_room" in clean_args:
                try: clean_args["to_room"] = int(clean_args["to_room"])
                except: pass
            
            proposal = {
                "id": str(uuid.uuid4()),
                "player_id": self.player_id,
                "action": { "type": action_type, "payload": clean_args }
            }
            
            msg = { "type": "Event", "payload": { "sequence_id": 0, "data": proposal } }
            if self.websocket:
                await self.websocket.send(json.dumps(msg))
                print(f"Sent action: {action_type}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--player", default="AI_Bot", help="Player ID")
    parser.add_argument("--room", default="Room_A", help="Room ID")
    parser.add_argument("--url", default="ws://localhost:3000/ws", help="Server URL")
    parser.add_argument("--max-turns", type=int, default=0, help="Max turns")
    args = parser.parse_args()

    agent = GameAgent(args.player, args.room, args.url, max_turns=args.max_turns)
    asyncio.run(agent.run())
