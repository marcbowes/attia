use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::{schema::*, Directory};
use tantivy::{Index, ReloadPolicy};

use crate::config::Config;
use crate::error::{ProgramError, Result};

fn schema() -> Schema {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("url", TEXT | STORED);
    schema_builder.add_text_field("content", TEXT);
    schema_builder.build()
}

pub struct Search {
    index: Index,
    pub schema: Schema,
    pub config: Config,
}

impl Search {
    pub fn new(config: Config) -> Result<Search> {
        let schema = schema();
        let path = config.data_dir.join("index");
        fs::create_dir_all(&path)?;
        let directory_path: Box<dyn Directory> = Box::new(MmapDirectory::open(path)?);
        let index = Index::open_or_create(directory_path, schema.clone())?;
        Ok(Search {
            index,
            schema,
            config,
        })
    }

    pub fn update(&mut self, documents: Vec<Document>) -> Result<()> {
        if documents.is_empty() {
            return Ok(());
        };

        let mut index_writer = self.index.writer(50_000_000)?; // memory budget
        for doc in documents {
            index_writer.add_document(doc)?;
        }
        index_writer.commit()?;
        self.touch()?;
        Ok(())
    }

    pub fn query(&self, query: &str) -> Result<()> {
        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;
        let searcher = reader.searcher();
        let title = self.schema.get_field("title").unwrap();
        let content = self.schema.get_field("content").unwrap();
        let query_parser = QueryParser::for_index(&self.index, vec![title, content]);
        let query = query_parser.parse_query(query)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            println!("{}", self.schema.to_json(&retrieved_doc));
        }
        Ok(())
    }

    /// Returns last time the index was modified or the epoch if it never was.
    /// This can be used to incrementally maintain the index by only adding
    /// documents that were modified after this time.
    pub fn touch_mtime(&self) -> SystemTime {
        let inner = || Ok::<_, ProgramError>(self.touch_marker_path().metadata()?.modified()?);
        inner().unwrap_or(SystemTime::UNIX_EPOCH)
    }

    fn touch(&self) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.touch_marker_path())?;
        file.write_all(b"TOUCH MARKER")?;
        Ok(())
    }

    fn touch_marker_path(&self) -> PathBuf {
        self.config.data_dir.join("index").join("TOUCH")
    }
}
