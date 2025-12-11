use sint_core::{
    logic::{actions::get_valid_actions, find_room_with_system_in_map, GameLogic},
    types::{
        Card, CardId, CardSolution, CardType, GameAction, GamePhase, HazardType, ItemType,
        PlayerStatus, SystemType,
    },
};

fn has_action(actions: &[sint_core::types::Action], target: &GameAction) -> bool {
    actions.iter().any(|a| {
        if let sint_core::types::Action::Game(ga) = a {
            // Partial matching is hard with enums that have data.
            // We'll exact match for simple ones, and discriminant match for complex?
            // Actually, let's just exact match since we construct the expected one fully.
            ga == target
        } else {
            false
        }
    })
}

#[test]
fn test_fire_blocks_system_action() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    // 1. Setup: P1 in Kitchen with Fire
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = kitchen;
        p.ap = 2;
    }
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.hazards.push(HazardType::Fire);
    }

    // 2. Get Actions
    let actions = get_valid_actions(&state, "P1");

    // 3. Assertions
    assert!(
        has_action(&actions, &GameAction::Extinguish),
        "Should show Extinguish"
    );
    assert!(
        !has_action(&actions, &GameAction::Bake),
        "Should NOT show Bake (Blocked by Fire)"
    );
    assert!(
        has_action(&actions, &GameAction::Move { to_room: 0 }),
        "Should show Move (to Hub)"
    );
}

#[test]
fn test_cannons_ammo_requirement() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let cannons = find_room_with_system_in_map(&state.map, SystemType::Cannons).unwrap();

    // Setup: P1 in Cannons
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = cannons;
        p.ap = 2;
        p.inventory.clear();
    }

    // Case A: No Ammo
    let actions_no_ammo = get_valid_actions(&state, "P1");
    assert!(
        !has_action(&actions_no_ammo, &GameAction::Shoot),
        "Should NOT show Shoot without ammo"
    );

    // Case B: With Ammo
    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Peppernut);
    }
    let actions_ammo = get_valid_actions(&state, "P1");
    assert!(
        has_action(&actions_ammo, &GameAction::Shoot),
        "Should show Shoot with ammo"
    );
}

#[test]
fn test_first_aid_adjacency() {
    let mut state = GameLogic::new_game(vec!["P1".to_string(), "P2".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let sickbay = find_room_with_system_in_map(&state.map, SystemType::Sickbay).unwrap();
    let neighbor = 0; // Hallway (Hub)
    let far_room = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sickbay;
        p.ap = 2;
    }

    // Case A: Neighbor
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = neighbor;
    }
    let actions_adj = get_valid_actions(&state, "P1");
    assert!(
        has_action(
            &actions_adj,
            &GameAction::FirstAid {
                target_player: "P2".to_string()
            }
        ),
        "Should show FirstAid for neighbor"
    );

    // Case B: Far away
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = far_room;
    }
    let actions_far = get_valid_actions(&state, "P1");
    assert!(
        !has_action(
            &actions_far,
            &GameAction::FirstAid {
                target_player: "P2".to_string()
            }
        ),
        "Should NOT show FirstAid for far player"
    );
}

#[test]
fn test_revive_same_room() {
    let mut state = GameLogic::new_game(vec!["P1".to_string(), "P2".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = kitchen;
        p.ap = 2;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = kitchen;
        p.status.push(PlayerStatus::Fainted);
    }

    let actions = get_valid_actions(&state, "P1");
    assert!(
        has_action(
            &actions,
            &GameAction::Revive {
                target_player: "P2".to_string()
            }
        ),
        "Should show Revive for fainted player in same room"
    );
}

#[test]
fn test_interact_situation_context() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();
    let hallway = 0;

    // Add Situation: Solvable in Kitchen
    let card = Card {
        id: CardId::StickyFloor,
        title: "Sticky Floor".to_string(),
        description: "Test".to_string(),
        card_type: CardType::Situation,
        options: vec![],
        solution: Some(CardSolution {
            target_system: Some(SystemType::Kitchen),
            ap_cost: 1,
            item_cost: None,
            required_players: 1,
        }),
    };
    state.active_situations.push(card);

    // Case A: P1 in Kitchen
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = kitchen;
        p.ap = 2;
    }
    let actions_kitchen = get_valid_actions(&state, "P1");
    assert!(
        has_action(&actions_kitchen, &GameAction::Interact),
        "Should show Interact in Kitchen"
    );

    // Case B: P1 in Hallway
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = hallway;
    }
    let actions_hallway = get_valid_actions(&state, "P1");
    assert!(
        !has_action(&actions_hallway, &GameAction::Interact),
        "Should NOT show Interact in Hallway"
    );
}
