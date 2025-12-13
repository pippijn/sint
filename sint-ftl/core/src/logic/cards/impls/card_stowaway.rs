use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameState, ItemType, SystemType},
};

pub struct StowawayCard;

impl CardBehavior for StowawayCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Stowaway,
            title: "The Stowaway".to_owned(),
            description: "Boom: All players lose all peppernuts.".to_owned(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![].into(),
            solution: Some(CardSolution {
                target_system: Some(SystemType::Dormitory),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn on_trigger(&self, state: &mut GameState) {
        // Theft: Lose all peppernuts
        for p in state.players.values_mut() {
            p.inventory.retain(|i| *i != ItemType::Peppernut);
        }
        state.active_situations.retain(|c| c.id != CardId::Stowaway);
    }
}
