use crate::{
    GameError,
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, ChatMessage, GameAction, GameState},
};

pub struct AfternoonNapCard;

impl CardBehavior for AfternoonNapCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::AfternoonNap,
            title: "Afternoon Nap".to_owned(),
            description: "The Reader falls asleep. Cannot spend AP.".to_owned(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: None, // Any room (Interacting wakes them up?)
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        let player_ids: Vec<String> = state.players.keys().cloned().collect();

        if player_ids.is_empty() {
            return;
        }

        // Logic: Turn count determines rotation.
        let index = (state.turn_count.saturating_sub(1) as usize) % player_ids.len();
        let reader_id = player_ids[index].clone();

        // Update the card state. We assume the newly drawn card is at the end.
        // We verify ID just in case.
        if let Some(card) = state.active_situations.last_mut()
            && card.id == CardId::AfternoonNap
        {
            card.affected_player = Some(reader_id.clone());
        }

        state.chat_log.push(ChatMessage {
            sender: "SYSTEM".to_owned(),
            text: format!("{} is the Reader and falls asleep!", reader_id),
            timestamp: 0,
        });
    }

    fn validate_action(
        &self,
        state: &GameState,
        player_id: &str,
        action: &GameAction,
    ) -> Result<(), GameError> {
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

        // Check if ANY Nap card targets THIS player
        let is_asleep = state.active_situations.iter().any(|c| {
            c.id == CardId::AfternoonNap && c.affected_player.as_deref() == Some(player_id)
        });

        if is_asleep {
            let name = state
                .players
                .get(player_id)
                .map(|p| p.name.as_str())
                .unwrap_or(player_id);
            return Err(GameError::InvalidAction(format!(
                "The Reader ({}) is asleep and cannot spend AP!",
                name
            )));
        }
        Ok(())
    }
}
