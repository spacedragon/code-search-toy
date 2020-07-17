

use tantivy::schema::*;
use tantivy::{directory::MmapDirectory, Index, IndexWriter};
use tantivy::{UserOperation};

use anyhow::{Result};


use std::path::{Path};
use std::sync::{RwLock};

pub struct IndexerWriter {
    writer: RwLock<IndexWriter>,
    source_field: Field,
    path_field: Field,
}

impl IndexerWriter {
    pub fn new(index_path: &Path) -> Result<Self> {
        if !index_path.exists() {
            std::fs::create_dir_all(&index_path)?;
        }
        let path = index_path.canonicalize()?;
        let mut schema_builder = Schema::builder();
        let source_field = schema_builder.add_text_field("source", TEXT | STORED);
        let path_field = schema_builder.add_text_field("path", STRING | STORED);
        let schema = schema_builder.build();
        let dir = MmapDirectory::open(&path)?;
        let index = Index::open_or_create(dir, schema.clone())
            .map_err(|_| anyhow!("can't create index in dir {:?}", &path))?;
        let index_writer = index
            .writer(50_000_000)
            .map_err(|_| anyhow!("can't create index writer"))?;
        let writer = RwLock::new(index_writer);
        Ok(Self {
            writer,
            source_field,
            path_field,
        })
    }

    pub fn index_doc(&self, path: &Path, source: &str) -> Result<()> {
        let path = String::from(path.to_string_lossy());
        let term = Term::from_field_text(self.path_field, &path);
        let writer = self
            .writer
            .read()
            .map_err(|_| anyhow!("index write failed."))?;
        writer.run(vec![
            UserOperation::Delete(term.clone()),
            UserOperation::Add(doc!(
             self.source_field => source,
             self.path_field => path
            )),
        ]);
        Ok(())
    }

    pub fn commit(&self) -> Result<()> {
        let mut writer = self
            .writer
            .write()
            .map_err(|_| anyhow!("index writer commit failed."))?;
        writer
            .commit()
            .map_err(|_| anyhow!("index writer commit failed."))?;
        Ok(())
    }
}
