use std::{fs::{self}, io, path::PathBuf, process::Command};
use itertools::Itertools;
use thiserror::Error;
use tempdir::TempDir;
use crate::types::Definition;

#[derive(Debug, Error)]
pub enum LuaLsError {
    #[error("file io error")]
    Io(#[from] io::Error),
    #[error("failed to execute lua-language-server")]
    Exec,
    #[error("unable to parse doc JSON")]
    JsonError(#[from] serde_json::Error),
}

pub fn generate_docs(definitions_path: &PathBuf) -> Result<Vec<Definition>,LuaLsError> { 
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
        return Err(LuaLsError::Exec)
    }

    let json_doc_path = tmp_dir.path().join("doc.json");

    let json_doc = fs::read_to_string(json_doc_path)?;

    let definitions: Vec<Definition> = serde_json::from_str(&json_doc)?;

    Ok(definitions)
}

pub fn clean_docs(path: &PathBuf, docs: Vec<Definition>) -> Vec<Definition> {
    // Exclude types that weren't defined locally
    let docs: Vec<Definition> = docs 
        .into_iter()
        .filter(|def| def 
                .defines
                .iter()
                .any(|define| {
                    let file_uri = define.file.clone();
                    let file = file_uri.strip_prefix("file://").unwrap_or(&file_uri);
                    let file_path = PathBuf::from(file);

                    file_path.starts_with(path)
                })
        )
        // Sort by file and location
        .sorted_by(|a, b| {
            let a_def= a.defines.first().expect("missing define");
            let b_def = b.defines.first().expect("missing define");

            a_def.file.cmp(&b_def.file).then(a_def.start.cmp(&b_def.start))
        })
        .collect();

    docs
}