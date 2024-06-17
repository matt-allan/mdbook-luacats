use std::fmt::Write;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocNode {
    desc: Option<String>,
    rawdesc: Option<String>,
    name: String,
    #[serde(rename = "type")]
    lua_type: LuaType,
    defines: Vec<Define>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct MarkdownOptions {
    /// Starting heading level
    heading_level: Option<u8>,
}

impl DocNode {
    /// Render the node to a markdown string.
    pub fn markdown(&self, options: Option<MarkdownOptions>) -> anyhow::Result<String> {
        let options = options.unwrap_or_default();
        let heading_level: usize = options.heading_level.unwrap_or(2).into();

        let mut str = String::new();

        write!(&mut str, "{} {}\n\n", "#".repeat(heading_level), self.name)?;

        if let Some(desc) = &self.desc {
            write!(&mut str, "{}\n\n", desc)?;
        }

        for def in self.defines.iter() {
            write!(&mut str, "```lua\n{}\n```\n\n", def.extends.view)?;
        }

        return Ok(str)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LuaType {
    // TODO: this is not yet exhaustive
    Function,
    #[serde(rename = "function.return")]
    FunctionReturn,
    Local,
    SetField,
    SetGlobal,
    String,
    Table,
    Variable,
    #[serde(rename = "...")]
    VarArg,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Define {
    start: u64,
    finish: u64,
    #[serde(rename = "type")]
    lua_type: LuaType,
    file: String,
    extends: Extend,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Extend {
    start: u64,
    finish: u64,
    #[serde(rename = "type")]
    lua_type: LuaType,
    view: String,
    desc: Option<String>,
    rawdesc: Option<String>,
    /// Only present for functions (type = "function") with args
    args: Option<Vec<FuncArg>>,
    /// Only present for functions (type = "function") with returns
    returns: Option<Vec<FuncReturn>>,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct FuncArg {
    /// The name is missing for varargs ("...")
    name: Option<String>,
    #[serde(rename = "type")]
    lua_type: LuaType,
    view: String,
    start: u64,
    finish: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct FuncReturn {
    name: Option<String>,
    #[serde(rename = "type")]
    lua_type: LuaType,
    view: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_json() -> anyhow::Result<()> {
        // TODO: not valid on windows?
        let input_json = include_str!("../test_doc/hello/doc.json");

        let _nodes: Vec<DocNode> = serde_json::from_str(input_json)?;

        Ok(())
    }

    #[test]
    fn markdown() -> anyhow::Result<()> {
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

        let node: DocNode = serde_json::from_str(input_json)?;

        let md = node.markdown(None)?;

        let want = r##"## hello

Say hello

```lua
function hello()
```

"##;

        assert_eq!(md, want);

        Ok(())
    }
}
