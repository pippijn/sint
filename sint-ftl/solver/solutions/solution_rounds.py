from execute_solution import RoundScope

def r1(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log, 3):
        p1.action("Move 0"); p1.action("Move 5")
        p5.action("Move 0"); p5.action("Move 6")
        p6.action("Move 0"); p6.action("Move 6")
        p2.action("Move 0"); p2.action("Move 3")
        p3.action("Move 0"); p3.action("Move 7")
        p4.action("Move 0"); p4.action("Move 4"); p4.action("PickUp Extinguisher")

def r2(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log, 3, sugar_rush=True):
        p4.action("Move 0", 0); p4.action("Move 6", 0); p4.action("Extinguish")
        p1.action("Bake"); p1.action("PickUp Peppernut"); p1.action("Move 0", 0)
        p2.action("PickUp Wheelbarrow"); p2.action("Move 0", 0); p2.action("Move 9", 0)
        p3.action("RaiseShields")
        p5.action("Move 0", 0)
        p6.action("Move 0", 0)

def r3(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log, 3, sugar_rush=True):
        p1.action("Move 5", 0); p1.action("Interact", 1)
        p1.action("Throw P5 0", 1)
        p5.action("Move 6", 1); p5.action("Shoot", 1)
        p2.action("PickUp Peppernut", 1); p2.action("PickUp Peppernut", 1); p2.action("Move 0", 1)
        p3.action("RaiseShields", 2)
        p4.action("Move 0", 1)
        p6.action("Move 6", 1)

def r4(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log, 2):
        p4.action("Extinguish"); p4.action("Move 4")
        p2.action("Move 6"); p2.action("Throw P5 1")
        p5.action("Shoot")
        p3.action("RaiseShields")
        p1.action("Bake"); p1.action("PickUp Peppernut")

def r5(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log, 2):
        p4.ap = 1
        p4.action("Extinguish")
        p2.action("Throw P5 1")
        p1.action("Move 0"); p1.action("Throw P2 0")
        p2.action("Throw P6 1")
        p5.action("Shoot")
        p6.action("Shoot")
        p3.action("RaiseShields")

def r6(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log, 2):
        p4.ap = 1
        p4.action("Interact")
        p1.action("Move 9"); p1.action("Interact")
        p2.action("Move 0"); p2.action("Move 9")
        p3.action("RaiseShields")

def r7(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log, 2):
        p2.action("PickUp Peppernut")
        p2.action("Move 0")
        p1.action("Move 0"); p1.action("Move 6")
        p4.action("Move 0")
        p3.action("RaiseShields")

def r8(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log, 2):
        p2.action("Move 6")
        p2.action("Throw P5 0")
        p4.action("Move 6")
        p5.action("Shoot")
        p3.action("RaiseShields")

def r9(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log, 2):
        p3.action("Move 0")
        p3.action("Drop 0")
        p3.action("Move 7")
        p4.action("Move 0")
        p1.action("Move 0")
        p2.action("Move 0")
        
def r10(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log, 2):
        p4.action("Move 5", 2)
        p3.action("Move 0")
        p1.action("Move 5", 2)
        p2.action("Move 0")
        p5.action("Move 0")
        p6.action("Move 0")

def r11(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log, 2):
        p4.action("Interact")
        p4.action("PickUp Peppernut")
        p1.action("PickUp Peppernut")
        p1.action("Move 0")
        p3.action("Move 5", 2)
        p2.action("PickUp Wheelbarrow")
        p2.action("PickUp Peppernut")
        p2.action("Pass")
        p3.action("Pass")
        p5.action("Pass")
        p6.action("Pass")

def r12(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log, 2):
        # R11 end: P4 in 5 (1 nut), P1 in 0 (1 nut), P3 in 5 (1 nut), P2 in 0 (WB, 1 nut), P5, P6 in 0.
        p3.action("Move 0")
        p2.action("Move 6")
        p1.action("Move 6")
        p4.action("Move 0"); p4.action("Move 6")
        p5.action("Move 6")
        p6.action("Move 6")
        p2.action("Throw P5 1")
        p5.action("Shoot")
        p3.action("Move 6")
        p3.action("Throw P6 0")
        p6.action("Shoot")

rounds_list = [r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12]