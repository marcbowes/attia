use bytes::Bytes;
use serde::{Deserialize, Serialize};

use std::{
    fs,
    io::{BufReader, BufWriter},
    path::Path,
};

use crate::error::Result;

pub const VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct HtmlFile {
    pub version: u32,
    pub url: String,
    pub content: Bytes, // probably should just be a String?
}

impl HtmlFile {
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        fs::create_dir_all(&path.as_ref().parent().unwrap())?;
        tracing::info!(path = %path.as_ref().display(), "saving");
        let file = fs::OpenOptions::new().create(true).write(true).open(path)?;
        let mut wtr = BufWriter::new(file);
        serde_json::to_writer(&mut wtr, &self)?;
        Ok(())
    }

    pub fn read_from_file(path: impl AsRef<Path>) -> Result<HtmlFile> {
        let rdr = BufReader::new(fs::File::open(path.as_ref())?);
        Ok(serde_json::from_reader(rdr)?)
    }
}
