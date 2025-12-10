import sint_core
import sint_solver
import sys

# Mock Player class that accumulates actions instead of printing them
class Player:
    def __init__(self, name, id):
        self.name = name
        self.id = id
        self.max_ap = 2
        self.ap = 2
        self.actions = []

    def reset_ap(self, ap=2):
        self.ap = ap
        self.max_ap = ap

    def action(self, cmd, cost=1):
        if self.ap < cost:
            # We allow it for recording, but verify might fail later
            pass
        self.ap -= cost
        
        # Parse command to Action dict
        act = self._parse_command(cmd)
        self.actions.append((self.id, act))

    def pass_turn(self):
        if self.ap > 0:
            self.actions.append((self.id, {"type": "Pass"}))
        self.ap = 0

    def _parse_command(self, cmd):
        parts = cmd.split()
        c = parts[0]
        
        if c == "Move":
            return {"type": "Move", "payload": {"to_room": int(parts[1])}}
        elif c == "Bake":
            return {"type": "Bake"}
        elif c == "Shoot":
            return {"type": "Shoot"}
        elif c == "Extinguish":
            return {"type": "Extinguish"}
        elif c == "Repair":
            return {"type": "Repair"}
        elif c == "PickUp":
            return {"type": "PickUp", "payload": {"item_type": "Peppernut"}}
        elif c == "Drop":
            return {"type": "Drop", "payload": {"item_index": int(parts[1])}}
        elif c == "Pass":
            return {"type": "Pass"}
        elif c == "VoteReady":
            return {"type": "VoteReady", "payload": {"ready": True}}
        elif c == "RaiseShields":
            return {"type": "RaiseShields"}
        elif c == "EvasiveManeuvers":
            return {"type": "EvasiveManeuvers"}
        elif c == "Lookout":
            return {"type": "Lookout"}
        elif c == "Interact":
            return {"type": "Interact"}
        elif c == "Throw":
            target = parts[1]
            idx = int(parts[2])
            return {"type": "Throw", "payload": {"target_player": target, "item_index": idx}}
        elif c == "Revive":
            target = parts[1]
            return {"type": "Revive", "payload": {"target_player": target}}
        elif c == "FirstAid":
            target = parts[1]
            return {"type": "FirstAid", "payload": {"target_player": target}}
        else:
            raise ValueError(f"Unknown command: {cmd}")

# Initialize Players
p1 = Player("P1", "P1")
p2 = Player("P2", "P2")
p3 = Player("P3", "P3")
p4 = Player("P4", "P4")
p5 = Player("P5", "P5")
p6 = Player("P6", "P6")
players = [p1, p2, p3, p4, p5, p6]

all_actions = []

def collect_actions():
    round_actions = []
    # Collect from all players, respecting some order?
    # The original script interleaved calls like:
    # p1.action(...)
    # p5.action(...)
    # This implies we should capture them in the order they are called.
    # But here players are objects.
    # The original script CALLED methods on p1, p2 etc in order.
    # We need to capture that order.
    pass

# We need to restructure slightly. 
# Instead of players storing actions, we can have a global list and players append to it.

actions_log = []

class LoggingPlayer(Player):
    def action(self, cmd, cost=1):
        super().action(cmd, cost)
        # The last action added to self.actions is the one we just did
        actions_log.append(self.actions[-1])

    def pass_turn(self):
        super().pass_turn()
        if self.actions and self.actions[-1][1]["type"] == "Pass":
             actions_log.append(self.actions[-1])

# Re-init with logging
p1 = LoggingPlayer("P1", "P1")
p2 = LoggingPlayer("P2", "P2")
p3 = LoggingPlayer("P3", "P3")
p4 = LoggingPlayer("P4", "P4")
p5 = LoggingPlayer("P5", "P5")
p6 = LoggingPlayer("P6", "P6")
players = [p1, p2, p3, p4, p5, p6]


def start_round(ap_override=None):
    for p in players:
        p.reset_ap(2 if ap_override is None else ap_override)

# --- Original Logic copied from generate_solution.py ---

def r1():
    # print("# Round 1: TurboMode, Enemy->5")
    start_round(3)
    p1.action("Move 7"); p1.action("Move 6")
    p5.action("Move 7"); p5.action("Move 8")
    p6.action("Move 7"); p6.action("Move 8")
    p2.action("Move 7"); p2.action("Move 4")
    p3.action("Move 7"); p3.action("Move 9")
    p4.action("Move 7")
    for p in players: p.pass_turn()

def r2():
    # print("# Round 2: SugarRush. Enemy->7. Fire in 5.")
    start_round(3)
    p4.action("Move 5", 0)
    p4.action("Extinguish", 1)
    p4.action("Interact", 1)
    p4.action("Move 7", 0)
    p3.action("EvasiveManeuvers", 2)
    p1.action("Bake", 1)
    p1.action("PickUp", 1)
    p2.action("Move 7", 0)
    p2.action("Move 8", 0)
    for p in players: p.pass_turn()

def r3():
    # print("# Round 3: Overheating. Enemy->5.")
    start_round()
    p4.ap -= 1
    p4.action("Move 7", 0)
    p3.action("Move 5", 0)
    p3.action("Move 9", 0)
    p2.action("Move 7", 0)
    p1.action("Interact", 1)
    p1.action("Throw P2 0", 1)
    p2.action("Throw P5 0", 1)
    p5.action("Shoot", 1)
    p3.action("EvasiveManeuvers", 2)
    for p in players: p.pass_turn()

def r4():
    # print("# Round 4: Rudderless. Enemy->7.")
    start_round()
    p4.action("Move 9", 1)
    p4.action("Interact", 1)
    p3.action("EvasiveManeuvers", 2)
    p2.action("Move 4", 1)
    p1.action("Bake", 1)
    for p in players: p.pass_turn()

def r5():
    # print("# Round 5: NoLight. Enemy->6.")
    start_round()
    p2.action("Interact", 1)
    p3.action("EvasiveManeuvers", 2)
    p1.action("PickUp", 1)
    p1.action("Move 7", 1)
    p1.action("Drop 0", 0)
    for p in players: p.pass_turn()

def r6():
    # print("# Round 6: FogBank. Enemy->9.")
    start_round()
    p3.action("EvasiveManeuvers", 2)
    p1.action("PickUp", 1)
    p1.action("Throw P5 0", 1)
    p5.action("Shoot", 1)
    p2.action("Move 8", 2)
    for p in players: p.pass_turn()

def r7():
    # print("# Round 7: Panic! Enemy->8.")
    start_round()
    p1.action("Move 7", 1); p1.action("Move 6", 1)
    p5.action("Move 7", 1); p5.action("Move 8", 1)
    p6.action("Move 7", 1); p6.action("Move 8", 1)
    p3.action("Move 7", 1); p3.action("Move 9", 1)
    p2.action("Move 7", 1)
    p4.action("Move 7", 1); p4.action("Move 5", 1)
    for p in players: p.pass_turn()

def r8():
    # print("# Round 8: ListingShip. Enemy->8. Fire in 9.")
    start_round()
    p4.action("Interact", 2) 
    p2.action("Move 9", 1)
    p2.action("Extinguish", 1)
    p3.action("EvasiveManeuvers", 2)
    p5.action("PickUp", 1)
    p6.action("PickUp", 1)
    p5.action("Shoot", 1)
    p6.action("Shoot", 1)
    p1.action("Move 6", 0)
    p1.action("PickUp", 1)
    for p in players: p.pass_turn()

def r9():
    # print("# Round 9: WeirdGifts. Enemy->6.")
    start_round()
    p5.action("Shoot", 1)
    p1.action("Move 7", 1)
    p1.action("Throw P6 0", 1)
    p6.action("Shoot", 1)
    p3.action("EvasiveManeuvers", 2)
    p2.action("Move 7", 1)
    p2.action("Move 4", 1)
    for p in players: p.pass_turn()

def r10():
    # print("# Round 10: Kill Boss / Defense.")
    start_round()
    p3.action("EvasiveManeuvers", 2)
    p4.action("Extinguish", 1)
    p4.action("Move 7", 1)
    p2.action("Interact", 1)
    p1.action("Move 6", 1)
    p1.action("Bake", 1)
    for p in players: p.pass_turn()

def r11():
    # print("# Round 11: Overheating. Enemy->9.")
    start_round()
    p4.action("Move 5", 1)
    p4.action("Interact", 1)
    p3.action("EvasiveManeuvers", 2)
    p1.action("Bake", 1)
    p1.action("PickUp", 1)
    p2.action("Move 7", 1)
    p2.action("Move 6", 1)
    for p in players: p.pass_turn()

def r12():
    # print("# Round 12: Monster Dough. Enemy->2.")
    start_round()
    p1.action("Interact", 1)
    p1.action("Move 7", 1)
    p2.action("PickUp", 1)
    p2.action("Move 7", 1)
    p3.action("EvasiveManeuvers", 2)
    p4.action("Move 7", 1)
    p4.action("Move 2", 1)
    for p in players: p.pass_turn()

def r13():
    # print("# Round 13: The Staff. Enemy->2.")
    start_round()
    p4.action("Interact", 1)
    p1.action("Move 8", 1)
    p1.action("Throw P5 0", 1)
    p2.action("Move 8", 1)
    p2.action("Throw P6 0", 1)
    p5.action("Shoot", 1)
    p6.action("Shoot", 1)
    p3.action("EvasiveManeuvers", 2)
    for p in players: p.pass_turn()

def r14():
    # print("# Round 14: Blockade. Enemy->8.")
    start_round()
    p4.action("Move 7", 1)
    p3.action("Move 7", 1)
    p4.action("Interact", 1)
    p1.action("Move 7", 1)
    p1.action("Move 6", 1)
    p2.action("Move 7", 1)
    p2.action("Move 6", 1)
    for p in players: p.pass_turn()

def r15():
    # print("# Round 15: Refill Bake.")
    start_round()
    p1.action("Bake", 1)
    p1.action("PickUp", 1)
    p2.action("Move 6", 1)
    p2.action("PickUp", 1)
    p4.action("Move 7", 1)
    p4.action("Move 6", 1)
    p3.action("Move 9", 1)
    for p in players: p.pass_turn()

def r16():
    # print("# Round 16: Return to Stations.")
    start_round()
    p3.action("EvasiveManeuvers", 2)
    p1.action("Move 7", 1)
    p1.action("Throw P5 0", 1)
    p2.action("Move 7", 1)
    p2.action("Throw P6 0", 1)
    p4.action("PickUp", 1)
    p4.action("Move 7", 1)
    
    # Correction from previous debug: P6 IS in 6, needs to move to 8
    p6.action("Move 7", 1)
    p6.action("Move 8", 1)
    
    p5.action("Extinguish", 1)
    p5.action("Shoot", 1)
    for p in players: p.pass_turn()

def r17():
    # print("# Round 17: Finish Monster.")
    start_round()
    p3.action("EvasiveManeuvers", 2)
    p4.action("Move 8", 1)
    p4.action("Throw P5 0", 1)
    p5.action("Shoot", 1)
    p6.action("Shoot", 1)
    p1.action("Move 6", 1)
    p1.action("Extinguish", 1)
    p2.action("Move 6", 1)
    p2.action("Bake", 1) 
    for p in players: p.pass_turn()

def r18():
    # print("# Round 18: Pickup & Reload.")
    start_round()
    p1.action("PickUp", 1)
    p1.action("Move 7", 1)
    p2.action("PickUp", 1)
    p2.action("Move 7", 1)
    p4.action("Move 7", 1)
    p4.action("Move 6", 1)
    p6.action("Move 7", 1)
    p6.action("Move 6", 1)
    p3.action("EvasiveManeuvers", 2)
    for p in players: p.pass_turn()

def r19():
    # print("# Round 19: Finish Monster (Again).")
    start_round()
    p3.action("EvasiveManeuvers", 2)
    p1.action("Move 8", 1)
    p1.action("Throw P5 0", 1)
    p6.action("Move 7", 1)
    p6.action("Move 8", 1)
    p2.action("Move 8", 1)
    p2.action("Throw P6 0", 1)
    p5.action("Shoot", 1)
    p4.action("Move 7", 1)
    p4.action("Move 8", 1)
    for p in players: p.pass_turn()

def r20():
    # print("# Round 20: Lucky Dip. Monster Low.")
    start_round()
    p3.action("EvasiveManeuvers", 2)
    p5.action("Shoot", 1)
    p6.action("Shoot", 1)
    p1.action("Move 7", 1)
    p1.action("Move 6", 1)
    p2.action("Move 7", 1)
    p2.action("Move 6", 1)
    p4.action("Move 7", 1)
    p4.action("Move 6", 1)
    for p in players: p.pass_turn()

def r21():
    # print("# Round 21: Reload. P1 Asleep.")
    start_round()
    p2.action("Bake", 1)
    p2.action("PickUp", 1)
    p4.action("PickUp", 1)
    p4.action("Move 7", 1)
    p3.action("EvasiveManeuvers", 2)
    for p in players: p.pass_turn()

def r22():
    # print("# Round 22: Deliver Ammo.")
    start_round()
    p3.action("EvasiveManeuvers", 2)
    p2.action("Move 7", 1)
    p2.action("Throw P5 0", 1)
    p4.action("Move 8", 1)
    p4.action("Throw P6 0", 1)
    p5.action("Shoot", 1)
    p6.action("Shoot", 1)
    for p in players: p.pass_turn()

def r23():
    # print("# Round 23: Solve Nap/Headwind/Clamp/Seasick.")
    start_round()
    p3.action("Interact", 1)
    p6.action("Interact", 1)
    p4.action("Interact", 1)
    p4.action("Interact", 1)
    p2.action("Move 6", 1)
    p2.action("Interact", 1)
    p5.action("Move 7", 2)
    for p in players: p.pass_turn()

def r24():
    # print("# Round 24: Heal. Solve Sticky. Move.")
    start_round()
    p3.action("FirstAid P5", 1)
    p5.action("Move 6", 1)
    p5.action("Interact", 1)
    p3.action("Move 7", 1)
    p1.action("Move 7", 1)
    p1.action("Move 5", 1)
    p2.action("Bake", 1)
    p2.action("PickUp", 1)
    p4.action("Move 7", 1)
    p4.action("Move 8", 1)
    p6.action("Move 7", 1)
    p6.action("Move 8", 1)
    for p in players: p.pass_turn()

def r25():
    # print("# Round 25: Extinguish. Bake. Shoot.")
    start_round()
    p4.action("Move 9", 1)
    p4.action("Extinguish", 1)
    p3.action("Move 7", 1)
    p3.action("Move 6", 1)
    p1.action("Extinguish", 1)
    p2.action("Move 8", 1)
    p2.action("Shoot", 1)
    p5.action("Move 6", 1)
    p5.action("Bake", 1)
    p6.action("Move 8", 1)
    for p in players: p.pass_turn()

def r26():
    # print("# Round 26: Seagull Attack. Extinguish & Relay.")
    start_round()
    p1.action("RaiseShields", 2)
    p3.action("Extinguish", 1)
    p3.action("PickUp", 1)
    p4.action("Move 7", 1)
    p5.action("PickUp", 1)
    p5.action("Throw P4 0", 1)
    p4.action("Throw P6 0", 1)
    p6.action("Shoot", 1)
    for p in players: p.pass_turn()

def r27():
    # print("# Round 27: Big Leak. Victory.")
    start_round()
    p1.action("RaiseShields", 2)
    p3.action("Throw P4 0", 1)
    p5.action("PickUp", 1)
    p5.action("Throw P4 0", 1)
    p4.action("Throw P6 0", 1)
    p4.action("Throw P2 0", 1)
    p6.action("Shoot", 1)
    p2.action("Shoot", 1)
    for p in players: p.pass_turn()

def r28():
    # print("# Round 28: Final Shot. Victory.")
    start_round()
    p5.action("Bake", 1)
    p5.action("PickUp", 1)
    p3.action("PickUp", 1)
    p3.action("Throw P4 0", 1)
    p4.action("Throw P6 0", 1)
    p6.action("Shoot", 1)
    p1.action("RaiseShields", 2)
    for p in players: p.pass_turn()

def r29():
    # print("# Round 29: Attack Wave. Shield & Kill.")
    start_round()
    p1.action("RaiseShields", 2)
    p5.action("Throw P4 0", 1)
    p3.action("PickUp", 1)
    p3.action("Throw P4 0", 1)
    p4.action("Throw P6 0", 1)
    p4.action("Throw P2 0", 1)
    p6.action("Shoot", 1)
    p2.action("Shoot", 1)
    for p in players: p.pass_turn()

def r30():
    # print("# Round 30: High Pressure. Solve Attack Wave. Reload.")
    start_round()
    p5.action("Move 6", 1)
    p5.action("Bake", 1)
    p6.action("Move 8", 1)
    p6.action("Interact", 1)
    p1.action("Move 6", 1)
    p1.action("PickUp", 1)
    p2.action("Move 6", 1)
    p2.action("PickUp", 1)
    p3.action("Move 6", 1)
    p3.action("PickUp", 1)
    p4.action("Move 7", 1)
    for p in players: p.pass_turn()

def r31():
    # print("# Round 31: Victory.")
    start_round()
    p5.action("Move 7", 1)
    p1.action("Throw P5 0", 1)
    p5.action("Throw P6 0", 1)
    p6.action("Shoot", 1)
    for p in players: p.pass_turn()

def main():
    seed = 12345
    
    # Run Rounds to record actions
    r1(); r2(); r3(); r4(); r5(); r6(); r7(); r8(); r9(); r10()
    r11(); r12(); r13(); r14(); r15(); r16(); r17(); r18(); r19(); r20()
    r21(); r22(); r23(); r24(); r25(); r26(); r27(); r28(); r29(); r30()
    r31()
    
    # Execute
    print(f"Executing {len(actions_log)} actions...")
    
    player_ids = ["P1", "P2", "P3", "P4", "P5", "P6"]
    initial_state = sint_core.new_game(player_ids, seed)
    
    result = sint_solver.verify_solution(initial_state, actions_log)
    
    if result["success"]:
        print("✅ SUCCESS!")
        print(f"Final Hull: {result['final_state']['hull_integrity']}")
        print(f"Final Boss HP: {result['final_state']['enemy']['hp']}")
        print(f"Score: {result['score']}")
        
        print("\n--- TRAJECTORY LOG ---")
        logs = sint_solver.get_trajectory_log(initial_state, result['history'])
        for l in logs:
            print(l, end='')

    else:
        print("❌ FAILURE!")
        print(result["failure_summary"])
        sys.exit(1)

if __name__ == "__main__":
    main()
