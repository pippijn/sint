# **SINT FTL: TEST SCENARIOS**

## **1. OVERVIEW**
Scenarios are preset Game States used for:
1.  **Unit Testing:** Verifying game logic (e.g., "Does fire spread correctly?").
2.  **AI Training:** Dropping agents into specific crises to test behavior.
3.  **Tutorials:** Scripted starts for new human players.

---

## **2. SCENARIO DEFINITIONS**

### **S01: The Shooting Range (Basic)**
*   **Goal:** Test movement, baking, and shooting mechanics.
*   **Setup:**
    *   **Players:** 1 (P1 starts in Kitchen).
    *   **Inventory:** P1 has 0 Peppernuts.
    *   **Hazards:** None.
    *   **Enemy:** "Dummy Target" (3 HP, Does not attack).
*   **Success Condition:** Enemy HP = 0.

### **S02: Fire Drill (Hazards)**
*   **Goal:** Test extinguishing and AP efficiency.
*   **Setup:**
    *   **Players:** 2 (P1 in Dormitory, P2 in Hallway).
    *   **Hazards:**
        *   **Fire:** 2 tokens in Kitchen (6).
        *   **Fire:** 1 token in Cargo (4).
    *   **Enemy:** Passive.
*   **Constraint:** Kitchen is critical. If it hits 3 Fire tokens, Scenario Failed.

### **S03: The Bucket Brigade (Coop)**
*   **Goal:** Test passing/throwing items.
*   **Setup:**
    *   **Players:** 3.
    *   **Map:** Fire in Hallway (Blocking movement).
    *   **Inventory:** P1 (Kitchen) has Nuts. P3 (Cannon) needs Nuts. P2 (Bridge) is the middleman.
    *   **Enemy:** Attacking in 2 turns.
*   **Challenge:** P1 cannot reach P3. Must use P2 to relay.

### **S04: The Kraken (Boss Fight)**
*   **Goal:** Stress test full combat loop.
*   **Setup:**
    *   **Players:** 4.
    *   **Enemy:** The Kraken (20 HP).
    *   **Status:** Hull at 50%.
    *   **Cards:** "Fog Bank" active (Blind).
    *   **Hazards:** Water leaking in Engine Room.

---

## **3. SCENARIO FILE FORMAT (JSON)**
Scenarios are stored as partial `GameState` objects that override the default "New Game".

```json
{
  "scenario_id": "S02_FIRE_DRILL",
  "description": "Put out the fires before the Kitchen burns down.",
  "override_state": {
    "players": {
      "P1": { "room_id": 3 },
      "P2": { "room_id": 7 }
    },
    "map": {
      "rooms": {
        "6": { "hazards": ["Fire", "Fire"] },
        "4": { "hazards": ["Fire"] }
      }
    },
    "enemy": { "hp": 99, "behavior": "Passive" }
  }
}
```
