//!LuaCATS documentation types.
use std::{collections::BTreeMap, fmt, marker::PhantomData, path::{Component, PathBuf}};
use itertools::Itertools;
use serde::{de::{self, MapAccess, Visitor}, Deserialize, Deserializer, Serialize};

use crate::error::Error;

/// A folder containing LuaCats type definitions.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Workspace {
    /// Absolute path to the root of the workspace.
    pub root_path: PathBuf,
    /// The files within the workspace.
    pub files: BTreeMap<PathBuf,MetaFile>,
}

impl Workspace {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root_path,
            files: BTreeMap::new(),
        }
    }

    pub fn add_definitions(&mut self, definitions: Vec<Definition>) -> Result<(), Error> {
        for definition in definitions.into_iter() {
            let file_path = definition.file_path();
            if file_path.is_none() {
                continue;
            }
            let file_path = file_path.unwrap();

            let rel_path = file_path.strip_prefix(&self.root_path);
            if rel_path.is_err() {
                continue;
            }
            let rel_path = rel_path.unwrap().to_path_buf();

            let file = self.files.entry(rel_path.clone())
                .or_insert_with(|| MetaFile {
                    path: rel_path.clone(),
                    definitions: Vec::new(),
                });
            file.definitions.push(definition)
        }

        Ok(())
    }

    /// Returns the workspace paths, sorted by depth then name.
    pub fn paths(&self) -> Vec<PathBuf> {
        self.files
            .keys()
            .cloned()
            .map(|path| {
                let depth = {
                    let components = path.components().collect::<Vec<_>>();
                    components.len()
                };

                (depth, path)
            })
            .sorted_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)))
            .map(|(_depth, path)| path)
            .collect()
    }
}

impl Iterator for Workspace {
    type Item = MetaFile;

    fn next(&mut self) -> Option<Self::Item> {
        // Sort by depth, then by name
        let paths: Vec<PathBuf> = self.files
            .keys()
            .cloned()
            .map(|path| {
                let depth = {
                    let components = path.components().collect::<Vec<_>>();
                    components.len()
                };

                (depth, path)
            })
            .sorted_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)))
            .map(|(_depth, path)| path)
            .collect();

        todo!()
    }
}

/// A meta (`@meta`) file containing LuaCats type definitions.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MetaFile {
    /// The path to the file, relative to the workspace root.
    pub path: PathBuf,
    /// The definitions from the file, sorted by location.
    pub definitions: Vec<Definition>,
}

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

impl Definition {
    pub fn file_path(&self) -> Option<PathBuf> {
        let loc = self.location();
        if loc.is_none() {
            return None
        }

        let (file_uri, _) = loc.unwrap();
        let file = file_uri.strip_prefix("file://").unwrap_or(&file_uri);
        Some(PathBuf::from(file))
    }

    pub fn location(&self) -> Option<(String, u64)> {
        self.defines.first().map(|def| (def.file.clone(), def.start))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DefinitionType {
    Binary,
    // TODO: this is probably not yet exhaustive
    #[serde(rename = "doc.alias")]
    DocAlias,
    #[serde(rename = "doc.class")]
    DocClass,
    #[serde(rename = "doc.extends.name")]
    DocExtendsName,
    #[serde(rename = "doc.enum")]
    DocEnum,
    #[serde(rename = "doc.type")]
    DocType,
    Function,
    #[serde(rename = "function.return")]
    FunctionReturn,
    Integer,
    Local,
    Nil,
    Number,
    #[serde(rename = "self")]
    SelfType,
    SetField,
    SetGlobal,
    SetMethod,
    String,
    Table,
    TableField,
    Type,
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
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_extends")]
    pub extends: Vec<Extend>,
}

/// Implement the value of "extends", which may be missing, null, an array
/// of maps, or a single map. We always deserialize into a vector of maps (which
/// may be empty) for consistency.
fn deserialize_extends<'de, D>(deserializer: D) -> Result<Vec<Extend>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ExtendData(PhantomData<fn() -> Vec<ExtendData>>);

    impl<'de> Visitor<'de> for ExtendData
    {
        type Value = Vec<Extend>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("array or map or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error, { 
            Ok(Vec::new())
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>, { 
            Ok(Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))?)
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            Ok(vec![Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))?])
        }
    }

    deserializer.deserialize_any(ExtendData(PhantomData))
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
