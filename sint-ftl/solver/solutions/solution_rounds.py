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
        # Move costs +1 due to Rudderless (Total 2 AP per move)
        # P1: Solve Man Overboard in Room 1 (Moves: 2x2=4, Interact: 1) = 5 AP
        p1.action("Move 0"); p1.action("Move 1"); p1.action("Interact"); p1.action("Pass")
        # P2, P3, P4: Repair Hull in Room 3
        p2.action("Move 0"); p2.action("Move 3"); p2.action("Repair"); p2.action("Pass")
        p3.action("Move 0"); p3.action("Move 3"); p3.action("Repair"); p3.action("Pass")
        p4.action("Move 0"); p4.action("Move 3"); p4.action("Repair"); p4.action("Pass")
        # P5, P6: Reload in Room 5
        p5.action("Move 0"); p5.action("Move 5"); p5.action("Bake"); p5.action("Pass")
        p6.action("Move 0"); p6.action("Move 5"); p6.action("PickUp Peppernut"); p6.action("Pass")

def r14(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 to Bridge
        p3.action("Move 0"); p3.action("Move 7")
        # P1, P2, P4 to Kitchen
        p1.action("Move 0"); p1.action("Move 5")
        p2.action("Move 0"); p2.action("Move 5")
        p4.action("Move 0"); p4.action("Move 5")
        # P5, P6 to Cannons
        p5.action("Move 0"); p5.action("Move 6")
        p6.action("Move 0"); p6.action("Move 6")

def r15(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 shields
        p3.action("RaiseShields")
        # P1, P2, P4 reload
        p1.action("PickUp Peppernut"); p1.action("Move 0")
        p2.action("PickUp Peppernut"); p2.action("Move 0")
        p4.action("PickUp Peppernut"); p4.action("Move 0")
        # P5, P6 shoot
        p5.action("Shoot"); p5.action("Pass")
        p6.action("Shoot"); p6.action("Pass")

def r16(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 shields
        p3.action("RaiseShields")
        # P1, P2, P4 move to 6 and shoot
        p1.action("Move 6"); p1.action("Shoot")
        p2.action("Move 6"); p2.action("Shoot")
        p4.action("Move 6"); p4.action("Shoot")
        # P5, P6 reload? (Kitchen is empty, Storage has nuts)
        p5.action("Move 0"); p5.action("Move 9")
        p6.action("Move 0"); p6.action("Move 9")

def r17(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3 shields
        p3.action("RaiseShields")
        # P5, P6 reload from Storage
        p5.action("PickUp Peppernut"); p5.action("Move 0")
        p6.action("PickUp Peppernut"); p6.action("Move 0")
        # Others shoot remaining Super Peppernuts (Recipe gave 2 each)
        p1.action("Shoot"); p1.action("Pass")
        p2.action("Shoot"); p2.action("Pass")
        p4.action("Shoot"); p4.action("Pass")

rounds_list = [r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, r13, r14, r15, r16, r17]
