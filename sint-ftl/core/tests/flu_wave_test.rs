use sint_core::{Action, CardId, CardType, GameLogic, GamePhase};

#[test]
fn test_flu_wave_effect_application() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);

    // 1. Inject Flu Wave with 1 round left
    let mut card = state
        .deck
        .iter()
        .find(|c| c.id == CardId::FluWave)
        .cloned()
        .unwrap();
    state.deck.clear(); // Isolate test from random draws

    card.card_type = CardType::Timebomb { rounds_left: 1 };
    state.active_situations.push(card);

    // 2. Advance Phase: Planning -> Execution -> EnemyAction -> MorningReport
    state.phase = GamePhase::Execution;
    // Set P1 AP to 0 so we advance
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
        p.is_ready = true;
    }

    // Execution -> EnemyAction
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::EnemyAction);

    // EnemyAction -> MorningReport (This triggers `on_round_end` then `on_round_start`)
    // `on_round_end` (in EnemyAction) ticks timer to 0.
    // `advance_phase` resets AP to 2.
    // `on_round_start` (in MorningReport) sees timer 0, sets AP to 1, removes card.
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::MorningReport);

    // 3. Verify AP is 1
    // Wait, AP is reset in MorningReport? No, in my recent edit:
    // "Reset AP (New Round)" is at start of EnemyAction block (transition FROM Execution TO EnemyAction?).
    // No, `advance_phase` matches CURRENT phase.
    // If state.phase is `Execution` and we call `advance_phase`:
    // It goes to `EnemyAction`.
    // My previous edit inserted `p.ap = 2` in `GamePhase::EnemyAction` block.
    // `match state.phase { GamePhase::EnemyAction => { ... } }` runs when we are IN EnemyAction and transitioning OUT.
    // So AP reset happens when moving EnemyAction -> MorningReport.

    // Let's trace carefully:
    // P1 votes ready in Execution. `advance_phase` called. `match Execution` -> `state.phase = EnemyAction`.
    // P1 votes ready in EnemyAction. `advance_phase` called. `match EnemyAction`:
    //   -> Reset AP to 2.
    //   -> Call `on_round_end` (Flu timer 1->0).
    //   -> Call `on_round_start` (Flu sees 0, AP=1, remove card).
    //   -> state.phase = MorningReport.

    assert_eq!(state.players["P1"].ap, 1, "Flu Wave should reduce AP to 1");

    // 4. Verify Card Removed
    assert!(
        !state
            .active_situations
            .iter()
            .any(|c| c.id == CardId::FluWave),
        "Flu Wave should be removed after triggering"
    );

    // 5. Advance to Planning
    // MorningReport -> EnemyTelegraph -> TacticalPlanning.
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap(); // To Telegraph
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap(); // To Planning

    // Verify AP persists (since Telegraph no longer resets it)
    assert_eq!(state.players["P1"].ap, 1, "AP Penalty should persist to Planning");

    // 6. Complete this round (Planning -> Execution)
    // Spend the 1 AP to Pass (or just Pass)
    state = GameLogic::apply_action(state, "P1", Action::Pass, None).unwrap();
    assert_eq!(state.phase, GamePhase::Execution);

    // Execution -> EnemyAction
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::EnemyAction);

    // 7. Next Round Start (EnemyAction -> MorningReport)
    // This should reset AP to 2. FluWave is gone, so no penalty.
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::MorningReport);

    // MorningReport -> Telegraph -> Planning
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::TacticalPlanning);

    // 8. Verify AP restored to 2
    assert_eq!(state.players["P1"].ap, 2, "AP should be restored to 2 in next round");
}
