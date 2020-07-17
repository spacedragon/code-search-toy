mod indexer;

use self::indexer::IndexerWriter;
use anyhow::{Context, Result};

use ignore::gitignore::Gitignore;
use ignore::Match::Ignore;
use rayon::prelude::*;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::str;
use tracing::{error, event, info, span, Level};
use tree_sitter::{Language, Parser};
use walkdir::{DirEntry, WalkDir};

// extern "C" { fn tree_sitter_javascript() -> Language; }
extern "C" {
    fn tree_sitter_typescript() -> Language;
}

extern "C" {
    fn tree_sitter_tsx() -> Language;
}

pub struct IndexerRunner {
    path: PathBuf,
}

impl IndexerRunner {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn run(&self) -> Result<()> {
        let span = span!(Level::DEBUG, "indexer run");
        let _enter = span.enter();
        info!("indexing project: {}", self.path.to_string_lossy());
        let index_path = PathBuf::from("./index");
        info!("write index to: {:?}", index_path);
        let indexer = IndexerWriter::new(&index_path)?;
        let root_path = self.path.canonicalize()?;

        let path_to_gitignore = root_path.join(".gitignore");
        let gitignore = Gitignore::new(path_to_gitignore).0;

        let walkdir = WalkDir::new(&root_path);
        let iter = walkdir.into_iter().filter_entry(|dir| {
            if dir.path().eq(&root_path) {
                return true;
            }
            let path = dir.path();
            if let Ignore(_) = gitignore.matched(path, path.is_dir()) {
                event!(Level::DEBUG, path = ?path, "gitignore file");
                return false;
            };
            if is_hidden(dir) {
                event!(Level::DEBUG, path = ?path, "ignore hidden file");
                return false;
            };
            return true;
        });
        iter.par_bridge().for_each(|e| {
            if let Ok(entry) = e {
                if !is_hidden(&entry) {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(Some(source)) = handle_file(path) {
                            let source = String::from_utf8_lossy(&source);
                            if indexer.index_doc(path, &source).is_err() {
                                error!("index doc {:?} failed", path);
                            };
                        }
                    }
                }
            } else {
                error!("{}", e.err().unwrap().description());
            }
        });
        indexer.commit()?;
        Ok(())

        // println!("indexing path: {}", self.path.to_string_lossy());
        // let mut parser = Parser::new();
        // let language = unsafe { tree_sitter_typescript() };
        // parser.set_language(language).unwrap();
        //
        // let source_code = "const a = 1 + 2";
        // let tree = parser.parse(source_code, None).unwrap();
        // let root_node = tree.root_node();
        // println!("{}", root_node.to_sexp());
    }
}

fn handle_file(path: &Path) -> Result<Option<Vec<u8>>> {
    if let Some(ext) = get_ext(path) {
        let lang_opt = match ext {
            "ts" => Some(unsafe { tree_sitter_typescript() }),
            "js" => Some(unsafe { tree_sitter_typescript() }),
            "tsx" => Some(unsafe { tree_sitter_tsx() }),
            _ => None,
        };
        if let Some(lang) = lang_opt {
            let span = span!(Level::DEBUG, "parsing file", path = ?path);
            let _enter = span.enter();
            let mut parser = Parser::new();
            parser.set_language(lang).expect("set language failed");
            let source_code = std::fs::read(path)?;
            let _tree = parser.parse(&source_code, None).context("parse failed")?;
            // println!("parsed source {:?}", path)
            // let root_node = tree.root_node();
            // println!("{}", root_node.to_sexp());
            return Ok(Some(source_code));
        }
    }
    Ok(None)
}

fn get_ext(path: &Path) -> Option<&str> {
    return path.extension()?.to_str();
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}
