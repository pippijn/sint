use crate::{
    GameError,
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, ItemType, SystemType},
};

pub struct SeagullAttackCard;

impl CardBehavior for SeagullAttackCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::SeagullAttack,
            title: "Seagull Attack".to_owned(),
            description: "Birds attacking ammo. Cannot Move while holding Peppernuts.".to_owned(),
            card_type: CardType::Situation,
            options: vec![].into(),
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bow),
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
        if let GameAction::Move { .. } = action
            && let Some(player) = state.players.get(player_id)
            && player.inventory.contains(&ItemType::Peppernut)
        {
            return Err(GameError::InvalidAction(
                "Cannot move while holding Peppernuts (Seagull Attack)".to_owned(),
            ));
        }
        Ok(())
    }

    fn check_resolution(
        &self,
        state: &mut GameState,
        player_id: &str,
        action: &GameAction,
    ) -> Result<(), GameError> {
        self.validate_action(state, player_id, action)
    }
}
