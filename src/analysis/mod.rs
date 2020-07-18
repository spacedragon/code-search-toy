mod ts_analyzer;

use crate::languages::Lang;
use tree_sitter::{Language, Tree, TreeCursor};

extern "C" {
    fn tree_sitter_typescript() -> Language;
}

trait SemanticAnalysis<L> {
    fn visit(self);
}

struct Analyzer<'a> {
    cursor: TreeCursor<'a>,
}

impl<'a> Analyzer<'a> {
    pub fn new(cursor: TreeCursor<'a>) -> Self {
        Self { cursor }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tree_sitter::{Parser, TreeCursor};

    #[test]
    fn test_walker() {
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter_typescript() };
        parser.set_language(language).unwrap();

        let source_code = "function test() { return Math.abs(1) }";
        let tree = parser.parse(source_code, None).unwrap();

        let mut cursor = tree.walk();
        let ts_analyzer = Analyzer::new(cursor);
        ts_analyzer.visit();
        println!("{}", tree.root_node().to_sexp());
    }
}
