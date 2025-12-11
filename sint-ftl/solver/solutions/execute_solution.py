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
            item = parts[1] if len(parts) > 1 else "Peppernut"
            return {"type": "PickUp", "payload": {"item_type": item}}
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

# Global log to capture actions in round blocks
rounds_log = []

class LoggingPlayer(Player):
    def action(self, cmd, cost=1):
        super().action(cmd, cost)
        # Append to the current round's log
        if rounds_log:
            rounds_log[-1].append(self.actions[-1])

    def pass_turn(self):
        super().pass_turn()
        if self.actions and self.actions[-1][1]["type"] == "Pass":
             if rounds_log:
                 rounds_log[-1].append(self.actions[-1])

# Re-init with logging
p1 = LoggingPlayer("P1", "P1")
p2 = LoggingPlayer("P2", "P2")
p3 = LoggingPlayer("P3", "P3")
p4 = LoggingPlayer("P4", "P4")
p5 = LoggingPlayer("P5", "P5")
p6 = LoggingPlayer("P6", "P6")
players = [p1, p2, p3, p4, p5, p6]


class RoundScope:
    def __init__(self, ap_override=None):
        self.ap = 2 if ap_override is None else ap_override

    def __enter__(self):
        rounds_log.append([])
        for p in players:
            p.reset_ap(self.ap)
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        if exc_type is not None:
             return False # Propagate exceptions
        for p in players:
            p.pass_turn()


# --- Original Logic copied from generate_solution.py ---

def r1():
    # print("# Round 1: TurboMode, Enemy->5")
    with RoundScope(3):
        p1.action("Move 0"); p1.action("Move 5") # P1 to Kitchen (5)
        p5.action("Move 0"); p5.action("Move 6") # P5 to Cannons (6)
        p6.action("Move 0"); p6.action("Move 6") # P6 to Cannons (6)
        p2.action("Move 0"); p2.action("Move 3") # P2 to Cargo (3)
        p3.action("Move 0"); p3.action("Move 7") # P3 to Bridge (7)
        p4.action("Move 0"); p4.action("Move 4"); p4.action("PickUp Extinguisher") # P4 to Engine (4) + Extinguisher

def r2():
    # print("# Round 2: SugarRush. Enemy->7. Fire in 5.")
    with RoundScope(3):
        # p4 already in 4 with Extinguisher
        p4.action("Extinguish", 1)
        p4.action("Interact", 1)
        p4.action("Move 0", 0)
        p3.action("EvasiveManeuvers", 2)
        p1.action("Bake", 1)
        p1.action("PickUp", 1)
        p2.action("Move 0", 0)
        p2.action("Move 6", 0) # Cannons (6)

def r3():
    # print("# Round 3: Overheating. Enemy->5.")
    with RoundScope():
        p4.ap -= 1
        p4.action("Move 0", 0)
        p3.action("Move 4", 0) # Engine (4)
        p3.action("Move 7", 0) # Bridge (7)
        p2.action("Move 0", 0)
        p1.action("Interact", 1)
        p1.action("Throw P2 0", 1)
        p2.action("Throw P5 0", 1)
        p5.action("Shoot", 1)
        p3.action("EvasiveManeuvers", 2)

def r4():
    # print("# Round 4: Rudderless. Enemy->7.")
    with RoundScope():
        p4.action("Move 7", 1) # Bridge (7)
        p4.action("Interact", 1)
        p3.action("EvasiveManeuvers", 2)
        p2.action("Move 3", 1) # Cargo (3)
        p1.action("Bake", 1)

def r5():
    # print("# Round 5: NoLight. Enemy->6.")
    with RoundScope():
        p2.action("Interact", 1)
        p3.action("EvasiveManeuvers", 2)
        p1.action("PickUp", 1)
        p1.action("Move 0", 1)
        p1.action("Drop 0", 0)

def r6():
    # print("# Round 6: FogBank. Enemy->9.")
    with RoundScope():
        p3.action("EvasiveManeuvers", 2)
        p1.action("PickUp", 1)
        p1.action("Throw P5 0", 1)
        p5.action("Shoot", 1)
        p2.action("Move 6", 2) # Cannons (6)

def r7():
    # print("# Round 7: Panic! Enemy->8.")
    with RoundScope():
        p1.action("Move 0", 1); p1.action("Move 5", 1) # Kitchen (5)
        p5.action("Move 0", 1); p5.action("Move 6", 1) # Cannons (6)
        p6.action("Move 0", 1); p6.action("Move 6", 1) # Cannons (6)
        p3.action("Move 0", 1); p3.action("Move 7", 1) # Bridge (7)
        p2.action("Move 0", 1)
        p4.action("Move 0", 1); p4.action("Move 4", 1) # Engine (4)

def r8():
    # print("# Round 8: ListingShip. Enemy->8. Fire in 9.")
    with RoundScope():
        p4.action("Interact", 2) 
        p2.action("Move 4", 1) # Engine (4)
        # p2.action("Extinguish", 1) # No fire to extinguish
        p3.action("EvasiveManeuvers", 2)
        p5.action("PickUp", 1)
        p6.action("PickUp", 1)
        p5.action("Shoot", 1)
        p6.action("Shoot", 1)
        # p1.action("Move 5", 0) # Kitchen (5) - Already there
        p1.action("PickUp", 1)

def r9():
    # print("# Round 9: WeirdGifts. Enemy->6.")
    with RoundScope():
        # p5.action("Shoot", 1) # No ammo
        p1.action("Move 0", 1)
        p1.action("Throw P6 0", 1)
        p6.action("Shoot", 1)
        p3.action("EvasiveManeuvers", 2)
        p2.action("Move 0", 1)
        p2.action("Move 3", 1) # Cargo (3)

def r10():
    # print("# Round 10: Kill Boss / Defense.")
    with RoundScope():
        p3.action("EvasiveManeuvers", 2)
        p4.action("Extinguish", 1)
        p4.action("Move 0", 1)
        p2.action("Interact", 1)
        p1.action("Move 5", 1) # Kitchen (5)
        p1.action("Bake", 1)

def r11():
    # print("# Round 11: Overheating. Enemy->9.")
    with RoundScope():
        p4.action("Move 4", 1) # Engine (4)
        p4.action("Interact", 1)
        p3.action("EvasiveManeuvers", 2)
        p1.action("Bake", 1)
        p1.action("PickUp", 1)
        p2.action("Move 0", 1)
        p2.action("Move 5", 1) # Kitchen (5)

def r12():
    # print("# Round 12: Monster Dough. Enemy->2.")
    with RoundScope():
        p1.action("Interact", 1)
        p1.action("Move 0", 1)
        p2.action("PickUp", 1)
        p2.action("Move 0", 1)
        p3.action("EvasiveManeuvers", 2)
        p4.action("Move 0", 1)
        p4.action("Move 1", 1) # Bow (1)

def r13():
    # print("# Round 13: The Staff. Enemy->2.")
    with RoundScope():
        p4.action("Interact", 1)
        p1.action("Move 6", 1) # Cannons (6)
        p1.action("Throw P5 0", 1)
        p2.action("Move 6", 1) # Cannons (6)
        p2.action("Throw P6 0", 1)
        p5.action("Shoot", 1)
        p6.action("Shoot", 1)
        p3.action("EvasiveManeuvers", 2)

def r14():
    # print("# Round 14: Blockade. Enemy->8.")
    with RoundScope():
        p4.action("Move 0", 1)
        p3.action("EvasiveManeuvers", 2)
        p4.action("Interact", 1)
        p1.action("Move 0", 1)
        p1.action("Move 5", 1) # Kitchen (5)
        p2.action("Move 0", 1)
        p2.action("Move 5", 1) # Kitchen (5)

def r15():
    # print("# Round 15: Refill Bake.")
    with RoundScope():
        p1.action("Bake", 1)
        p1.action("PickUp", 1)
        p2.action("EvasiveManeuvers", 2)
        p3.action("Move 5", 1)
        p3.action("PickUp", 1)
        p4.action("Move 0", 1)
        p4.action("Move 5", 1) # Kitchen (5)

def r16():
    # print("# Round 16: Return to Stations.")
    with RoundScope():
        p2.action("EvasiveManeuvers", 2)
        p1.action("Move 0", 1)
        p1.action("Throw P5 0", 1)
        p3.action("Move 0", 1)
        p3.action("Throw P6 0", 1)
        p4.action("PickUp", 1)
        p4.action("Move 0", 1)
        
        # Correction from previous debug: P6 IS in 5, needs to move to 6
        p6.action("Move 0", 1)
        p6.action("Move 6", 1) # Cannons (6)
        
        p5.action("Shoot", 1)

def r17():
    # print("# Round 17: Finish Monster.")
    with RoundScope():
        p2.action("EvasiveManeuvers", 2)
        p4.action("Move 6", 1) # Cannons (6)
        p4.action("Throw P5 1", 1) # Throw Peppernut (Index 1, as 0 is Extinguisher)
        p5.action("Shoot", 1)
        p6.action("Shoot", 1)
        p1.action("Move 6", 1) # Cannons (6)
        # p1.action("Extinguish", 1)
        p3.action("Move 5", 1) # Kitchen (5)
        p3.action("Bake", 1) 

def r18():
    # print("# Round 18: Pickup & Reload.")
    with RoundScope():
        p1.action("Move 0", 1)
        p1.action("Move 5", 1)
        
        p3.action("PickUp", 1)
        p3.action("Move 0", 1)
        
        p4.action("Move 0", 1)
        p4.action("Move 5", 1) # Kitchen (5)
        p6.action("Move 0", 1)
        p6.action("Move 5", 1) # Kitchen (5)
        p2.action("EvasiveManeuvers", 2)

def r19():
    # print("# Round 19: Finish Monster (Again).")
    with RoundScope():
        p2.action("EvasiveManeuvers", 2)
        
        # P3 supplies P5
        p3.action("Throw P5 0", 1)
        p5.action("Shoot", 1)
        
        # Others reload and move to Hub
        p1.action("PickUp", 1)
        p1.action("Move 0", 1)
        
        p4.action("PickUp", 1)
        p4.action("Move 0", 1)
        
        p6.action("PickUp", 1)
        p6.action("Move 0", 1)
        
        p3.action("Move 5", 1) # P3 goes back to kitchen

def r20():
    # print("# Round 20: Lucky Dip. Monster Low.")
    with RoundScope():
        p2.action("EvasiveManeuvers", 2)
        
        # P1 supplies P5
        p1.action("Throw P5 0", 1)
        p5.action("Shoot", 1)
        
        # P6 moves in and shoots
        p6.action("Move 6", 1)
        p6.action("Shoot", 1)
        
        # Reinforcements
        p4.action("Move 6", 1) # P4 moves to Cannons
        
        p3.action("PickUp", 1) # P3 grabs form Kitchen
        p3.action("Move 0", 1) # Moves to Hub

def r21():
    # print("# Round 21: Reload. P3 Asleep (Solved by P1).")
    with RoundScope():
        
        # P1 Solves Nap (Reader Rotation means P3 was asleep)
        p1.action("Interact", 1)

        p3.action("Move 5", 1)
        p3.action("Bake", 1)
        
        p4.action("Move 0", 1)
        p4.action("Move 5", 1)
        
        p2.action("EvasiveManeuvers", 2)

def r22():
    # print("# Round 22: Deliver Ammo.")
    with RoundScope():
        p2.action("EvasiveManeuvers", 2)
        p3.action("Move 0", 1)
        p3.action("Throw P5 0", 1)
        
        p4.action("Move 0", 1)
        p4.action("Throw P6 0", 1)
        
        p5.action("Shoot", 1)
        p6.action("Shoot", 1)

def r23():
    # print("# Round 23: Solve Headwind/Clamp/Seasick.")
    with RoundScope():
        
        # P4: Nap solved in R21. No action needed.
        # p4.action("Interact", 1) 
        
        # Bridge Team (P5, P6) solve local issues (Headwind, Clamp, Static)
        p6.action("Interact", 1)
        p6.action("Interact", 1)
        p5.action("Interact", 1)
        
        # P2 moves towards Kitchen (Stops at 0 due to Sticky Floor cost in next step)
        p2.action("Move 0", 1)
        # p2.action("Move 5", 1) # Cannot do this turn (Cost 2)
        
        # P3 is already in 0.

def r24():
    # print("# Round 24: Heal. Solve Sticky. Move.")
    with RoundScope():
        
        # P3 moves to Engine (Skip Heal - P5 is full HP)
        p3.action("Move 0", 1)
        p3.action("Move 4", 1)
        
        # P5 Regroups
        p5.action("Move 0", 1)
        
        # P6 solves The Book (in Bridge)
        p6.action("Interact", 1)
        p6.action("Move 0", 1) # Regroup
        
        # P2 enters Kitchen (Cost 2 due to Sticky)
        p2.action("Move 5", 2)
        
        # Others regroup
        p1.action("Move 0", 1)
        p4.action("Move 0", 1)

def r25():
    # print("# Round 25: Recovery/Solve.")
    with RoundScope():
        
        # P2 (in R0 - pushed by Waves) moves to Kitchen
        p2.action("Move 5", 2) # Cost 2 (Sticky)
        
        # P3 (in Engine) Moves to Hub (No fire in 4 to extinguish)
        # p3.action("Extinguish", 1) # REMOVED: No fire here
        p3.action("Move 0", 1)
        
        # P4 moves to Cargo to Repair (Unchanged)
        p4.action("Move 0", 1)
        p4.action("Move 3", 1)
        
        # Others (P1, P5, P6) in Engine (pushed by Waves). Move to Hub.
        p1.action("Move 0", 1)
        p5.action("Move 0", 1)
        p6.action("Move 0", 1)

def r26():
    # print("# Round 26: Recovery & Reload.")
    with RoundScope():
        
        # P2 (in Kitchen) Solves Sticky + Seasick
        p2.action("Interact", 1)
        p2.action("Interact", 1)
        
        # P3 (in R4 now?) moves to Cannons?
        # Previous: P3 (in R0) moves to 6.
        # New: P3 starts in R4. Move 0 -> Move 6. (2 AP).
        p3.action("Move 6", 1)
        p3.action("Move 0", 1) # Burn AP to satisfy Seasick (Cannot Pass)
        
        # P4 moves to Bow
        p4.action("Move 0", 1)
        p4.action("Move 1", 1)
        
        # P1, P5, P6 ABORT Kitchen (Fire + Sticky Floor = Impossible)
        # Redirect to Engine (4) to help P3 or just wait safely
        p1.action("Move 0", 1)
        # p1.action("Move 4", 1) # Optional positioning
        
        p5.action("Move 0", 1)
        # p5.action("Move 6", 1) # Go to Cannons
        
        p6.action("Move 0", 1)
        # p6.action("Move 6", 1) # Go to Cannons

def r27():
    # print("# Round 27: Big Leak. Victory.")
    with RoundScope():
        
        # P4 Solves Seagull (Bow)
        p4.action("Interact", 1)
        p4.action("Move 0", 1)
        
        # P2 Extinguishes Dormitory (2) then Recovers
        # P2 starts in 2 (Respawned)
        # p2.action("Extinguish", 1) # Fire not spawned yet!
        p2.action("Move 0", 1)
        
        # P1 fetches ammo
        p1.action("Move 5", 1)
        p1.action("PickUp", 1)
        
        # P5 Extinguishes Kitchen (5)
        p5.action("Move 5", 1)
        p5.action("Extinguish", 1)
        
        # Gunners to Cannons
        p3.action("Move 6", 1)
        # p5 moved to 5
        p6.action("Move 6", 1)

def r28():
    # print("# Round 28: Recipe Reload & Volley.")
    with RoundScope():
        p4.action("Move 1", 1); p4.action("Interact", 1)
        p2.action("Move 7", 1); p2.action("Extinguish", 1)
        p5.action("Move 0", 1); p5.action("Move 6", 1)
        p6.action("Shoot", 1)
        
        # P3 Shoots ONCE. Saves 1 for R29.
        p3.action("Shoot", 1)
        
        p1.action("Move 0", 1)

def r29():
    # print("# Round 29: Kill Monster.")
    with RoundScope():
        # P3 starts with 1 nut.
        p3.action("Shoot", 1) # 1->0
        
        p1.action("Move 2", 1); p1.action("Extinguish", 1)
        p2.action("Extinguish", 1); p2.action("Move 0", 1)
        
        # P4 Refills P3
        p4.action("Move 0", 1); p4.action("Throw P3 0", 1)
        
        # P3 Shoots Again
        p3.action("Shoot", 1) # 1->0
        
        p5.action("Shoot", 1)
        p6.action("Shoot", 1)

def r30():
    # print("# Round 30: High Pressure. Solve Attack Wave. Reload.")
    with RoundScope():
        
        # P5 (in 0) Moves to 6. Shoots (Has ammo).
        p5.action("Move 6", 1)
        p5.action("Shoot", 1)
        
        # P1 (in 0) Feeds Everyone!
        # P1 has plenty of ammo (Recipe gave 2, +1 from R27).
        p1.action("Throw P6 0", 1)
        p1.action("Throw P3 0", 1)
        
        # P6 (in 0) Moves to 6, Shoots (Refilled by P1).
        p6.action("Move 6", 1)
        p6.action("Shoot", 1)
        
        # P3 (in 0) Moves to 6, Shoots (Refilled by P1).
        p3.action("Move 6", 1)
        p3.action("Shoot", 1)
        
        # P2 scattered. Skip.
        pass

def r31():
    # print("# Round 31: Victory.")
    with RoundScope():
        # P4 helps feed P6 (proven to work)
        p4.action("Move 0", 1)
        p4.action("Throw P6 0", 1)
        
        # P6 Shoots.
        p6.action("Shoot", 1)

def main():
    seed = 12345
    
    # Run Rounds to record actions
    r1(); r2(); r3(); r4(); r5(); r6(); r7(); r8(); r9(); r10()
    r11(); r12(); r13(); r14(); r15(); r16(); r17(); r18(); r19(); r20()
    r21(); r22(); r23(); r24(); r25(); r26(); r27(); r28(); r29(); r30()
    r31()
    
    # Execute
    print(f"Executing {len(rounds_log)} rounds...")
    
    player_ids = ["P1", "P2", "P3", "P4", "P5", "P6"]
    initial_state = sint_core.new_game(player_ids, seed)
    
    result = sint_solver.verify_solution(initial_state, rounds_log)

    is_success = result["success"]
    is_error = result.get("error") is not None

    if is_success:
        print("✅ SUCCESS!")
    else:
        print("❌ FAILURE!")

    print(f"Final Hull: {result['final_state']['hull_integrity']}")
    print(f"Final Boss HP: {result['final_state']['enemy']['hp']}")
    print(f"Score: {result['score']}")

    if not is_success:
        print(result["failure_summary"])

    # Show trajectory if it's a valid game run (Success or Defeat), but not on Execution Error
    if not is_error:
        print("\n--- TRAJECTORY LOG ---")
        logs = sint_solver.get_trajectory_log(initial_state, result['history'])
        for l in logs:
            print(l, end='')

    if not is_success:
        sys.exit(1)

if __name__ == "__main__":
    main()
