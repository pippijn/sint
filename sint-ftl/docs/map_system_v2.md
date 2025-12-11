# SINT FTL: Map System v2 Design

## 1. Core Concept: Separation of Concerns

In v1, **Room ID** and **System ID** were synonymous. Room 7 was "The Hallway" and "System 7" was "The Hallway System".
In v2, we decouple these concepts to allow for complex topologies (like the Torus layout) and varied game setups.

### Definitions
*   **System ID (u32):** A logical identifier for a ship component (e.g., Engine, Cannons). These correspond to the dice faces of the Enemy Attack (2d6).
*   **Room ID (u32):** A unique identifier for a physical node on the map graph (0-based index). Used for movement and adjacency.
*   **Empty Room:** A Room that contains `None` for its `system` field.
    *   *Design Note:* We use `Option<SystemType>::None` rather than a `SystemType::Empty` variant. This strictly enforces that Empty Rooms cannot be targeted by the Enemy (which rolls for a `SystemType`). It physically separates "Transit Areas" from "Critical Systems".
    *   **Action:** `SystemType::Hallway` will be removed from the enum.

---

## 2. System Identifiers (The Dice Roll)

The Enemy AI rolls **2d6** to determine which **System** to attack. The game then resolves this by finding the **Room** that currently houses that System.

| ID | System Name | Function | Note |
|:---|:---|:---|:---|
| **2** | **Bow** | Lookout | Unchanged |
| **3** | **Dormitory** | Respawn | Unchanged |
| **4** | **Cargo** | Fuel / Storage | Unchanged |
| **5** | **Engine** | Shields | Unchanged |
| **6** | **Kitchen** | Baking | Unchanged |
| **7** | **Cannons** | Shooting | **Changed** (Was Hallway) |
| **8** | **Bridge** | Evasion | **Changed** (Was Cannons) |
| **9** | **Sickbay** | Healing | **Changed** (Was Bridge) |
| **10** | **Storage** | Safe Item Storage | **Changed** (Was Sickbay) |
| **11** | **(None)** | **Lucky Miss** | Old Storage removed |
| **12** | **(None)** | **Lucky Miss** | - |

> **Update:** The `EnemyAttack` struct will store **both**:
> *   `target_system: Option<SystemType>` (New: The logical intent. None = Miss).
> *   `target_room: u32` (Legacy/Display: The room ID resolved at telegraph time. 0 if Miss).
>
> We will also add `AttackEffect::Miss` to explicitly handle Lucky Misses (Rolls 11/12) without damaging the ship.

---

## 3. Map Layouts

The game supports multiple topologies. **Room IDs** are 0-indexed integers. The game will default to the **Star** layout.

### A. The "Star" Layout (Default)
Mimics the classic setup with a central hub.

*   **Structure:** 1 Central Empty Room (Hub) connected to all System Rooms.
*   **Total Rooms:** 10 (IDs 0-9).
*   **Room 0:** Empty (The Hub/Hallway).
*   **Rooms 1-9:** Contain Systems 2-10 distributed arbitrarily.
*   **Connectivity:** Room 0 is adjacent to Rooms 1-9. Rooms 1-9 are ONLY adjacent to Room 0.

### B. The "Torus" Layout
A circular corridor layout with two "Empty" choke points.

*   **Structure:** A ring of Rooms.
*   **Total Rooms:** 11 (IDs 0-10).
*   **Sequence:**
    *   Room 0 (Bow/2) <-> Room 1 (Dormitory/3) <-> **Room 2 (Empty A)**
    *   <-> Room 3 (Cargo/4) <-> Room 4 (Engine/5) <-> Room 5 (Kitchen/6)
    *   <-> **Room 6 (Empty B)** <-> Room 7 (Cannons/7) <-> Room 8 (Bridge/8)
    *   <-> Room 9 (Sickbay/9) <-> Room 10 (Storage/10) <-> Room 0 (Loop).
*   **Connectivity:** `i` connects to `(i+1)%N` and `(i-1)%N`.

---

## 4. Card Logic Updates (Dynamic Lookups)

Cards must no longer hardcode Room IDs. We will implement helper functions in `GameLogic` or `GameState`:
*   `find_room_with_system(SystemType) -> Option<RoomId>`
*   `find_empty_rooms() -> Vec<RoomId>`

| Card | New Logic |
|:---|:---|
| **Amerigo** (The Horse) | **Hungry Horse in Storage:** Amerigo appears in the Room containing **Storage (10)**. At the end of every round, if there are **Peppernuts** in the room, he eats 1 (he ignores other items). **Solution:** Interact in Storage to shoo him away. |
| **Big Leak** | **Leak in Cargo:** Spawns Water in the Room containing **Cargo (4)**. |
| **False Note** | **Flee to Nearest Hallway:** Players in the Room containing **Cannons (7)** move to the nearest Empty Room (Calculated via **BFS** using `find_empty_rooms`). If multiple are equidistant, pick the one with the lowest Room ID. **Fallback:** If no Empty Room is reachable, players stay put. |
| **Sticky Floor** | **Syrup Spill:** Moving into the Room containing **Kitchen (6)** costs +1 AP. |
| **Turbo Mode** | **Engine Explosion:** 2 Fire in **Engine (5)**. 1 Fire in a deterministic neighbor: Prefer an Empty Room; otherwise pick neighbor with lowest Room ID. |
| **Wheel Clamp** | **Spatial Rotation:** All players move to `(Current_Room_ID + 1) % Total_Rooms`. This is an intended "Glitch" effect that ignores walls/topology. |

---

## 5. Enemy Targeting Logic

1.  **Roll 2d6** (Result `R`).
2.  **Intent:** Create `EnemyAttack`.
    *   If R is 2-10: `target_system = Some(SystemType::from_u32(R))`, `effect = Fireball/Leak`.
    *   If R is 11-12: `target_system = None`, `effect = AttackEffect::Miss`.
3.  **Resolution (Telegraph Time):**
    *   Lookup `Room` where `Room.system == target_system`.
    *   Set `target_room` to that ID (or 0 if Miss).
4.  **Resolution (Damage Time):**
    *   Apply Hazard to `target_room` (if not Miss).
    *   If Miss, log "Lucky Miss!".
5.  **Note:** Empty Rooms (Hallways) effectively have "System ID: None" and cannot be directly targeted by the dice. They only take damage via Fire Spread.

---

## 6. Client / UI

*   **Status:** **Implemented.**
*   The **Map Layout Selector** is available in the **Game View** (under "Status Report") while in the **Lobby Phase**.
*   **Star Layout:** Uses the classic visualizer (Hub + Rows).
*   **Torus Layout:** Uses a **4x4 Grid** layout to visualize the ring topology (Room 0 at Top Left, winding clockwise).
*   **Compatibility:** We preserve `target_room` in `EnemyAttack` so the Client (and AI Agent) can still visualize targets without schema breaking changes.