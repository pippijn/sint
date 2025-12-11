TASK:
Generate a valid sequence of actions for the players to execute this strategy.
Check CURRENT STATE for available AP (usually 2, but can be more/less).
**IMPORTANT:** Moving to a room costs 1 AP *per step*. Check "Path Costs" in Player status. Moving 2 steps costs 2 AP.

**REALITY CHECK:**
- The **STRATEGY GUIDE** is a general plan. The **CURRENT STATE** is reality.
- **IF** Strategy says "Extinguish Fire" but **CURRENT STATE** shows NO FIRE in that room -> **DO NOT EXTINGUISH.** You will fail.
- **IF** Strategy says "Shoot" but you have NO AMMO -> **DO NOT SHOOT.**

INSTRUCTIONS:
1. **ANALYZE:** Look at "CURRENT STATE" above. Who has a "FULL" inventory? Who is where?
2. **PLAN:** Mentally simulate the sequence.
   - *Example:* "P1 is in Kitchen. Bake (Create items). PickUp (Get item). Move (Go to Hallway)." -> VALID.
   - *Bad Example:* "P1 Move to Hallway. Bake." -> INVALID (Not in Kitchen anymore).
3. **EXECUTE:** Call `submit_plan` with the valid action list.

COMMON MISTAKES TO AVOID:
- **Inventory Overflow:** If P2 has `[FULL! Cannot PickUp]`, they MUST `Throw` or `Drop` first.
- **Throw vs PickUp:** `Throw` transfers items directly (ignoring limit). `PickUp` RESPECTS limit.
  - If P2 holds 1 Peppernut, they CANNOT PickUp.
  - If P1 Throws to P2, P2 now has 2. P2 STILL cannot PickUp.
- **Ghost Items:** Do not PickUp if the room is empty.
- **Wrong Order:** Action first, THEN Move (usually). If you Move, you leave the station!
- **MAP:** Outer Rooms (1-9) are **NOT** connected to each other. You MUST go through Hallway (0).
  - *Example:* Kitchen (5) -> Cannons (6) is **IMPOSSIBLE**. Must go 5 -> 0 -> 6.

CONSTRAINTS:
- Validate dependencies (e.g. P1 must bake before P2 can pickup).
- **ACTIVE SITUATIONS:** Read them! They can FORBID actions (e.g. Sugar Rush = No Shoot) or change costs.
- **INVENTORY LIMIT:** Max **1 Peppernut** per player. (Max 5 if holding **Wheelbarrow**).
- **CHECK CURRENT INVENTORY:** If a player already has a Peppernut, they CANNOT PickUp another. They must Throw/Drop it first.
- **LOCATION & ITEMS:** Items stay in the room. They do NOT follow you. If you Move, you are in the new room. subsequent actions happen THERE.
- COMMAND SIGNATURES:
  * "Move": {{"to_room": int}} (Cost: 1 AP per hop).
  * "Bake": {{}} (Requires Kitchen Room 5. Creates 3 Peppernuts **ON THE FLOOR**. You must **PickUp** (1 AP) to hold them.)
  * "Shoot": {{}} (Requires Cannons Room 6 + 1 Peppernut in Inventory)
  * "Extinguish": {{}} (Removes Fire)
  * "Repair": {{}} (Removes Water)
  * "Throw": {{"target_player": "Px", "item_index": int}} (Requires item in INVENTORY. Target must be in/adj room. Transfers directly to Target's INVENTORY. Target does NOT need to PickUp.)
  * "PickUp": {{"item_type": "Peppernut"|"Wheelbarrow"|"Extinguisher"}} (Cost: 1 AP. Adds item from Room to Inventory.)
  * "Drop": {{"item_index": int}}
  * "Pass": {{}}
  * "Interact": {{}}
  * "EvasiveManeuvers": {{}} (Requires Bridge Room 7)
  * "RaiseShields": {{}} (Requires Engine Room 4)

TIPS:
- To move items to another room: PickUp -> Move -> Drop (or Throw).
- **BAKING SEQUENCE:** Bake (1 AP) -> PickUp (1 AP) -> Throw (1 AP) = 3 AP total.
- **STARTING POSITIONS:** Players usually start in Dormitory (Room 2). Kitchen (5) is 2 hops away (2->0->5). Cost: 2 AP.