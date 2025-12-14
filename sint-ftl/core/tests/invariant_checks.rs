use proptest::prelude::*;
use sint_core::logic::resolution;
use sint_core::{GameLogic, types::*};

proptest! {
    /// Test that standing in a room with Fire deals 1 damage to players at the end of the round.
    #[test]
    fn test_fire_damages_players(seed in 0u64..u64::MAX) {
        let mut state = GameLogic::new_game(vec!["P1".to_owned()], seed);
        state.phase = GamePhase::EnemyAction;
        let room_id = state.players.get("P1").unwrap().room_id;
        state.map.rooms.get_mut(&room_id).unwrap().add_hazard(HazardType::Fire);
        let hp_before = state.players.get("P1").unwrap().hp;

        resolution::resolve_hazards(&mut state);

        let hp_after = state.players.get("P1").unwrap().hp;
        prop_assert_eq!(hp_after, hp_before - 1, "Player should take 1 damage from fire");
    }

    /// Test that Peppernuts in Storage are NOT destroyed by Water.
    #[test]
    fn test_storage_protects_nuts(seed in 0u64..u64::MAX) {
        let mut state = GameLogic::new_game(vec!["P1".to_owned()], seed);
        state.phase = GamePhase::EnemyAction;

        let storage_id = sint_core::logic::find_room_with_system_in_map(&state.map, SystemType::Storage).unwrap();
        let room = state.map.rooms.get_mut(&storage_id).unwrap();
        room.add_hazard(HazardType::Water);
        room.add_item(ItemType::Peppernut);

        resolution::resolve_hazards(&mut state);

        let has_nuts = state.map.rooms.get(&storage_id).unwrap().items.contains(&ItemType::Peppernut);
        prop_assert!(has_nuts, "Peppernuts in Storage should be protected from Water");
    }

    /// Test that Fainted players respawn in the Dormitory with full HP at the start of the next round.
    #[test]
    fn test_respawn_logic(seed in 0u64..u64::MAX) {
        let mut state = GameLogic::new_game(vec!["P1".to_owned(), "P2".to_owned()], seed);
        state.phase = GamePhase::EnemyAction;

        let dormitory_id = sint_core::logic::find_room_with_system_in_map(&state.map, SystemType::Dormitory).unwrap_or(0);

        // Faint P1, keep P2 alive
        if let Some(p1) = state.players.get_mut("P1") {
            p1.hp = 0;
            p1.status.push(PlayerStatus::Fainted);
            p1.room_id = 999; // Away from dormitory
        }
        if let Some(p2) = state.players.get_mut("P2") {
            p2.hp = 3;
            p2.is_ready = true;
        }

        // Transition from EnemyAction -> MorningReport
        let state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::VoteReady { ready: true }), None).unwrap();

        let p1 = state.players.get("P1").unwrap();
        prop_assert_eq!(p1.hp, 3, "Respaced player should have full HP");
        prop_assert_eq!(p1.room_id, dormitory_id, "Respaced player should be in Dormitory");
        prop_assert!(!p1.status.contains(&PlayerStatus::Fainted), "Respaced player should not be Fainted");
    }

    /// Test that First Aid restores 1 HP (up to 3).
    #[test]
    fn test_first_aid_healing(seed in 0u64..u64::MAX) {
        let mut state = GameLogic::new_game(vec!["P1".to_owned()], seed);
        state.phase = GamePhase::TacticalPlanning;

        let sickbay_id = sint_core::logic::find_room_with_system_in_map(&state.map, SystemType::Sickbay).unwrap();
        if let Some(p) = state.players.get_mut("P1") {
            p.room_id = sickbay_id;
            p.hp = 1;
            p.ap = 10; // Give plenty of AP
        }

        let state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::FirstAid { target_player: "P1".to_owned() }), None).unwrap();

        let mut state = state;
        resolution::resolve_proposal_queue(&mut state, false);

        let p = state.players.get("P1").unwrap();
        prop_assert_eq!(p.hp, 2, "First Aid should restore 1 HP");
    }

    /// Test that Extinguisher removes 2 Fire tokens.
    #[test]
    fn test_extinguisher_bonus(seed in 0u64..u64::MAX) {
        let mut state = GameLogic::new_game(vec!["P1".to_owned()], seed);
        state.phase = GamePhase::TacticalPlanning;

        let room_id = state.players.get("P1").unwrap().room_id;
        let room = state.map.rooms.get_mut(&room_id).unwrap();
        room.add_hazard(HazardType::Fire);
        room.add_hazard(HazardType::Fire);
        room.add_hazard(HazardType::Fire);

        if let Some(p) = state.players.get_mut("P1") {
            p.inventory.push(ItemType::Extinguisher);
        }

        let state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Extinguish), None).unwrap();
        let mut state = state;
        resolution::resolve_proposal_queue(&mut state, false);

        let fire_count = state.map.rooms.get(&room_id).unwrap().hazards.iter().filter(|&&h| h == HazardType::Fire).count();
        prop_assert_eq!(fire_count, 1, "Extinguisher should have removed 2 fire tokens");
    }

    /// Test that Static Noise (Situation Card) restricts chat to non-alphabetic characters.
    #[test]
    fn test_static_noise_chat_restriction(seed in 0u64..u64::MAX) {
        let mut state = GameLogic::new_game(vec!["P1".to_owned()], seed);
        state.phase = GamePhase::TacticalPlanning;

        // Add Static Noise card
        let static_noise = sint_core::logic::cards::registry::get_behavior(CardId::StaticNoise).get_struct();
        state.active_situations.push(static_noise);

        // Try to chat with letters
        let action_fail = Action::Game(GameAction::Chat { message: "Hello".to_owned() });
        let res = GameLogic::apply_action(state.clone(), "P1", action_fail, None);
        prop_assert!(res.is_err(), "Chat with letters should fail during Static Noise");

        // Try to chat with numbers/symbols
        let action_ok = Action::Game(GameAction::Chat { message: "123! 😊".to_owned() });
        let res = GameLogic::apply_action(state.clone(), "P1", action_ok, None);
        prop_assert!(res.is_ok(), "Chat with numbers/emojis should pass during Static Noise");
    }

    /// Test that Fire deals Hull damage only when a system explodes (system_health reaches 0).
    #[test]
    fn test_system_explosion_hull_damage(seed in 0u64..u64::MAX) {
        let mut state = GameLogic::new_game(vec!["P1".to_owned()], seed);
        state.phase = GamePhase::EnemyAction;
        state.hull_integrity = 20;

        // Find a room with a system and set its health to 1
        let room_id = state.map.rooms.values().find(|r| r.system.is_some()).unwrap().id;
        if let Some(r) = state.map.rooms.get_mut(&room_id) {
            r.system_health = 1;
            r.add_hazard(HazardType::Fire);
        }

        resolution::resolve_hazards(&mut state);

        prop_assert_eq!(state.hull_integrity, 19, "Ship should take 1 hull damage when a system explodes");
        prop_assert!(state.map.rooms.get(&room_id).unwrap().is_broken, "System should be marked as broken");
    }
}
