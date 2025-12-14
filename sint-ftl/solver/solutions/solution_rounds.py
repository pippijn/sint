from execute_solution import RoundScope

def r1(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P1: Bake in Kitchen (Room 5)
        p1.action("Move 0"); p1.action("Move 5"); p1.action("Bake")
        # P2: Get Wheelbarrow in Cargo (Room 3)
        p2.action("Move 0"); p2.action("Move 3"); p2.action("PickUp Wheelbarrow")
        # P3: To Bridge (Room 7)
        p3.action("Move 0"); p3.action("Move 7"); p3.action("Pass")
        # P4: Get Extinguisher in Engine (Room 4)
        p4.action("Move 0"); p4.action("Move 4"); p4.action("PickUp Extinguisher")
        # P5 & P6: To Hallway (Room 0)
        p5.action("Move 0"); p5.action("Pass")
        p6.action("Move 0"); p6.action("Pass")

def r2(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P4 (Extinguisher): Clear Room 6, grab a nut from Kitchen
        p4.action("Move 0"); p4.action("Move 6"); p4.action("Extinguish")
        p4.action("Move 0"); p4.action("Move 5"); p4.action("PickUp Peppernut"); p4.action("Move 0"); p4.action("Pass")
        # P1 (Kitchen): Deliver one nut to P5, keep one
        p1.action("PickUp Peppernut"); p1.action("Move 0"); p1.action("Throw P5 0")
        p1.action("Move 5"); p1.action("PickUp Peppernut")
        # P2 (Storage): Collect 3 nuts with Wheelbarrow
        p2.action("Move 0"); p2.action("Move 9")
        p2.action("PickUp Peppernut"); p2.action("PickUp Peppernut"); p2.action("PickUp Peppernut")
        # P3: Shields
        p3.action("RaiseShields"); p3.action("Pass")
        # P5 & P6: Stay in 0
        p5.action("Pass")
        p6.action("Pass")

def r3(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P4: Stay in 0 and prepare to clear Turbo Mode fire
        p4.action("Move 0"); p4.action("Pass")
        # P2: Finish collecting nuts in Room 9 (now has 5)
        p2.action("PickUp Peppernut"); p2.action("PickUp Peppernut"); p2.action("Move 0"); p2.action("Pass")
        # P1: Move 0, Throw to P6, Move 5, Bake
        p1.action("Move 0"); p1.action("Throw P6 0"); p1.action("Move 5"); p1.action("Bake"); p1.action("Pass")
        # P3: Shields
        p3.action("RaiseShields"); p3.action("Pass")
        # P5 & P6: Wait
        p5.action("Pass")
        p6.action("Pass")

def r4(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P4: Clear fire in Room 0, move to 6
        p4.action("Extinguish"); p4.action("Move 6"); p4.action("Pass")
        # P5 & P6: Move to 6
        p5.action("Move 6"); p5.action("Pass")
        p6.action("Move 6"); p6.action("Pass")
        # P1: Move to 0, Move 6
        p1.action("Move 0"); p1.action("Move 6"); p1.action("Pass")
        # P2: Move to 6, Drop nuts
        p2.action("Move 6")
        for _ in range(5): p2.action("Drop 1")
        p2.action("Pass")
        # P3: Shields
        p3.action("RaiseShields")

def r5(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # Shooters already holding nuts
        p4.action("Shoot"); p4.action("PickUp Peppernut")
        p5.action("Shoot"); p5.action("PickUp Peppernut")
        p6.action("Shoot"); p6.action("PickUp Peppernut")
        # Shooters needing nuts
        p1.action("PickUp Peppernut"); p1.action("Shoot")
        p2.action("PickUp Peppernut"); p2.action("Shoot")
        # P3: Shields
        p3.action("RaiseShields")

def r6(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P4: Clear Room 4 fire and solve Overheating (Moves: 2, Action: 1, Interact: 1)
        # Wait, if Overheating is active, do actions cost more?
        # description: "End turn in Engine -> Lose 1 AP next round."
        # It doesn't say actions cost more.
        p4.action("Move 0"); p4.action("Move 4"); p4.action("Extinguish"); p4.action("Interact"); p4.action("Pass")
        # P2: Repair Hull in Room 3 (Moves: 2, Repair: 4) = 6 AP
        p2.action("Move 0"); p2.action("Move 3"); p2.action("Repair"); p2.action("Repair"); p2.action("Repair"); p2.action("Repair")
        # P1: Solve Sticky Floor and bake (Moves: 3, Interact: 1, Bake: 1) = 5 AP
        p1.action("Move 0"); p1.action("Move 5"); p1.action("Interact"); p1.action("Bake"); p1.action("Pass")
        # P5: Solve Mice Plague in Room 9
        p5.action("Move 0"); p5.action("Move 9"); p5.action("Interact"); p5.action("Drop 0"); p5.action("Pass")
        # P6: Drop nut in Storage
        p6.action("Move 0"); p6.action("Move 9"); p6.action("Drop 0"); p6.action("Pass")
        # P3: To Hallway
        p3.action("Move 0"); p3.action("Pass")

def r7(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 solves Recipe in Room 1
        p3.action("Move 1"); p3.action("Interact")
        # Everyone else moves to Room 6
        p1.action("Move 0"); p1.action("Move 6")
        p2.action("Move 0"); p2.action("Move 6")
        p4.action("Move 0"); p4.action("Move 6")
        p5.action("Move 0"); p5.action("Move 6")
        p6.action("Move 0"); p6.action("Move 6")

def r8(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 moves toward Bridge
        p3.action("Move 0"); p3.action("Move 7")
        # Everyone in 6 shoots!
        p1.action("Shoot"); p1.action("Shoot")
        p2.action("Shoot"); p2.action("Shoot")
        p4.action("Shoot"); p4.action("Shoot")
        p5.action("Shoot"); p5.action("Shoot")
        p6.action("Shoot"); p6.action("Shoot")

def r9(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 blocks damage
        p3.action("RaiseShields")
        # P4 shoots and moves
        p4.action("Shoot"); p4.action("Move 0")
        # Others move to Kitchen
        p1.action("Move 0"); p1.action("Move 5")
        p2.action("Move 0"); p2.action("Move 5")
        p5.action("Move 0"); p5.action("Move 5")
        p6.action("Move 0"); p6.action("Move 5")
        
def r10(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 blocks damage
        p3.action("RaiseShields")
        # P5 clears fire and reloads
        p5.action("Extinguish"); p5.action("PickUp Peppernut")
        # P1, P2, P6 reload and move out
        p1.action("PickUp Peppernut"); p1.action("Move 0")
        p2.action("PickUp Peppernut"); p2.action("Move 0")
        p6.action("PickUp Peppernut"); p6.action("Move 0")
        # P4 moves to reload
        p4.action("Move 5"); p4.action("Pass")


def r11(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 shields
        p3.action("RaiseShields")
        # P1, P2, P6 move and shoot
        p1.action("Move 6"); p1.action("Shoot")
        p2.action("Move 6"); p2.action("Shoot")
        p6.action("Move 6"); p6.action("Shoot")
        # P5 moves to shoot next round
        p5.action("Move 0"); p5.action("Move 6")
        # P4 reloads
        p4.action("PickUp Peppernut"); p4.action("Move 0")

def r12(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 shields
        p3.action("RaiseShields")
        # P5 shoots
        p5.action("Shoot"); p5.action("Pass")
        # P4 moves and shoots
        p4.action("Move 6"); p4.action("Shoot")
        # Others wait
        p1.action("Pass")
        p2.action("Pass")
        p6.action("Pass")

def r13(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # Rest Round: 6 AP each. No hazards present. Hull starts at 15. Max is 20.
        # P1: Solve Man Overboard in Room 1 (Moves 2, Interact 1, Moves to 3: 2, Repair 1) = 6 AP. Hull 15 -> 16.
        p1.action("Move 0"); p1.action("Move 1"); p1.action("Interact")
        p1.action("Move 0"); p1.action("Move 3"); p1.action("Repair")
        # P2: Repair in Room 3 (Moves 2, Repair 4) = 6 AP. Hull 16 -> 20.
        p2.action("Move 0"); p2.action("Move 3"); p2.action("Repair"); p2.action("Repair"); p2.action("Repair"); p2.action("Repair")
        # P3: To Bridge (Room 7) (Moves 2, Pass)
        p3.action("Move 0"); p3.action("Move 7"); p3.action("Pass")
        # P4: Stay in Room 0 or move to Room 8 (Sickbay) to heal if needed
        p4.action("Move 0"); p4.action("Move 8"); p4.action("Pass")
        # P5, P6: Reload in Room 5
        p5.action("Move 0"); p5.action("Move 5"); p5.action("Bake"); p5.action("PickUp Peppernut"); p5.action("Move 0"); p5.action("Pass")
        p6.action("Move 0"); p6.action("Move 5"); p6.action("PickUp Peppernut"); p6.action("Move 0"); p6.action("Pass")

def r14(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 to Bridge (Room 7)
        p3.action("Move 0"); p3.action("Move 7")
        # P4 to Engine (Room 4) - wait, check where extinguisher is.
        # P4 already has extinguisher.
        # P1, P2 to Kitchen (Room 5)
        p1.action("Move 0"); p1.action("Move 5")
        p2.action("Move 0"); p2.action("Move 5")
        # P5, P6 to Cannons (Room 6)
        p5.action("Move 0"); p5.action("Move 6")
        p6.action("Move 0"); p6.action("Move 6")
        # P4 moves to Room 3 (Cargo) to prepare for fires
        p4.action("Move 0"); p4.action("Move 3")

def r15(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 (Bridge) moves to Hallway (already has 2 nuts + Wheelbarrow)
        p3.action("Move 0"); p3.action("Pass")
        # P4 (Cargo) moves to Hallway
        p4.action("Move 0"); p4.action("Pass")
        # P1, P2 (Kitchen) pick up nuts and move to Hallway
        p1.action("PickUp Peppernut"); p1.action("Move 0")
        p2.action("PickUp Peppernut"); p2.action("Move 0")
        # P5 (Cannons) fixes jam and moves to Hallway (has Extinguisher + 1 nut)
        p5.action("Interact"); p5.action("Move 0")
        # P6 (Cannons) shoots and moves to Hallway
        p6.action("Shoot"); p6.action("Move 0")

def r16(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # State: P1: R1(1 nut), P2: R6(1 nut), P3: R4(2 nuts, WB), P4: R3, P5: R3(Ext), P6: R9
        # Hazards: R0 (1 fire), R9 (2 fires)
        # P5 (Extinguisher) clears fire in Hallway
        p5.action("Move 0"); p5.action("Extinguish")
        # P6 moves out of fire in Storage
        p6.action("Move 0"); p6.action("Pass")
        # P2 shoots and moves to Hallway
        p2.action("Shoot"); p2.action("Move 0")
        # P1 moves to Cannons
        p1.action("Move 0"); p1.action("Move 6")
        # P3 moves to Cannons
        p3.action("Move 0"); p3.action("Move 6")
        # P4 moves to Hallway
        p4.action("Move 0"); p4.action("Pass")

def r17(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # State: P1:R6(nut), P2:R0, P3:R6(2 nuts,WB), P4:R0, P5:R0(Ext), P6:R0
        # Hazards: R0(1 fire), R3(1 water), R9(2 fires)
        # Enemy targets R5 with Fireball.
        # P5 (Extinguisher) clears fire in Hallway and moves to Storage
        p5.action("Extinguish"); p5.action("Move 9")
        # P1 shoots and moves to Hallway
        p1.action("Shoot"); p1.action("Move 0")
        # P3 drops nuts and moves to Hallway
        p3.action("Drop 0"); p3.action("Drop 0"); p3.action("Move 0"); p3.action("Pass")
        # P2 moves to Cannons and picks up a nut
        p2.action("Move 6"); p2.action("PickUp Peppernut")
        # P4 moves to Cannons and picks up a nut
        p4.action("Move 6"); p4.action("PickUp Peppernut")
        # P6 moves to Cargo to prepare for repair
        p6.action("Move 3"); p6.action("Pass")

def r18(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # State (Predicted): P1:R0, P2:R6(nut), P3:R0(WB), P4:R6(nut), P5:R9(Ext), P6:R3
        # Hazards: R3(1 water), R9(2 fires), R5(2 fires)
        # P5 (Extinguisher) clears fire in Storage and moves to Hallway
        p5.action("Extinguish"); p5.action("Move 0")
        # P6 repairs water in Cargo and moves to Hallway
        p6.action("Repair"); p6.action("Move 0")
        # P2 and P4 shoot and move to Hallway
        p2.action("Shoot"); p2.action("Move 0")
        p4.action("Shoot"); p4.action("Move 0")
        # P1 and P3 clear fires in Kitchen
        p1.action("Move 5"); p1.action("Extinguish")
        p3.action("Move 5"); p3.action("Extinguish")

def r19(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # Storm blew everyone to Room 0.
        # P3 moves to Bridge (Room 7)
        p3.action("Move 7"); p3.action("Pass")
        # P5, P6 move to Sickbay (Room 8)
        p5.action("Move 8"); p5.action("Pass")
        p6.action("Move 8"); p6.action("Pass")
        # Reloading
        p1.action("Move 5"); p1.action("PickUp Peppernut")
        p2.action("Move 5"); p2.action("PickUp Peppernut")
        p4.action("Move 9"); p4.action("PickUp Peppernut")

def r20(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 raises shields
        p3.action("RaiseShields")
        # P5, P6 heal in Sickbay
        p5.action("FirstAid P5"); p5.action("Move 0")
        p6.action("FirstAid P6"); p6.action("Move 0")
        # P1, P2, P4 move to Cannons (Room 6)
        p1.action("Move 0"); p1.action("Move 6")
        p2.action("Move 0"); p2.action("Move 6")
        p4.action("Move 0"); p4.action("Move 6")

def r21(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 raises shields
        p3.action("RaiseShields")
        # P5 moves to Cargo (Room 3) to clear hazards
        p5.action("Move 3"); p5.action("Extinguish")
        # P6 moves to reload
        p6.action("Move 5"); p6.action("Bake")
        # P1, P2, P4 shoot
        p1.action("Shoot"); p1.action("Pass")
        p2.action("Shoot"); p2.action("Pass")
        p4.action("Shoot"); p4.action("Pass")

def r22(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 raises shields
        p3.action("RaiseShields")
        # P5 clears Cargo
        p5.action("Repair"); p5.action("Repair")
        # P6 picks up nut and moves to 0
        p6.action("PickUp Peppernut"); p6.action("Move 0")
        # P1, P2, P4 move to reload
        p1.action("Move 0"); p1.action("Move 5")
        p2.action("Move 0"); p2.action("Move 5")
        p4.action("Move 0"); p4.action("Move 9")

def r23(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 raises shields
        p3.action("RaiseShields")
        # P5 repairs hull
        p5.action("Repair"); p5.action("Repair")
        # Reloading
        p1.action("PickUp Peppernut"); p1.action("Move 0")
        p2.action("PickUp Peppernut"); p2.action("Move 0")
        p4.action("PickUp Peppernut"); p4.action("Move 0")
        # P6 moves to shoot
        p6.action("Move 6"); p6.action("Shoot")

def r24(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 raises shields
        p3.action("RaiseShields")
        # Final shots
        p1.action("Move 6"); p1.action("Shoot")
        p2.action("Move 6"); p2.action("Shoot")
        p4.action("Move 6"); p4.action("Shoot")
        p6.action("Shoot"); p6.action("Pass")
        p5.action("Repair"); p5.action("Repair")

def r25(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 raises shields
        p3.action("RaiseShields")
        p1.action("Shoot"); p1.action("Pass")
        p2.action("Shoot"); p2.action("Pass")
        p4.action("Shoot"); p4.action("Pass")
        p6.action("Pass")
        p5.action("Repair"); p5.action("Pass")

rounds_list = [r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, r13, r14, r15, r16, r17, r18, r19, r20, r21, r22, r23, r24, r25]
