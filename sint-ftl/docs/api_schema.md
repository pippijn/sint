# **SINT FTL: API & DATA SCHEMA**

## **1. CORE TYPES (Shared Rust/TS/Python)**

These types are defined in `sint-core` and exported via `schemars` to JSON Schema.

### **A. GameState**
```json
{
  "version": 1,
  "sequence_id": 105,
  "phase": "Planning", // Planning, Execution, EnemyAction, GameOver
  "total_team_ap": 8,
  "hull_integrity": 18,
  "rng_seed": "SECRET", // Hidden from clients in practice, or used for deterministic replay
  "turn_count": 3,
  
  "players": {
    "P1": {
      "id": "P1",
      "name": "Captain",
      "room_id": 9, // Bridge
      "hp": 3,
      "ap": 2,
      "inventory": ["Extinguisher", "Peppernut"],
      "status": [] // e.g., "Fainted"
    }
  },

  "map": {
    "rooms": {
      "6": { 
        "id": 6, 
        "name": "Kitchen", 
        "systems": ["Oven"], 
        "hazards": ["Fire", "Fire"], // 2 Fire tokens
        "items": ["Peppernut", "Peppernut"] 
      }
    }
  },

  "enemy": {
    "name": "The Petty Thief",
    "hp": 5,
    "next_attack": {
      "target_room": 6,
      "type": "Fireball"
    }
  },

  "chat_log": [
    { "sender": "P1", "msg": "Go to kitchen!", "timestamp": 123456 }
  ],

  "proposal_queue": [
    { "player_id": "P2", "action": "Move(6)" }
  ]
}
```

---

## **2. ACTIONS (The Toolset)**

The AI and Clients send these payloads.

### **`Action` Enum**

| Type | Payload | Description |
| :--- | :--- | :--- |
| **`Move`** | `{ "to_room": int }` | Move pawn to adjacent room. Costs 1 AP. |
| **`Bake`** | `{}` | Kitchen: Create Peppernuts. Costs 1 AP. |
| **`Shoot`** | `{}` | Cannons: Load & Fire at Enemy. Costs 1 AP + 1 Ammo. |
| **`RaiseShields`** | `{}` | Bridge: Block next damage. Costs 2 AP. |
| **`EvasiveManeuvers`** | `{}` | Engine: Dodge next attack. Costs 2 AP. |
| **`Interact`** | `{}` | Generic Interaction (e.g. solve card). Costs 1 AP. |
| **`Extinguish`** | `{}` | Remove 1 Fire token. Costs 1 AP. |
| **`Repair`** | `{}` | Remove 1 Water token. Costs 1 AP. |
| **`Throw`** | `{ "target_player": "ID", "item": "Peppernut" }` | Throw item to adjacent player. Costs 1 AP. |
| **`PickUp`** | `{ "item_type": ItemType }` | Pick up item from floor. Costs 1 AP. |
| **`Drop`** | `{ "item_index": int }` | Drop item to floor. Free? (Or 1 AP). |
| **`Revive`** | `{ "target_player": "ID" }` | Revive a Fainted player in the same room. Costs 1 AP. |
| **`Chat`** | `{ "message": string }` | Send a message. Free. (Restricted during Silence). |
| **`Pass`** | `{}` | Forfeit remaining AP for this round. |
| **`VoteReady`** | `{ "ready": bool }` | Vote to Execute the current batch. |

---

## **3. AI TOOL DEFINITIONS**

The AI Agent receives these Function Declarations (generated from the Schema).

### **`get_state()`**
*   **Returns:** Full `GameState` JSON (sanitized: no hidden info).

### **`simulate_plan(actions: List[Action])`**
*   **Input:** `["Move(6)", "Extinguish"]`
*   **Returns:**
    ```json
    {
      "valid": true,
      "outcome_description": "You moved to Kitchen. You extinguished 1 Fire. 0 AP Remaining.",
      "stopped_at_rng": false
    }
    ```

### **`propose_action(action: Action)`**
*   **Input:** `Move { to_room: 6 }`
*   **Effect:** Adds action to the global `proposal_queue`.

### **`vote_ready()`**
*   **Effect:** Signals "I am done planning."

---

## **4. ERROR CODES**

*   `ERR_NO_AP`: "Not enough Action Points."
*   `ERR_INVALID_MOVE`: "Cannot move there (No door)."
*   `ERR_BLOCKED`: "Room is blocked by Hazard."
*   `ERR_SILENCED`: "Chat forbidden (Emoji only)."
