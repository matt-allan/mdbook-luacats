use std::error::Error;
use luacats_doc::types::Definition;

#[test]
fn parse_json() -> Result<(), Box<dyn Error>> {
    let input_json = include_str!("./test_doc/hello/doc.json");

    let _nodes: Vec<Definition> = serde_json::from_str(input_json)?;

    Ok(())
}