pub mod print;
pub mod types;

use std::convert::Infallible;
use std::{fs::{self}, io, path::PathBuf, process::Command};
use itertools::Itertools;
use tempdir::TempDir;
use crate::types::Definition;
use mdbook::book::{Book, Chapter};
use mdbook::errors::Error as MdBookError;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::BookItem;
use toml::value::Table;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Missing required configuration `{0}`")]
    MissingConfig(String),
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("failed to execute lua-language-server")]
    Exec,
    #[error("unable to parse doc JSON")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Debug, Default)]
pub struct Config {
    definitions_path: Option<String>,
    part_title: Option<String>,
    nav_depth: Option<u8>,
}

impl<'a> TryFrom<Option<&'a Table>> for Config {
    type Error = Infallible;

    fn try_from(table: Option<&'a Table>) -> Result<Self,Infallible> { 
        let mut config = Config::default();

        if let Some(table) = table {
            config.definitions_path = table
                .get("part-title")
                .and_then(|v| v.as_str())
                .and_then(|v| Some(v.to_owned()));

            config.definitions_path = table
                .get("definitions-path")
                .and_then(|v| v.as_str())
                .and_then(|v| Some(v.to_owned()))            ;

            config.nav_depth = table
                .get("nav-depth")
                .and_then(|v| v.as_integer())
                .and_then(|v| Some(v.try_into().expect("integer overflow")));
        }

        Ok(config)
    }    
}

/// A mdbook preprocessor that generates LuaCATS API docs.
pub struct LuaCats;

impl LuaCats {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LuaCats {
    fn default() -> Self {
        Self {}
    }
}

impl Preprocessor for LuaCats {
    fn name(&self) -> &str {
        "luacats-preprocessor"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, MdBookError> {
        let config: Config = ctx.config.get_preprocessor(self.name()).try_into().unwrap();

        let definitions_path = config.definitions_path
            .ok_or_else(|| Error::MissingConfig(String::from("definitions-path")))?;
        let definitions_path = PathBuf::from(definitions_path).canonicalize()?;

        let docs = generate_docs(&definitions_path)?;
        let docs = clean_docs(&definitions_path, docs);

        let part_title = config.part_title.unwrap_or("API Reference".into());

        // Group by file / depth
        // Generate a table of contents

        // book.for_each_mut(|section: &mut BookItem| {
        //     // 
        // });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html" || renderer == "epub"
    }
}

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
        return Err(Error::Exec)
    }

    let json_doc_path = tmp_dir.path().join("doc.json");

    let json_doc = fs::read_to_string(json_doc_path)?;

    // For debug!
    print!("{}", json_doc);

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn nop_preprocessor_run() {
        let input_json = r##"[
            {
                "root": "/path/to/book",
                "config": {
                    "book": {
                        "authors": ["AUTHOR"],
                        "language": "en",
                        "multilingual": false,
                        "src": "src",
                        "title": "TITLE"
                    },
                    "preprocessor": {
                        "luacats": {}
                    }
                },
                "renderer": "html",
                "mdbook_version": "0.4.21"
            },
            {
                "sections": [
                    {
                        "Chapter": {
                            "name": "Chapter 1",
                            "content": "# Chapter 1\n",
                            "number": [1],
                            "sub_items": [],
                            "path": "chapter_1.md",
                            "source_path": "chapter_1.md",
                            "parent_names": []
                        }
                    }
                ],
                "__non_exhaustive": null
            }
        ]"##;
        let input_json = input_json.as_bytes();

        let (ctx, book) = mdbook::preprocess::CmdPreprocessor::parse_input(input_json).unwrap();
        let expected_book = book.clone();
        let result = LuaCats::new().run(&ctx, book);
        assert!(result.is_ok());

        let actual_book = result.unwrap();
        // TODO: assert changes
        assert_eq!(actual_book, expected_book);
    }
}