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
    parser.add_argument("--room", default=None, help="Room ID")
    parser.add_argument("--url", default="ws://localhost:3000/ws", help="Server URL")
    parser.add_argument("--max-turns", type=int, default=0, help="Max turns")
    parser.add_argument("--debug", action="store_true", help="Enable debug logging")
    args = parser.parse_args()

    if args.room is None:
        import urllib.request
        import json
        
        # Convert WS URL to HTTP API URL
        # "ws://localhost:3000/ws" -> "http://localhost:3000/api/rooms"
        api_url = args.url.replace("ws://", "http://").replace("wss://", "https://").replace("/ws", "/api/rooms")
        
        print(f"Fetching active rooms from {api_url}...")
        try:
            with urllib.request.urlopen(api_url) as response:
                if response.status == 200:
                    data = json.loads(response.read().decode())
                    rooms = data.get("rooms", [])
                    if rooms:
                        print("Active Rooms:")
                        for r in rooms:
                            print(f" - {r}")
                        print("\nUsage: python ai/agent.py --room <ROOM_ID>")
                    else:
                        print("No active rooms found.")
                else:
                    print(f"Error fetching rooms: HTTP {response.status}")
        except Exception as e:
            print(f"Failed to connect to server: {e}")
        
        sys.exit(0)

    agent = GameAgent(args.player, args.room, args.url, max_turns=args.max_turns, debug=args.debug)
    asyncio.run(agent.run())