use std::{
    collections::HashMap, path::PathBuf
};

use anyhow::{anyhow, Ok};
use itertools::Itertools;
use serde::Serialize;
use url::Url;

use crate::lua_cats::Definition;

/// A folder containing LuaCats definition files.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Default)]
pub struct Workspace {
    /// The absolute path to the root folder of the workspace.
    pub root: PathBuf,
    /// The workspace's meta files.
    pub files: Vec<MetaFile>,
}

impl Workspace {
    pub fn new<P: Into<PathBuf>>(p: P) -> Self {
        Self {
            root: p.into(),
            ..Default::default()
        }
    }

    /// Load the workspace from the given doc definitions.
    pub fn load(&mut self, docs: Vec<Definition>) -> anyhow::Result<()> {
        // Index definitions by their file in the order they were defined
        let mut defs_by_file: HashMap<PathBuf, Vec<(u64, Definition)>> = HashMap::new();

        for definition in docs.into_iter() {
            for define in definition.defines.iter() {
                let file_url = Url::parse(&define.file)?;
                let file_path = file_url
                    .to_file_path()
                    .map_err(|_| anyhow!("inalid file url"))?;
                defs_by_file
                    .entry(file_path)
                    .or_default()
                    .push((define.start, definition.clone()));
            }
        }

        let root = &self.root;
        let meta_files: Vec<MetaFile> = defs_by_file
            .into_iter()
            .filter_map(|(path, definitions)| {
                path.strip_prefix(root)
                    .ok() // discard definitions from outside the root (system definitions)
                    .map(|path| MetaFile::from((path.to_owned(), definitions)))
            })
            .sorted_by(|a, b| {
                a.depth
                    .cmp(&b.depth) // first by depth so we add parents before children
                    .then(a.file_name().cmp(&b.file_name())) // ...then alphabetically
            })
            .collect();

        for file in meta_files.into_iter() {
            self.add_file(file)
        }

        Ok(())
    }

    fn add_file(&mut self, file: MetaFile) {
        let depth = file.depth;

        // Top level files can be added directly to the workspace
        if depth == 0 {
            self.files.push(file);
            return;
        }

        // Otherwise we have to find the file's parent
        for other_file in self.files.iter_mut() {
            if other_file.depth == depth - 1 && other_file.file_stem() == file.directory_name().unwrap()
            {
                other_file.add_sub_file(file);
                return;
            }
        }

        // It's possible for a file to not have a direct parent on disk, but that
        // doesn't work with the mdbook chapter heirarchy so we just fallback to
        // adding those files to the root.
        log::warn!("No parent found for {}", file.file_name());
        self.files.push(file);
    }
}


/// A Lua file containing only LuaCats meta.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Default, Serialize)]
pub struct MetaFile {
    /// The file path, relative to the workspace root.
    pub path: PathBuf,
    /// Parsed definitions from this file.
    pub definitions: Vec<Definition>,
    /// The depth in the directory tree.
    pub depth: usize,
    /// Files considered below this one in the heirarchy.
    pub sub_files: Vec<MetaFile>,
}

impl MetaFile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn directory_name(&self) -> Option<String> {
        if self.depth == 0 {
            return None;
        }

        let dirname = self.path.parent().unwrap();

        Some(
            dirname
                .strip_prefix(dirname.parent().unwrap())
                .unwrap()
                .as_os_str()
                .to_string_lossy()
                .into_owned(),
        )
    }

    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }

    pub fn file_stem(&self) -> String {
        self.path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }

    pub fn add_sub_file(&mut self, file: MetaFile) {
        self.sub_files.push(file)
    }
}

impl From<(PathBuf, Vec<(u64, Definition)>)> for MetaFile {
    fn from(value: (PathBuf, Vec<(u64, Definition)>)) -> Self {
        let (path, definitions) = value;

        let depth = path.components().count() - 1;

        let definitions = definitions
            .into_iter()
            .sorted_by(|(a, _), (b, _)| a.cmp(b))
            .map(|(_, def)| def)
            .collect();

        MetaFile {
            path,
            definitions,
            depth,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod test {

    use crate::lua_cats::{Define, DefinitionType};

    use super::*;

    fn test_definition<U: Into<String>>(file: U) -> Definition {
        Definition {
        desc: None,
        rawdesc: None,
        name: "test".into(),
        lua_type: DefinitionType::Nil,
        fields: Vec::new(),
        defines: vec![Define {
            start: 0,
            finish: 10,
            lua_type: DefinitionType::Nil,
            file: file.into(),
            extends: Vec::new(),
        }],
        }
    }

    #[test]
    fn load_workspace() -> anyhow::Result<()> {
        let file_urls = vec![
            "file:///my/definitions/path/standard.lua",
            "file:///my/definitions/path/renoise.lua",
            "file:///my/definitions/path/renoise/midi.lua",
            "file:///my/definitions/path/bit.lua",
        ];

        let docs: Vec<Definition> = file_urls
            .iter()
            .map(|&file| test_definition(file))
            .collect();

        let mut ws = Workspace::new("/my/definitions/path");

        ws.load(docs)?;

        let root_files: Vec<String> = ws.files.iter()
            .map(|f| f.file_name())
            .collect();

        assert_eq!(root_files, vec!["bit.lua", "renoise.lua", "standard.lua"]);

        assert_eq!(ws.files.get(1).unwrap().sub_files.get(0).unwrap().file_name(), "midi.lua");

        Ok(())
    }
}
