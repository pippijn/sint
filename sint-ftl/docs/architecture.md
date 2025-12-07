# **SINT FTL: SYSTEM ARCHITECTURE**

## **1. OVERVIEW**
This project implements a **Deterministic Peer-to-Peer (P2P)** architecture via a **Sequenced Relay**.
There is **NO HOST**. Every client (Human or AI) is equal. The Server acts as the "Timekeeper" by ordering messages, ensuring all clients execute the same actions in the same order on their local copies of the game state.

---

## **2. COMPONENT STRUCTURE**

### **A. `sint-core` (The Brain)**
*   **Language:** Rust.
*   **Role:** The single source of truth for Game State, Rules, and Reducers.
*   **Compilation Targets:**
    1.  **WASM (`.wasm`):** For the Browser Client.
    2.  **Python Extension (`.so`/`.pyd`):** For the AI Agent (via `PyO3` / `maturin`).
    3.  **Native Binary:** For the Server (optional validation).
*   **Key Responsibilities:**
    *   `apply_action(State, Action) -> Result<State, Error>`
    *   `validate_action(State, Action) -> bool`
    *   `json_schema()`: Exporting tool definitions for AI.

### **B. `sint-server` (The Relay)**
*   **Language:** Rust (Axum/Tokio).
*   **Role:** A "dumb" WebSocket relay and sequence stamper.
*   **Persistence:** None (Ephemeral RAM). Recovery relies on Clients.
*   **Responsibility:**
    *   Accepts `Message` from Client A.
    *   Broadcasts `Message` to All Clients.
    *   Does **not** validate rules.

### **C. `sint-client` (The Interface)**
*   **Language:** Rust (Leptos framework).
*   **Role:** The human user interface.
*   **Logic:** Runs a local copy of `sint-core`.
*   **Mode:** **Optimistic Prediction with Rollback.**
    1.  **User Input:** User clicks "Move".
    2.  **Prediction:** Client applies "Move" to `LocalState` immediately (Zero Latency).
    3.  **Queue:** Client adds "Move" to `PendingOutbox`.
    4.  **Network:** Client sends "Move" to Server.
    5.  **Reconciliation:**
        *   Receive `ConfirmedEvent(Seq 100)`.
        *   **Rollback:** Reset `LocalState` to `VerifiedState`.
        *   **Apply:** Apply `ConfirmedEvent`. Update `VerifiedState`.
        *   **Replay:** Re-apply all items in `PendingOutbox`.
    6.  **Resolution:** If `PendingOutbox` items act on invalid state (due to the new Event), they fail/vanish locally.

### **D. `sint-ai` (The Agent)**
*   **Language:** Python.
*   **Role:** An autonomous player simulated by an LLM (Gemini).
*   **Logic:** Imports `sint_core` as a native Python library.
*   **Flow:**
    *   Connects to Relay as a standard client.
    *   Maintains its own local `GameState` (Optimistically or Sequentially).
    *   On `ConfirmedEvent`, updates local state.
    *   Decides move -> Sends `Request`.

---

## **3. DATA FLOW (EVENT SOURCING)**

The game state is a derivative of the **Action History**.

### **The Event Log**
The "Truth" is a linear list of `ConfirmedEvents` sequenced by the Server.

### **The Sequenced Handshake**
1.  **Proposal:** Player A clicks "Move".
    *   Msg: `Type: Request, From: A, Action: Move` -> Sent to Relay.
2.  **Relay:** Assigns `SequenceID` (e.g., 100). Broadcasts `ConfirmedEvent(100)`.
3.  **Client:**
    *   Receives `ConfirmedEvent(100)`.
    *   Applies `sint-core.apply_action()`.
    *   State updates officially.

---

## **4. RECOVERY & RESILIENCE**

### **Scenario: Server Restart**
1.  **Crash:** Relay Server dies. Connections drop.
2.  **Reconnect:** Clients auto-reconnect.
3.  **Handshake:**
    *   Client A sends: `Hello { last_seq: 105 }`
    *   Client B sends: `Hello { last_seq: 105 }`
4.  **Consensus:** Server sees clients agree on Seq 105.
5.  **Resume:** Server sets its internal counter to 106. Game continues.

### **Scenario: Packet Loss**
1.  **Gap:** Client A has Seq 100. Receives Seq 102.
2.  **Detection:** `102 > 100 + 1`.
3.  **Action:** Client A sends `RequestResend(101)`.
4.  **Recovery:** Server (or Peer) sends 101. Client A processes 101, then 102.

---

## **5. SIMULATION ARCHITECTURE**

To prevent AI hallucination and RNG cheating:

*   **Real Game:** Uses `Seed_Real` (Shared in GameState).
*   **Simulation Tool:**
    *   AI calls `sint_core.simulate(actions, seed=Empty)`.
    *   Engine executes deterministic logic (movement, AP cost).
    *   **Stop Condition:** If Engine hits a RNG check (e.g., "Roll for Fire Spread"), it halts and returns:
        *   `"State valid up to Step 3. Step 4 requires Random Roll. Stopping."`
    *   AI receives the partial state and validity confirmation.
