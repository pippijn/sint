mod bindings;
pub mod field_map;
pub mod logic;
pub mod small_map;
pub mod types;

pub use logic::{GameError, GameLogic};
pub use small_map::{SmallMap, SmallSet};
pub use types::*;

pub fn export_schema() -> String {
    #[derive(schemars::JsonSchema)]
    #[allow(dead_code)]
    struct FullSchema {
        state: types::GameState,
        action: types::Action,
    }
    let schema = schemars::schema_for!(FullSchema);
    serde_json::to_string_pretty(&schema).unwrap()
}
