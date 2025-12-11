use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardType, GameState},
};

pub struct CostumePartyCard;

impl CardBehavior for CostumePartyCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::CostumeParty,
            title: "Costume Party".to_owned(),
            description: "Players swap positions (Cyclic shift).".to_owned(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
            affected_player: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Effect: Players swap positions (Cyclic shift: P1->P2, P2->P3...).
        // Use sorted IDs to ensure deterministic cycle
        let player_ids: Vec<String> = state.players.keys().cloned().collect();

        if player_ids.is_empty() {
            return;
        }

        // Capture current rooms
        let current_rooms: Vec<u32> = player_ids
            .iter()
            .map(|id| state.players.get(id).unwrap().room_id)
            .collect();

        // Rotate rooms: Room of P_last goes to P_first?
        // Or "P1 moves to P2's spot".
        // P1 -> P2's room.
        // P2 -> P3's room.
        // P_last -> P1's room.

        let len = player_ids.len();
        for (i, pid) in player_ids.iter().enumerate() {
            // Target is (i + 1) % len
            let target_room_idx = (i + 1) % len;
            let new_room = current_rooms[target_room_idx];

            if let Some(p) = state.players.get_mut(pid) {
                p.room_id = new_room;
            }
        }
    }
}
