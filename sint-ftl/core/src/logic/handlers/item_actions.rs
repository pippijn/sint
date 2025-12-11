use super::ActionHandler;
use crate::types::{GameState, ItemType};
use crate::GameError;

// --- PICK UP ---
pub struct PickUpHandler {
    pub item_type: ItemType,
}
impl ActionHandler for PickUpHandler {
    fn validate(&self, state: &GameState, player_id: &str) -> Result<(), GameError> {
        let p = state
            .players
            .get(player_id)
            .ok_or(GameError::PlayerNotFound)?;
        let room = state
            .map
            .rooms
            .get(&p.room_id)
            .ok_or(GameError::RoomNotFound)?;

        if !room.items.contains(&self.item_type) {
            return Err(GameError::InvalidAction(format!(
                "Item {:?} not in room (or already picked up)",
                self.item_type
            )));
        }

        if self.item_type == ItemType::Peppernut {
            let nut_count = p
                .inventory
                .iter()
                .filter(|i| **i == ItemType::Peppernut)
                .count();
            let has_wheelbarrow = p.inventory.contains(&ItemType::Wheelbarrow);
            let limit = if has_wheelbarrow { 5 } else { 1 };

            if nut_count >= limit {
                return Err(GameError::InventoryFull);
            }
        }
        Ok(())
    }

    fn execute(
        &self,
        state: &mut GameState,
        player_id: &str,
        _simulation: bool,
    ) -> Result<(), GameError> {
        self.validate(state, player_id)?;

        let p = state
            .players
            .get_mut(player_id)
            .ok_or(GameError::PlayerNotFound)?;
        let room_id = p.room_id;

        // Remove from room
        if let Some(room) = state.map.rooms.get_mut(&room_id) {
            if let Some(pos) = room.items.iter().position(|x| *x == self.item_type) {
                let item = room.items.remove(pos);
                // Add to inventory (re-borrow player)
                if let Some(p) = state.players.get_mut(player_id) {
                    p.inventory.push(item);
                }
            }
        }
        Ok(())
    }
}

// --- DROP ---
pub struct DropHandler {
    pub item_index: usize,
}
impl ActionHandler for DropHandler {
    fn validate(&self, state: &GameState, player_id: &str) -> Result<(), GameError> {
        let p = state
            .players
            .get(player_id)
            .ok_or(GameError::PlayerNotFound)?;
        if self.item_index >= p.inventory.len() {
            return Err(GameError::InvalidItem);
        }

        // Cannot drop Wheelbarrow if holding excess Peppernuts
        let item_to_drop = &p.inventory[self.item_index];
        if *item_to_drop == ItemType::Wheelbarrow {
            let nut_count = p
                .inventory
                .iter()
                .filter(|i| **i == ItemType::Peppernut)
                .count();
            if nut_count > 1 {
                return Err(GameError::InvalidAction(
                    "Cannot drop Wheelbarrow while holding >1 Peppernuts".to_owned(),
                ));
            }
        }
        Ok(())
    }

    fn execute(
        &self,
        state: &mut GameState,
        player_id: &str,
        _simulation: bool,
    ) -> Result<(), GameError> {
        self.validate(state, player_id)?;

        let mut item = None;
        let mut room_id = 0;

        if let Some(p) = state.players.get_mut(player_id) {
            room_id = p.room_id;
            item = Some(p.inventory.remove(self.item_index));
        }

        if let Some(it) = item {
            if let Some(room) = state.map.rooms.get_mut(&room_id) {
                room.items.push(it);
            }
        }
        Ok(())
    }

    fn base_cost(&self) -> i32 {
        0
    }
}

// --- THROW ---
pub struct ThrowHandler {
    pub target_player: String,
    pub item_index: usize,
}
impl ActionHandler for ThrowHandler {
    fn validate(&self, state: &GameState, player_id: &str) -> Result<(), GameError> {
        let p = state
            .players
            .get(player_id)
            .ok_or(GameError::PlayerNotFound)?;
        if self.item_index >= p.inventory.len() {
            return Err(GameError::InvalidItem);
        }

        let target = state
            .players
            .get(&self.target_player)
            .ok_or(GameError::PlayerNotFound)?;
        let room = state
            .map
            .rooms
            .get(&p.room_id)
            .ok_or(GameError::RoomNotFound)?;

        let is_adjacent = (p.room_id == target.room_id) || room.neighbors.contains(&target.room_id);

        if !is_adjacent {
            return Err(GameError::InvalidAction("Target not in range".to_owned()));
        }
        Ok(())
    }

    fn execute(
        &self,
        state: &mut GameState,
        player_id: &str,
        _simulation: bool,
    ) -> Result<(), GameError> {
        self.validate(state, player_id)?;

        let mut item = None;
        if let Some(p) = state.players.get_mut(player_id) {
            item = Some(p.inventory.remove(self.item_index));
        }

        if let Some(it) = item {
            if let Some(target) = state.players.get_mut(&self.target_player) {
                target.inventory.push(it);
            }
        }
        Ok(())
    }
}
