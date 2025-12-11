use crate::{
    logic::{
        cards::behavior::CardBehavior, find_empty_rooms, find_room_with_system,
        pathfinding::find_path,
    },
    types::{Card, CardId, CardType, GameState, SystemType},
};

pub struct FalseNoteCard;

impl CardBehavior for FalseNoteCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::FalseNote,
            title: "False Note".to_owned(),
            description: "Everyone in Cannons flees to the nearest Empty Room.".to_owned(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
            affected_player: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        let start_room = find_room_with_system(state, SystemType::Cannons);
        let empty_rooms = find_empty_rooms(state);

        if let Some(start) = start_room {
            // Find nearest
            let mut best_target = None;
            let mut min_dist = usize::MAX;

            // Empty rooms are sorted by ID for tie-breaking (lowest ID wins)
            for target in empty_rooms {
                if let Some(path) = find_path(&state.map, start, target) {
                    // path includes end but excludes start? find_path returns steps.
                    // Len is distance.
                    if path.len() < min_dist {
                        min_dist = path.len();
                        best_target = Some(target);
                    }
                }
            }

            if let Some(target) = best_target {
                for p in state.players.values_mut() {
                    if p.room_id == start {
                        p.room_id = target;
                    }
                }
            }
        }
    }
}
