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

        if !p.can_add_item(self.item_type) {
            return Err(GameError::InventoryFull);
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
        if *item_to_drop == ItemType::Wheelbarrow && p.peppernut_count() > 2 {
            return Err(GameError::InvalidAction(
                "Cannot drop Wheelbarrow while holding >2 Peppernuts".to_owned(),
            ));
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
            // Water destroys Peppernuts immediately (except in Storage)
            if item == ItemType::Peppernut
                && room.hazards.contains(&crate::types::HazardType::Water)
                && room.system != Some(crate::types::SystemType::Storage)
            {
                // Destroyed! Do not push to room.items
                log::info!(
                    "Peppernut dropped into water and destroyed in room {}",
                    room_id
                );
            } else {
                room.add_item(item);
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

        // Only Peppernuts can be thrown
        let item_to_throw = &p.inventory[self.item_index];
        if *item_to_throw != ItemType::Peppernut {
            return Err(GameError::InvalidAction(
                "Only Peppernuts can be thrown".to_owned(),
            ));
        }

        // Check Target Capacity
        if !target.can_add_item(ItemType::Peppernut) {
            return Err(GameError::InventoryFull);
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
