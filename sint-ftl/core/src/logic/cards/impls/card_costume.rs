use crate::logic::cards::behavior::CardBehavior;
use crate::types::GameState;
use crate::types::{Card, CardId, CardType};

pub struct CostumePartyCard;

impl CardBehavior for CostumePartyCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::CostumeParty,
            title: "Costume Party".to_string(),
            description: "Players swap positions (Cyclic shift).".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Effect: Players swap positions (Cyclic shift: P1->P2, P2->P3...).
        // Sort IDs to ensure deterministic cycle
        let mut sorted_ids: Vec<String> = state.players.keys().cloned().collect();
        sorted_ids.sort();

        if sorted_ids.is_empty() {
            return;
        }

        // Capture current rooms
        let current_rooms: Vec<u32> = sorted_ids
            .iter()
            .map(|id| state.players.get(id).unwrap().room_id)
            .collect();

        // Rotate rooms: Room of P_last goes to P_first?
        // Or "P1 moves to P2's spot".
        // P1 -> P2's room.
        // P2 -> P3's room.
        // P_last -> P1's room.

        let len = sorted_ids.len();
        for (i, pid) in sorted_ids.iter().enumerate() {
            // Target is (i + 1) % len
            let target_room_idx = (i + 1) % len;
            let new_room = current_rooms[target_room_idx];

            if let Some(p) = state.players.get_mut(pid) {
                p.room_id = new_room;
            }
        }
    }
}
