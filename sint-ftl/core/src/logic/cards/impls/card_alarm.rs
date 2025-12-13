use crate::{
    GameError,
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState},
};

pub struct WailingAlarmCard;

impl CardBehavior for WailingAlarmCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::WailingAlarm,
            title: "Wailing Alarm".to_owned(),
            description: "No Bonuses. Solve in any Empty Room.".to_owned(),
            card_type: CardType::Situation,
            options: vec![].into(),
            solution: Some(CardSolution {
                target_system: None, // Any room (Checked in validate_action to be Empty)
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn validate_action(
        &self,
        _state: &GameState,
        _player_id: &str,
        action: &GameAction,
    ) -> Result<(), GameError> {
        match action {
            GameAction::RaiseShields => Err(GameError::InvalidAction(
                "Wailing Alarm! Shields are disabled.".to_owned(),
            )),
            GameAction::EvasiveManeuvers => Err(GameError::InvalidAction(
                "Wailing Alarm! Evasive Maneuvers are disabled.".to_owned(),
            )),
            _ => Ok(()),
        }
    }

    fn can_solve(&self, state: &GameState, player_id: &str) -> bool {
        if let Some(player) = state.players.get(player_id)
            && let Some(room) = state.map.rooms.get(&player.room_id)
        {
            return room.system.is_none();
        }
        false
    }
}
