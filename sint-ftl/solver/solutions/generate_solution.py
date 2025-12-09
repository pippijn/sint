
class Player:
    def __init__(self, name, ap=2):
        self.name = name
        self.max_ap = ap
        self.ap = ap
    
    def reset_ap(self, ap=2):
        self.ap = ap
        self.max_ap = ap

    def action(self, cmd, cost=1):
        if self.ap < cost:
            pass
        self.ap -= cost
        print(f"{self.name}: {cmd}")

    def pass_turn(self):
        if self.ap > 0:
            print(f"{self.name}: Pass")
        self.ap = 0

# Initialize Players
p1 = Player("P1")
p2 = Player("P2")
p3 = Player("P3")
p4 = Player("P4")
p5 = Player("P5")
p6 = Player("P6")
players = [p1, p2, p3, p4, p5, p6]

def start_round(ap_override=None):
    for p in players:
        p.reset_ap(2 if ap_override is None else ap_override)

def r1():
    print("# Round 1: TurboMode, Enemy->5")
    start_round()
    p1.action("Move 7"); p1.action("Move 6")
    p5.action("Move 7"); p5.action("Move 8")
    p6.action("Move 7"); p6.action("Move 8")
    p2.action("Move 7"); p2.action("Move 4")
    p3.action("Move 7"); p3.action("Move 9")
    p4.action("Move 7")
    for p in players: p.pass_turn()

def r2():
    print("# Round 2: SugarRush. Enemy->7. Fire in 5.")
    start_round()
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
    print("# Round 3: Overheating. Enemy->5.")
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
    print("# Round 4: Rudderless. Enemy->7.")
    start_round()
    p4.action("Move 9", 1)
    p4.action("Interact", 1)
    p3.action("EvasiveManeuvers", 2)
    p2.action("Move 4", 1)
    p1.action("Bake", 1)
    for p in players: p.pass_turn()

def r5():
    print("# Round 5: NoLight. Enemy->6.")
    start_round()
    p2.action("Interact", 1)
    p3.action("EvasiveManeuvers", 2)
    p1.action("PickUp", 1)
    p1.action("Move 7", 1)
    p1.action("Drop 0", 0)
    for p in players: p.pass_turn()

def r6():
    print("# Round 6: FogBank. Enemy->9.")
    start_round()
    p3.action("EvasiveManeuvers", 2)
    p1.action("PickUp", 1)
    p1.action("Throw P5 0", 1)
    p5.action("Shoot", 1)
    p2.action("Move 8", 2)
    for p in players: p.pass_turn()

def r7():
    print("# Round 7: Panic! Enemy->8.")
    start_round()
    p1.action("Move 7", 1); p1.action("Move 6", 1)
    p5.action("Move 7", 1); p5.action("Move 8", 1)
    p6.action("Move 7", 1); p6.action("Move 8", 1)
    p3.action("Move 7", 1); p3.action("Move 9", 1)
    p2.action("Move 7", 1)
    p4.action("Move 7", 1); p4.action("Move 5", 1)
    for p in players: p.pass_turn()

def r8():
    print("# Round 8: ListingShip. Enemy->8. Fire in 9.")
    start_round()
    p4.action("Interact", 2) # Solve Listing. Costs Normal immediately after.
    
    # Optimization: P2 (in 7) -> 9 (Free/Move 7->9). Extinguish (1 AP).
    # Wait, Listing Ship is solved by P4 FIRST.
    # So costs are NORMAL (1 AP). Moves are NOT Free.
    # So P2 (in 7) -> 9 (1 AP). Extinguish (1 AP). Total 2 AP.
    # P2 can do it!
    
    p2.action("Move 9", 1)
    p2.action("Extinguish", 1) # Fire 9 Cleared.
    
    # P3 (in 9) is Free to Evasive!
    p3.action("EvasiveManeuvers", 2) # Block Attack on 8!
    
    # P5, P6 PickUp (Ammo from R5 Rain in 8).
    p5.action("PickUp", 1)
    p6.action("PickUp", 1)
    
    # P5 Shoot.
    p5.action("Shoot", 1)
    p6.action("Shoot", 1)
    
    p1.action("Move 6", 0)
    p1.action("PickUp", 1)
    
    for p in players: p.pass_turn()

def r9():
    print("# Round 9: WeirdGifts. Enemy->6.")
    start_round()
    
    # Room 8 is Clear! No need to Extinguish.
    # P5 Shoot.
    p5.action("Shoot", 1)
    # Boss Dead.
    
    # P1 Relay to P6.
    p1.action("Move 7", 1)
    p1.action("Throw P6 0", 1)
    p6.action("Shoot", 1)
    
    # P3 Evasive (Block Attack on 6).
    p3.action("EvasiveManeuvers", 2)
    
    # P2 (in 9) -> 7 -> 4.
    p2.action("Move 7", 1)
    p2.action("Move 4", 1)
    
    # P4 (in 5) -> 4? Prep for WeirdGifts?
    # WeirdGifts solve in 4. P2 needs help?
    # P2 in 4. Interact (1 AP) in R10.
    # We solve it early.
    
    for p in players: p.pass_turn()

def r10():
    print("# Round 10: Kill Boss / Defense.")
    start_round()
    
    # Enemy targets 2 (The Monster).
    # P3 (in 9). Evasive.
    p3.action("EvasiveManeuvers", 2)
    
    # Short Circuit Fire in 5.
    # P4 (in 5). Extinguish.
    p4.action("Extinguish", 1)
    p4.action("Move 7", 1) # Escape Engine
    
    # P2 (in 4). Interact (Solve WeirdGifts).
    p2.action("Interact", 1)
    
    # P1 Stockpile.
    p1.action("Move 6", 1)
    p1.action("Bake", 1)
    
    for p in players: p.pass_turn()

def main():
    print("SEED 12345")
    print("PLAYERS 6")
    print("")
    r1()
    r2()
    r3()
    r4()
    r5()
    r6()
    r7()
    r8()
    r9()
    r10()

if __name__ == "__main__":
    main()
