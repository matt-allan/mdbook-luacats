use mdbook::{book::{Book, Chapter, SectionNumber}, preprocess::{Preprocessor, PreprocessorContext}, BookItem};
use mdbook::errors::Error as MdBookError;
use std::{env, path::PathBuf};
use toml::value::Table;
use log::*;

use crate::{luals::generate_docs, print::{MarkdownOptions, MarkdownPrinter}, workspace::{MetaFile, Workspace}};


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
                .and_then(|v| Some(v.to_owned()));

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

        debug!("Using mdbook root: {:?}", ctx.root);
        debug!("Using definitions path: {:?}", config.definitions_path);

        let mut root = ctx.root.clone();
        if root.is_relative() {
            root = env::current_dir()?.join(ctx.root.clone())
        }
        let mut root_path = config.definitions_path
            .unwrap_or_else(|| PathBuf::from("library"));
        if root_path.is_relative() {
            root_path = root.join(root_path);
        }
        debug!("Using root path: {:?}", root_path);

        let docs = generate_docs(&root_path)?;
        debug!("Generated {} definitions", docs.len());

        let mut workspace = Workspace::new(root_path);
        workspace.load(docs)?;
        debug!("Loaded {} root files", workspace.files.len());

        let part_title = config.part_title.unwrap_or("API Reference".into());
        book.push_item(BookItem::PartTitle(part_title));
        
        for (index, file) in workspace.files.iter().enumerate() {
            let chapter = build_chapter(file, index, None)?;
            book.push_item(BookItem::Chapter(chapter));
         }

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html" || renderer == "epub"
    }
}

fn build_chapter(file: &MetaFile, index: usize, parent: Option<&Chapter>) -> anyhow::Result<Chapter> {
    let name = file.file_stem(); // todo: get from first def if possible
    // todo: replace with hbars
    let content = MarkdownPrinter::new(MarkdownOptions::default()).print(&file.definitions)?;
    let md_path = file.path.with_extension("md");
    let number = match parent {
        Some(parent) => {
            let mut number = parent.number.clone().unwrap_or_else(|| SectionNumber(Vec::new()));
            number.0.push(u32::try_from(index).unwrap()+1);
            number
        },
        None => SectionNumber(vec![u32::try_from(index).unwrap()+1])
    };
    let parent_names = match parent {
        Some(parent) => {
            let mut names = parent.parent_names.clone();
            names.push(parent.name.clone());
            names
        },
        None => Vec::new(),
    };

    let mut chapter = Chapter {
        name,
        content,
        number: Some(number),
        sub_items: Vec::new(),
        path: Some(md_path),
        source_path: None,
        parent_names,
    };

    let mut sub_items = Vec::with_capacity(file.sub_files.len());
    for (sub_index, sub_file) in file.sub_files.iter().enumerate() {
        let sub_item = build_chapter(sub_file, sub_index, Some(&chapter))?;
        sub_items.push(BookItem::Chapter(sub_item));
    }

    chapter.sub_items = sub_items;

    Ok(chapter)
}

#[cfg(test)]
mod test {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn preprocessor_run() {
        init();

        let input_json = r##"[
            {
                "root": "../test_book",
                "config": {
                    "book": {
                        "authors": ["AUTHOR"],
                        "language": "en",
                        "multilingual": false,
                        "src": "src",
                        "title": "TITLE"
                    },
                    "preprocessor": {
                        "luacats": {
                          "definitions-path": "library" 
                        }
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
        let result = LuaCats::new().run(&ctx, book);
        assert!(result.is_ok(), "preprocessor failed: {:#?}", result.err());

        let actual_book = result.unwrap();

        // TODO: better asserts
        assert_eq!(actual_book.sections.len(), 2); // Chapter 1, Chapter "hello"
    }
}
