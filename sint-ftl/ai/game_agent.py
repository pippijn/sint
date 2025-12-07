import asyncio
import json
import uuid
import websockets
import textwrap
import os
from typing import Any, Dict, Optional
import google.generativeai as genai
import sint_core # type: ignore
from context import MemoryBank
from tools import load_game_tools

class GameAgent:
    def __init__(self, player_id: str, room_id: str, server_url: str, max_turns: int = 0, debug: bool = False) -> None:
        self.player_id = player_id
        self.room_id = room_id
        self.server_url = server_url
        self.max_turns = max_turns
        self.debug = debug
        self.state_json: Dict[str, Any] = {}
        self.memory = MemoryBank()
        
        self.debounce_task: Optional[asyncio.Task[None]] = None
        self.turns_taken: int = 0
        
        self.tools = load_game_tools()
        self.model = genai.GenerativeModel(
            model_name='gemini-2.5-flash-lite',
            tools=[self.tools],
        )
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
                await self.send_event("Join", { "name": self.player_id })
                
                # Sync
                await ws.send(json.dumps({ "type": "SyncRequest", "payload": None }))
                
                # Kickstart
                self.schedule_think(delay=2.0)

                async for message in ws:
                    data = json.loads(message)
                    await self.handle_message(data)
        except websockets.exceptions.ConnectionClosed:
            print("WebSocket connection closed.")

    async def send_event(self, action_type: str, payload: Dict[str, Any]) -> None:
        event = {
            "id": str(uuid.uuid4()),
            "player_id": self.player_id,
            "action": { "type": action_type, "payload": payload }
        }
        msg = { "type": "Event", "payload": { "sequence_id": 0, "data": event } }
        if self.websocket:
            await self.websocket.send(json.dumps(msg))

    async def handle_message(self, data: Dict[str, Any]) -> None:
        msg_type = data.get("type")
        payload = data.get("payload")
        if payload is None: return

        if msg_type == "Welcome":
            print(f"Joined room: {payload.get('room_id')}")
            
        elif msg_type == "SyncRequest":
            if self.state_json and self.state_json.get("sequence_id", 0) > 0:
                print("Responding to SyncRequest...")
                state_str = json.dumps(self.state_json)
                await self.send_event("FullSync", { "state_json": state_str })
            
        elif msg_type == "Event":
            try:
                if not self.state_json:
                     self.state_json = sint_core.new_game([self.player_id], 12345)

                event_data = payload.get("data", {})
                pid = event_data.get("player_id")
                action = event_data.get("action", {})
                
                self.state_json = sint_core.apply_action_with_id(self.state_json, pid, action, None)
                
                action_type = action.get("type")
                action_payload = action.get("payload", {})
                
                desc = f"Player {pid} performed {action_type}"
                if action_type == "Move":
                    desc += f" to room {action_payload.get('to_room')}"
                elif action_type == "Chat":
                    desc += f": '{action_payload.get('message')}'"
                
                print(f"Event Received: {desc}")
                self.memory.add_log(desc)
                
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
            resp = await self.model.generate_content_async(summary_prompt)
            if resp.text:
                self.memory.commit_summary(resp.text)

        # Context
        state = self.state_json
        me = state['players'].get(self.player_id)
        if not me: return
        
        room_id = me['room_id']
        room = state['map']['rooms'].get(room_id) or state['map']['rooms'].get(str(room_id))
        
        # History
        memory_text = self.memory.get_full_context_text()
        
        # Chat Log (Source of Truth)
        chat_log = state.get('chat_log', [])
        recent_chat = chat_log[-10:] # Last 10 messages
        chat_text = "\n".join([f"CHAT: {msg['sender']}: {msg['text']}" for msg in recent_chat])
        
        status_desc = f"HP {me['hp']}/3, AP {me['ap']}/2. Inventory: {me['inventory']}"
        room_desc = f"Room {room_id} ({room.get('name')}). Hazards: {room.get('hazards')}. People: {[p['name'] for p in state['players'].values() if p['room_id'] == room_id]}"
        
        system_instruction = self._load_system_prompt()
        
        prompt_parts = [
            system_instruction,
            "",
            "RECENT EVENTS:",
            memory_text,
            "",
            "CHAT HISTORY:",
            chat_text,
            "",
            "CURRENT SITUATION:",
            room_desc,
            status_desc,
            f"Hull: {state['hull_integrity']}. Turn: {state['turn_count']}.",
            "",
            "What is your next move?"
        ]
        
        prompt = "\n".join(prompt_parts)
        
        if self.debug:
            print(f"DEBUG: Prompt sent to AI:\n{prompt}")
        
        try:
            response = await self.model.generate_content_async(contents=[prompt])
            
            for part in response.parts:
                if fn := part.function_call:
                    print(f"AI decided to: {fn.name}")
                    await self.execute_tool(fn.name, dict(fn.args))
                if part.text:
                    print(f"AI Thought: {part.text}")
                    
        except Exception as e:
            print(f"AI Generation Error: {e}")

        # Check exit condition after turn completion
        if self.max_turns > 0 and self.turns_taken >= self.max_turns:
            print(f"Max turns ({self.max_turns}) reached. Closing.")
            if self.websocket: await self.websocket.close()

    def _load_system_prompt(self) -> str:
        try:
            with open(os.path.join(os.path.dirname(__file__), 'system_prompt.txt'), 'r') as f:
                return f.read().format(player_id=self.player_id)
        except Exception as e:
            print(f"Error loading system prompt: {e}")
            return f"You are {self.player_id}. Cooperate to survive."

    async def execute_tool(self, tool_name: str, args: Dict[str, Any]) -> None:
        if tool_name.startswith("action_"):
            action_type = tool_name.replace("action_", "").capitalize()
            clean_args = dict(args)
            if "to_room" in clean_args:
                try: clean_args["to_room"] = int(clean_args["to_room"])
                except: pass
            
            await self.send_event(action_type, clean_args)
            print(f"Sent action: {action_type}")
