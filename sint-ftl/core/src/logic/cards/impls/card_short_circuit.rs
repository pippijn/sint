use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardType, GameState, HazardType},
};

pub struct ShortCircuitCard;

impl CardBehavior for ShortCircuitCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::ShortCircuit,
            title: "Short Circuit".to_string(),
            description: format!(
                "Spawn 1 Fire in the Engine Room ({}).",
                crate::types::SystemType::Engine.as_u32()
            )
            .to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Effect: Spawn 1 Fire in the Engine Room (5).
        if let Some(room) = state
            .map
            .rooms
            .get_mut(&crate::types::SystemType::Engine.as_u32())
        {
            room.hazards.push(HazardType::Fire);
        }
    }
}
