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
        # State: P1:R0, P2:R4(1HP), P3:R0(WB), P4:R4(2HP), P5:R2(Ext,3HP), P6:R4(1HP)
        # Hazards: R0(1 fire), R7(2 fires)
        # Enemy targets R6 with Fireball.
        # P5 (Extinguisher) clears fire in Hallway
        p5.action("Move 0"); p5.action("Extinguish")
        # P3 moves to Bridge and clears one fire
        p3.action("Move 7"); p3.action("Extinguish")
        # P1 moves to Bridge to clear second fire
        p1.action("Move 7"); p1.action("Extinguish")
        # P2, P4, P6 move out of Engine (Room 4) to Hallway
        p2.action("Move 0"); p2.action("Pass")
        p4.action("Move 0"); p4.action("Pass")
        p6.action("Move 0"); p6.action("Pass")

def r20(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # State: P1:R7, P2:R0(1HP), P3:R7(WB), P4:R0(2HP), P5:R0(Ext,3HP), P6:R0(1HP)
        # Hazards: R6(2 fires). Enemy targets R6 with Fireball.
        # P1 performs RaiseShields in Bridge (Room 7) - Game logic says RaiseShields is here.
        p1.action("RaiseShields")
        # P5 (Extinguisher) clears fire in Cannons (Room 6)
        p5.action("Move 6"); p5.action("Extinguish")
        # P2 and P6 move to Sickbay (Room 8) and heal
        p2.action("Move 8"); p2.action("FirstAid P2")
        p6.action("Move 8"); p6.action("FirstAid P6")
        # P4 moves to Storage to pick up a nut
        p4.action("Move 9"); p4.action("PickUp Peppernut")
        # P3 moves to Hallway
        p3.action("Move 0"); p3.action("Pass")

def r21(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # State: P1:R7, P2:R8(Fire), P3:R0(WB), P4:R9(Nut), P5:R6(Ext), P6:R8(Fire)
        # Hazards: R3(3 fires), R8(1 fire). Enemy targets R0 with Miss.
        # P2 clears fire in Sickbay and moves to Hallway
        p2.action("Extinguish"); p2.action("Move 0")
        # P6 moves to Hallway
        p6.action("Move 0"); p6.action("Pass")
        # P5 (Extinguisher) moves to Cargo (Room 3)
        p5.action("Move 0"); p5.action("Move 3")
        # P1 moves to Hallway
        p1.action("Move 0"); p1.action("Pass")
        # P3 moves to Hallway (already there) or Cargo
        p3.action("Move 3"); p3.action("Pass")
        # P4 moves to Hallway
        p4.action("Move 0"); p4.action("Pass")

def r22(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # State: P1,P2,P4(nut),P6 in 0. P3(WB),P5(Ext) in 3.
        # Hazards: 0. Enemy targets R5 with Fireball.
        # P3 and P5 repair hull in Cargo
        p3.action("Repair"); p3.action("Repair")
        p5.action("Repair"); p5.action("Repair")
        # P4 shoots
        p4.action("Move 6"); p4.action("Shoot")
        # P1 bakes in Kitchen
        p1.action("Move 5"); p1.action("Bake")
        # P2 and P6 move to Kitchen to prepare for reload
        p2.action("Move 5"); p2.action("Pass")
        p6.action("Move 5"); p6.action("Pass")

def r23(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # State: P1,P2,P6 in 5 (2 fires, 1HP). P3,P5 in 3. P4 in 6.
        # Situations: The Staff (1 AP). Hazards: R0(1 fire), R5(2 fires).
        # P1, P2, P6 MUST move out of Room 5 to avoid fainting, even if R0 has fire
        p1.action("Move 0")
        p2.action("Move 0")
        p6.action("Move 0")
        # P4 moves to Hallway
        p4.action("Move 0")
        # P3 and P5 repair hull in Cargo
        p3.action("Repair")
        p5.action("Repair")

def r24(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # State: P1,P2,P6 in 2. P3,P5(Ext) in 3. P4 in 0.
        # Hazards: R0(1), R3(2), R5(2). Enemy targets R7 with Fireball.
        # P5 (Extinguisher) clears fire in Cargo and repairs hull
        p5.action("Extinguish"); p5.action("Repair")
        # P3 repairs hull twice in Cargo
        p3.action("Repair"); p3.action("Repair")
        # P4 moves to Cargo and repairs hull
        p4.action("Move 3"); p4.action("Repair")
        # P1 moves to Hallway and clears fire
        p1.action("Move 0"); p1.action("Extinguish")
        # P2 and P6 move to Kitchen to clear fires next round
        p2.action("Move 0"); p2.action("Move 5")
        p6.action("Move 0"); p6.action("Move 5")

def r25(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # State: P1:R0, P2,P6 in 5. P3,P4,P5 in 3.
        # Hazards: R7(2). Enemy targets R7 with Fireball.
        # P3, P4, P5 repair hull in Cargo
        p3.action("Repair"); p3.action("Repair")
        p4.action("Repair"); p4.action("Repair")
        p5.action("Repair"); p5.action("Repair")
        # P2, P6 pick up nuts and move to Hallway
        p2.action("PickUp Peppernut"); p2.action("Move 0")
        p6.action("PickUp Peppernut"); p6.action("Move 0")
        # P1 moves to Bridge to clear fire
        p1.action("Move 7"); p1.action("Pass")

def r26(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # State: P1:R7(4 fires). P2,P6 in 0(nuts). P3,P4,P5 in 3.
        # Hazards: R7(4). Enemy targets R2 with Fireball.
        # P1 moves out of Bridge to avoid fainting
        p1.action("Move 0"); p1.action("Pass")
        # P2 lifts Blockade in Room 0 and moves to Cannons
        p2.action("Interact"); p2.action("Move 6")
        # P6 moves to Cannons and shoots
        p6.action("Move 6"); p6.action("Shoot")
        # P3, P4 repair hull in Cargo
        p3.action("Repair"); p3.action("Repair")
        # P4 repairs hull and moves to Hallway
        p4.action("Repair"); p4.action("Move 0")
        # P5 (Extinguisher) moves to Hallway
        p5.action("Move 0"); p5.action("Pass")

def r27(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # State: P2,P6 in 6. P1,P4,P5 in 0. P3 in 3.
        # Hazards: R0(1), R2(2), R7(4). Enemy targets R5 with Fireball.
        # P2 shoots and moves to Hallway
        p2.action("Shoot"); p2.action("Move 0")
        # P6 moves to reload in Kitchen
        p6.action("Move 0"); p6.action("Move 5")
        # P5 (Extinguisher) clears fire in Hallway and moves to Bridge
        p5.action("Extinguish"); p5.action("Move 7")
        # P1 moves to Dormitory to clear fire
        p1.action("Move 2"); p1.action("Extinguish")
        # P4 moves to Kitchen to reload
        p4.action("Move 5"); p4.action("PickUp Peppernut")
        # P3 repairs hull
        p3.action("Repair"); p3.action("Repair")

def r28(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # State: P2:R1, P6:R6, P4,P5:R2, P1:R3, P3:R4(Water).
        # Hazards: R0(1), R2(1), R4(1W), R5(2), R7(4). Enemy targets R7 with Fireball.
        # P2 moves to Kitchen to prepare for reload
        p2.action("Move 0"); p2.action("Move 5")
        # P6 moves to Kitchen to prepare for reload
        p6.action("Move 0"); p6.action("Move 5")
        # P4 (has nut) moves to Hallway
        p4.action("Move 0"); p4.action("Pass")
        # P5 (Extinguisher) moves to Hallway
        p5.action("Move 0"); p5.action("Pass")
        # P1 repairs hull
        p1.action("Repair"); p1.action("Repair")
        # P3 repairs water in Engine
        p3.action("Repair"); p3.action("Move 0")

def r29(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # State: P4,P5:R1. P3,P6:R2(Fire). P1:R4. P2:R6.
        # Hazards: R7(6), R5(2), R2(1), R0(1).
        # P3 clears fire in Dormitory and moves to Hallway
        p3.action("Extinguish"); p3.action("Move 0")
        # P6 moves to Hallway
        p6.action("Move 0"); p6.action("Pass")
        # P2 moves to Hallway
        p2.action("Move 0"); p2.action("Pass")
        # P1 moves to Cargo to repair
        p1.action("Move 0"); p1.action("Move 3")
        # P5 (Extinguisher) moves to Bridge
        p5.action("Move 0"); p5.action("Move 7")
        # P4 moves to Bridge
        p4.action("Move 0"); p4.action("Move 7")

def r30(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # State: P1:R4, P2:R2, P3:R1, P4,P5:R8, P6:R1.
        # Hazards: R7(6), R5(2), R0(1), R2(1W), R4(1W).
        # P5 (Extinguisher) moves to Bridge (Room 7) and clears 4 fires
        p5.action("Move 0"); p5.action("Move 7")
        p5.action("Extinguish"); p5.action("Extinguish"); p5.action("Pass")
        # P4 moves to Bridge and clears 1 fire
        p4.action("Move 0"); p4.action("Move 7")
        p4.action("Extinguish"); p4.action("Pass")
        # P2 clears fire in Kitchen (Room 5)
        p2.action("Move 0"); p2.action("Move 5")
        p2.action("Extinguish"); p2.action("Extinguish"); p2.action("Pass")
        # P3 and P6 move to Cargo to repair hull
        p3.action("Move 0"); p3.action("Move 3")
        p3.action("Repair"); p3.action("Repair"); p3.action("Pass")
        p6.action("Move 0"); p6.action("Move 3")
        p6.action("Repair"); p6.action("Repair"); p6.action("Pass")
        # P1 moves to Cargo to repair hull
        p1.action("Move 0"); p1.action("Move 3")
        p1.action("Repair"); p1.action("Repair"); p1.action("Pass")

def r31(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # State: P1,P4,P5 in 2(W). P3,P6 in 4(W). P2 in 6.
        # Hazards: R3(2F,1W), R0(1F), R7(1F), R8(1W), R2(1W), R4(1W).
        # P1 clears water in Dormitory (Room 2) and moves to Cargo
        p1.action("Repair"); p1.action("Move 0"); p1.action("Move 3"); p1.action("Pass")
        # P4 moves to Cargo
        p4.action("Move 0"); p4.action("Move 3"); p4.action("Pass")
        # P5 (Extinguisher) moves to Cargo
        p5.action("Move 0"); p5.action("Move 3"); p5.action("Pass")
        # P3 clears water in Engine (Room 4) and moves to Hallway
        p3.action("Repair"); p3.action("Move 0"); p3.action("Pass")
        # P6 moves to Cargo
        p6.action("Move 0"); p6.action("Move 3"); p6.action("Pass")
        # P2 moves to Hallway
        p2.action("Move 0"); p2.action("Pass")

def r32(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # State: P1,P4,P5 in 4. P3,P6 in 2. P2 in 1.
        # Hazards: R3(2F,1W), R5(2F), R0(1F), R7(1F), R2(1W), R8(1W).
        # P5 (Extinguisher) clears Cargo
        p5.action("Move 0"); p5.action("Move 3")
        p5.action("Extinguish"); p5.action("Repair"); p5.action("Pass")
        # P1, P4 repair hull in Cargo
        p1.action("Move 0"); p1.action("Move 3")
        p1.action("Repair"); p1.action("Repair"); p1.action("Pass")
        p4.action("Move 0"); p4.action("Move 3")
        p4.action("Repair"); p4.action("Repair"); p4.action("Pass")
        # P3, P6 clear fire in Kitchen next round
        p3.action("Move 0"); p3.action("Move 5"); p3.action("Pass")
        p6.action("Move 0"); p6.action("Move 5"); p6.action("Pass")
        # P2 moves to Hallway
        p2.action("Move 0"); p2.action("Pass")

def r33(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # Clear fires in Kitchen and Bridge
        p3.action("Extinguish"); p3.action("Extinguish"); p3.action("Pass")
        p6.action("PickUp Peppernut"); p6.action("Move 0"); p6.action("Pass")
        p2.action("Move 7"); p2.action("Extinguish"); p2.action("Pass")
        # Repair hull
        for p in [p1, p4, p5]:
            p.action("Repair"); p.action("Repair"); p.action("Pass")

def r34(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # Shooting
        p2.action("Move 6"); p2.action("Shoot"); p2.action("Pass")
        p6.action("Move 6"); p6.action("Shoot"); p6.action("Pass")
        # Reloading
        p3.action("PickUp Peppernut"); p3.action("Move 0"); p3.action("Pass")
        # Repairing
        for p in [p1, p4, p5]:
            p.action("Repair"); p.action("Repair"); p.action("Pass")

def r35(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p3.action("Move 6"); p3.action("Shoot"); p3.action("Pass")
        p1.action("Move 6"); p1.action("PickUp Peppernut"); p1.action("Shoot"); p1.action("Pass")
        for p in [p2, p4, p5, p6]:
            p.action("Repair"); p.action("Repair"); p.action("Pass")

rounds_list = [r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, r13, r14, r15, r16, r17, r18, r19, r20, r21, r22, r23, r24, r25, r26, r27, r28, r29, r30, r31, r32, r33, r34, r35]
