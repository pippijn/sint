mod bindings;
pub mod logic;
pub mod types;
pub mod small_map;

pub use logic::{GameError, GameLogic};
pub use types::*;

#[cfg(feature = "schema")]
pub fn export_schema() -> String {
    let schema = schemars::schema_for!(types::Action);
    serde_json::to_string_pretty(&schema).unwrap()
}
