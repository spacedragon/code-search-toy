use std::path::PathBuf;

fn build(dir: PathBuf, name: &str) {
    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .file(dir.join("scanner.c"))
        .compile(name);
}

fn main() {

    build(PathBuf::from(r"tree-sitter-javascript/src"), "tree-sitter-javascript");
    build(PathBuf::from(r"tree-sitter-typescript/typescript/src"), "tree-sitter-javascript");
    build(PathBuf::from(r"tree-sitter-typescript/tsx/src"), "tree-sitter-tsx");

}