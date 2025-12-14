from execute_solution import RoundScope

def r1(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P1: Kitchen (5) - Bake
        p1.action("Move 0"); p1.action("Move 5"); p1.action("Bake")
        # P2: Cargo (3) - Wheelbarrow
        p2.action("Move 0"); p2.action("Move 3"); p2.action("PickUp Wheelbarrow")
        # P3: Engine (4) - For Evasive Maneuvers
        p3.action("Move 0"); p3.action("Move 4"); p3.action("Pass")
        # P4: Engine (4) - Extinguisher
        p4.action("Move 0"); p4.action("Move 4"); p4.action("PickUp Extinguisher")
        # P5, P6: Storage (9) - Get nuts
        p5.action("Move 0"); p5.action("Move 9"); p5.action("PickUp Peppernut")
        p6.action("Move 0"); p6.action("Move 9"); p6.action("PickUp Peppernut")

def r2(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3: Evasive Maneuvers in Room 4 (2 AP)
        p3.action("EvasiveManeuvers"); p3.action("Pass")
        # P1: Bake (1), PickUp (1).
        p1.action("Bake"); p1.action("PickUp Peppernut"); p1.action("Pass")
        # P2: Move 0, 9, PickUp x3.
        p2.action("Move 0"); p2.action("Move 9")
        p2.action("PickUp Peppernut"); p2.action("PickUp Peppernut"); p2.action("PickUp Peppernut")
        # P4: Move 0, 6, Extinguish.
        p4.action("Move 0"); p4.action("Move 6"); p4.action("Extinguish"); p4.action("Pass")
        # P5, P6: Move 0, 6.
        p5.action("Move 0"); p5.action("Move 6"); p5.action("Pass")
        p6.action("Move 0"); p6.action("Move 6"); p6.action("Pass")

def r3(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3: Evasive Maneuvers
        p3.action("EvasiveManeuvers")
        # P1: Bake. Move to Cannons.
        p1.action("Bake"); p1.action("Move 0"); p1.action("Move 6"); p1.action("Pass")
        # P2: Has 3 nuts. Move to Cannons (6) and Drop them.
        p2.action("Move 0"); p2.action("Move 6")
        p2.action("Drop 1"); p2.action("Drop 1"); p2.action("Drop 1"); p2.action("Pass")
        # P4, P5, P6: Stay at Cannons, PickUp nuts.
        p4.action("PickUp Peppernut"); p4.action("Pass")
        p5.action("Pass")
        p6.action("Pass")

def r4(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # P3: Sticky Floor active. Cannot Move with 1 AP.
        p3.action("Pass")
        # P2: Move 0.
        p2.action("Move 0"); p2.action("Pass")
        p1.action("Pass")
        p4.action("Pass")
        p5.action("Pass")
        p6.action("Pass")

def r5(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # SHOOT! (Enemy 5 HP -> 1 HP)
        p1.action("Shoot"); p1.action("PickUp Peppernut")
        p4.action("Shoot"); p4.action("PickUp Peppernut")
        p5.action("Shoot"); p5.action("Pass")
        p6.action("Shoot"); p6.action("Pass")
        # P2: Move to Cannons
        p2.action("Move 6"); p2.action("Pass")
        p3.action("Pass")

def r6(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # KILL BOSS (Enemy 1 HP -> 0 HP)
        p1.action("Shoot"); p1.action("Pass")
        p4.action("Shoot"); p4.action("Pass")
        p2.action("Pass")
        p3.action("Pass")
        p5.action("Pass")
        p6.action("Pass")

def r7(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        # REST ROUND! (6 AP each)
        p4.action("Move 0"); p4.action("Extinguish"); p4.action("Move 3"); p4.action("Extinguish")
        p4.action("Pass")
        p2.action("Move 0"); p2.action("Move 3"); p2.action("Repair"); p2.action("Repair")
        p2.action("Pass")
        p1.action("Move 0"); p1.action("Move 3"); p1.action("Repair"); p1.action("Repair")
        p1.action("Pass")
        p3.action("Move 0"); p3.action("Move 3"); p3.action("Repair"); p3.action("Repair")
        p3.action("Pass")
        p5.action("Move 0"); p5.action("Move 5"); p5.action("Extinguish"); p5.action("Extinguish")
        p5.action("Pass")
        p6.action("Move 0"); p6.action("Move 4"); p6.action("Extinguish"); p6.action("Extinguish")
        p6.action("Move 0"); p6.action("Pass")

def r8(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("Move 0"); p1.action("Move 7")
        p2.action("Move 0"); p2.action("Pass")
        p3.action("Move 0"); p3.action("Pass")
        p4.action("Move 0"); p4.action("Pass")
        p5.action("Bake"); p5.action("PickUp Peppernut")
        p6.action("Move 0"); p6.action("Pass")

def r9(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("RaiseShields")
        p2.action("Move 5")
        p3.action("Move 5")
        p4.action("Move 5")
        p5.action("Move 0"); p5.action("Move 4")
        p6.action("Move 4"); p6.action("Pass")

def r10(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("RaiseShields")
        p2.action("PickUp Peppernut"); p2.action("Move 0")
        p3.action("PickUp Peppernut"); p3.action("Move 0")
        p4.action("PickUp Peppernut"); p4.action("Move 0")
        # P5 has 1 AP (Overheating).
        p5.action("Extinguish")
        # P6 has 1 AP (Overheating).
        p6.action("Move 0")

def r11(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("RaiseShields")
        p2.action("Move 6"); p2.action("Shoot")
        p3.action("Move 6"); p3.action("Shoot")
        p4.action("Move 6"); p4.action("Shoot")
        # P5 move to 0 (1 AP).
        p5.action("Move 0")
        # P6 move to Kitchen (costs 2).
        p6.action("Move 5")

def r12(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("RaiseShields")
        # P6 in Kitchen. Bake and PickUp.
        p6.action("Bake"); p6.action("PickUp Peppernut")
        # Others move and shoot? No, they have no nuts.
        # P2, P3, P4 move to 0.
        p2.action("Move 0"); p2.action("Pass")
        p3.action("Move 0"); p3.action("Pass")
        p4.action("Move 0"); p4.action("Pass")
        # P5 move to help.
        p5.action("Move 5"); p5.action("Pass") # Costs 2.

def r13(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("RaiseShields")
        # P5, P6 in Kitchen.
        p5.action("PickUp Peppernut"); p5.action("Move 0")
        p6.action("Move 0"); p6.action("Pass")
        # Others move to Kitchen.
        p2.action("Move 5")
        p3.action("Move 5")
        p4.action("Move 5")

def r14(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("RaiseShields")
        # P2, P3, P4 pick up and move to 0.
        p2.action("PickUp Peppernut"); p2.action("Move 0")
        p3.action("PickUp Peppernut"); p3.action("Move 0")
        p4.action("PickUp Peppernut"); p4.action("Move 0")
        # P5 move to cannons.
        p5.action("Move 6"); p5.action("Shoot")
        # P6 move to cannons.
        p6.action("Move 6"); p6.action("Shoot")

def r15(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("RaiseShields")
        p2.action("Move 6"); p2.action("Shoot")
        p3.action("Move 6"); p3.action("Shoot")
        p4.action("Move 6"); p4.action("Shoot")
        p5.action("Move 0"); p5.action("Pass")
        p6.action("Move 0"); p6.action("Pass")

rounds_list = [r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, r13, r14, r15]