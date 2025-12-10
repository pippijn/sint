use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState},
    GameError,
};

pub struct AfternoonNapCard;

impl CardBehavior for AfternoonNapCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::AfternoonNap,
            title: "Afternoon Nap".to_string(),
            description: "The Reader falls asleep. Cannot spend AP.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: None,
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn validate_action(
        &self,
        state: &GameState,
        player_id: &str,
        action: &GameAction,
    ) -> Result<(), GameError> {
        // Logic: "The Reader" cannot spend AP.
        // Definition of "Reader": The player whose ID is lexicographically first.
        let mut sorted_ids: Vec<&String> = state.players.keys().collect();
        sorted_ids.sort();

        let reader_id = sorted_ids.first();

        if let Some(&rid) = reader_id {
            if rid == player_id {
                // Check if action costs AP
                // Hardcoded knowledge of costs here, or we'd need to invoke cost calc?
                // But validate is called BEFORE cost calc in apply_action.
                // We'll duplicate the base cost logic slightly or just block all non-free actions.
                let is_free = matches!(
                    action,
                    GameAction::Chat { .. }
                        | GameAction::VoteReady { .. }
                        | GameAction::Pass
                        | GameAction::Undo { .. }
                );

                if !is_free {
                    return Err(GameError::InvalidAction(
                        "The Reader (You) is asleep and cannot spend AP!".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}
