# Sint FTL - Operation Peppernut

A cooperative multiplayer game where players (Human and AI) crew a steamboat to deliver Sinterklaas safely, managing hazards like fire and water leaks while fighting off enemies.

## **Project Structure**
This is a Rust workspace monorepo containing:
*   **`core/`**: The shared game logic (Rust). Compiles to WASM for the client and a Python extension (`.so`) for the AI.
*   **`server/`**: A lightweight WebSocket relay (Rust/Axum).
*   **`client/`**: The web frontend (Rust/Leptos).
*   **`ai/`**: The LLM-powered autonomous agent (Python).
*   **`solver/`**: Verification tools and strategy solver (Rust/Python).
*   **`scripts/`**: Automation scripts for setup, building, and launching (`start_game.py`).
*   **`docs/`**: Detailed documentation (`rules.md`, `architecture.md`, `strategy.md`).

## **Prerequisites**
1.  **Rust Toolchain**: `cargo`, `rustc`.
2.  **Python 3.10+**: `python3`, `pip`, `venv`.
3.  **WASM Tools**:
    ```bash
    cargo install trunk
    rustup target add wasm32-unknown-unknown
    ```
4.  **Maturin** (for building Python bindings):
    ```bash
    pip install maturin
    ```

## **Quick Start (Recommended)**
The easiest way to run the game is using the helper script. It automatically handles **virtual environment creation**, **dependency installation**, and **building Python bindings**.

```bash
# Export API Key (if using AI)
export GEMINI_API_KEY="your_api_key_here"

# Launch Server, Client, and AI Agent in one terminal
python3 scripts/start_game.py --ai
```
*The script monitors the processes. Press `Ctrl+C` to stop everything.*

## **Manual Setup & Installation**
If you prefer to run components individually:

### **1. Python Environment**
The project uses a virtual environment at `sint-ftl/.venv`. The scripts create this automatically, but to do it manually:
```bash
# Create venv (if not exists)
python3 -m venv .venv

# Activate
source .venv/bin/activate

# Install dependencies
pip install -r ai/requirements.txt
```

### **2. Build Core Bindings**
To allow the Python AI to use the Rust game logic, you must compile `sint-core` as a Python extension.
```bash
# With venv activated:
cd core
maturin develop --release
```
*This installs the `sint_core` package directly into your virtual environment.*

## **Manual Execution**
You need to run three components simultaneously in separate terminals.

### **Terminal 1: Server**
The WebSocket relay that synchronizes all players.
```bash
cargo run -p sint-server
```
*Listens on `ws://localhost:3000/ws`.*

### **Terminal 2: Web Client**
The human player interface.
```bash
cd client
trunk serve
```
*Opens in browser at `http://localhost:8080`.*
*If you are not automatically joined, click the **"JOIN GAME"** button.*
*New in v2.1: Use the dropdown in the **Status Report** panel (Lobby Phase) to select **Star** or **Torus** map layout.*

### **Terminal 3: AI Agent**
The autonomous Gemini-powered crewmate.
```bash
# With venv activated:
export GEMINI_API_KEY="your_api_key_here"
python3 ai/agent.py
```
*   **Single-Shot Mode:** Use `--max-turns 1` to run the agent for a single decision cycle (useful for debugging/cost saving).
*   **Debug Mode:** Use `--debug` to print the exact prompt context sent to the LLM.
*   **Configuration:** Edit `ai/system_prompt.txt` to change the AI's persona and instructions.
*   *The agent will connect, join the room, and start playing.*

## **Docker**
You can also run the game server and client using Docker. **Note:** This container *only* hosts the Game Server and Web Client. You must run the AI Agent separately on your host machine.

```bash
# Build
docker build -t sint-ftl .

# Run (Exposes Web Client on 8080, Server is proxied internally)
docker run -p 8080:80 sint-ftl
```

*   **Play:** Open `http://localhost:8080`
*   **Run AI:** In a new terminal (on your host):
    ```bash
    # Note: The AI needs to connect to the exposed port (8080) which Nginx proxies to the WS server
    export GEMINI_API_KEY="..."
    # The default agent URL is ws://localhost:3000/ws. We must override it.
    python3 scripts/run_agent.py --url ws://localhost:8080/ws
    ```
## **Development Notes**
*   **Synchronization**: The game uses an "Event Sourcing" pattern. All actions are sent to the server, sequenced, and broadcast back.
*   **Optimistic UI**: The client predicts the outcome of your actions instantly but rolls back if the server contradicts them. The UI shows a "Connected" indicator.
*   **AI Context**: The AI receives a rich text summary of its surroundings (Room, Hazards, Players) to make decisions.
