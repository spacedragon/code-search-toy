use std::path::{PathBuf, Path};

use tree_sitter::{Parser, Language};

use walkdir::{DirEntry, WalkDir};
use rayon::prelude::*;
use std::error::Error;
use anyhow::{Result, Context};
use ignore::gitignore::Gitignore;
use ignore::Match::Ignore;
use tracing::{span, info, Level, event};

// extern "C" { fn tree_sitter_javascript() -> Language; }
extern "C" {
    fn tree_sitter_typescript() -> Language;
}

extern "C" { fn tree_sitter_tsx() -> Language; }

pub struct Indexer {
    path: PathBuf
}


impl Indexer {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path
        }
    }

    pub fn run(&self) -> Result<()> {
        let span = span!(Level::DEBUG, "indexer run");
        let _enter = span.enter();
        info!("indexing path: {}", self.path.to_string_lossy());
        let root_path = self.path.canonicalize()?;

        let path_to_gitignore = root_path.join(".gitignore");
        let mut gitignore = Gitignore::new(path_to_gitignore).0;

        let walkdir = WalkDir::new(&root_path);
        let iter = walkdir.into_iter()
            .filter_entry(|dir| {
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
                return true
            });
        iter.par_bridge().for_each(|e| {
            if let Ok(entry) = e {
                if !is_hidden(&entry) {
                    let path = entry.path();
                    if path.is_file() {
                        handle_file(path).expect(format!("parse {:?} failed", path).as_str());
                    }
                }
            } else {
                eprintln!("{}", e.err().unwrap().description());
            }
        });
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

fn handle_file(path: &Path) -> Result<()> {
    if let Some(ext) = get_ext(path) {
        let langOpt = match ext {
            "ts" => Some(unsafe { tree_sitter_typescript() }),
            "js" => Some(unsafe { tree_sitter_typescript() }),
            "tsx" => Some(unsafe { tree_sitter_tsx() }),
            _ => None
        };
        if let Some(lang) = langOpt {

            let span = span!(Level::DEBUG, "parsing file", path = ?path);
            let _enter = span.enter();
            let mut parser = Parser::new();
            parser.set_language(lang).expect("set language failed");
            let source_code = std::fs::read_to_string(path)?;
            let tree = parser.parse(source_code, None).context("parse failed")?;
            // println!("parsed source {:?}", path)
            // let root_node = tree.root_node();
            // println!("{}", root_node.to_sexp());
        }
    }
    Ok(())
}

fn get_ext(path: &Path) -> Option<&str> {
    return path.extension()?.to_str();
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}