use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Definition {
    pub desc: Option<String>,
    pub rawdesc: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub lua_type: DefinitionType,
    pub defines: Vec<Define>,
    // TODO: missing "fields"?
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DefinitionType {
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
    pub start: u64,
    pub finish: u64,
    #[serde(rename = "type")]
    pub lua_type: DefinitionType,
    pub file: String,
    // TODO: may be optional? 
    // https://github.com/LuaLS/lua-language-server/blob/85cb44556575f81a31267fe3c443822f3e97699e/script/cli/doc2md.lua#L23
    pub extends: Extend,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Extend {
    pub start: u64,
    pub finish: u64,
    #[serde(rename = "type")]
    pub lua_type: DefinitionType,
    pub view: String,
    pub desc: Option<String>,
    pub rawdesc: Option<String>,
    /// Only present for functions (type = "function") with args
    pub args: Option<Vec<FuncArg>>,
    /// Only present for functions (type = "function") with returns
    pub returns: Option<Vec<FuncReturn>>,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct FuncArg {
    /// The name is missing for varargs ("...")
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub lua_type: DefinitionType,
    pub view: String,
    pub start: u64,
    pub finish: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct FuncReturn {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub lua_type: DefinitionType,
    pub view: String,
}
