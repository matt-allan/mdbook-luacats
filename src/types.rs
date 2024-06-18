use std::{fmt, marker::PhantomData};
use serde::{de::{self, MapAccess, Visitor}, Deserialize, Deserializer, Serialize};

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

/// It's not worth making a real enum for this but it is an enumerated type.
type DefinitionType = String;

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
