class Player:
    def __init__(self, name, ap=2):
        self.name = name
        self.max_ap = ap
        self.ap = ap
    
    def reset_ap(self, ap=2):
        self.ap = ap
        self.max_ap = ap

    def action(self, cmd, cost=1):
        # Handle "Free" moves logic or manual cost override
        if self.ap < cost:
            # We print a warning but allow it, so we can debug with verify tool
            pass
        self.ap -= cost
        print(f"{self.name}: {cmd}")

    def free_action(self, cmd):
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
    """
    Round 1:
    Event: Turbo Mode (Timebomb 3). Boom: Fire in Engine/Hallway.
    Enemy: The Petty Thief (5 HP). Targets Engine (5).
    Strategy: 
    - Position players for future rounds.
    - P4 stays in Hallway (7) to avoid initial fire in Engine (5).
    - P3 to Bridge (9) for Evasive Maneuvers.
    - P1 to Kitchen (6) to Bake later.
    - P5/P6 to Cannons (8).
    - P2 to Cargo (4).
    """
    print("# Round 1: TurboMode, Enemy->5")
    start_round()
    
    p1.action("Move 7")
    p1.action("Move 6")
    
    p5.action("Move 7")
    p5.action("Move 8")
    
    p6.action("Move 7")
    p6.action("Move 8")
    
    p2.action("Move 7")
    p2.action("Move 4")
    
    p3.action("Move 7")
    p3.action("Move 9")
    
    p4.action("Move 7")
    
    for p in players: p.pass_turn()

def r2():
    """
    Round 2:
    Event: Sugar Rush (Situation). Moves Free. Cannons Prohibited.
    Active: Turbo Mode.
    Enemy: Targets Hallway (7).
    Hazards: Fire in Engine (5).
    Strategy:
    - P4 (in 7) moves to Engine (5) (Free), Extinguishes Fire (1 AP), and Solves Turbo Mode (1 AP).
    - P3 (in 9) uses Evasive Maneuvers (2 AP) to block attack on Hallway.
    - P1 (in 6) Bakes and Picks Up ammo (Stockpile).
    - P2 moves to Cannons (8) to prepare for Relay/Shooting.
    - Sugar Rush prevents shooting, so no attacks.
    """
    print("# Round 2: SugarRush. Enemy->7. Fire in 5.")
    start_round()
    
    p4.action("Move 5", 0)
    p4.action("Extinguish", 1)
    p4.action("Interact", 1)
    
    p3.action("EvasiveManeuvers", 2)
    
    p1.action("Bake", 1)
    p1.action("PickUp", 1)
    
    p2.action("Move 7", 0)
    p2.action("Move 8", 0)
    
    for p in players: p.pass_turn()

def r3():
    """
    Round 3:
    Event: Overheating (Situation). End turn in Engine -> Lose 1 AP.
    Active: Sugar Rush (Moves Free).
    Enemy: Targets Engine (5).
    Strategy:
    - P4 (in 5) has 1 AP (Overheating penalty). Moves to 7 (Free) to escape Engine.
    - P1 (in 6) Solves Sugar Rush (Interact). This removes "Moves Free" and "No Shoot".
    - Because P1 solves Sugar Rush mid-round, ordering is critical.
    - P2 (in 8) moves to 7 (Free, before Solve) to receive ammo.
    - P1 Throws ammo to P2.
    - P2 Throws ammo to P5 (in 8).
    - P5 Shoots (Allowed after Solve). Boss HP 5 -> 4.
    - P3 (in 9) Moves to 5 and back to 9 (Free, before Solve).
    - Note: P3 does NOT Shield/Evasive here, so Attack hits Engine (5). Fire spawns.
    """
    print("# Round 3: Overheating. Enemy->5.")
    start_round()
    p4.ap -= 1 # Overheating penalty
    
    p4.action("Move 7", 0) # Escape
    
    p3.action("Move 5", 0)
    p3.action("Move 9", 0)
    
    p2.action("Move 7", 0)
    
    p1.action("Interact", 1) # Solves Sugar Rush!
    p1.action("Throw P2 0", 1)
    
    p2.action("Throw P5 0", 1)
    
    p5.action("Shoot", 1)
    
    for p in players: p.pass_turn()

def r4():
    """
    Round 4:
    Event: Rudderless (Situation). Enemy Damage +1. Solve in Bridge (9).
    Active: Rudderless.
    Enemy: Targets Sickbay (10).
    Hazards: Fire in Engine (5).
    Strategy:
    - P4 (in 7) Moves to Bridge (9) and Solves Rudderless (Interact).
    - P3 (in 9) uses Evasive Maneuvers to block attack on Sickbay.
    - P2 Moves to Cargo (4) to prepare for next round (NoLight).
    - P1 Bakes more ammo.
    """
    print("# Round 4: Rudderless. Enemy->7.")
    start_round()
    
    p4.action("Move 9", 1)
    p4.action("Interact", 1) # Solve Rudderless
    
    p3.action("EvasiveManeuvers", 2)
    
    p2.action("Move 4", 1)
    p2.action("Move 7", 1)
    
    p1.action("Bake", 1)
    
    for p in players: p.pass_turn()

def r5():
    """
    Round 5:
    Event: No Light (Situation). Cannons Disabled. Solve in Cargo (4).
    Active: No Light.
    Enemy: Targets Dormitory (3).
    Hazards: Fire in Engine (5).
    Strategy:
    - P2 (in 7) Moves to Cargo (4) and Solves No Light (Interact).
    - P3 (in 9) uses Evasive Maneuvers to block attack.
    - P1 (in 6) Picks Up ammo, Moves to Hallway (7), Drops it for relay.
    - No Shooting this round.
    """
    print("# Round 5: NoLight. Enemy->6.")
    start_round()
    
    p2.action("Move 4", 1)
    p2.action("Interact", 1) # Solve NoLight
    
    p3.action("EvasiveManeuvers", 2)
    
    p1.action("PickUp", 1)
    p1.action("Move 7", 1)
    p1.action("Drop 0", 0)
    
    for p in players: p.pass_turn()

def r6():
    """
    Round 6:
    Event: Fog Bank (Situation). Telegraph Hidden.
    Active: Fog Bank.
    Enemy: Hidden Target (Actually Cannons 8).
    Strategy:
    - P1 (in 7) Picks Up dropped ammo, Throws to P5 (in 8).
    - P5 Shoots. Boss HP 4 -> 3.
    - P3 (in 9) uses Evasive Maneuvers. Blocks attack on 8.
    - P2 moves to 8 to help later.
    """
    print("# Round 6: FogBank. Enemy->9.")
    start_round()
    
    p3.action("EvasiveManeuvers", 2)
    
    p1.action("PickUp", 1)
    p1.action("Throw P5 0", 1)
    
    p5.action("Shoot", 1)
    
    p2.action("Move 8", 2) # 4->7->8 (Cost 2)
    
    for p in players: p.pass_turn()

def r7():
    """
    Round 7:
    Event: Panic! (Flash). Everyone moves to Dormitory (3).
    Active: Fog Bank.
    Enemy: Hidden Target (Actually Bridge 9).
    Strategy:
    - Everyone is displaced to Room 3.
    - Massive AP spend to return to stations.
    - P3 tries to reach Bridge (9) but cannot Evasive (AP exhausted).
    - Attack hits Bridge (9). Fire spawns in 9. Hull -1.
    """
    print("# Round 7: Panic! Enemy->8.")
    start_round()
    
    p1.action("Move 7", 1)
    p1.action("Move 6", 1)
    
    p5.action("Move 7", 1)
    p5.action("Move 8", 1)
    
    p6.action("Move 7", 1)
    p6.action("Move 8", 1)
    
    p3.action("Move 7", 1)
    p3.action("Move 9", 1)
    
    p2.action("Move 7", 1)
    
    p4.action("Move 7", 1)
    p4.action("Move 5", 1)
    
    for p in players: p.pass_turn()

def r8():
    """
    Round 8:
    Event: Listing Ship (Situation). Work costs 2x AP. Walk is Free.
    Active: Fog Bank, Listing Ship.
    Enemy: Hidden Target (Actually Cannons 8).
    Hazards: Fire in Engine (5), Bridge (9).
    Strategy:
    - P4 (in 5) Solves Listing Ship (Interact). Removes 2x Cost penalty immediately.
    - P3 (in 9) Extinguishes Fire in Bridge (1 AP).
    - P5, P6 (in 8) Pick Up ammo (from R5 Rain).
    - P5, P6 Shoot. Boss HP 3 -> 1.
    - P1 (in 6) Picks Up ammo.
    - Attack hits Cannons (8). Fire spawns in 8. Hull -1.
    """
    print("# Round 8: ListingShip. Enemy->5.")
    start_round()
    
    p4.action("Interact", 2) # Solve Listing.
    
    p3.action("Extinguish", 1)
    
    p5.action("PickUp", 1)
    p6.action("PickUp", 1)
    
    p5.action("Shoot", 1)
    p6.action("Shoot", 1)
    
    p1.action("Move 6", 0) # Same room
    p1.action("PickUp", 1)
    
    for p in players: p.pass_turn()

def r9():
    """
    Round 9:
    Event: Weird Gifts (Timebomb).
    Active: Fog Bank, Weird Gifts.
    Enemy: Hidden Target (Actually Engine 5).
    Hazards: Fire in Engine (5), Cannons (8).
    Strategy:
    - Room 8 has Fire. P2 (in 7) moves to 8 and Extinguishes.
    - Room 8 clear. P6 Shoots.
    - P5 Shoots.
    - Boss HP 1 -> -1. Boss Dead.
    - P3 (in 9) Picks Up ammo (Rain), moves to 7 to setup R10 (Victory Lap/Next Boss).
    """
    print("# Round 9: WeirdGifts. Enemy->6.")
    start_round()
    
    p2.action("Move 8", 1)
    p2.action("Extinguish", 1)
    
    # P5/P6 have 0 items? No, they used them in R8.
    # Wait, my logic for R9 was P6/P5 Shoot.
    # In my last successful verification, P6/P5 did NOT Shoot in R9?
    # Because they had no ammo?
    # R8 Output showed Shoot P5/P6.
    # So they used ammo in R8.
    # So in R9, they have 0 ammo.
    # So P5/P6 cannot shoot in R9!
    # I need to fix this.
    # P1 has ammo (PickUp in R8).
    # P1 (in 6). Move 7 (1). Throw P5 (1).
    # P5 (in 8). Shoot (1).
    # This works.
    
    p1.action("Move 7", 1)
    p1.action("Throw P5 0", 1)
    
    p5.action("Shoot", 1)
    
    p3.action("EvasiveManeuvers", 2)
    
    for p in players: p.pass_turn()

def r10():
    print("# Round 10: Victory/Transition.")
    start_round()
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