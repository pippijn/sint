#!/usr/bin/env python3
import argparse
import subprocess
import time
import sys
import os
import signal
import shutil

# --- Configuration ---
ROOT_DIR = os.path.dirname(os.path.abspath(__file__))
VENV_DIR = os.path.join(ROOT_DIR, ".venv")
VENV_BIN = os.path.join(VENV_DIR, "bin")
PYTHON_EXEC = os.path.join(VENV_BIN, "python3")

# --- Helpers ---
def run_cmd(cmd, cwd=ROOT_DIR, env=None, background=False):
    print(f"🚀 Running: {cmd}")
    if env is None:
        env = os.environ.copy()
    
    # Ensure venv is in path
    env["PATH"] = f"{VENV_BIN}:{env['PATH']}"
    
    if background:
        return subprocess.Popen(cmd, shell=True, cwd=cwd, env=env)
    else:
        ret = subprocess.call(cmd, shell=True, cwd=cwd, env=env)
        if ret != 0:
            print(f"❌ Command failed: {cmd}")
            sys.exit(ret)

def check_venv():
    if not os.path.exists(VENV_DIR):
        print("🔧 Creating Python virtual environment...")
        subprocess.check_call([sys.executable, "-m", "venv", ".venv"])
        # Use the python executable inside the venv
        venv_python = os.path.join(VENV_BIN, "python")
        run_cmd(f"{venv_python} -m pip install -r ai/requirements.txt")
        run_cmd(f"{venv_python} -m pip install maturin")

def build_core():
    print("🔨 Building Core & Python Bindings...")
    # Build Rust lib
    run_cmd("cargo build -p sint-core")
    # Build Python bindings using maturin in venv
    run_cmd("maturin develop --release", cwd=os.path.join(ROOT_DIR, "core"))

def run_tests():
    print("🧪 Running Tests...")
    run_cmd("cargo test -p sint-core")
    run_cmd("cargo check -p sint-client")
    run_cmd("cargo check -p sint-server")

def main():
    parser = argparse.ArgumentParser(description="Sint FTL Launcher")
    parser.add_argument("--ai", action="store_true", help="Start an AI agent")
    parser.add_argument("--build-only", action="store_true", help="Build and exit")
    parser.add_argument("--clean", action="store_true", help="Clean build artifacts first")
    args = parser.parse_args()

    # 1. Setup
    if args.clean:
        run_cmd("cargo clean")
        shutil.rmtree(VENV_DIR, ignore_errors=True)

    check_venv()

    # 2. Build & Test
    build_core()
    run_tests()
    
    if args.build_only:
        print("✅ Build Complete.")
        return

    # 3. Start Processes
    procs = []
    try:
        print("\n🌐 Starting Server...")
        procs.append(run_cmd("cargo run -p sint-server", background=True))
        time.sleep(1) # Wait for server port

        print("\n🖥️  Starting Web Client...")
        # trunk serve needs to be in client dir
        procs.append(run_cmd("trunk serve", cwd=os.path.join(ROOT_DIR, "client"), background=True))

        if args.ai:
            print("\n🤖 Starting AI Agent...")
            if not os.environ.get("GEMINI_API_KEY"):
                print("⚠️  GEMINI_API_KEY not set. AI might fail.")
            procs.append(run_cmd(f"{PYTHON_EXEC} ai/agent.py --player AI_Bot", background=True))

        print("\n✅ System Running. Press Ctrl+C to stop.\n")
        
        # Keep alive
        while True:
            time.sleep(1)
            # Check if server died
            if procs[0].poll() is not None:
                print("❌ Server process died!")
                break

    except KeyboardInterrupt:
        print("\n🛑 Stopping...")
    finally:
        for p in procs:
            if p.poll() is None:
                # Send SIGTERM to process group if possible, or just kill
                os.kill(p.pid, signal.SIGTERM)
        print("👋 Bye.")

if __name__ == "__main__":
    main()
