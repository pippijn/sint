import asyncio
import json
import os
import sys
from typing import Any, Dict, List, Optional
import google.generativeai as genai 
from google.generativeai.types import FunctionDeclaration, Tool 
import sint_core # type: ignore

# Configure Gemini
API_KEY = os.environ.get("GEMINI_API_KEY")
if not API_KEY:
    print("Error: GEMINI_API_KEY not set")
    # sys.exit(1) # Commented out for now to allow testing without key

genai.configure(api_key=API_KEY) # type: ignore[attr-defined]

class GameAgent:
    def __init__(self, player_id: str, room_id: str, server_url: str = "ws://localhost:3000/ws") -> None:
        self.player_id = player_id
        self.room_id = room_id
        self.server_url = server_url
        self.state: Optional[Dict[str, Any]] = None
        self.history: List[str] = []
        
        # Load Tools from Rust Schema
        self.tools = self._load_tools()
        
        # Initialize Model
        self.model = genai.GenerativeModel( # type: ignore[attr-defined]
            model_name='gemini-2.5-flash-lite',
            tools=[self.tools],
            system_instruction="""
            You are a crew member of The Steamboat playing 'Operation Peppernut'.
            Your goal is to cooperate with other players to survive.
            
            GUIDELINES:
            1. Always discuss plans before committing.
            2. Use 'simulate_plan' to check if your move is valid.
            3. Be strictly logical but roleplay your character.
            4. If the status is 'SILENCE', use only Emojis.
            
            You have 2 AP per round.
            """
        )
        self.chat = self.model.start_chat(enable_automatic_function_calling=True)

    def _load_tools(self) -> Tool:
        """
        Dynamically converts the Rust JSON Schema into Gemini Function Declarations.
        Explodes the 'oneOf' Action enum into individual tools (e.g., Move, Bake).
        """
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
        # Rust schema structure for Enum with Tag="type", Content="payload":
        # { "oneOf": [ 
        #    { "properties": { "type": { "const": "Move" }, "payload": { ... } } },
        #    { "properties": { "type": { "const": "Bake" } } }
        # ]}
        
        if "oneOf" in schema:
            for variant in schema["oneOf"]:
                # Extract Action Name (e.g., "Move")
                props = variant.get("properties", {})
                type_field = props.get("type", {})
                action_name = type_field.get("const") or type_field.get("enum", [None])[0]
                
                if not action_name:
                    continue
                    
                tool_name = f"action_{action_name.lower()}"
                
                # Extract Payload Schema (Arguments)
                payload_schema = props.get("payload", {"type": "object", "properties": {}})
                
                # Sanitize payload schema (remove $schema, title, validation keywords)
                def clean(s: Dict[str, Any]) -> Dict[str, Any]:
                    if isinstance(s, dict):
                        # List of allowed keys in Gemini Function Schema
                        allowed_keys = {"type", "format", "description", "nullable", "enum", "properties", "required", "items"}
                        
                        # Keys to remove
                        keys_to_remove = [k for k in s.keys() if k not in allowed_keys]
                        for k in keys_to_remove:
                            s.pop(k, None)
                        
                        # Recurse
                        for v in s.values(): 
                            if isinstance(v, dict):
                                clean(v)
                        
                        # Fixup: Ensure 'required' only lists keys that actually exist in 'properties'
                        if "required" in s and "properties" in s:
                            s["required"] = [k for k in s["required"] if k in s["properties"]]
                            if not s["required"]:
                                s.pop("required") # Remove empty required list
                                
                    return s
                    
                clean(payload_schema)

                funcs.append(FunctionDeclaration(
                    name=tool_name,
                    description=f"Propose action: {action_name}",
                    parameters=payload_schema
                ))

        # 3. Simulate Tool (Advanced: Needs full schema, but simplified)
        # Since we can't pass 'oneOf' to Gemini easily for the array items,
        # we might skip simulate_plan for the very first test, or use a simplified generic object.
        # For now, let's omit simulate_plan to verify the individual actions work first.
        
        return Tool(function_declarations=funcs)

    async def run(self) -> None:
        print(f"Agent {self.player_id} starting...")
        # TODO: WebSocket Connection Logic
        # For now, we just test the loop locally
        
        # Mock State for testing
        print("Tools loaded successfully.")
        
        # Example: Ask LLM what to do
        response = self.chat.send_message("I just woke up. What is the situation?")
        
        # Handle response (Text or Function Call)
        if response.parts:
            for part in response.parts:
                if fn := part.function_call:
                    print(f"Agent chose to call tool: {fn.name} with args: {dict(fn.args)}")
                if part.text:
                    print(f"Agent Text: {part.text}")

if __name__ == "__main__":
    agent = GameAgent("AI_1", "Room_Test")
    asyncio.run(agent.run())
