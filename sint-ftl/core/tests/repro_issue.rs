#[cfg(test)]
mod tests {
    use sint_core::{GameLogic, types::*};

    #[test]
    fn test_player_spawn_location() {
        let state = GameLogic::new_game(vec!["p1".to_owned()], 12345);
        let p1 = state.players.get("p1").unwrap();

        println!("Player 1 spawned in Room ID: {}", p1.room_id);

        let room = state.map.rooms.get(&p1.room_id).unwrap();
        println!("Room Name: {}", room.name);
        println!("Room System: {:?}", room.system);

        // Assertions
        assert_eq!(room.system, Some(SystemType::Dormitory));
        assert_eq!(room.name, "Dormitory");

        // Check ID assumptions
        // If logic.rs is as read, Dormitory should be ID 2.
        // If the user says they are in Cargo, maybe they are in ID 3?
        assert_eq!(p1.room_id, 2, "Expected Dormitory to be Room 2");

        let cargo_room = state
            .map
            .rooms
            .values()
            .find(|r| r.system == Some(SystemType::Cargo))
            .unwrap();
        println!("Cargo is Room ID: {}", cargo_room.id);
        assert_eq!(cargo_room.id, 3, "Expected Cargo to be Room 3");
    }
}
