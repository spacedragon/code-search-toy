use crate::analysis::{Analyzer, SemanticAnalysis};
use crate::languages::Typescript;

impl<'a> SemanticAnalysis<Typescript> for Analyzer<'a> {
    fn visit(mut self) {
        let token = Typescript::from(self.cursor.node().kind_id());
        let str: &str = token.into();
        println!("{}", str);
    }
}
