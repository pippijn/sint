use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSentiment, CardType, GameState, ItemType},
};

pub struct LuckyDipCard;

impl CardBehavior for LuckyDipCard {
    fn get_sentiment(&self) -> CardSentiment {
        CardSentiment::Neutral
    }

    fn get_struct(&self) -> Card {
        Card {
            id: CardId::LuckyDip,
            title: "Lucky Dip".to_owned(),
            description: "Tool Swap! All players pass their Special Item to the left.".to_owned(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
            affected_player: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Swap Special Items to LEFT.
        let player_ids: Vec<String> = state.players.keys().cloned().collect();

        let len = player_ids.len();
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

        for (i, pid) in player_ids.iter().enumerate() {
            if let Some(p) = state.players.get_mut(pid) {
                // Find first special item
                if let Some(pos) = p.inventory.iter().position(|it| special_types.contains(it)) {
                    extracted_items[i] = Some(p.inventory.remove(pos));
                }
            }
        }

        // Swap Special Items to the left (i receives from i-1 in sorted list).

        for (i, pid) in player_ids.iter().enumerate() {
            let source_idx = if i == 0 { len - 1 } else { i - 1 };
            if let Some(item) = &extracted_items[source_idx]
                && let Some(p) = state.players.get_mut(pid)
            {
                p.inventory.push(*item);
            }
        }
    }
}
