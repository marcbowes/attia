use std::fs;
use std::path::Path;

use crate::{
    cache::HtmlFile,
    config::Config,
    error::{ProgramError, Result},
};

use tantivy::schema::*;

pub fn html_to_tantivy(schema: &Schema, config: &Config) -> Result<Vec<Document>> {
    fs::read_dir(&config.data_dir.join("html-cache"))?
        .map(|p| html_to_tantivy_at_path(&p?.path(), &schema))
        .collect::<Result<Vec<_>>>()
}

fn html_to_tantivy_at_path(p: impl AsRef<Path>, schema: &Schema) -> Result<Document> {
    let hf = HtmlFile::read_from_file(p.as_ref())?;
    let scraped =
        scraper::Html::parse_document(&String::from_utf8(hf.content.into()).expect("valid utf8"));
    let title_selector = scraper::Selector::parse("div.header__title h1.heading--page")?;
    let title = scraped
        .select(&title_selector)
        .map(|x| x.text().collect::<Vec<_>>().join("\n").trim().to_string())
        .next()
        .ok_or(ProgramError::Unexpected(format!(
            "no title in {}",
            p.as_ref().display()
        )))?;
    let content_selector = scraper::Selector::parse("div.content--post")?;
    let content = scraped
        .select(&content_selector)
        .map(|x| x.inner_html())
        .next()
        .ok_or(ProgramError::Unexpected(format!(
            "no content in {}",
            p.as_ref().display()
        )))?;
    let mut tantivy = Document::default();
    tantivy.add_text(schema.get_field("title").unwrap(), title); // FIXME: remove the html stuffs
    tantivy.add_text(schema.get_field("url").unwrap(), hf.url);
    tantivy.add_text(schema.get_field("content").unwrap(), content);
    Ok(tantivy)
}
