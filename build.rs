use std::env;
use std::path::PathBuf;

fn cwd() -> PathBuf {
    let path = env::current_dir().unwrap();
    if path.ends_with("language-gen") {
        path.join("..")
    } else {
        path
    }
}

fn build(dir: PathBuf, name: &str) {
    let dir = cwd().join(dir);
    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .file(dir.join("scanner.c"))
        .compile(name);
}

fn main() {
    build(
        PathBuf::from(r"tree-sitter-javascript/src"),
        "tree-sitter-javascript",
    );
    build(
        PathBuf::from(r"tree-sitter-typescript/typescript/src"),
        "tree-sitter-javascript",
    );
    build(
        PathBuf::from(r"tree-sitter-typescript/tsx/src"),
        "tree-sitter-tsx",
    );
}
