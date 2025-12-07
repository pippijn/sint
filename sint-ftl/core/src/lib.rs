pub mod types;
pub mod logic;
mod bindings;

pub use types::*;

#[cfg(feature = "schema")]
pub fn export_schema() -> String {
    let schema = schemars::schema_for!(types::Action);
    serde_json::to_string_pretty(&schema).unwrap()
}