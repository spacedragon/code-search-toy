

use tantivy::query::QueryParserError;


pub trait ResultExt<F> {
    fn map_error(self) -> anyhow::Result<F>;
}

impl<F> ResultExt<F> for tantivy::Result<F> {
    fn map_error(self) -> anyhow::Result<F> {
        self.map_err(|e| anyhow!("{}: {:?}", "Tantivy Error", e))
    }
}

impl<F> ResultExt<F> for Result<F, QueryParserError> {
    fn map_error(self) -> anyhow::Result<F> {
        self.map_err(|e| anyhow!("{}: {:?}", "Query Parser Error", e))
    }
}
