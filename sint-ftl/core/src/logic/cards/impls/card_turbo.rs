use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, HazardType, SystemType},
};

pub struct TurboModeCard;

impl CardBehavior for TurboModeCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::TurboMode,
            title: "Turbo Mode".to_owned(),
            description: "Advantage: +1 AP. Boom: 2 Fire in Engine, 1 in neighbor.".to_owned(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Engine),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn validate_action(
        &self,
        _state: &GameState,
        _player_id: &str,
        _action: &crate::types::GameAction,
    ) -> Result<(), crate::GameError> {
        Ok(())
    }

    fn on_round_end(&self, state: &mut GameState) {
        let mut triggered = false;
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::TurboMode {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                        if *rounds_left == 0 {
                            triggered = true;
                        }
                    }
                }
            }
        }

        if triggered {
            if let Some(engine_id) = find_room_with_system(state, SystemType::Engine) {
                // 1. Identify Target Neighbor
                let mut target_neighbor = None;
                if let Some(engine_room) = state.map.rooms.get(&engine_id) {
                    let neighbors = &engine_room.neighbors;
                    let mut best_non_empty_id = u32::MAX;

                    for &nid in neighbors {
                        if let Some(n_room) = state.map.rooms.get(&nid) {
                            if n_room.system.is_none() {
                                target_neighbor = Some(nid);
                                break; // Found empty room, priority 1
                            }
                            if nid < best_non_empty_id {
                                best_non_empty_id = nid;
                            }
                        }
                    }
                    if target_neighbor.is_none() && best_non_empty_id != u32::MAX {
                        target_neighbor = Some(best_non_empty_id);
                    }
                }

                // 2. Apply Hazards
                if let Some(room) = state.map.rooms.get_mut(&engine_id) {
                    room.hazards.push(HazardType::Fire);
                    room.hazards.push(HazardType::Fire);
                }
                if let Some(tid) = target_neighbor {
                    if let Some(room) = state.map.rooms.get_mut(&tid) {
                        room.hazards.push(HazardType::Fire);
                    }
                }
            }

            state
                .active_situations
                .retain(|c| c.id != CardId::TurboMode);
        }
    }

    fn on_round_start(&self, state: &mut GameState) {
        // Advantage: +1 AP.
        for p in state.players.values_mut() {
            p.ap += 1;
        }
    }
}
