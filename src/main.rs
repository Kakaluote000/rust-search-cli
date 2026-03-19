//! Rust Search CLI - A fast file search tool similar to ripgrep
//! 
//! Usage: rust-search-cli [OPTIONS] <PATTERN> <PATH>

mod error;
mod search;
mod walker;
mod output;

use anyhow::Result;
use clap::{Parser, ArgAction};
use error::AppError;
use search::SearchConfig;
use std::path::PathBuf;
use std::time::Instant;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use walker::FileWalker;
use output::OutputFormatter;

use rayon::prelude::*;

/// Search pattern and path
#[derive(Parser, Debug)]
#[command(name = "rust-search")]
#[command(version = "0.1.0")]
#[command(about = "A fast file search tool similar to ripgrep", long_about = None)]
struct Cli {
    /// The search pattern (regex supported)
    #[arg(value_name = "PATTERN")]
    pattern: String,

    /// The path to search in
    #[arg(value_name = "PATH", default_value = ".")]
    path: PathBuf,

    /// Case insensitive search
    #[arg(short = 'i', long = "ignore-case")]
    ignore_case: bool,

    /// Invert match - show lines that don't match
    #[arg(short = 'v', long = "invert-match")]
    invert_match: bool,

    /// Show line numbers
    #[arg(short = 'n', long = "line-number")]
    line_number: bool,

    /// Show only matching parts of lines
    #[arg(short = 'o', long = "only-matching")]
    only_matching: bool,

    /// Recurse into directories
    #[arg(short = 'r', long = "recursive", default_value = "true")]
    recursive: bool,

    /// Maximum directory depth
    #[arg(long = "max-depth", value_name = "DEPTH")]
    max_depth: Option<usize>,

    /// Exclude files matching this pattern
    #[arg(long = "exclude", value_name = "PATTERN")]
    exclude: Vec<String>,

    /// Include only files matching this pattern
    #[arg(short = 't', long = "type", value_name = "TYPE")]
    file_type: Vec<String>,

    /// Search hidden files
    #[arg(long = "hidden", action = ArgAction::SetTrue)]
    hidden: bool,

    /// Follow symbolic links
    #[arg(long = "follow-symlinks", action = ArgAction::SetTrue)]
    follow_symlinks: bool,

    /// Show count of matches per file
    #[arg(short = 'c', long = "count")]
    count: bool,

    /// Suppress normal output, show only file names
    #[arg(short = 'l', long = "files-with-matches")]
    files_with_matches: bool,

    /// Don't print matches, just exit status
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,

    /// Show colored output (auto/always/never)
    #[arg(long = "color", value_name = "WHEN", default_value = "auto")]
    color: String,

    /// Enable verbose output
    #[arg(long = "verbose", action = ArgAction::SetTrue)]
    verbose: bool,
}

fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::new(
                if cli.verbose { "rust_search_cli=debug" } else { "rust_search_cli=info" }
            )
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    tracing::info!("Starting rust-search-cli");
    tracing::debug!("CLI args: {:?}", cli);
    
    // Validate color argument
    let color_mode = match cli.color.as_str() {
        "auto" | "always" | "never" => cli.color.clone(),
        _ => return Err(AppError::InvalidArgument(
            format!("Invalid color value: {}. Use 'auto', 'always', or 'never'", cli.color)
        ).into()),
    };
    
    // Validate path
    if !cli.path.exists() {
        return Err(AppError::PathNotFound(cli.path).into());
    }

    // Build search configuration
    let config = SearchConfig {
        pattern: cli.pattern.clone(),
        path: cli.path.clone(),
        ignore_case: cli.ignore_case,
        invert_match: cli.invert_match,
        line_number: cli.line_number,
        only_matching: cli.only_matching,
        recursive: cli.recursive,
        max_depth: cli.max_depth,
        exclude: cli.exclude.clone(),
        file_type: cli.file_type.clone(),
        hidden: cli.hidden,
        follow_symlinks: cli.follow_symlinks,
        count: cli.count,
        files_with_matches: cli.files_with_matches,
        quiet: cli.quiet,
        color: color_mode.clone(),
    };

    // Build search engine
    let engine = config.build_engine()?;
    tracing::info!("Search engine created");

    // Build file walker
    let walker = FileWalker::new(
        config.max_depth,
        config.exclude.clone(),
        config.file_type.clone(),
        config.hidden,
        config.follow_symlinks,
    );

    // Collect files to search
    let files = walker.walk(&cli.path);
    tracing::info!("Found {} files to search", files.len());

    // Start timing
    let start_time = Instant::now();

    // Search files in parallel
    let results: Vec<_> = files
        .par_iter()
        .filter_map(|file| {
            match engine.search_file(file) {
                Ok(Some(result)) => Some(result),
                Ok(None) => None,
                Err(e) => {
                    tracing::warn!("Error searching {:?}: {}", file, e);
                    None
                }
            }
        })
        .collect();

    let elapsed = start_time.elapsed();

    // Output results
    let formatter = OutputFormatter::new(&color_mode, cli.line_number, cli.only_matching);
    let mut total_matches = 0;
    let results_count = results.len();

    for result in &results {
        total_matches += result.total_matches;

        if cli.files_with_matches {
            formatter.print_filename(&result.file_path);
        } else if cli.count {
            formatter.print_count(&result.file_path, result.total_matches);
        } else if !cli.quiet {
            formatter.print_file_header(&result.file_path);
            formatter.print_result(result);
        }
    }

    // Print stats if verbose or not quiet
    if cli.verbose || !cli.quiet {
        println!("\n--- Search Statistics ---");
        println!("Files searched: {}", files.len());
        println!("Files with matches: {}", results_count);
        println!("Total matches: {}", total_matches);
        println!("Time elapsed: {:?}", elapsed);
    }

    tracing::info!("Search complete: {} matches in {:?}", total_matches, elapsed);

    Ok(())
}
