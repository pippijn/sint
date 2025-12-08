use crate::types::*;
use super::get_behavior;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

pub fn initialize_deck(rng: &mut StdRng) -> Vec<Card> {
    let mut deck = vec![
        // Tier 1
        Card {
            id: CardId::AfternoonNap,
            title: "Afternoon Nap".to_string(),
            description: "The Reader falls asleep. Cannot spend AP.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution { room_id: None, ap_cost: 1, item_cost: None, required_players: 1 }),
        },
        Card {
            id: CardId::StaticNoise,
            title: "Static Noise".to_string(),
            description: "Radio interference. Chat restricted to Emoji Only.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution { room_id: Some(9), ap_cost: 1, item_cost: None, required_players: 1 }),
        },
        Card {
            id: CardId::SeagullAttack,
            title: "Seagull Attack".to_string(),
            description: "Birds attacking ammo. Cannot Move while holding Peppernuts.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution { room_id: Some(2), ap_cost: 1, item_cost: None, required_players: 1 }),
        },
        Card {
            id: CardId::SlipperyDeck,
            title: "Slippery Deck".to_string(),
            description: "Soap everywhere. Move costs 0 AP, but Actions cost +1 AP.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution { room_id: Some(5), ap_cost: 1, item_cost: None, required_players: 1 }),
        },
        Card {
            id: CardId::PeppernutRain,
            title: "Peppernut Rain".to_string(),
            description: "+2 Peppernuts dropped in every occupied room.".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        },
        // Tier 2
        Card {
            id: CardId::HighWaves,
            title: "High Waves".to_string(),
            description: "All players are pushed 1 Room towards the Engine (5).".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        },
        Card {
            id: CardId::CostumeParty,
            title: "Costume Party".to_string(),
            description: "Players swap positions (Cyclic shift).".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        },
        Card {
            id: CardId::MicePlague,
            title: "Mice Plague".to_string(),
            description: "At end of round, lose 2 Peppernuts from Storage (11).".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution { room_id: Some(11), ap_cost: 1, item_cost: None, required_players: 1 }),
        },
        Card {
            id: CardId::ShortCircuit,
            title: "Short Circuit".to_string(),
            description: "Spawn 1 Fire in the Engine Room (5).".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        },
        Card {
            id: CardId::Leak,
            title: "Leak!".to_string(),
            description: "Spawn 1 Water in the Cargo Room (4).".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        },
        // Tier 3
        Card {
            id: CardId::Mutiny,
            title: "Mutiny?".to_string(),
            description: "If not solved, Game Over (or -10 Hull).".to_string(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution { room_id: Some(9), ap_cost: 1, item_cost: None, required_players: 2 }),
        },
        Card {
            id: CardId::FogBank,
            title: "Fog Bank".to_string(),
            description: "Cannot see Enemy Intent (Telegraph disabled).".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution { room_id: Some(2), ap_cost: 2, item_cost: None, required_players: 1 }),
        },
        Card {
            id: CardId::AnchorStuck,
            title: "Anchor Stuck".to_string(),
            description: "Evasion action (Engine) is disabled.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution { room_id: Some(2), ap_cost: 1, item_cost: None, required_players: 3 }),
        },
        Card {
            id: CardId::JammedCannon,
            title: "Jammed Cannon".to_string(),
            description: "Cannons (8) are disabled.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution { room_id: Some(8), ap_cost: 1, item_cost: Some(ItemType::Peppernut), required_players: 1 }),
        },
        Card {
            id: CardId::ManOverboard,
            title: "Man Overboard!".to_string(),
            description: "Target Player (Random) is removed from play.".to_string(),
            card_type: CardType::Timebomb { rounds_left: 2 },
            options: vec![],
            solution: Some(CardSolution { room_id: Some(2), ap_cost: 1, item_cost: None, required_players: 1 }), // Throw rope? Interact?
        },
        Card {
            id: CardId::StrongHeadwind,
            title: "Strong Headwind".to_string(),
            description: "Cannons are inaccurate. Hit Threshold is 5+.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution { room_id: Some(9), ap_cost: 1, item_cost: None, required_players: 1 }),
        },
        Card {
            id: CardId::Listing,
            title: "Listing Ship".to_string(),
            description: "Walking is easy (0 AP), but working is hard (2x Cost).".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution { room_id: Some(5), ap_cost: 1, item_cost: None, required_players: 1 }),
        },
        Card {
            id: CardId::CloggedPipe,
            title: "Clogged Pipe".to_string(),
            description: "Kitchen (6) is disabled.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution { room_id: Some(6), ap_cost: 1, item_cost: None, required_players: 1 }),
        },
        Card {
            id: CardId::AttackWave,
            title: "Attack Wave".to_string(),
            description: "Enemy attacks twice this round!".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution { room_id: Some(8), ap_cost: 1, item_cost: None, required_players: 1 }),
        },
        Card {
            id: CardId::SingASong,
            title: "Sing a Song".to_string(),
            description: "Morale boost! Removes 2 Fire/Water tokens.".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        },
    ];

    deck.shuffle(rng);
    deck
}

pub fn draw_card(state: &mut GameState) {
    if let Some(card) = state.deck.pop() {
        state.latest_event = Some(card.clone());

        match card.card_type {
            CardType::Flash => {
                get_behavior(card.id).on_activate(state);
                state.discard.push(card);
            }
            CardType::Situation | CardType::Timebomb { .. } => {
                state.active_situations.push(card.clone());
                get_behavior(card.id).on_activate(state);
            }
        }
    }
}