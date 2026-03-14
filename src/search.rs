//! Search module - handles pattern matching in files

use anyhow::Result;
use regex::Regex;
use std::path::Path;

/// Search result for a single match
#[derive(Debug, Clone)]
pub struct Match {
    pub line_number: usize,
    pub line: String,
    pub start: usize,
    pub end: usize,
}

/// Search result for a file
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: String,
    pub matches: Vec<Match>,
    pub total_matches: usize,
    #[allow(dead_code)]
    line_count: usize,
}

/// Search engine with pattern matching
#[allow(dead_code)]
pub struct SearchEngine {
    regex: Regex,
    ignore_case: bool,
    invert_match: bool,
    only_matching: bool,
}

impl SearchEngine {
    /// Create a new search engine with the given pattern
    pub fn new(pattern: &str, ignore_case: bool) -> Result<Self> {
        let regex_pattern = if ignore_case {
            format!("(?i){}", pattern)
        } else {
            pattern.to_string()
        };

        let regex = Regex::new(&regex_pattern)
            .map_err(|e| anyhow::anyhow!("Invalid regex pattern: {}", e))?;

        Ok(Self {
            regex,
            ignore_case,
            invert_match: false,
            only_matching: false,
        })
    }

    /// Set invert match mode
    pub fn with_invert_match(mut self, invert: bool) -> Self {
        self.invert_match = invert;
        self
    }

    /// Set only matching mode
    pub fn with_only_matching(mut self, only: bool) -> Self {
        self.only_matching = only;
        self
    }

    /// Search a file and return results
    pub fn search_file(&self, path: &Path) -> Result<Option<SearchResult>> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Ok(None), // Skip unreadable files
        };

        let mut matches = Vec::new();
        let mut line_count = 0;

        for (line_num, line) in content.lines().enumerate() {
            line_count = line_num + 1;
            let has_match = self.regex.is_match(line);

            let should_match = if self.invert_match {
                !has_match
            } else {
                has_match
            };

            if should_match {
                if self.only_matching {
                    // Find all matches in the line
                    for mat in self.regex.find_iter(line) {
                        matches.push(Match {
                            line_number: line_num + 1,
                            line: line.to_string(),
                            start: mat.start(),
                            end: mat.end(),
                        });
                    }
                } else {
                    // Just record the line
                    matches.push(Match {
                        line_number: line_num + 1,
                        line: line.to_string(),
                        start: 0,
                        end: 0,
                    });
                }
            }
        }

        if matches.is_empty() {
            return Ok(None);
        }

        let total = matches.len();

        Ok(Some(SearchResult {
            file_path: path.to_string_lossy().to_string(),
            matches,
            total_matches: total,
            line_count,
        }))
    }
}

/// Search configuration shared across threads
#[allow(dead_code)]
#[derive(Clone)]
pub struct SearchConfig {
    pub pattern: String,
    pub path: std::path::PathBuf,
    pub ignore_case: bool,
    pub invert_match: bool,
    pub line_number: bool,
    pub only_matching: bool,
    pub recursive: bool,
    pub max_depth: Option<usize>,
    pub exclude: Vec<String>,
    pub file_type: Vec<String>,
    pub hidden: bool,
    pub follow_symlinks: bool,
    pub count: bool,
    pub files_with_matches: bool,
    pub quiet: bool,
    pub color: String,
}

impl SearchConfig {
    pub fn build_engine(&self) -> Result<SearchEngine> {
        let mut engine = SearchEngine::new(&self.pattern, self.ignore_case)?;
        engine.invert_match = self.invert_match;
        engine.only_matching = self.only_matching;
        Ok(engine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_match() {
        let engine = SearchEngine::new("hello", false).unwrap();
        let result = engine.search_file(Path::new("Cargo.toml")).unwrap();
        // Should find some matches in Cargo.toml
        println!("Result: {:?}", result);
    }

    #[test]
    fn test_case_insensitive() {
        let engine = SearchEngine::new("hello", true).unwrap();
        let result = engine.search_file(Path::new("Cargo.toml")).unwrap();
        println!("Result: {:?}", result);
    }
}
