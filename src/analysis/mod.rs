mod java;

use rust_code_analysis::*;
use std::marker::PhantomData;
use tree_sitter::{Language, Parser, Tree, TreeCursor};

extern "C" {
    fn tree_sitter_typescript() -> Language;
}

trait AnalyzerTrait {
    fn new(code: Vec<u8>) -> Self;
}

trait LanguageAnalysis {
    fn visit(self);
}

pub struct Analyzer<T: TSLanguage> {
    code: Vec<u8>,
    tree: Tree,
    phantom: PhantomData<T>,
}

impl<T: 'static + TSLanguage> AnalyzerTrait for Analyzer<T> {
    fn new(code: Vec<u8>) -> Self {
        let mut parser = Parser::new();
        parser.set_language(T::get_language()).unwrap();
        let tree = parser.parse(&code, None).unwrap();
        Self {
            code,
            tree,
            phantom: PhantomData,
        }
    }
}

pub type JavaAnalyzer = Analyzer<JavaCode>;
pub type RustAnalyzer = Analyzer<RustCode>;

#[cfg(test)]
mod test {
    use super::*;
    use rust_code_analysis::JavaCode;
    use tree_sitter::{Parser, TreeCursor};

    #[test]
    fn test_java() {
        let source_code = "public class Main {  public void main() {} }";

        let analyzer = JavaAnalyzer::new(Vec::from(source_code));
        analyzer.visit();
    }

    #[test]
    fn test_rust() {
        let source_code = "fn test() { return 1 + 1 }";

        let analyzer = RustAnalyzer::new(Vec::from(source_code));
        analyzer.visit();
    }
}
