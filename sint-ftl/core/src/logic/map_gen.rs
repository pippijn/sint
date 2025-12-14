use crate::types::{GameMap, HazardType, ItemType, MapLayout, Room, RoomName, SystemType};

pub struct RoomDef {
    pub name: RoomName,
    pub system: Option<SystemType>,
    pub items: Vec<ItemType>,
    pub hazards: Vec<HazardType>,
}

impl RoomDef {
    fn new(name: RoomName, system: Option<SystemType>) -> Self {
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
        (0, RoomDef::new(RoomName::CentralHallway, None)),
        (1, RoomDef::new(RoomName::Bow, Some(SystemType::Bow))),
        (
            2,
            RoomDef::new(RoomName::Dormitory, Some(SystemType::Dormitory)),
        ),
        (3, RoomDef::new(RoomName::Cargo, Some(SystemType::Cargo))),
        (4, RoomDef::new(RoomName::Engine, Some(SystemType::Engine))),
        (
            5,
            RoomDef::new(RoomName::Kitchen, Some(SystemType::Kitchen)),
        ),
        (
            6,
            RoomDef::new(RoomName::Cannons, Some(SystemType::Cannons)),
        ),
        (7, RoomDef::new(RoomName::Bridge, Some(SystemType::Bridge))),
        (
            8,
            RoomDef::new(RoomName::Sickbay, Some(SystemType::Sickbay)),
        ),
        (
            9,
            RoomDef::new(RoomName::Storage, Some(SystemType::Storage)),
        ),
    ];

    let hub_id = 0;

    let rooms = definitions
        .into_iter()
        .map(|(id, def)| {
            let neighbors = if id == hub_id {
                // Hub connects to all other rooms (1..9)
                (1..=9).collect()
            } else {
                // Spoke connects only to Hub
                vec![hub_id]
            };

            (
                id,
                Room {
                    id,
                    name: def.name,
                    system: def.system,
                    system_health: crate::types::SYSTEM_HEALTH,
                    is_broken: false,
                    hazards: def.hazards.into(),
                    items: def.items.into(),
                    neighbors: neighbors.into(),
                },
            )
        })
        .collect();

    GameMap { rooms }
}

fn generate_torus() -> GameMap {
    // 12 Rooms: 9 Systems + 3 Empty
    // Ring Topology
    let definitions = vec![
        (0, RoomDef::new(RoomName::Bow, Some(SystemType::Bow))),
        (
            1,
            RoomDef::new(RoomName::Dormitory, Some(SystemType::Dormitory)),
        ),
        (2, RoomDef::new(RoomName::CorridorA, None)),
        (3, RoomDef::new(RoomName::Cargo, Some(SystemType::Cargo))),
        (4, RoomDef::new(RoomName::Engine, Some(SystemType::Engine))),
        (
            5,
            RoomDef::new(RoomName::Kitchen, Some(SystemType::Kitchen)),
        ),
        (6, RoomDef::new(RoomName::CorridorB, None)),
        (
            7,
            RoomDef::new(RoomName::Cannons, Some(SystemType::Cannons)),
        ),
        (8, RoomDef::new(RoomName::Bridge, Some(SystemType::Bridge))),
        (
            9,
            RoomDef::new(RoomName::Sickbay, Some(SystemType::Sickbay)),
        ),
        (
            10,
            RoomDef::new(RoomName::Storage, Some(SystemType::Storage)),
        ),
        (11, RoomDef::new(RoomName::CorridorC, None)),
    ];

    let count = definitions.len() as u32;

    let rooms = definitions
        .into_iter()
        .map(|(id, def)| {
            let prev = if id == 0 { count - 1 } else { id - 1 };
            let next = (id + 1) % count;
            let neighbors = vec![prev, next];

            (
                id,
                Room {
                    id,
                    name: def.name,
                    system: def.system,
                    system_health: crate::types::SYSTEM_HEALTH,
                    is_broken: false,
                    hazards: def.hazards.into(),
                    items: def.items.into(),
                    neighbors: neighbors.into(),
                },
            )
        })
        .collect();

    GameMap { rooms }
}
