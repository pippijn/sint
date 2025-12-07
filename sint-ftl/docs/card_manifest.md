# **SINT FTL: CARD MANIFEST**

## **1. CARD TYPES**

*   **⚡ FLASH (Immediate):** The effect happens instantly upon drawing. The card is then discarded.
*   **📌 SITUATION (Persistent):** The card stays active until a **Solution** is performed.
    *   *Constraint:* Limits actions (e.g., "Silence", "No Baking").
*   **💣 TIMEBOMB (Countdown):** The card stays active for X rounds. If not solved by then, a bad effect triggers.

---

## **2. THE DECK (Initial Set: 15 Cards)**

### **TIER 1: ANNOYANCES (Common)**

| ID | Title | Type | Description / Effect | Solution |
| :--- | :--- | :--- | :--- | :--- |
| **C01** | **Afternoon Nap** | 📌 Sit | The Reader falls asleep. **Effect:** Current Reader cannot spend AP. | **Wake Up:** Spend 1 AP in Reader's room. |
| **C02** | **Static Noise** | 📌 Sit | Radio interference. **Effect:** Chat restricted to **Emoji Only**. | **Re-tune:** Spend 1 AP in **Bridge (9)**. |
| **C03** | **Seagull Attack** | 📌 Sit | Birds attacking ammo. **Effect:** Cannot **Move** while holding a Peppernut. | **Scare:** Spend 1 AP in **Bow (2)**. |
| **C04** | **Slippery Deck** | 📌 Sit | Soap everywhere. **Effect:** Movement costs 0 AP, but stops only at walls? (Simplification: **Move costs 0, Actions cost 2**). | **Mop:** Spend 1 AP in **Engine (5)**. |
| **C05** | **Peppernut Rain** | ⚡ Flash | **Effect:** Every player receives +2 Peppernuts (Overflow drops to floor). | N/A |

### **TIER 2: HAZARDS (Uncommon)**

| ID | Title | Type | Description / Effect | Solution |
| :--- | :--- | :--- | :--- | :--- |
| **C06** | **High Waves** | ⚡ Flash | **Effect:** All players are pushed 1 Room towards the **Engine (5)**. | N/A |
| **C07** | **Costume Party** | ⚡ Flash | **Effect:** Players swap positions (Cyclic shift: P1->P2, P2->P3...). | N/A |
| **C08** | **Mice Plague** | 📌 Sit | **Effect:** At end of round, lose **2 Peppernuts** from Storage (11). | **Catch:** Spend 1 AP in **Storage (11)**. |
| **C09** | **Short Circuit** | ⚡ Flash | **Effect:** Spawn **1 Fire** in the **Engine Room (5)**. | N/A |
| **C10** | **Leak!** | ⚡ Flash | **Effect:** Spawn **1 Water** in the **Cargo Room (4)**. | N/A |

### **TIER 3: CRISES (Rare / Boss Phase)**

| ID | Title | Type | Description / Effect | Solution |
| :--- | :--- | :--- | :--- | :--- |
| **C11** | **Mutiny?** | 💣 Bomb | **Countdown:** 3 Rounds. **Effect:** If not solved, Game Over (or -10 Hull). | **Negotiate:** 2 Players must be in **Bridge (9)** together and spend 1 AP each. |
| **C12** | **Fog Bank** | 📌 Sit | **Effect:** Cannot see Enemy Intent (Telegraph disabled). | **Spotlight:** Spend 2 AP in **Bow (2)**. |
| **C13** | **Anchor Stuck** | 📌 Sit | **Effect:** **Evasion** action (Engine) is disabled. | **Hoist:** 3 Players must be in **Bow (2)** to lift it. |
| **C14** | **Jammed Cannon** | 📌 Sit | **Effect:** **Cannons (8)** are disabled. | **Grease:** Spend 1 AP in **Cannons** + pay 1 Peppernut. |
| **C15** | **Man Overboard!** | 💣 Bomb | **Countdown:** 2 Rounds. **Effect:** Target Player (Random) is removed from play. | **Rescue:** Another player must Throw a Rope (Interact) from **Bow (2)**. |

---

## **3. DILEMMA LOGIC**
For cards that offer a choice (e.g., "Pay or Pain"), the card data will include:
```rust
options: vec![
  CardOption { text: "Pay 5 Nuts", cost: Cost::Resource(5), effect: Effect::None },
  CardOption { text: "Take Damage", cost: Cost::None, effect: Effect::DamageHull(2) }
]
```
The Game State enters `Phase::Decision` until a Vote confirms an option.
