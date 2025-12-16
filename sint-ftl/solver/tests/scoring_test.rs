use sint_core::logic::GameLogic;
use sint_core::logic::pathfinding::MapDistances;
use sint_solver::scoring::beam::{BeamScoringWeights, score_static};

#[test]
fn test_scoring_consistency() {
    let player_ids = vec![
        "P1".to_string(),
        "P2".to_string(),
        "P3".to_string(),
        "P4".to_string(),
        "P5".to_string(),
        "P6".to_string(),
    ];
    let state = GameLogic::new_game(player_ids, 42);
    let weights = BeamScoringWeights::default();
    let distances = MapDistances::new(&state.map);
    let history = vec![];

    let score1 = score_static(&state, &history, &weights, &distances);
    let score2 = score_static(&state, &history, &weights, &distances);

    assert_eq!(score1, score2, "Scoring must be deterministic");
}

#[test]
fn test_scoring_components_not_nan() {
    let player_ids = vec![
        "P1".to_string(),
        "P2".to_string(),
        "P3".to_string(),
        "P4".to_string(),
        "P5".to_string(),
        "P6".to_string(),
    ];
    let state = GameLogic::new_game(player_ids, 42);
    let weights = BeamScoringWeights::default();
    let distances = MapDistances::new(&state.map);
    let history = vec![];

    let score = score_static(&state, &history, &weights, &distances);

    assert!(!score.total.is_nan());
    assert!(!score.vitals.is_nan());
    assert!(!score.hazards.is_nan());
    assert!(!score.offense.is_nan());
    assert!(!score.panic.is_nan());
    assert!(!score.logistics.is_nan());
    assert!(!score.situations.is_nan());
    assert!(!score.threats.is_nan());
    assert!(!score.progression.is_nan());
    assert!(!score.anti_oscillation.is_nan());
}

#[test]
fn test_scoring_fire_penalty() {
    let player_ids = vec![
        "P1".to_string(),
        "P2".to_string(),
        "P3".to_string(),
        "P4".to_string(),
        "P5".to_string(),
        "P6".to_string(),
    ];
    let state_clean = GameLogic::new_game(player_ids.clone(), 42);
    let mut state_fire = state_clean.clone();

    // Add fire to the first room
    let first_room_id = state_fire.map.rooms.keys().next().unwrap();
    state_fire
        .map
        .rooms
        .get_mut(&first_room_id)
        .unwrap()
        .hazards
        .push(sint_core::types::HazardType::Fire);

    let weights = BeamScoringWeights::default();
    let distances = MapDistances::new(&state_clean.map);
    let history = vec![];

    let score_clean = score_static(&state_clean, &history, &weights, &distances);
    let score_fire = score_static(&state_fire, &history, &weights, &distances);

    assert!(
        score_fire.total < score_clean.total,
        "Ship with fire must have lower score than clean ship. Clean: {}, Fire: {}",
        score_clean.total,
        score_fire.total
    );
    assert!(
        score_fire.hazards < score_clean.hazards,
        "Hazards component must reflect fire"
    );
}

#[test]
fn test_scoring_hull_loss_penalty() {
    let player_ids = vec![
        "P1".to_string(),
        "P2".to_string(),
        "P3".to_string(),
        "P4".to_string(),
        "P5".to_string(),
        "P6".to_string(),
    ];
    let state_full = GameLogic::new_game(player_ids.clone(), 42);
    let mut state_damaged = state_full.clone();
    state_damaged.hull_integrity = 5; // Significant damage to trigger critical state

    let weights = BeamScoringWeights::default();
    let distances = MapDistances::new(&state_full.map);
    let history = vec![];

    let score_full = score_static(&state_full, &history, &weights, &distances);
    let score_damaged = score_static(&state_damaged, &history, &weights, &distances);

    assert!(
        score_damaged.total < score_full.total,
        "Damaged ship (hull 5) must have lower score than full ship (hull 20). Full: {}, Damaged: {}",
        score_full.total,
        score_damaged.total
    );
}
