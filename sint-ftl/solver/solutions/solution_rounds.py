from execute_solution import RoundScope

def r1(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("Move 0"); p1.action("Move 5"); p1.action("Pass")
        p5.action("Move 0"); p5.action("Move 6"); p5.action("Pass")
        p6.action("Move 0"); p6.action("Move 6"); p6.action("Pass")
        p2.action("Move 0"); p2.action("Move 3"); p2.action("Pass")
        p3.action("Move 0"); p3.action("Move 7"); p3.action("Pass")
        p4.action("Move 0"); p4.action("Move 4"); p4.action("PickUp Extinguisher")

def r2(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p4.action("Move 0"); p4.action("Move 6"); p4.action("Extinguish"); p4.action("Pass")
        p1.action("Bake"); p1.action("PickUp Peppernut"); p1.action("Move 0"); p1.action("Pass")
        p2.action("PickUp Wheelbarrow"); p2.action("Move 0"); p2.action("Move 9"); p2.action("Pass")
        p3.action("RaiseShields"); p3.action("Pass")
        p5.action("Move 0"); p5.action("Pass")
        p6.action("Move 0"); p6.action("Pass")

def r3(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("Move 5"); p1.action("Interact")
        p1.action("Throw P5 0"); p1.action("Pass")
        p5.action("Move 6"); p5.action("Shoot"); p5.action("Pass")
        p2.action("PickUp Peppernut"); p2.action("PickUp Peppernut"); p2.action("Move 0")
        p3.action("RaiseShields"); p3.action("Pass")
        p4.action("Move 0"); p4.action("Pass")
        p6.action("Move 6"); p6.action("Pass")

def r4(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p4.action("Extinguish"); p4.action("Move 4")
        p2.action("Move 6"); p2.action("Throw P5 1")
        p5.action("Shoot"); p5.action("Pass")
        p3.action("RaiseShields")
        p1.action("Bake"); p1.action("PickUp Peppernut")
        p6.action("Pass")

def r5(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p4.action("Extinguish")
        p2.action("Throw P5 1")
        p1.action("Move 0"); p1.action("Throw P2 0")
        p2.action("Throw P6 1")
        p5.action("Shoot"); p5.action("Pass")
        p6.action("Shoot"); p6.action("Pass")
        p3.action("RaiseShields")

def r6(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p4.action("Interact")
        p1.action("Move 9"); p1.action("Interact")
        p2.action("Move 0"); p2.action("Move 9")
        p3.action("RaiseShields")
        p5.action("Pass")
        p6.action("Pass")

def r7(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p2.action("PickUp Peppernut")
        p2.action("Move 0")
        p1.action("Move 0"); p1.action("Move 6")
        p4.action("Move 0"); p4.action("Pass")
        p3.action("RaiseShields")
        p5.action("Pass")
        p6.action("Pass")

def r8(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p2.action("Move 6")
        p2.action("Throw P5 0")
        p4.action("Move 6"); p4.action("Pass")
        p5.action("Shoot"); p5.action("Pass")
        p3.action("RaiseShields")
        p1.action("Pass")
        p6.action("Pass")

def r9(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p3.action("Move 0")
        p3.action("Drop 0")
        p3.action("Move 7")
        p4.action("Move 0"); p4.action("Pass")
        p1.action("Move 0"); p1.action("Pass")
        p2.action("Move 0"); p2.action("Pass")
        p5.action("Pass")
        p6.action("Pass")
        
def r10(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("Move 5")
        p2.action("Move 5")
        p3.action("Move 0"); p3.action("Pass")
        p4.action("Move 6"); p4.action("Pass")
        p5.action("Move 0"); p5.action("Pass")
        # p6 stays in 6
        p6.action("Pass")

def r11(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("PickUp Peppernut")
        p1.action("Throw P3 0")
        p2.action("PickUp Peppernut")
        p2.action("Throw P5 0")
        p3.action("Throw P4 0"); p3.action("Pass")
        p5.action("Throw P6 1"); p5.action("Pass")
        p4.action("Shoot"); p4.action("Pass")
        p6.action("Shoot"); p6.action("Pass")

def r12(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("Move 0"); p1.action("Pass")
        p2.action("Move 0"); p2.action("Pass")
        p3.action("Move 0"); p3.action("Pass")
        p4.action("Move 0"); p4.action("Pass")
        p5.action("Move 0"); p5.action("Pass")
        p6.action("Move 0"); p6.action("Pass")

def r13(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("Move 5")
        p2.action("Move 6")
        p2.action("Pass")
        p3.action("Pass")
        p5.action("Pass")
        p6.action("Pass")

def r14(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("Move 5")
        p2.action("Pass")
        p3.action("Pass")
        p5.action("Pass")
        p6.action("Pass")

def r15(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("PickUp Peppernut")
        p1.action("Pass")
        p6.action("Pass")
        p2.action("Pass")
        p3.action("Pass")
        p5.action("Pass")

def r16(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p1.action("Throw P6 0")
        p1.action("Pass")
        p6.action("Pass")
        p2.action("Pass")
        p3.action("Pass")
        p5.action("Pass")

def r17(players, rounds_log):
    p1, p2, p3, p4, p5, p6 = players
    with RoundScope(players, rounds_log):
        p6.action("Shoot")
        p6.action("Pass")
        p1.action("Pass")
        p2.action("Pass")
        p3.action("Pass")
        p5.action("Pass")

rounds_list = [r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, r13, r14, r15, r16, r17]
