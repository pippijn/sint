use schemars::schema_for;
use sint_core::Action;

fn main() {
    let schema = schema_for!(Action);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
