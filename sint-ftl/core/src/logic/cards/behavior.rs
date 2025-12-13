use crate::{
    GameError,
    types::{Card, CardId, CardSentiment, CardType, GameAction, GameState},
};

pub trait CardBehavior: Send + Sync {
    /// Returns the static definition of the card (Title, Description, etc.)
    fn get_struct(&self) -> Card;

    /// Returns the sentiment of the card (Positive, Neutral, Negative).
    /// Default: Negative.
    fn get_sentiment(&self) -> CardSentiment {
        CardSentiment::Negative
    }

    /// Modify the AP cost of an action.
    /// Default: return base_cost unmodified.
    fn modify_action_cost(
        &self,
        _state: &GameState,
        _player_id: &str,
        _action: &GameAction,
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
        _action: &GameAction,
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

    /// Modify the number of hazard tokens spawned per hit (Default 0).
    /// Used by Rudderless.
    fn get_hazard_modifier(&self, _state: &GameState) -> u32 {
        0
    }

    /// Hook to modify the enemy attack telegraph (e.g. Masking it).
    fn modify_telegraph(&self, _attack: &mut crate::types::EnemyAttack) {}

    /// Hook to resolve/reveal the enemy attack before execution (e.g. Unmasking Fog).
    fn resolve_telegraph(&self, _state: &mut GameState, _attack: &mut crate::types::EnemyAttack) {}

    /// Hook called during execution phase, before the action is applied.
    /// Can be used for RNG checks (Sticky Floor) or late validation.
    /// Returns Ok(()) to proceed, or Err to skip action.
    fn check_resolution(
        &self,
        _state: &mut GameState,
        _player_id: &str,
        _action: &GameAction,
    ) -> Result<(), GameError> {
        Ok(())
    }

    /// Hook called when the card is successfully solved/removed via Interaction.
    fn on_solved(&self, _state: &mut GameState) {}

    /// Check if the card can be solved by the player in their current state.
    fn can_solve(&self, state: &GameState, player_id: &str) -> bool {
        if let Some(sol) = self.get_struct().solution {
            let p = if let Some(player) = state.players.get(player_id) {
                player
            } else {
                return false;
            };

            let room_match = if let Some(sys) = sol.target_system {
                crate::logic::find_room_with_system(state, sys) == Some(p.room_id)
            } else {
                true
            };

            let item_match =
                sol.item_cost.is_none() || p.inventory.contains(sol.item_cost.as_ref().unwrap());

            room_match && item_match
        } else {
            false
        }
    }
}

// A default behavior that does nothing
pub struct NoOpBehavior;
impl CardBehavior for NoOpBehavior {
    fn get_struct(&self) -> Card {
        // Fallback for missing behaviors
        Card {
            id: CardId::AfternoonNap, // Dummy ID
            title: "Unknown Card".to_owned(),
            description: "Missing implementation.".to_owned(),
            card_type: CardType::Situation,
            options: vec![],
            solution: None,
            affected_player: None,
        }
    }
}
