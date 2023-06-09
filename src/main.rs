use clap::Parser;

use config::Config;
use search::Search;

mod cache;
mod config;
mod convert;
mod download;
mod error;
mod search;

#[tokio::main]
async fn main() -> error::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let config = Config::load()?;

    // TODO: Use a heuristic to automatically download. For example, episodes
    // usually come out on a Monday, and so it should be possible to determine
    // whether it's worth checking.
    if args.download {
        download::run(&config).await?;
    }
    let mut search = Search::new(config.clone())?;
    let tantivies = convert::html_to_tantivy(&search.schema, &config, search.touch_mtime())?;
    search.update(tantivies)?;
    search.query(&args.query)?;
    Ok(())
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    download: bool,

    // FIXME: maybe move to subcommand otherwise you can't run download without a query
    #[arg(short, long)]
    query: String,
}
