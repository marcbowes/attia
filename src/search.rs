use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, ReloadPolicy};
use tempdir::TempDir;

use crate::error::Result;

pub fn schema() -> Schema {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("url", TEXT | STORED);
    schema_builder.add_text_field("content", TEXT);
    schema_builder.build()
}

// TODO: strip all the html stuff
pub fn index_then_search(schema: &Schema, documents: Vec<Document>, query: &str) -> Result<()> {
    let index_path = TempDir::new("attia")?;
    let index = Index::create_in_dir(&index_path, schema.clone())?;
    let mut index_writer = index.writer(50_000_000)?; // memory budget
    let title = schema.get_field("title").unwrap();
    let content = schema.get_field("content").unwrap();
    for doc in documents {
        index_writer.add_document(doc)?;
    }
    index_writer.commit()?;

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;
    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![title, content]);
    let query = query_parser.parse_query(query)?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        println!("{}", schema.to_json(&retrieved_doc));
    }

    Ok(())
}
