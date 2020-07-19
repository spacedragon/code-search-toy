extern crate cc;

use std::path::PathBuf;
use std::{env, fs};

const TREE_SITTER: &str = "tree-sitter-";

fn get_opt_level() -> u32 {
    env::var("OPT_LEVEL").unwrap().parse::<u32>().unwrap()
}

fn get_debug() -> bool {
    env::var("DEBUG").unwrap() == "true"
}

fn get_cwd() -> &'static str {
    if is_enums() {
        ".."
    } else {
        "."
    }
}

fn is_enums() -> bool {
    let path = env::current_dir().unwrap();
    path.ends_with("language-gen")
}

fn collect_tree_sitter_dirs(ignore: Vec<String>) -> Vec<String> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(get_cwd()).unwrap() {
        if let Ok(entry) = entry {
            let path = entry.path();
            let dir = path.file_name().unwrap().to_str().unwrap().to_string();
            if dir.starts_with(TREE_SITTER) && !ignore.contains(&dir) {
                dirs.push(dir);
            }
        }
    }
    dirs
}

fn collect_src_files(dir: &str) -> (Vec<String>, Vec<String>) {
    eprintln!("Collect files for {}", dir);

    let mut c_files = Vec::new();
    let mut cpp_files = Vec::new();
    let path = PathBuf::from(get_cwd()).join(&dir).join("src");
    for entry in fs::read_dir(path).unwrap() {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("binding")
            {
                continue;
            }
            if let Some(ext) = path.extension() {
                if ext == "c" {
                    c_files.push(path.to_str().unwrap().to_string());
                } else if ext == "cc" || ext == "cpp" || ext == "cxx" {
                    cpp_files.push(path.to_str().unwrap().to_string());
                }
            }
        }
    }
    (c_files, cpp_files)
}

fn build_c(files: Vec<String>, language: &str) {
    let mut build = cc::Build::new();
    for file in files {
        build
            .file(&file)
            .include(PathBuf::from(file).parent().unwrap())
            .pic(true)
            .opt_level(get_opt_level())
            .debug(get_debug())
            .warnings(false)
            .flag_if_supported("-std=c99");
    }
    build.compile(&format!("tree-sitter-{}-c", language));
}

fn build_cpp(files: Vec<String>, language: &str) {
    let mut build = cc::Build::new();
    for file in files {
        build
            .file(&file)
            .include(PathBuf::from(file).parent().unwrap())
            .pic(true)
            .opt_level(get_opt_level())
            .debug(get_debug())
            .warnings(false)
            .cpp(true);
    }
    build.compile(&format!("tree-sitter-{}-cpp", language));
}

fn build_dir(dir: &str, language: &str) {
    println!("Build language {}", language);
    if PathBuf::from(get_cwd())
        .join(dir)
        .read_dir()
        .unwrap()
        .next()
        .is_none()
    {
        eprintln!(
            "The directory {} is empty, did you use 'git clone --recursive'?",
            dir
        );
        eprintln!("You can fix in using 'git submodule init && git submodule update --recursive'.");
        std::process::exit(1);
    }
    let (c, cpp) = collect_src_files(&dir);
    if !c.is_empty() {
        build_c(c, &language);
    }
    if !cpp.is_empty() {
        build_cpp(cpp, &language);
    }
}

fn main() {
    let ignore = vec![
        "tree-sitter-typescript".to_string(),
        "tree-sitter-cpp".to_string(),
    ];
    let dirs = collect_tree_sitter_dirs(ignore);
    for dir in dirs {
        let language = &dir[TREE_SITTER.len()..];
        build_dir(&dir, &language);
    }
    build_dir("tree-sitter-typescript/tsx", "tsx");
    build_dir("tree-sitter-typescript/typescript", "typescript");
}
