//! Rust Search CLI - A fast file search tool similar to ripgrep
//! 
//! Usage: rust-search-cli [OPTIONS] <PATTERN> <PATH>

use anyhow::Result;
use clap::{Parser, ArgAction};
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
        _ => anyhow::bail!("Invalid color value: {}. Use 'auto', 'always', or 'never'", cli.color),
    };
    
    // Build search config
    let config = SearchConfig {
        pattern: cli.pattern,
        path: cli.path,
        ignore_case: cli.ignore_case,
        invert_match: cli.invert_match,
        line_number: cli.line_number,
        only_matching: cli.only_matching,
        recursive: cli.recursive,
        max_depth: cli.max_depth,
        exclude: cli.exclude,
        file_type: cli.file_type,
        hidden: cli.hidden,
        follow_symlinks: cli.follow_symlinks,
        count: cli.count,
        files_with_matches: cli.files_with_matches,
        quiet: cli.quiet,
        color: color_mode,
    };
    
    tracing::info!("Search config: {:?}", config);
    
    println!("✓ CLI parsing complete - Ready to implement search logic");
    println!("  Pattern: {}", config.pattern);
    println!("  Path: {:?}", config.path);
    
    Ok(())
}

/// Search configuration
#[derive(Debug, Clone)]
struct SearchConfig {
    pattern: String,
    path: PathBuf,
    ignore_case: bool,
    invert_match: bool,
    line_number: bool,
    only_matching: bool,
    recursive: bool,
    max_depth: Option<usize>,
    exclude: Vec<String>,
    file_type: Vec<String>,
    hidden: bool,
    follow_symlinks: bool,
    count: bool,
    files_with_matches: bool,
    quiet: bool,
    color: String,
}
