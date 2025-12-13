use sint_core::logic::GameLogic;
use sint_core::types::{Action, GameAction, GamePhase, GameState};

#[derive(Clone, Debug)]
pub struct GameDriver {
    pub state: GameState,
}

impl GameDriver {
    pub fn new(state: GameState) -> Self {
        let mut driver = Self { state };
        driver.stabilize();
        driver
    }

    /// Advances the game state until a strategic decision is required or the game ends.
    ///
    /// "Stable" means:
    /// 1. We are in `TacticalPlanning` phase.
    /// 2. At least one player has AP > 0 and is not ready.
    ///    OR
    /// 3. The game is over (Victory/GameOver).
    fn stabilize(&mut self) {
        let mut loop_safety = 0;
        loop {
            loop_safety += 1;
            if loop_safety > 5000 {
                // Prevent infinite loops if logic is broken
                break;
            }

            if self.state.phase == GamePhase::GameOver || self.state.phase == GamePhase::Victory {
                return;
            }

            if self.state.phase != GamePhase::TacticalPlanning {
                // If not in planning, everyone votes ready to advance
                let unready_players: Vec<String> = self
                    .state
                    .players
                    .values()
                    .filter(|p| !p.is_ready)
                    .map(|p| p.id.clone())
                    .collect();

                if unready_players.is_empty() {
                    // This implies we are waiting for something else, or logic failed?
                    // In sint-core, apply_action usually advances phase if everyone is ready.
                    // If we are stuck here, force advance via ready on first player?
                    // Actually, if unready is empty, the phase SHOULD have advanced in the previous apply.
                    // If we are here, something weird happened.
                    // Let's assume apply_action handles phase transition on last ready.
                    break;
                }

                for pid in unready_players {
                    let _ = GameLogic::apply_action(
                        self.state.clone(),
                        &pid,
                        Action::Game(GameAction::VoteReady { ready: true }),
                        None,
                    )
                    .map(|s| self.state = s);

                    if self.state.phase == GamePhase::TacticalPlanning
                        || self.state.phase == GamePhase::GameOver
                        || self.state.phase == GamePhase::Victory
                    {
                        break;
                    }
                }
                continue;
            }

            // We are in TacticalPlanning.
            // Check for players with 0 AP who are NOT ready. They must be marked ready.
            let zero_ap_unready: Vec<String> = self
                .state
                .players
                .values()
                .filter(|p| p.ap <= 0 && !p.is_ready)
                .map(|p| p.id.clone())
                .collect();

            if !zero_ap_unready.is_empty() {
                for pid in zero_ap_unready {
                    let _ = GameLogic::apply_action(
                        self.state.clone(),
                        &pid,
                        Action::Game(GameAction::VoteReady { ready: true }),
                        None,
                    )
                    .map(|s| self.state = s);

                    if self.state.phase != GamePhase::TacticalPlanning {
                        break; // Phase changed (everyone was 0 AP), loop again to fast forward next phases
                    }
                }
                continue;
            }

            // If we are here:
            // 1. Phase is TacticalPlanning.
            // 2. No players have 0 AP AND unready status.
            // This means all unready players have AP > 0 (Decisions to make!), OR everyone is ready (which would advance phase).
            break;
        }
    }

    /// Applies a strategic action and then stabilizes the state.
    pub fn apply(&mut self, player_id: &str, action: GameAction) -> Result<(), String> {
        match GameLogic::apply_action(self.state.clone(), player_id, Action::Game(action), None) {
            Ok(new_state) => {
                self.state = new_state;
                self.stabilize();
                Ok(())
            }
            Err(e) => Err(format!("{:?}", e)),
        }
    }
}
