use super::*;
use rust_code_analysis::Java;

impl LanguageAnalysis for JavaAnalyzer {
    fn visit(self) {
        let kind: Java = self.tree.root_node().kind_id().into();
        assert_eq!(kind, Java::Program);
        println!("{}", self.tree.root_node().to_sexp());
    }
}

impl LanguageAnalysis for RustAnalyzer {
    fn visit(self) {
        println!("{}", self.tree.root_node().to_sexp());
    }
}
