use crate::types::{Action, Card, CardId, CardType, GameState};
use crate::GameError;

pub trait CardBehavior: Send + Sync {
    /// Returns the static definition of the card (Title, Description, etc.)
    fn get_struct(&self) -> Card;

    /// Modify the AP cost of an action.
    /// Default: return base_cost unmodified.
    fn modify_action_cost(
        &self,
        _state: &GameState,
        _player_id: &str,
        _action: &Action,
        base_cost: i32,
    ) -> i32 {
        base_cost
    }

    /// Validate if an action is allowed.
    /// Default: return Ok(()).
    fn validate_action(
        &self,
        _state: &GameState,
        _player_id: &str,
        _action: &Action,
    ) -> Result<(), GameError> {
        Ok(())
    }

    /// Hook for when the card is drawn/activated (optional).
    fn on_activate(&self, _state: &mut GameState) {}

    /// Hook for round end / cleanup (optional).
    fn on_round_end(&self, _state: &mut GameState) {}

    /// Hook for start of turn (before AP reset or after?).
    /// Called in `advance_phase` when entering TacticalPlanning?
    /// Or MorningReport?
    /// Let's call it `on_round_start`.
    fn on_round_start(&self, _state: &mut GameState) {}

    /// Modify the success roll threshold for Shooting (Default 3).
    fn get_hit_threshold(&self, _state: &GameState) -> u32 {
        3
    }

    /// Modify the number of attacks the enemy performs (Default 1).
    fn get_enemy_attack_count(&self, _state: &GameState) -> u32 {
        1
    }

    /// Hook called during execution phase, before the action is applied.
    /// Can be used for RNG checks (Sticky Floor) or late validation.
    /// Returns Ok(()) to proceed, or Err to skip action.
    fn check_resolution(
        &self,
        _state: &mut GameState,
        _player_id: &str,
        _action: &Action,
    ) -> Result<(), GameError> {
        Ok(())
    }
}

// A default behavior that does nothing
pub struct NoOpBehavior;
impl CardBehavior for NoOpBehavior {
    fn get_struct(&self) -> Card {
        // Fallback for missing behaviors
        Card {
            id: CardId::AfternoonNap, // Dummy ID
            title: "Unknown Card".to_string(),
            description: "Missing implementation.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: None,
        }
    }
}
