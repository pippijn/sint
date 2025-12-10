#!/usr/bin/env python3
import argparse
import time
import sys
import os
import signal
import shutil

# Ensure we can import common
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
import common

def run_tests():
    print("🧪 Running Tests...")
    common.run_cmd("cargo check --all")
    common.run_cmd("cargo test --all")

def main():
    parser = argparse.ArgumentParser(description="Sint FTL Launcher")
    parser.add_argument("--ai", action="store_true", help="Start an AI agent")
    parser.add_argument("--build-only", action="store_true", help="Build and exit")
    parser.add_argument("--clean", action="store_true", help="Clean build artifacts first")
    parser.add_argument("--rebuild", action="store_true", help="Force rebuild of bindings")
    args = parser.parse_args()

    # 1. Setup
    if args.clean:
        print("🧹 Cleaning...")
        common.run_cmd("cargo clean")
        shutil.rmtree(common.VENV_DIR, ignore_errors=True)

    common.check_venv()

    # 2. Build & Test
    # For start_game, we usually want to ensure everything is correct, so maybe we keep cargo build?
    # But common.build_bindings is smart now.
    # However, start_game needs 'sint-server' binary which maturin doesn't build.
    # So we MUST run cargo build/run for the server.
    
    # Let's trust cargo's incrementalism for the server binary.
    # But for python bindings, use our smart builder.
    common.build_bindings(force=args.rebuild)
    
    # Run cargo check/test if requested or implied?
    # Original ran them always.
    run_tests()
    
    if args.build_only:
        print("✅ Build Complete.")
        return

    # 3. Start Processes
    procs = []
    try:
        print("\n🌐 Starting Server...")
        # cargo run will build the server binary if needed
        procs.append(common.run_cmd("cargo run -p sint-server", background=True))
        time.sleep(1) # Wait for server port

        print("\n🖥️  Starting Web Client...")
        procs.append(common.run_cmd("trunk serve", cwd=os.path.join(common.ROOT_DIR, "client"), background=True))

        if args.ai:
            print("\n🤖 Starting AI Agent...")
            if not os.environ.get("GEMINI_API_KEY"):
                print("⚠️  GEMINI_API_KEY not set. AI might fail.")
            procs.append(common.run_cmd(f"{common.PYTHON_EXEC} ai/agent.py --player AI_Bot --room Room_A", background=True))

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