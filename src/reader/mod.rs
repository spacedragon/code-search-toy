use crate::error::*;
use anyhow::{Context, Result};
use std::path::PathBuf;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::{Index, ReloadPolicy};

pub fn search(index_path: PathBuf, q: &str) -> Result<()> {
    let index = Index::open_in_dir(&index_path)
        .map_error()
        .context("open index failed.")?;
    let schema = index.schema();
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()
        .map_error()
        .context("open index reader failed")?;

    let searcher = reader.searcher();

    let path_field = schema.get_field("path").unwrap();
    let source_field = schema.get_field("source").unwrap();
    let query_parser = QueryParser::for_index(&index, vec![path_field, source_field]);

    let query = query_parser.parse_query(q).map_error()?;

    let top_docs = searcher
        .search(&query, &TopDocs::with_limit(10))
        .map_error()?;

    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address).map_error()?;
        println!("{}", schema.to_json(&retrieved_doc));
    }

    Ok(())
}
