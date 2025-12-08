# **SINT FTL: SYSTEM ARCHITECTURE**

## **1. OVERVIEW**
This project implements a **Deterministic Peer-to-Peer (P2P)** architecture. 
There is **NO AUTHORITATIVE HOST**. Every client (Human or AI) runs the full game simulation locally (`sint-core`). 

To ensure all clients stay in sync (Deterministic Lockstep/Rollback), we need a **Total Ordering** of events. Currently, a lightweight WebSocket Server acts as this **Sequencer**. In the future, this can be replaced by a true P2P protocol (e.g., Tox, Libp2p) using consensus algorithms or vector clocks.

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
    *   `apply_action(State, Action) -> Result<State, Error>`: Deterministic reducer. Given State S and Action A, ALWAYS produces S'.
    *   `validate_action(State, Action) -> bool`
    *   `json_schema()`: Exporting tool definitions for AI.

### **B. `sint-server` (The Sequencer)**
*   **Language:** Rust (Axum/Tokio).
*   **Role:** A "dumb" Sequenced Relay.
*   **Persistence:** None (Ephemeral RAM). Recovery relies on Clients.
*   **Responsibility:**
    *   Accepts `Message` from Client A.
    *   Broadcasts `Message` to All Clients.
    *   **Ordering:** Ensures strict causal ordering of messages so all clients apply them in the same sequence. It does **NOT** validate game rules.

### **C. `sint-client` (The Interface)**
*   **Language:** Rust (Leptos framework).
*   **Role:** The human user interface.
*   **Logic:** Runs a local copy of `sint-core`.
*   **Mode:** **Optimistic Prediction with Rollback.**
    1.  **User Input:** User clicks "Move".
    2.  **Prediction:** Client applies "Move" to `LocalState` immediately (Zero Latency).
    3.  **Queue:** Client adds "Move" to `PendingOutbox`.
    4.  **Network:** Client sends "Move" to Server/Swarm.
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
The "Truth" is a linear list of `ConfirmedEvents`.

### **The Sequenced Handshake**
1.  **Proposal:** Player A clicks "Move".
    *   Msg: `Type: Request, From: A, Action: Move` -> Sent to Relay.
2.  **Relay:** Broadcasts `Event(A, Move)`.
3.  **Client:**
    *   Receives `Event`.
    *   Applies `sint-core.apply_action()`.
    *   State updates officially.

---

## **4. RECOVERY & RESILIENCE**

### **Scenario: New Peer / Reconnect**
1.  **Join:** Client C connects.
2.  **Request:** Client C sends `SyncRequest`.
3.  **Response:** Client A (or B) serializes their current `VerifiedState` and sends `FullSync` action.
4.  **Catchup:** Client C loads the state.

---

## **5. SIMULATION ARCHITECTURE**

To prevent AI hallucination and RNG cheating:

*   **Real Game:** Uses `Seed_Real` (Shared in GameState).
*   **Planning:**
    *   AI (and Human Clients) use the `Proposal Queue`.
    *   This queue is part of the shared state.
    *   Peers can see "Ghost" actions of others before they are committed (`VoteReady`).
    *   This allows for "Shared Simulation" and collaborative planning without committing to RNG outcomes yet.
