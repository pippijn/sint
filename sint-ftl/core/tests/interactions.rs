use sint_core::{
    logic::{apply_action, cards::get_behavior, GameLogic},
    types::{Action, Card, CardId, CardType, GameAction, GamePhase, ItemType},
};

fn get_player_ids(state: &sint_core::types::GameState) -> Vec<String> {
    state.players.keys().cloned().collect()
}

#[test]
fn test_multiple_situation_cards_stack_effects() {
    // 1. Setup: Create a game with two active situation cards:
    // - Flu Wave: Reduces AP to 1 for the round.
    // - Overheating: Player was in the engine room, so they lose 1 AP.
    let mut state = GameLogic::new_game(vec!["p1".to_owned()], 0);
    state.phase = GamePhase::TacticalPlanning;
    let flu_card = Card {
        id: CardId::FluWave,
        title: "Flu Wave".to_owned(),
        description: "".to_owned(),
        card_type: CardType::Timebomb { rounds_left: 0 }, // Make it trigger
        options: vec![],
        solution: None,
        affected_player: None,
    };
    let overheating_card = Card {
        id: CardId::Overheating,
        title: "Overheating".to_owned(),
        description: "".to_owned(),
        card_type: CardType::Situation,
        options: vec![],
        solution: None,
        affected_player: None,
    };
    state.active_situations.push(flu_card.clone());
    state.active_situations.push(overheating_card.clone());

    let player_id = get_player_ids(&state)[0].clone();

    // Manually place the player in the Engine room to trigger Overheating
    let engine_room_id = state
        .map
        .rooms
        .values()
        .find(|r| r.system == Some(sint_core::types::SystemType::Engine))
        .unwrap()
        .id;
    let p = state.players.get_mut(&player_id).unwrap();
    p.room_id = engine_room_id;
    p.ap = 2; // Start with normal AP

    // 2. Action: Manually trigger the `on_round_start` effects.
    get_behavior(flu_card.id).on_round_start(&mut state);
    get_behavior(overheating_card.id).on_round_start(&mut state);

    // 3. Assert: The player's AP should be reduced to 0.
    // FluWave sets AP to 1. Overheating then removes 1 AP.
    let player = state.players.get(&player_id).unwrap();
    assert_eq!(
        player.ap, 0,
        "Player should have 0 AP after FluWave and Overheating effects."
    );

    // 4. Action & Assert: Attempt to move, which should fail.
    let start_room = player.room_id;
    let target_room = state.map.rooms.get(&start_room).unwrap().neighbors[0];
    let move_action = Action::Game(GameAction::Move {
        to_room: target_room,
    });

    let result = apply_action(state.clone(), &player_id, move_action);
    assert!(result.is_err(), "Move action should fail with 0 AP.");
    let error = result.unwrap_err();
    assert!(
        matches!(error, sint_core::logic::GameError::NotEnoughAP),
        "Expected NotEnoughAP, got {:?}",
        error
    );
}

#[test]
fn test_hazard_blocks_card_effect() {
    // 1. Setup: A player has a Peppernut, and the Seagull Attack card is active.
    let mut state = GameLogic::new_game(vec!["p1".to_owned()], 0);
    state.phase = GamePhase::TacticalPlanning;
    let player_id = get_player_ids(&state)[0].clone();

    // Add Seagull Attack card
    state.active_situations.push(Card {
        id: CardId::SeagullAttack,
        title: "Seagull Attack".to_owned(),
        description: "".to_owned(),
        card_type: CardType::Situation,
        options: vec![],
        solution: None,
        affected_player: None,
    });

    // Give the player a Peppernut, which is the condition for the blockade.
    let p = state.players.get_mut(&player_id).unwrap();
    p.inventory.push(ItemType::Peppernut);

    let room_id = p.room_id;

    // 2. Action: Attempt to move.
    let target_room = state.map.rooms.get(&room_id).unwrap().neighbors[0];
    let move_action = Action::Game(GameAction::Move {
        to_room: target_room,
    });

    let result = apply_action(state.clone(), &player_id, move_action);

    // 3. Assert: The move action is blocked because the player has a Peppernut.
    assert!(
        result.is_err(),
        "Move should be blocked by Seagull Attack when holding a Peppernut."
    );
    let error = result.unwrap_err();
    assert!(
        matches!(error, sint_core::logic::GameError::InvalidAction(_)),
        "Expected InvalidAction due to blockade, got {:?}",
        error
    );
}
