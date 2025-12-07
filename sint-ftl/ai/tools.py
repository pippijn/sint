import json
from typing import Any, Dict, List
import google.generativeai as genai
from google.generativeai.types import Tool, FunctionDeclaration
import sint_core # type: ignore

def load_game_tools() -> Tool:
    """Dynamically converts Rust Schema into Gemini Tools."""
    schema_json: str = sint_core.get_schema_json()
    schema = json.loads(schema_json)
    
    funcs: List[FunctionDeclaration] = []
    
    if "oneOf" in schema:
        for variant in schema["oneOf"]:
            props = variant.get("properties", {})
            type_field = props.get("type", {})
            action_name = type_field.get("const") or type_field.get("enum", [None])[0]
            
            if not action_name:
                continue
            
            # Skip internal/system actions if needed (like Join/FullSync might not be for AI decision?)
            # Actually AI uses Join/FullSync via manual calls, but decision model uses "action_..." tools.
            # We can filter out unwanted tools here if necessary.
            
            tool_name = f"action_{action_name.lower()}"
            payload_schema = props.get("payload", {"type": "object", "properties": {}})
            
            clean_schema(payload_schema)
            
            funcs.append(FunctionDeclaration(
                name=tool_name,
                description=f"Propose action: {action_name}",
                parameters=payload_schema
            ))
            
    return Tool(function_declarations=funcs)

def clean_schema(s: Dict[str, Any]) -> None:
    """Sanitizes JSON schema for Gemini."""
    if not isinstance(s, dict):
        return

    allowed_keys = {"type", "format", "description", "nullable", "enum", "properties", "required", "items"}
    keys_to_remove = [k for k in s.keys() if k not in allowed_keys]
    for k in keys_to_remove:
        s.pop(k, None)
        
    # Recurse
    if "properties" in s and isinstance(s["properties"], dict):
        for prop_schema in s["properties"].values():
            clean_schema(prop_schema)
            
    if "items" in s:
        if isinstance(s["items"], dict):
            clean_schema(s["items"])
        elif isinstance(s["items"], list):
            for item in s["items"]:
                clean_schema(item)

    # Fixup required
    if "required" in s and "properties" in s:
        s["required"] = [k for k in s["required"] if k in s["properties"]]
        if not s["required"]:
            s.pop("required")
