use std::fmt::{self, Write};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use crate::lua_cats::Definition;

#[derive(Debug, Default)]
pub struct MarkdownPrinter {
    options: MarkdownOptions,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct MarkdownOptions {
    /// Starting heading level
    heading_level: Option<u8>,
}

impl MarkdownPrinter {
    pub fn new(options: MarkdownOptions) -> Self {
        Self {
            options,
        }
    }

    pub fn print(&self, definitions: &[Definition]) -> Result<String, fmt::Error> {
        let chunks: Result<Vec<String>, fmt::Error> = definitions
            .iter()
            .map(|def| self.print_definition(def))
            .collect();

        return chunks.map(|chunks| chunks.iter().join("\n"))
    }

    pub fn print_definition(&self, node: &Definition) -> Result<String, fmt::Error> {
        let heading_level: usize = self.options.heading_level.unwrap_or(2).into();

        let mut str = String::new();

        write!(&mut str, "{} {}\n\n", "#".repeat(heading_level), node.name)?;

        if let Some(desc) = &node.desc {
            write!(&mut str, "{}\n\n", desc)?;
        }

        for def in node.defines.iter() {
            for extend in def.extends.iter() {
                write!(&mut str, "```lua\n{}\n```\n\n", extend.view)?;
            }
        }

        return Ok(str)
    }
}