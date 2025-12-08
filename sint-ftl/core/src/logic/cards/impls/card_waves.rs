use crate::logic::cards::behavior::CardBehavior;
use crate::logic::pathfinding::find_path;
use crate::types::GameState;

pub struct HighWavesCard;

use crate::types::{Card, CardId, CardType};

impl CardBehavior for HighWavesCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::HighWaves,
            title: "High Waves".to_string(),
            description: "All players are pushed 1 Room towards the Engine (5).".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Effect: All players are pushed 1 Room towards the Engine (5).
        let engine_id = 5;
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
