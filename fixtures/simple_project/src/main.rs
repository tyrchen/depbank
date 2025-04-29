use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    age: u32,
}

fn main() -> Result<()> {
    let person = Person {
        name: "John Doe".to_string(),
        age: 30,
    };

    println!("Person: {:?}", person);
    Ok(())
}
