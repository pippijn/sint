use crate::{
    GameError,
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
};

pub struct SeasickCard;

impl CardBehavior for SeasickCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Seasick,
            title: "Seasick".to_owned(),
            description: "Nauseous. You may EITHER Walk OR do Actions (not both).".to_owned(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Kitchen),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn validate_action(
        &self,
        state: &GameState,
        player_id: &str,
        action: &GameAction,
    ) -> Result<(), GameError> {
        let is_move = matches!(action, GameAction::Move { .. });
        let is_free = matches!(
            action,
            GameAction::Chat { .. }
                | GameAction::VoteReady { .. }
                | GameAction::Pass
                | GameAction::Undo { .. }
        );

        if is_free {
            return Ok(());
        }

        let has_moves = state
            .proposal_queue
            .iter()
            .any(|p| p.player_id == player_id && matches!(p.action, GameAction::Move { .. }));
        let has_others = state.proposal_queue.iter().any(|p| {
            p.player_id == player_id
                && !matches!(
                    p.action,
                    GameAction::Move { .. }
                        | GameAction::Chat { .. }
                        | GameAction::VoteReady { .. }
                        | GameAction::Pass
                        | GameAction::Undo { .. }
                )
        });

        if is_move {
            if has_others {
                return Err(GameError::InvalidAction(
                    "Seasick! Cannot Walk if you already performed Actions.".to_owned(),
                ));
            }
        } else if has_moves {
            return Err(GameError::InvalidAction(
                "Seasick! Cannot perform Actions if you already Walked.".to_owned(),
            ));
        }

        Ok(())
    }
}
