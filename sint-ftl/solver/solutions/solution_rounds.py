rounds = (
    ( # Round 1
        ("P1", "Move 0"), ("P1", "Move 5"), ("P1", "Bake"),
        ("P2", "Move 0"), ("P2", "Move 3"), ("P2", "PickUp Wheelbarrow"),
        ("P3", "Move 0"), ("P3", "Move 4"), ("P3", "Pass"),
        ("P4", "Move 0"), ("P4", "Move 4"), ("P4", "PickUp Extinguisher"),
        ("P5", "Move 0"), ("P5", "Move 9"), ("P5", "PickUp Peppernut"),
        ("P6", "Move 0"), ("P6", "Move 9"), ("P6", "PickUp Peppernut"),
    ),
    ( # Round 2
        ("P3", "EvasiveManeuvers"), ("P3", "Pass"),
        ("P1", "Bake"), ("P1", "PickUp Peppernut"), ("P1", "Pass"),
        ("P2", "Move 0"), ("P2", "Move 9"), ("P2", "PickUp Peppernut"), ("P2", "PickUp Peppernut"), ("P2", "PickUp Peppernut"),
        ("P4", "Move 0"), ("P4", "Move 6"), ("P4", "Extinguish"), ("P4", "Pass"),
        ("P5", "Move 0"), ("P5", "Move 6"), ("P5", "Pass"),
        ("P6", "Move 0"), ("P6", "Move 6"), ("P6", "Pass"),
    ),
    ( # Round 3
        ("P3", "EvasiveManeuvers"),
        ("P1", "Bake"), ("P1", "Move 0"), ("P1", "Move 6"), ("P1", "Pass"),
        ("P2", "Move 0"), ("P2", "Move 6"), ("P2", "Drop 1"), ("P2", "Drop 1"), ("P2", "Drop 1"), ("P2", "Pass"),
        ("P4", "PickUp Peppernut"), ("P4", "Pass"),
        ("P5", "Pass"),
        ("P6", "Pass"),
    ),
    ( # Round 4
        ("P3", "Pass"), ("P2", "Move 0"), ("P2", "Pass"), ("P1", "Pass"), ("P4", "Pass"), ("P5", "Pass"), ("P6", "Pass"),
    ),
    ( # Round 5
        ("P1", "Shoot"), ("P1", "PickUp Peppernut"),
        ("P4", "Shoot"), ("P4", "PickUp Peppernut"),
        ("P5", "Shoot"), ("P5", "Pass"),
        ("P6", "Shoot"), ("P6", "Pass"),
        ("P2", "Move 6"), ("P2", "Pass"),
        ("P3", "Pass"),
    ),
    ( # Round 6
        ("P1", "Shoot"), ("P1", "Pass"),
        ("P4", "Shoot"), ("P4", "Pass"),
        ("P2", "Pass"), ("P3", "Pass"), ("P5", "Pass"), ("P6", "Pass"),
    ),
    ( # Round 7 (Rest Round - 6 AP)
        ("P1", "Move 5"), ("P1", "Bake"), ("P1", "Bake"), ("P1", "Bake"), ("P1", "PickUp Peppernut"), ("P1", "PickUp Peppernut"),
        ("P2", "Move 5"), ("P2", "PickUp Peppernut"), ("P2", "PickUp Peppernut"), ("P2", "Move 0"), ("P2", "Move 6"), ("P2", "Drop 0"),
        ("P3", "Move 5"), ("P3", "PickUp Peppernut"), ("P3", "PickUp Peppernut"), ("P3", "Move 0"), ("P3", "Move 6"), ("P3", "Drop 0"),
        ("P4", "Move 0"), ("P4", "Move 4"), ("P4", "Extinguish"), ("P4", "Extinguish"), ("P4", "Move 0"), ("P4", "Pass"),
        ("P5", "Move 0"), ("P5", "Extinguish"), ("P5", "Move 9"), ("P5", "Extinguish"), ("P5", "Move 0"), ("P5", "Pass"),
        ("P6", "Move 0"), ("P6", "Move 9"), ("P6", "Extinguish"), ("P6", "PickUp Peppernut"), ("P6", "PickUp Peppernut"), ("P6", "Move 0"),
    ),
    ( # Round 8
        ("P1", "Move 0"), ("P1", "Move 7"), ("P1", "Pass"),
        ("P2", "Move 0"), ("P2", "Pass"),
        ("P3", "Move 0"), ("P3", "Move 6"), ("P3", "Pass"),
        ("P4", "Move 0"),
        ("P5", "Move 0"), ("P5", "Move 6"), ("P5", "Pass"),
        ("P6", "Pass"),
    ),
    ( # Round 9
        ("P4", "Move 7"), ("P4", "Extinguish"), # Clear Bridge (7). 1+1=2 AP.
        ("P1", "RaiseShields"), # Possible now. 2 AP.
        ("P2", "Move 5"), # Move costs 2 due to sticky floor. 2 AP.
        ("P3", "Pass"),
        ("P5", "Pass"),
        ("P6", "Move 0"), ("P6", "Pass"), # Move costs 2 due to sticky floor. 2 AP.
    ),
    ( # Round 10
        ("P1", "RaiseShields"),
        ("P2", "PickUp Peppernut"), ("P2", "PickUp Peppernut"),
        ("P6", "Pass"),
        ("P3", "Pass"), ("P4", "Pass"), ("P5", "Pass"),
    ),
    ( # Round 11
        ("P1", "Pass"),
        ("P2", "Bake"), ("P2", "Move 0"),
        ("P3", "Pass"), ("P4", "Pass"), ("P5", "Pass"), ("P6", "Pass"),
    ),
    ( # Round 12
        ("P1", "Move 0"), ("P1", "Move 2"),
        ("P2", "Move 6"), ("P2", "Drop 0"), ("P2", "Drop 0"), ("P2", "Move 0"),
        ("P3", "PickUp Peppernut"), ("P3", "PickUp Peppernut"),
        ("P4", "Move 0"), ("P4", "Move 2"),
        ("P5", "Move 0"), ("P5", "Move 7"), ("P5", "Extinguish"),
        ("P6", "Pass"),
    ),
    ( # Round 13
        ("P1", "Pass"),
        ("P2", "Bake"),
        ("P3", "Pass"), ("P4", "Pass"), ("P5", "Pass"), ("P6", "Pass"),
    ),
    ( # Round 14
        ("P1", "Pass"),
        ("P2", "Pass"),
        ("P3", "Pass"), ("P4", "Pass"), ("P5", "Pass"), ("P6", "Pass"),
    ),
    ( # Round 15
        ("P1", "Pass"),
        ("P2", "Pass"),
        ("P3", "Pass"), ("P4", "Pass"), ("P5", "Pass"), ("P6", "Pass"),
    ),
)
