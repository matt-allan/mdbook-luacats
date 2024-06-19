use itertools::Itertools;
use mdbook::{book::Book, preprocess::{Preprocessor, PreprocessorContext}};
use mdbook::errors::Error as MdBookError;
use std::path::PathBuf;
use toml::value::Table;

use crate::{lua_cats::Workspace, luals::generate_docs};


/// Configuration for the preprocessor.
#[derive(Debug, Default)]
pub struct Config {
    definitions_path: Option<PathBuf>,
    part_title: Option<String>,
    nav_depth: Option<u8>,
}

impl<'a> From<Option<&'a Table>> for Config {
    fn from(table: Option<&'a Table>) -> Config {
        let mut config = Config::default();

        if let Some(table) = table {
            config.definitions_path = table
                .get("definitions-path")
                .and_then(|v| v.as_str())
                .and_then(|v| Some(v.to_owned().into()));

            config.part_title = table
                .get("part-title")
                .and_then(|v| v.as_str())
                .and_then(|v| Some(v.to_owned()))            ;

            config.nav_depth = table
                .get("nav-depth")
                .and_then(|v| v.as_integer())
                .and_then(|v| Some(v.try_into().expect("nav-depth overflow")));
        }

        config
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
        let config: Config = ctx.config.get_preprocessor(self.name()).into();

        let root_path = config.definitions_path
            .unwrap_or_else(|| PathBuf::from(ctx.root.join("library")))
            .canonicalize()?;

        let docs = generate_docs(&root_path)?;

        let mut workspace = Workspace::new(root_path);

        workspace.add_definitions(docs)?;

        let part_title = config.part_title.unwrap_or("API Reference".into());

        // // Group by file / depth
        // // Generate a table of contents

        // // book.for_each_mut(|section: &mut BookItem| {
        // //     // 
        // // });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html" || renderer == "epub"
    }
}
