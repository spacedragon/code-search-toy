mod analysis;
mod error;

#[macro_use]
extern crate tantivy;

#[macro_use]
extern crate anyhow;

mod indexer;
mod reader;
use crate::indexer::IndexerRunner;
use anyhow::Result;
use clap::Clap;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber;
use tree_sitter::{Language, Parser};

// extern "C" { fn tree_sitter_javascript() -> Language; }
extern "C" {
    fn tree_sitter_typescript() -> Language;
}

#[derive(Clap)]
#[clap(version = "1.0", author = "Draco <allendragon@gmail.com>")]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    // #[clap(short, long, default_value = "default.conf")]
    // config: String,
    /// A level of verbosity, and can be used multiple times
    // #[clap(short, long, parse(from_occurrences))]
    // verbose: i32,
    /// Print debug info
    #[clap(short)]
    debug: bool,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Index(Index),
    Test,
    Search(Search),
}

/// A subcommand for controlling testing
#[derive(Clap)]
struct Index {
    #[clap()]
    project_path: PathBuf,
}

#[derive(Clap)]
struct Search {
    #[clap(short, long)]
    index: PathBuf,
    #[clap()]
    keyword: String,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let level = if opts.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = tracing_subscriber::fmt()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(level)
        // completes the builder
        .finish();

    // and sets the constructed `Subscriber` as the default.
    tracing::subscriber::set_global_default(subscriber).expect("no global subscriber has been set");
    match opts.subcmd {
        SubCommand::Index(index) => {
            let indexer_runner = IndexerRunner::new(index.project_path);
            indexer_runner.run()?;
        }
        SubCommand::Test => {
            info!("Do a test");
            let mut parser = Parser::new();
            let language = unsafe { tree_sitter_typescript() };
            parser.set_language(language).unwrap();
            let source_code = "var a=1;";
            let tree = parser.parse(source_code, None).unwrap();
            println!("{}", tree.root_node().to_sexp());
        }
        SubCommand::Search(search) => {
            reader::search(search.index, &search.keyword)?;
        }
    }
    return Ok(());
}
