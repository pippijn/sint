import json
from typing import Any, Dict, List, Tuple
from google import genai
from google.genai import types
from bindings_wrapper import SintBindings

def load_game_tools() -> Tuple[types.Tool, Dict[str, str]]:
    """Dynamically converts Rust Schema into Gemini Tools."""
    schema_json: str = SintBindings.get_schema()
    full_schema = json.loads(schema_json)
    
    # The schema now contains state and action under properties, and models in $defs
    defs = full_schema.get("$defs", {})
    action_schema = defs.get("Action", {})
    
    funcs: List[types.FunctionDeclaration] = []
    name_map: Dict[str, str] = {}
    
    # Action is a Union of GameAction and MetaAction
    # These are further unions of specific action variants
    
    def resolve_refs(s: Any) -> Any:
        if isinstance(s, dict):
            if "$ref" in s:
                ref_path = s["$ref"].split("/")[-1]
                return resolve_refs(defs.get(ref_path, {}))
            return {k: resolve_refs(v) for k, v in s.items()}
        elif isinstance(s, list):
            return [resolve_refs(x) for x in s]
        return s

    resolved_action = resolve_refs(action_schema)
    
    # Now walk the nested unions
    # Action -> anyOf [GameAction, MetaAction]
    # GameAction -> oneOf [Move, Bake, ...]
    
    variants: List[Dict[str, Any]] = []
    
    def collect_variants(s: Any) -> None:
        if not isinstance(s, dict):
            return
        if "anyOf" in s:
            for sub in s["anyOf"]:
                collect_variants(sub)
        elif "oneOf" in s:
            for sub in s["oneOf"]:
                collect_variants(sub)
        elif "properties" in s and "type" in s["properties"]:
            variants.append(s)

    collect_variants(resolved_action)

    for variant in variants:
        props = variant.get("properties", {})
        type_field = props.get("type", {})
        action_name = type_field.get("const") or type_field.get("enum", [None])[0]
        
        if not action_name:
            continue
        
        tool_name = f"action_{action_name.lower()}"
        name_map[tool_name] = action_name
        
        payload_schema = props.get("payload", {"type": "object", "properties": {}})
        
        clean_schema(payload_schema)
        
        funcs.append(types.FunctionDeclaration(
            name=tool_name,
            description=f"Propose action: {action_name}",
            parameters=payload_schema
        ))
            
    return types.Tool(function_declarations=funcs), name_map

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
