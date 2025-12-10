use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardType, GameState, ItemType},
};

pub struct LuckyDipCard;

impl CardBehavior for LuckyDipCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::LuckyDip,
            title: "Lucky Dip".to_string(),
            description: "Tool Swap! All players pass their Special Item to the left.".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Swap Special Items to LEFT.
        // Sorted IDs
        let mut sorted_ids: Vec<String> = state.players.keys().cloned().collect();
        sorted_ids.sort();

        let len = sorted_ids.len();
        if len < 2 {
            return;
        }

        // Extract special items
        let special_types = [
            ItemType::Extinguisher,
            ItemType::Keychain,
            ItemType::Wheelbarrow,
            ItemType::Mitre,
        ];

        let mut extracted_items = vec![None; len];

        for (i, pid) in sorted_ids.iter().enumerate() {
            if let Some(p) = state.players.get_mut(pid) {
                // Find first special item
                if let Some(pos) = p.inventory.iter().position(|it| special_types.contains(it)) {
                    extracted_items[i] = Some(p.inventory.remove(pos));
                }
            }
        }

        // Rotate Left: Player i gets item from (i + 1) % len (Right)?
        // "Pass ... to the player to their LEFT".
        // If I sit in circle: P1 - P2 - P3.
        // P1 passes to Left (P3?). P3 passes to Left (P2?).
        // Usually Left means index - 1 or + 1 depending on view.
        // Let's assume P1 -> P2 -> P3 -> P1.
        // So i receives from i-1.

        for (i, pid) in sorted_ids.iter().enumerate() {
            let source_idx = if i == 0 { len - 1 } else { i - 1 };
            if let Some(item) = &extracted_items[source_idx] {
                if let Some(p) = state.players.get_mut(pid) {
                    p.inventory.push(item.clone());
                }
            }
        }
    }
}
