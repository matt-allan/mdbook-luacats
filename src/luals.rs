use std::{fs::{self}, path::PathBuf, process::Command};
use anyhow::{anyhow, Error};
use tempdir::TempDir;
use crate::lua_cats::Definition;

/// Spawn the lua-language-server to generate docs.
pub fn generate_docs(definitions_path: &PathBuf) -> Result<Vec<Definition>,Error> { 
    let tmp_dir = TempDir::new("luals-docs")?;
    let tmp_path = tmp_dir.path();

    let output = Command::new("lua-language-server")
        .arg("--doc")
        .arg(definitions_path)
        .arg("--doc_out_path")
        .arg(tmp_path)
        .arg("--logpath")
        .arg(tmp_path)
        .output()?;

    if !output.status.success() {
        let err = match output.status.code() {
            Some(code) => anyhow!("LuaLS process exited with status code {}", code),
            None => anyhow!("LuaLS process terminated by signal"),
        };
        return Err(err)
    }

    let json_doc_path = tmp_dir.path().join("doc.json");

    let json_doc = fs::read_to_string(json_doc_path)?;

    let definitions: Vec<Definition> = serde_json::from_str(&json_doc)?;

    Ok(definitions)
}