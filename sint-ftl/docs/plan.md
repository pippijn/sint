# **PROJECT STATUS & ROADMAP**

## **1. CURRENT STATE (Prototype Complete)**

### **Architecture**
*   **Monorepo:** `/sint-ftl` (Rust Workspace).
*   **Core (`sint-core`):** Rust Library. Contains `GameState`, `Action` enum, `GameLogic` reducer. Compiles to Native (Server), Python Extension (AI), and WASM (Client).
*   **Server (`sint-server`):** Rust Binary (Axum). Sequenced WebSocket relay.
*   **AI (`sint-ai`):** Python project. `agent.py` connects to Gemini, loads Rust tools dynamically, and executes actions. Validated.
*   **Client (`sint-client`):** Rust/Leptos Web App. Implements Optimistic Prediction, Map Visualization, and Contextual Actions.

### **Documentation**
*   `README.md`: Setup and Run instructions.
*   `rules.md`: v2.0 Gameplay Rules (FTL-style).
*   `architecture.md`: Deterministic P2P design (Sequenced Relay).

### **Verification**
*   [x] `sint-core` compiles (WASM & Python).
*   [x] `sint-server` relays messages correctly.
*   [x] `sint-client` connects, renders map, handles actions optimistically, and shows connection status.
*   [x] `sint-ai` connects, reads state via Rust bindings, executes moves, and supports single-shot mode (`--max-turns`).
*   [x] **Full Loop:** Client <-> Server <-> AI synchronization verified.

---

## **2. REMAINING WORK (Refinement & Content)**

### **A. Gameplay Depth**
*   **Tasks:**
    1.  Implement `Card` effects and resolution loop.
    2.  Implement `Fire` spreading logic (Simulation).
    3.  Implement `Enemy` attack resolution (Telegraphing).

### **B. UI Polish**
*   **Tasks:**
    1.  Better visual representation of Hazards (Fire/Water icons).
    2.  Chat UI (Emoji/Text communication between Human and AI).
    3.  "Ready" voting system visibility.
    *   (Completed: Connection Status Indicator, Join Game Button)

### **C. AI Improvement**
*   **Tasks:**
    1.  Give AI memory of past turns (Conversation History).
    2.  Implement `simulate_plan` tool for AI to self-validate complex moves.
    *   (Completed: Single-shot execution, Context-rich prompts, Robust Schema handling)

---

## **3. HOW TO RESUME**
1.  **Read Docs:** Start with `docs/rules.md` and `docs/architecture.md`.
2.  **Frontend:** Focus on `sint-client`.
3.  **Command:** `cargo leptos watch` (once setup) to run the frontend.