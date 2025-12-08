pub mod behavior;
pub mod deck;
pub mod impls;
pub mod registry;

pub use behavior::CardBehavior;
pub use deck::{draw_card, initialize_deck};
pub use registry::get_behavior;
