# **SINT FTL: OPERATION PEPPERNUT (v2.0)**

## **1. OVERVIEW**
**Operation Peppernut** is a cooperative, tactical survival game where players crew "The Steamboat" against waves of enemies. Unlike the previous party-game iteration, this version emphasizes resource scarcity, strategic positioning, and crisis management (inspired by *FTL: Faster Than Light*).

*   **Goal:** Defeat 4 Bosses in sequence (The Thief, The Monster, The Armada, The Kraken).
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

| Room | Name | Function / Action |
| :--- | :--- | :--- |
| **2** | **The Bow** | *Lookout.* Reveals the top card of the Enemy Deck (Forecasting). |
| **3** | **Dormitory** | *Respawn Point.* Fainted players revive here at the start of the next round. |
| **4** | **Cargo** | *Fuel.* Contains flammable gifts. Fire spreads 2x faster here. |
| **5** | **Engine** | *Evasion.* Action: **"Evasive Maneuvers"** (2 AP). Forces the next enemy attack to **MISS**. |
| **6** | **Kitchen** | *Ammo.* Action: **"Bake"** (1 AP). Create 3 Peppernuts (placed in room). |
| **7** | **Hallway** | *Transit.* Connects all rooms. Central hub. |
| **8** | **Cannons** | *Attack.* Action: **"Load & Fire"** (1 AP + 1 Nut). Deals 1 Dmg to Enemy (Chance to Hit). |
| **9** | **Bridge** | *Defense.* Action: **"Raise Shields"** (2 AP). Blocks the next incoming **Damage** event. |
| **10** | **Sickbay** | *Heal.* Action: **"First Aid"** (1 AP). Restore 1 HP to self or adjacent player. |
| **11** | **Storage** | *Vault.* Secure storage. Items here are safe from Water damage. |

---

## **3. THE CREW (PLAYERS)**

### **Stats**
*   **HP:** Each player has **3 HP**.
    *   **0 HP = Fainted.** The pawn is removed from the board. Respawns in Dormitory (Room 3) at the start of the next round with full HP.
*   **AP:** Each player has **2 Action Points** per round.
*   **Carry Capacity:**
    *   **Hand:** Max **1 Peppernut**.
    *   **Wheelbarrow (Item):** Max **5 Peppernuts**.

### **Player Actions (Cost: 1 AP unless specified)**
1.  **Move:** Move to an adjacent room.
2.  **Interact:** Perform the specific Room Action (Bake, Shoot, Shield, etc.).
3.  **Extinguish:** Remove 1 Fire token from current room.
4.  **Repair:** Remove 1 Water token from current room.
5.  **Throw:** Toss 1 Peppernut to a player in an adjacent room (100% success).
6.  **Pick Up / Drop:** Manage inventory.
7.  **Revive:** Help a Fainted player in the same room (Revives them immediately with 1 HP).

---

## **4. GAME LOOP (PHASES)**

The game does not have fixed turns. It plays in a series of **Rounds**, each containing 4 Phases.

### **PHASE 1: MORNING REPORT (Event)**
*   The system draws a **Situation Card**.
*   **Immediate Effect:** Applied instantly (e.g., "Storm: All players blown to Hallway").
*   **Decision:** If the card is a Dilemma (e.g., "Pay 5 Nuts or Lose 2 Hull"), players must Vote.

### **PHASE 2: ENEMY TELEGRAPH**
*   The Enemy AI declares its intent.
*   **Example:** "The Pirate aims a Cannonball at the **Kitchen (6)**!"
*   Players now know exactly what threat to counter.

### **PHASE 3: TACTICAL PLANNING (The Core Loop)**
*   **Free-Form Discussion:** Players discuss via Chat.
*   **Proposal:** Players queue up actions ("I will move to Bridge," "I will Bake").
    *   *Ghost Mode:* Players see the projected outcome of proposed actions.
*   **Commit:** Players vote to **"Execute Batch"**.
*   **Resolution:** Actions occur in sequence.
    *   *Note:* Players do not need to spend all AP at once. They can execute a few moves, see the result, and plan again.
*   **End of Phase:** Triggered when all players have **0 AP** or have voted to **Pass**.

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
*   **Spread:** If >2 Fire tokens in a room, it automatically spills to adjacent rooms.

### **WATER (Blue Token)**
*   **Effect:** Disables Room Function.
*   **Danger:** Does not damage Hull directly, but renders systems useless.
*   **Cleanup:** Requires 'Repair' (Mop).

---

## **6. AI & SIMULATION**

*   **The "Oracle" Limit:** The AI (and players) can simulate actions to see validity (pathfinding, AP cost).
*   **Fog of War:** Simulations stop at **Random Events**.
    *   *Example:* Simulating "Shoot Cannon" will confirm you can load it, but will **not** reveal if the shot hits or misses.

## **7. COMMUNICATION**
*   **Standard:** Open Chat.
*   **"Static Noise" Event:** Communications systems down. Chat is restricted to **Emoji Only**.
