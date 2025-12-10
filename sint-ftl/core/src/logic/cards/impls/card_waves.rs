use crate::{
    logic::{cards::behavior::CardBehavior, pathfinding::find_path},
    types::{Card, CardId, CardType, GameState},
};

pub struct HighWavesCard;

impl CardBehavior for HighWavesCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::HighWaves,
            title: "High Waves".to_string(),
            description: format!(
                "All players are pushed 1 Room towards the Engine ({}).",
                crate::types::SystemType::Engine.as_u32()
            )
            .to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Effect: All players are pushed 1 Room towards the Engine (5).
        let engine_id = crate::types::SystemType::Engine.as_u32();
        let player_ids: Vec<String> = state.players.keys().cloned().collect();

        for pid in player_ids {
            let current_room = state.players.get(&pid).unwrap().room_id;

            // Calculate path to Engine
            if let Some(path) = find_path(&state.map, current_room, engine_id) {
                if let Some(&next_step) = path.first() {
                    // Update player position
                    if let Some(p) = state.players.get_mut(&pid) {
                        p.room_id = next_step;
                    }
                }
            }
        }
    }
}
