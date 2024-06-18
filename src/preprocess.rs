use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

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

    fn run(&self, _ctx: &PreprocessorContext, book: Book) -> Result<Book, Error> {
        // TODO: implement
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html" || renderer == "epub"
    }
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
                        "nop": {}
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
