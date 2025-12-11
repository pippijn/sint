use sint_core::{
    logic::GameLogic,
    types::{Action, GameAction, GamePhase},
};

#[test]
fn test_enemy_targeting_logic_consistency() {
    // Try multiple seeds to hit different dice rolls (2-12)
    let seeds = [1, 123, 999, 10101, 55555];

    for seed in seeds {
        let mut state = GameLogic::new_game(vec!["P1".to_string()], seed);
        
        // 1. Advance Lobby -> MorningReport
        state = GameLogic::apply_action(
            state,
            "P1",
            Action::Game(GameAction::VoteReady { ready: true }),
            None,
        ).unwrap();
        assert_eq!(state.phase, GamePhase::MorningReport);

        // 2. Advance MorningReport -> EnemyTelegraph (Triggering the attack generation logic)
        state = GameLogic::apply_action(
            state,
            "P1",
            Action::Game(GameAction::VoteReady { ready: true }),
            None,
        ).unwrap();
        assert_eq!(state.phase, GamePhase::EnemyTelegraph);

        // 3. Inspect the generated attack
        let attack = state.enemy.next_attack.as_ref().expect("Enemy should have an attack planned");
        
        println!("Seed {}: Attack targeted Room ID {}", seed, attack.target_room);

        // Assertion 1: The target room MUST exist in the map.
        assert!(
            state.map.rooms.contains_key(&attack.target_room),
            "Target Room ID {} does not exist in the map!", 
            attack.target_room
        );

        // Assertion 2: Verify consistency between Target System and Target Room.
        if let Some(sys) = attack.target_system {
             // Case A: A System was targeted.
             // 1. The target room must exist.
             let room = state.map.rooms.get(&attack.target_room)
                 .expect("Attack targets a non-existent room!");
            
             // 2. The target room must actually contain the targeted system.
             assert_eq!(
                room.system, 
                Some(sys),
                "Mismatch! Attack targets {:?} but resolved to Room {} which contains {:?}",
                sys, attack.target_room, room.system
            );
        } else {
             // Case B: No System targeted (Miss).
             // In v2 logic, dice rolls 11-12 result in target_system=None and effect=Miss.
             assert!(
                 matches!(attack.effect, sint_core::types::AttackEffect::Miss),
                 "If target_system is None, expected AttackEffect::Miss, but got {:?}", 
                 attack.effect
             );
             // Note: We do not enforce `target_room` for misses as it is a fallback/cosmetic value.
        }
    }
}