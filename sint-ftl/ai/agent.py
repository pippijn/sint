import asyncio
import json
import os
import sys
import uuid
import websockets
from typing import Any, Dict, List, Optional
import google.generativeai as genai 
from google.generativeai.types import FunctionDeclaration, Tool 
import sint_core # type: ignore

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
        self.state_json: Dict[str, Any] = {} # Raw JSON state
        
        # Load Tools
        self.tools = self._load_tools()
        
        # Initialize Model
        self.model = genai.GenerativeModel( # type: ignore[attr-defined]
            model_name='gemini-2.5-flash-lite',
            tools=[self.tools],
            system_instruction=f"""
            You are {player_id}, a crew member of The Steamboat playing 'Operation Peppernut'.
            Your goal is to cooperate with other players to survive.
            
            GUIDELINES:
            1. Always discuss plans before committing.
            2. Be strictly logical but roleplay your character.
            3. You have 2 AP per round.
            
            When you decide to perform an action, call the corresponding tool.
            If you want to chat, call 'action_chat' and provide the 'message' argument with your text.
            """
        )
        self.chat = self.model.start_chat(enable_automatic_function_calling=True)
        self.websocket: Optional[Any] = None

    def _load_tools(self) -> Tool:
        schema_json: str = sint_core.get_schema_json()
        schema = json.loads(schema_json)
        
        funcs: List[FunctionDeclaration] = []
        
        # 1. get_state Tool
        funcs.append(FunctionDeclaration(
            name="get_state",
            description="Get the full current game state JSON.",
            parameters={"type": "object", "properties": {}}
        ))
        
        # 2. Parse 'oneOf' to create specific Action tools
        if "oneOf" in schema:
            for variant in schema["oneOf"]:
                props = variant.get("properties", {})
                type_field = props.get("type", {})
                action_name = type_field.get("const") or type_field.get("enum", [None])[0]
                
                if not action_name:
                    continue
                    
                tool_name = f"action_{action_name.lower()}"
                payload_schema = props.get("payload", {"type": "object", "properties": {}})
                
                # Sanitize payload schema
                def clean(s: Dict[str, Any]) -> None:
                    if not isinstance(s, dict):
                        return

                    allowed_keys = {"type", "format", "description", "nullable", "enum", "properties", "required", "items"}
                    keys_to_remove = [k for k in s.keys() if k not in allowed_keys]
                    for k in keys_to_remove:
                        s.pop(k, None)
                        
                    # Recurse specifically into container keywords
                    if "properties" in s and isinstance(s["properties"], dict):
                        for prop_schema in s["properties"].values():
                            clean(prop_schema)
                            
                    if "items" in s:
                        if isinstance(s["items"], dict):
                            clean(s["items"])
                        elif isinstance(s["items"], list):
                            for item in s["items"]:
                                clean(item)

                    # Fixup: Ensure 'required' only lists keys that actually exist in 'properties'
                    if "required" in s and "properties" in s:
                        s["required"] = [k for k in s["required"] if k in s["properties"]]
                        if not s["required"]:
                            s.pop("required")

                clean(payload_schema)
                # DEBUG: Print schema for chat to verify required fields
                if action_name == "Chat":
                    print(f"DEBUG: Chat Schema: {json.dumps(payload_schema)}")
                    
                funcs.append(FunctionDeclaration(name=tool_name, description=f"Propose action: {action_name}", parameters=payload_schema))
        
        return Tool(function_declarations=funcs)

    async def run(self) -> None:
        print(f"Agent {self.player_id} connecting to {self.server_url}...")
        async with websockets.connect(self.server_url) as ws:
            self.websocket = ws
            
            # Join
            join_msg = {
                "type": "Join",
                "payload": { "room_id": self.room_id, "player_id": self.player_id }
            }
            await ws.send(json.dumps(join_msg))
            
            # Send Join Action to GameState
            join_action_payload = {
                "id": str(uuid.uuid4()),
                "player_id": self.player_id,
                "action": {
                    "type": "Join",
                    "payload": { "name": self.player_id }
                }
            }
            
            msg = {
                "type": "Event",
                "payload": {
                    "sequence_id": 0,
                    "data": join_action_payload
                }
            }
            await ws.send(json.dumps(msg))
            
            # Initial AI Prompt
            asyncio.create_task(self.ai_loop())
            
            # Message Loop
            try:
                async for message in ws:
                    data = json.loads(message)
                    await self.handle_message(data)
            except websockets.exceptions.ConnectionClosed:
                print("WebSocket connection closed.")

    async def handle_message(self, data: Dict[str, Any]) -> None:
        msg_type = data.get("type")
        payload = data.get("payload")
        
        if payload is None:
             return

        if msg_type == "Welcome":
            print(f"Joined room: {payload.get('room_id')}")
            
        elif msg_type == "Event":
            # Apply to local state
            # payload is the ProposedAction from server
            # We need to construct the Action object and apply it
            try:
                # We need the current state object (Python dict -> Rust struct is handled in bindings)
                # But we don't have the full state yet unless we initialized it.
                # For now, let's just assume we start from a new game or we need to implement Sync.
                # Simplification: We blindly restart state on new execution or just track updates?
                # The server doesn't send full state, only events. 
                # The agent needs to initialize state same as client.
                
                if not self.state_json:
                     # Initialize
                     # TODO: Sync with server seed/players
                     # For now, create new game
                     print("Initializing local state...")
                     # We need to call new_game with correct players
                     # This is a limitation of current simple Agent: It doesn't know other players yet.
                     # Let's just create a single player game for itself to not crash.
                     self.state_json = sint_core.new_game([self.player_id], 12345)

                # Extract action and player_id
                event_data = payload.get("data", {})
                pid = event_data.get("player_id")
                action_data = event_data.get("action")
                
                # Apply
                self.state_json = sint_core.apply_action_with_id(
                    self.state_json, 
                    pid, 
                    action_data, 
                    None
                )
                print(f"State updated. Seq: {self.state_json.get('sequence_id')}")
                
            except Exception as e:
                print(f"Error applying action: {e}")

    async def ai_loop(self) -> None:
        """Periodically ask AI what to do."""
        turns = 0
        while True:
            if self.max_turns > 0 and turns >= self.max_turns:
                print(f"Max turns ({self.max_turns}) reached. Exiting.")
                if self.websocket:
                    await self.websocket.close()
                return

            await asyncio.sleep(10) # Wait 10 seconds between thoughts
            
            if not self.state_json:
                continue
                
            print("AI Thinking...")
            turns += 1
            
            # Construct Rich Context
            try:
                state = self.state_json
                me = state['players'].get(self.player_id)
                if not me:
                    continue
                    
                room_id = me['room_id']
                # pythonize preserves integer keys, but JSON uses strings. Try both.
                room = state['map']['rooms'].get(room_id) or state['map']['rooms'].get(str(room_id))
                
                if not room:
                    print(f"Error: Room {room_id} not found in map.")
                    continue
                
                # Nearby Info
                location_desc = f"You are in Room {room_id} ({room.get('name', 'Unknown')})."
                doors = f"Doors to: {room.get('neighbors', [])}."
                
                # Players here
                players_here = [p['name'] for p in state['players'].values() if p['room_id'] == room_id]
                people_desc = f"People here: {players_here}."
                
                # Hazards
                hazards = room.get('hazards', [])
                hazard_desc = f"Hazards: {hazards}." if hazards else "Room is safe."
                
                # System
                system = room.get('system')
                sys_desc = f"System here: {system}." if system else "No system."
                
                status_desc = f"Your Status: HP {me['hp']}/3, AP {me['ap']}/2. Inventory: {me['inventory']}."
                
                prompt = f"""
                STATUS UPDATE:
                {location_desc}
                {sys_desc}
                {hazard_desc}
                {doors}
                {people_desc}
                {status_desc}
                
                Hull Integrity: {state['hull_integrity']}.
                Turn: {state['turn_count']}.
                
                What is your next move? (Call a tool)
                """
                
                print(f"DEBUG: Prompt sent to AI:\n{prompt}")
                response = await self.chat.send_message_async(prompt)
                
                for part in response.parts:
                    if fn := part.function_call:
                        print(f"AI decided to: {fn.name}")
                        await self.execute_tool(fn.name, dict(fn.args))
                    if part.text:
                         print(f"AI Thought: {part.text}")
            except Exception as e:
                print(f"AI Error: {e}")

    async def execute_tool(self, tool_name: str, args: Dict[str, Any]) -> None:
        print(f"DEBUG: Tool {tool_name} called with args: {args}")
        if tool_name == "get_state":
            # The AI already has access via prompt, but maybe it wants full json
            pass
            
        elif tool_name.startswith("action_"):
            action_type = tool_name.replace("action_", "").capitalize()
            
            # Clean up Args
            clean_args = dict(args)
            if "to_room" in clean_args:
                try:
                    clean_args["to_room"] = int(clean_args["to_room"])
                except:
                    pass
            
            # Construct Action payload
            action_payload = {
                "type": action_type,
                "payload": clean_args
            }
            
            # Wrap in ProposedAction for Server
            proposal = {
                "id": str(uuid.uuid4()),
                "player_id": self.player_id,
                "action": action_payload
            }
            
            msg = {
                "type": "Event",
                "payload": {
                    "sequence_id": 0,
                    "data": proposal
                }
            }
            
            if self.websocket:
                await self.websocket.send(json.dumps(msg))
                print(f"Sent action: {action_type}")

if __name__ == "__main__":
    # Ensure API Key
    if not os.environ.get("GEMINI_API_KEY"):
         print("Skipping execution: No API Key")
         sys.exit(0)

    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--player", default="AI_Bot", help="Player ID")
    parser.add_argument("--room", default="Room_A", help="Room ID")
    parser.add_argument("--url", default="ws://localhost:3000/ws", help="Server URL")
    parser.add_argument("--max-turns", type=int, default=0, help="Max turns to run (0=infinite)")
    args = parser.parse_args()

    agent = GameAgent(args.player, args.room, args.url, max_turns=args.max_turns)
    asyncio.run(agent.run())