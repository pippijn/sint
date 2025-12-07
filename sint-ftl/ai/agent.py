import argparse
import asyncio
import os
import sys
from game_agent import GameAgent

if __name__ == "__main__":
    if not os.environ.get("GEMINI_API_KEY"):
         print("Skipping execution: No API Key")
         sys.exit(0)

    parser = argparse.ArgumentParser()
    parser.add_argument("--player", default="AI_Bot", help="Player ID")
    parser.add_argument("--room", default="Room_A", help="Room ID")
    parser.add_argument("--url", default="ws://localhost:3000/ws", help="Server URL")
    parser.add_argument("--max-turns", type=int, default=0, help="Max turns")
    parser.add_argument("--debug", action="store_true", help="Enable debug logging")
    args = parser.parse_args()

    agent = GameAgent(args.player, args.room, args.url, max_turns=args.max_turns, debug=args.debug)
    asyncio.run(agent.run())