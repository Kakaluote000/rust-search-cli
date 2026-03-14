# Rust Search CLI

A fast file search tool similar to ripgrep, written in Rust.

## Features

- **Regex Pattern Matching** - Search using powerful regular expressions
- **Concurrent Search** - Parallel file processing using Rayon
- **Flexible Filtering** - Exclude files, filter by type, control depth
- **Color Output** - Syntax highlighting with auto/always/never modes
- **Rich Statistics** - Performance metrics (files searched, matches found, time elapsed)

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
git clone <repository-url>
cd rust-search-cli
cargo build --release
./target/release/rust-search-cli --help
```

## Usage

```bash
# Basic search
rust-search-cli "pattern" /path/to/search

# Case insensitive search
rust-search-cli -i "pattern" .

# Show line numbers
rust-search-cli -n "pattern" .

# Only show filenames with matches
rust-search-cli -l "pattern" .

# Count matches per file
rust-search-cli -c "pattern" .

# Exclude files
rust-search-cli --exclude "target" "pattern" .

# Search only specific file types
rust-search-cli -t rs "pattern" .

# Maximum directory depth
rust-search-cli --max-depth 2 "pattern" .

# Search hidden files
rust-search-cli --hidden "pattern" .

# Invert match (show non-matching lines)
rust-search-cli -v "pattern" .

# Only matching parts
rust-search-cli -o "pattern" .

# Color output control
rust-search-cli --color always "pattern" .

# Verbose output with statistics
rust-search-cli --verbose "pattern" .
```

## Command Line Options

| Option | Description |
|--------|-------------|
| `-i, --ignore-case` | Case insensitive search |
| `-v, --invert-match` | Show lines that don't match |
| `-n, --line-number` | Show line numbers |
| `-o, --only-matching` | Show only matching parts |
| `-r, --recursive` | Recurse into directories (default: true) |
| `--max-depth` | Maximum directory depth |
| `--exclude` | Exclude files matching pattern |
| `-t, --type` | Include only files of specific type |
| `--hidden` | Search hidden files |
| `--follow-symlinks` | Follow symbolic links |
| `-c, --count` | Show count of matches per file |
| `-l, --files-with-matches` | Show only filenames with matches |
| `-q, --quiet` | Suppress output, just exit status |
| `--color` | Color output (auto/always/never) |
| `--verbose` | Enable verbose output |

## Architecture

```
src/
├── main.rs      # CLI parsing and orchestration
├── search.rs    # Regex search engine
├── walker.rs    # File traversal
└── output.rs    # Result formatting
```

### Key Components

- **SearchEngine** - Regex-based pattern matching with support for case-insensitive and inverted matches
- **FileWalker** - Directory traversal with filtering (depth, hidden files, symlinks)
- **OutputFormatter** - Colored output with TTY detection

## Performance

The tool uses Rayon for parallel file processing, making it efficient for large codebases.

## License

MIT
