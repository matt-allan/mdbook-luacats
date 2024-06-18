use clap::{Arg, Command};
use mdbook_luacats::{
    clean_docs, generate_docs, print::MarkdownPrinter
};
use std::{io::{self, Write}, path::PathBuf};

pub fn make_app() -> Command {
    Command::new("luacats-to-markdown")
        .about("Generate markdown API docs from luaCATS type definitions")
        .arg(Arg::new("path").required(true).help("Path to the lua definitions"))
}

fn main() -> anyhow::Result<()> {
    let matches = make_app().get_matches();

    let definitions_path = matches
        .get_one::<String>("path")
        .expect("required argument");
    let definitions_path = PathBuf::from(definitions_path).canonicalize()?;

    let docs = generate_docs(&definitions_path)?;

    let docs = clean_docs(&definitions_path, docs);

    let printer = MarkdownPrinter::default();

    let md = printer.print(&docs[..])?;

    io::stdout().write_all(md.as_bytes())?;

    Ok(())
}