use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, Card, CardId, CardSolution, CardType, GameState};
use crate::GameError;

pub struct WailingAlarmCard;

impl CardBehavior for WailingAlarmCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::WailingAlarm,
            title: "Wailing Alarm".to_string(),
            description: "No Bonuses. Special items and skills don't work.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Hallway.as_u32()),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn validate_action(
        &self,
        _state: &GameState,
        _player_id: &str,
        action: &Action,
    ) -> Result<(), GameError> {
        match action {
            Action::RaiseShields => Err(GameError::InvalidAction(
                "Wailing Alarm! Shields are disabled.".to_string(),
            )),
            Action::EvasiveManeuvers => Err(GameError::InvalidAction(
                "Wailing Alarm! Evasive Maneuvers are disabled.".to_string(),
            )),
            _ => Ok(()),
        }
    }
}
