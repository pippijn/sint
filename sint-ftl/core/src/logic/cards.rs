use crate::types::*;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

pub fn initialize_deck(rng: &mut StdRng) -> Vec<Card> {
    let mut deck = vec![
        // C01: Afternoon Nap
        Card {
            id: "C01".to_string(),
            title: "Afternoon Nap".to_string(),
            description: "The Reader falls asleep. Cannot spend AP.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: None, // Any room where the reader is? Simplified: Any room.
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        },
        // C02: Static Noise
        Card {
            id: "C02".to_string(),
            title: "Static Noise".to_string(),
            description: "Radio interference. Chat restricted to Emoji Only.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(9), // Bridge
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        },
        // C03: Seagull Attack
        Card {
            id: "C03".to_string(),
            title: "Seagull Attack".to_string(),
            description: "Birds attacking ammo. Cannot Move while holding Peppernuts.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(2), // Bow
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        },
        // C04: Slippery Deck (Simplified Effect)
        Card {
            id: "C04".to_string(),
            title: "Slippery Deck".to_string(),
            description: "Soap everywhere. Move costs 0 AP, but Actions cost +1 AP.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(5), // Engine
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        },
        // C05: Peppernut Rain
        Card {
            id: "C05".to_string(),
            title: "Peppernut Rain".to_string(),
            description: "+2 Peppernuts for everyone.".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        },
    ];

    deck.shuffle(rng);
    deck
}

pub fn draw_card(state: &mut GameState) {
    // Simple draw logic
    if let Some(card) = state.deck.pop() {
        state.latest_event = Some(card.clone());

        match card.card_type {
            CardType::Flash => {
                // C05: Peppernut Rain logic
                if card.id == "C05" {
                    for p in state.players.values_mut() {
                        p.inventory.push(ItemType::Peppernut);
                        p.inventory.push(ItemType::Peppernut);
                        // TODO: Handle overflow
                    }
                }
                state.discard.push(card);
            }
            CardType::Situation | CardType::Timebomb { .. } => {
                state.active_situations.push(card);
            }
        }
    } else {
        // Reshuffle discard?
    }
}
