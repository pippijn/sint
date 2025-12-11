# **SINT FTL: OPERATION PEPPERNUT (v2.0)**

## **1. OVERVIEW**
**Operation Peppernut** is a cooperative, tactical survival game where players crew "The Steamboat" against waves of enemies. Unlike the previous party-game iteration, this version emphasizes resource scarcity, strategic positioning, and crisis management (inspired by *FTL: Faster Than Light*).

*   **Goal:** Defeat 4 Bosses in sequence (The Petty Thief, The Monster, The Armada, The Kraken).
*   **Lose Conditions:**
    *   **Hull Integrity reaches 0.**
    *   **All players are Fainted simultaneously** (Crew Wipe).

---

## **2. THE SHIP & SYSTEMS**

### **Hull Integrity**
*   The Ship starts with **20 Hull Points**.
*   **Damage Sources:**
    *   Enemy Cannonballs (Direct Hit).
    *   **Structural Burn:** If a room contains **Fire** at the end of the round, the ship takes **-1 Hull Damage**.

### **Rooms & Systems**
The ship is divided into numbered rooms. Each room houses a specific system.
**System Status:**
*   **Operational:** No Fire or Water tokens present.
*   **Disabled:** If a room has **1 or more** Fire/Water tokens, its Action is unusable.
*   **Repair:** Players must `Extinguish` (Fire) or `Repair` (Water) to restore function.

### **Map Layout (Default: Star)**
*   **Room 0 (Central Hallway):** The Hub. Connects to **all** other rooms (1-9).
*   **Outer Rooms (1-9):** Connect only to the Hub. No direct travel between outer rooms.

### **Systems (Dice Rolls)**
The Enemy targets specific **Systems** determined by a **2d6 Dice Roll**. The Room ID containing each system depends on the Map Layout.

| Roll | System | Function / Action | Starting Items |
| :--- | :--- | :--- | :--- |
| **2** | **The Bow** | *Lookout.* Reveals the top card of the Enemy Deck (Forecasting). | - |
| **3** | **Dormitory** | *Respawn Point.* Fainted players revive here at the start of the next round. | - |
| **4** | **Cargo** | *Fuel.* Contains flammable gifts. Fire spreads 2x faster here (Spread Threshold 1). | **Wheelbarrow** |
| **5** | **Engine** | *Power.* Action: **"Raise Shields"** (2 AP). Blocks **all** incoming **Damage** events this round. | **Extinguisher** |
| **6** | **Kitchen** | *Ammo.* Action: **"Bake"** (1 AP). Create 3 Peppernuts (placed in room). | - |
| **7** | **Cannons** | *Attack.* Action: **"Load & Fire"** (1 AP + 1 Nut). Deals 1 Dmg to Enemy (**1d6 >= 3**). | - |
| **8** | **Bridge** | *Steering.* Action: **"Evasive Maneuvers"** (2 AP). Forces **all** enemy attacks to **MISS** this round. | - |
| **9** | **Sickbay** | *Heal.* Action: **"First Aid"** (1 AP). Restore 1 HP to self or adjacent player. | - |
| **10** | **Storage** | *Vault.* Secure storage. Items here are safe from Water damage. | **5 Peppernuts** |

*   **Hallway / Empty Rooms:** Transit areas. Cannot be directly targeted by the enemy (no system ID), but hazards can spread to them.

---

## **3. THE CREW (PLAYERS)**

### **Stats**
*   **HP:** Each player has **3 HP**.
    *   **0 HP = Fainted.** The pawn is removed from the board. Respawns in Dormitory (System 3) at the start of the next round with full HP.
*   **AP:** Each player has **2 Action Points** per round.
*   **Carry Capacity:**
    *   **Hand:** Max **1 Peppernut**.
    *   **Wheelbarrow (Item):** Max **5 Peppernuts**.

### **Player Actions (Cost: 1 AP unless specified)**
1.  **Move:** Move to an adjacent room.
2.  **Interact:** Perform the specific Room Action (Bake, Shoot, Shield, etc.).
3.  **Extinguish:** Remove 1 Fire token from current room (Removes **2** if holding **Extinguisher**).
4.  **Repair:** Remove 1 Water token from current room.
5.  **Throw:** Toss 1 Peppernut to a player in an adjacent room (100% success).
6.  **Pick Up:** Add item from room to inventory.
7.  **Drop:** Drop item from inventory to room (**Free**).
8.  **Revive:** Help a Fainted player in the same room (Revives them immediately with 1 HP).

---

## **4. GAME LOOP (PHASES)**

The game does not have fixed turns. It plays in a series of **Rounds**, each containing 4 Phases.

### **PHASE 1: MORNING REPORT (Event)**
*   The system draws a **Situation Card**.
*   **Immediate Effect:** Applied instantly (e.g., "Storm: All players blown to Hallway").
*   **Decision:** If the card is a Dilemma (e.g., "Pay 5 Nuts or Lose 2 Hull"), players must Vote.

### **PHASE 2: ENEMY TELEGRAPH**
*   The Enemy AI declares its intent.
*   **Example:** "The Pirate aims a Cannonball at the **Kitchen (System 6)**!"
*   Players now know exactly what threat to counter.

### **PHASE 3: TACTICAL PLANNING (The Core Loop)**
*   **Iterative Planning:** This is a safe planning phase. You can Queue actions, see the result, discuss, and modify.
    *   **Propose:** Use actions (Move, Bake, etc) to add them to your `Proposal Queue`.
    *   **Simulate:** The system shows "Ghost" outcomes.
    *   **Correct:** If a plan is wrong, use `Undo` to remove actions and regain AP.
    *   **Coordinate:** Talk to other players. "I'll go left, you go right."
*   **Commit:** ONLY when you are happy with the plan, vote to **"Execute Batch"** (`VoteReady`).
*   **Resolution:** Actions occur in sequence.
*   **End of Phase:** Triggered when all players have voted `Ready`.
    *   **VoteReady:** Execute queued actions. If ANY player has AP left, the game returns to `Tactical Planning` for another batch.
    *   **Pass:** Sets your AP to 0 and votes Ready. Use this only if you are completely done for the round.

### **PHASE 4: ENEMY ACTION & HAZARDS**
1.  **Enemy Attack Resolves:**
    *   If **Shields** were raised: Blocked.
    *   If **Evasion** was active: Missed.
    *   Otherwise: The target room takes the hit (Fire, Water, or Special).
2.  **Fire Damage (The Burn):**
    *   Any player standing in a room with Fire takes **1 Damage**.
    *   Any room with Fire deals **1 Hull Damage**.
3.  **Spread:** Fire may spread (Dice Roll).
4.  **Respawn:** Fainted players return to the Dormitory.

---

## **5. HAZARDS**

### **FIRE (Red Token)**
*   **Effect:** Disables Room Function.
*   **Danger:** Deals damage to Players and Hull at end of round.
*   **Spread:** If >= 2 Fire tokens in a room (>= 1 in Cargo), it has a 50% chance to spread to adjacent rooms.

### **WATER (Blue Token)**
*   **Effect:** Disables Room Function. Destroys all loose **Items** in the room (except in **Storage**).
*   **Danger:** Does not damage Hull directly, but renders systems useless and destroys ammo.
*   **Cleanup:** Requires 'Repair' (Mop).

## **6. COMMUNICATION**
*   **Standard:** Open Chat.
*   **"Static Noise" Event:** Communications systems down. Chat is restricted to **No alphabetic characters**.