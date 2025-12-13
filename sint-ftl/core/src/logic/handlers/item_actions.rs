use super::ActionHandler;
use crate::GameError;
use crate::types::{GameState, ItemType};

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
        } else {
            // Special Item Limit: Can only carry 1 special item (non-Peppernut)
            let has_special = p.inventory.iter().any(|i| *i != ItemType::Peppernut);
            if has_special {
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
        let room = state
            .map
            .rooms
            .get_mut(&room_id)
            .ok_or(GameError::RoomNotFound)?;
        let pos = room
            .items
            .iter()
            .position(|x| *x == self.item_type)
            .ok_or_else(|| GameError::InvalidAction("Item disappeared".to_owned()))?;
        let item = room.items.remove(pos);

        // Add to inventory
        state
            .players
            .get_mut(player_id)
            .ok_or(GameError::PlayerNotFound)?
            .inventory
            .push(item);

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

        let p = state
            .players
            .get_mut(player_id)
            .ok_or(GameError::PlayerNotFound)?;
        let room_id = p.room_id;
        let item = p.inventory.remove(self.item_index);

        if let Some(room) = state.map.rooms.get_mut(&room_id) {
            room.items.push(item);
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

        // Check Target Capacity
        let item_to_throw = &p.inventory[self.item_index];
        if *item_to_throw == ItemType::Peppernut {
            let nut_count = target
                .inventory
                .iter()
                .filter(|i| **i == ItemType::Peppernut)
                .count();
            let has_wheelbarrow = target.inventory.contains(&ItemType::Wheelbarrow);
            let limit = if has_wheelbarrow { 5 } else { 1 };

            if nut_count >= limit {
                return Err(GameError::InventoryFull);
            }
        } else {
            // Special Item Limit
            let has_special = target.inventory.iter().any(|i| *i != ItemType::Peppernut);
            if has_special {
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

        let item = state
            .players
            .get_mut(player_id)
            .ok_or(GameError::PlayerNotFound)?
            .inventory
            .remove(self.item_index);

        state
            .players
            .get_mut(&self.target_player)
            .ok_or(GameError::PlayerNotFound)?
            .inventory
            .push(item);

        Ok(())
    }
}
