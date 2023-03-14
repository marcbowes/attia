use async_trait::async_trait;
use fantoccini::{Client, ClientBuilder, Locator};
use http::Method;
use sha2::{Digest, Sha256};
use tokio;
use tokio::time::Duration;
use tracing;

use std::fs;

use crate::{
    cache::{self, HtmlFile},
    config::Config,
    error::{ProgramError, Result},
};

#[async_trait]
trait ClientExt {
    async fn expect_url(&self, expect: &str) -> Result<()>;
}

#[async_trait]
impl ClientExt for Client {
    async fn expect_url(&self, expect: &str) -> Result<()> {
        let url = self.current_url().await?;
        if url.as_ref() != expect {
            return Err(ProgramError::WrongUrl(url.to_string(), expect.to_string()));
        }
        Ok(())
    }
}

const LOGIN_PAGE: &str = "https://peterattiamd.com/login/";
const ARCHIVE_PAGE: &str = "https://peterattiamd.com/podcast/archive/";

// TODO: client and config are passed through methods here, probably makes sense
// to have a struct and implement the methods on that struct.

pub async fn run(config: &Config) -> Result<()> {
    let c = ClientBuilder::native()
        .connect("http://localhost:4444") // TODO: make customizable
        .await
        .expect("failed to connect to WebDriver");

    let result = run_with_client(&c, config).await;
    // it's important to close even if there is an error otherwise you can't
    // rerun the program without manually restarting the driver
    c.close().await?;
    result
}

async fn run_with_client(c: &Client, config: &Config) -> Result<()> {
    ensure_login(c, config).await?;
    download_show_notes(&c, config).await?;
    Ok(())
}

async fn download_show_notes(c: &Client, config: &Config) -> Result<()> {
    c.goto(ARCHIVE_PAGE).await?;
    let elems = c
        .find_all(Locator::Css(".display-posts-listing a.title"))
        .await?;

    let mut links = vec![];
    for elem in elems {
        let url = elem
            .attr("href")
            .await?
            .ok_or(ProgramError::Unexpected(
                "all links should have a href".to_string(),
            ))?
            .to_string();
        links.push(url);
    }
    for link in links {
        if download_show_notes_at(c, config, &link).await? {
            tracing::debug!("pausing to rate limit");
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    Ok(())
}

// returns false if nothing was downloaded (already cached)
async fn download_show_notes_at(c: &Client, config: &Config, url: &str) -> Result<bool> {
    let mut hasher = Sha256::new();
    hasher.update(url);
    let hash = hasher.finalize();
    let encoded = base16ct::lower::encode_string(&hash);
    let cache_location = config.data_dir.join("html-cache");
    let path = cache_location.join(format!("{}.json", encoded));

    if path.exists() {
        match HtmlFile::read_from_file(&path) {
            Ok(h) => {
                if h.version == cache::VERSION {
                    tracing::info!(%url, "already downloaded");
                    return Ok(false);
                } else {
                    tracing::warn!(path = %path.display(), "old format, discarding");
                    fs::remove_file(&path)?;
                }
            }
            Err(err) => {
                tracing::error!(%err, "unable to deserialize file, maybe really old format, discarding");
                fs::remove_file(&path)?;
            }
        }
    }

    tracing::info!(%url, "downloading page");

    let raw = c.raw_client_for(Method::GET, &url).await?;
    use futures_util::TryStreamExt;
    let page_bytes = raw
        .into_body()
        .try_fold(Vec::new(), |mut data, chunk| async move {
            data.extend_from_slice(&chunk);
            Ok(data)
        })
        .await
        .map_err(fantoccini::error::CmdError::from)?;
    if page_bytes.len() == 0 {
        return Err(ProgramError::Unexpected(format!(
            "page at {} is empty?",
            url
        )));
    }

    let hf = HtmlFile {
        version: cache::VERSION,
        url: url.to_string(),
        content: page_bytes.into(),
    };
    hf.write_to_file(&path)?;

    Ok(true)
}

async fn ensure_login(c: &Client, config: &Config) -> Result<()> {
    c.goto(LOGIN_PAGE).await?;

    let url = c.current_url().await?;
    if url.as_ref() == LOGIN_PAGE {
        tracing::info!("logging in");
        login(c, config).await?;
        tracing::info!("logged in");
    } else {
        tracing::info!("already logged in");
    }

    c.expect_url("https://peterattiamd.com/members/").await?;

    Ok(())
}

async fn login(c: &Client, config: &Config) -> Result<()> {
    let f = c.form(Locator::Css("#mepr_loginform")).await?;
    f.set_by_name("log", &config.username)
        .await?
        .set_by_name("pwd", &config.password)
        .await?
        .submit()
        .await?;

    Ok(())
}
