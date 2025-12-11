use crate::types::{GameMap, HazardType, ItemType, MapLayout, Room, SystemType};
use std::collections::HashMap;

pub struct RoomDef {
    pub name: &'static str,
    pub system: Option<SystemType>,
    pub items: Vec<ItemType>,
    pub hazards: Vec<HazardType>,
}

impl RoomDef {
    fn new(name: &'static str, system: Option<SystemType>) -> Self {
        let mut items = vec![];

        // Default Items based on System
        if system == Some(SystemType::Storage) {
            items = vec![ItemType::Peppernut; 5];
        } else if system == Some(SystemType::Cargo) {
            items = vec![ItemType::Wheelbarrow];
        } else if system == Some(SystemType::Engine) {
            items = vec![ItemType::Extinguisher];
        }

        Self {
            name,
            system,
            items,
            hazards: vec![],
        }
    }
}

pub fn generate_map(layout: MapLayout) -> GameMap {
    match layout {
        MapLayout::Star => generate_star(),
        MapLayout::Torus => generate_torus(),
    }
}

fn generate_star() -> GameMap {
    let definitions = vec![
        (0, RoomDef::new("Central Hallway", None)),
        (1, RoomDef::new("The Bow", Some(SystemType::Bow))),
        (2, RoomDef::new("Dormitory", Some(SystemType::Dormitory))),
        (3, RoomDef::new("Cargo", Some(SystemType::Cargo))),
        (4, RoomDef::new("Engine", Some(SystemType::Engine))),
        (5, RoomDef::new("Kitchen", Some(SystemType::Kitchen))),
        (6, RoomDef::new("Cannons", Some(SystemType::Cannons))),
        (7, RoomDef::new("Bridge", Some(SystemType::Bridge))),
        (8, RoomDef::new("Sickbay", Some(SystemType::Sickbay))),
        (9, RoomDef::new("Storage", Some(SystemType::Storage))),
    ];

    let mut rooms = HashMap::new();
    let hub_id = 0;

    for (id, def) in definitions {
        let mut neighbors = vec![];

        if id == hub_id {
            // Hub connects to all other rooms (1..9)
            for j in 1..=9 {
                neighbors.push(j);
            }
        } else {
            // Spoke connects only to Hub
            neighbors.push(hub_id);
        }

        rooms.insert(
            id,
            Room {
                id,
                name: def.name.to_string(),
                system: def.system,
                hazards: def.hazards,
                items: def.items,
                neighbors,
            },
        );
    }

    GameMap { rooms }
}

fn generate_torus() -> GameMap {
    // 12 Rooms: 9 Systems + 3 Empty
    // Ring Topology
    let definitions = vec![
        (0, RoomDef::new("The Bow", Some(SystemType::Bow))),
        (1, RoomDef::new("Dormitory", Some(SystemType::Dormitory))),
        (2, RoomDef::new("Corridor A", None)),
        (3, RoomDef::new("Cargo", Some(SystemType::Cargo))),
        (4, RoomDef::new("Engine", Some(SystemType::Engine))),
        (5, RoomDef::new("Kitchen", Some(SystemType::Kitchen))),
        (6, RoomDef::new("Corridor B", None)),
        (7, RoomDef::new("Cannons", Some(SystemType::Cannons))),
        (8, RoomDef::new("Bridge", Some(SystemType::Bridge))),
        (9, RoomDef::new("Sickbay", Some(SystemType::Sickbay))),
        (10, RoomDef::new("Storage", Some(SystemType::Storage))),
        (11, RoomDef::new("Corridor C", None)),
    ];

    let mut rooms = HashMap::new();
    let count = definitions.len() as u32;

    for (id, def) in definitions {
        let prev = if id == 0 { count - 1 } else { id - 1 };
        let next = (id + 1) % count;

        let neighbors = vec![prev, next];

        rooms.insert(
            id,
            Room {
                id,
                name: def.name.to_string(),
                system: def.system,
                hazards: def.hazards,
                items: def.items,
                neighbors,
            },
        );
    }

    GameMap { rooms }
}
