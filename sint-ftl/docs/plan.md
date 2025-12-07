# **PROJECT STATUS & ROADMAP**

## **1. CURRENT STATE (Backend & AI Complete)**

### **Architecture**
*   **Monorepo:** `/sint-ftl` (Rust Workspace).
*   **Core (`sint-core`):** Rust Library. Contains `GameState`, `Action` enum, `GameLogic` reducer. Compiles to Native (Server), Python Extension (AI), and WASM (Client).
*   **Server (`sint-server`):** Rust Binary (Axum). Sequenced WebSocket relay.
*   **AI (`sint-ai`):** Python project. `agent.py` connects to Gemini, loads Rust tools dynamically, and executes actions. Validated.

### **Documentation**
*   `rules.md`: v2.0 Gameplay Rules (FTL-style).
*   `architecture.md`: Deterministic P2P design (Sequenced Relay).
*   `api_schema.md`: JSON Data contracts.
*   `ai_guidelines.md`: Instructions for LLM Agents.

### **Verification**
*   [x] `sint-core` compiles.
*   [x] `sint-core` builds Python extension via `maturin`.
*   [x] `sint-core` compiles to WASM (`wasm32-unknown-unknown`).
*   [x] `agent.py` successfully calls Gemini and receives tool calls.

---

## **2. REMAINING WORK (Next Session)**

### **A. Web Client (`sint-client`)**
*   **Tech:** Rust + Leptos (WASM).
*   **Tasks:**
    1.  Initialize Leptos project.
    2.  Implement WebSocket client to connect to `sint-server`.
    3.  Implement **Event Loop**: Receive `Event` -> `sint_core::apply_action(Event)`.
    4.  Render the `GameState` (Room grid, Players, Fire/Water tokens).
    5.  Implement UI for "Propose Action" and "Vote Ready".

### **B. Game Logic Expansion**
*   **Tasks:**
    1.  Implement `Card` effects (currently just types).
    2.  Implement `Fire` spreading logic (in `GameLogic`).
    3.  Implement `Enemy` attack resolution.

### **C. Integration**
*   **Tasks:**
    1.  Run Server, Client, and AI Agent simultaneously.
    2.  Verify AI appears in the Web Client.
    3.  Verify AI Chat (Emoji mode).

---

## **3. HOW TO RESUME**
1.  **Read Docs:** Start with `docs/rules.md` and `docs/architecture.md`.
2.  **Frontend:** Focus on `sint-client`.
3.  **Command:** `cargo leptos watch` (once setup) to run the frontend.