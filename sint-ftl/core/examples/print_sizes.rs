use sint_core::types::*;
use std::mem::size_of;

fn main() {
    println!("Size of Player: {} bytes", size_of::<Player>());
    println!(
        "Size of FieldMap<Player>: {} bytes",
        size_of::<sint_core::field_map::FieldMap<Player>>()
    );
    println!("Size of GameState: {} bytes", size_of::<GameState>());
    println!("Size of Room: {} bytes", size_of::<Room>());
}
