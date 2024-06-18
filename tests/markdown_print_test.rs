use std::error::Error;
use mdbook_luacats::{types::Definition, print::MarkdownPrinter};

#[test]
fn print_definition() -> Result<(), Box<dyn Error>> {
    // A subset of doc.json
    let input_json = r##"{
        "defines": [
            {
                "extends": {
                    "args": [],
                    "desc": "Say hello",
                    "finish": 30020,
                    "rawdesc": "Say hello",
                    "start": 30000,
                    "type": "function",
                    "view": "function hello()"
                },
                "file": "file:///Users/matt/Code/luacats-doc/./test_doc/hello/hello.lua",
                "finish": 30014,
                "start": 30009,
                "type": "setglobal"
            }
        ],
        "desc": "Say hello",
        "name": "hello",
        "rawdesc": "Say hello",
        "type": "variable"
    }"##;

    let def: Definition = serde_json::from_str(input_json)?;

    let printer = MarkdownPrinter::default();

    let md = printer.print_definition(&def)?;

    let want = r##"## hello

Say hello

```lua
function hello()
```

"##;

    assert_eq!(md, want);

    Ok(())
}