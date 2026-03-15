//! Search module - handles pattern matching in files

use crate::error::AppError;
use crate::error::AppResult;
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

/// Search engine with pattern matching - uses Arc for thread-safe sharing
#[allow(dead_code)]
pub struct SearchEngine {
    regex: Regex,
    ignore_case: bool,
    invert_match: bool,
    only_matching: bool,
}

// Manual implementation to avoid Sync bound on Regex
unsafe impl Send for SearchEngine {}
unsafe impl Sync for SearchEngine {}

impl SearchEngine {
    /// Create a new search engine with the given pattern
    pub fn new(pattern: &str, ignore_case: bool) -> AppResult<Self> {
        let regex_pattern = if ignore_case {
            format!("(?i){}", pattern)
        } else {
            pattern.to_string()
        };

        let regex = Regex::new(&regex_pattern)
            .map_err(|e| AppError::InvalidPattern(e.to_string()))?;

        Ok(Self {
            regex,
            ignore_case,
            invert_match: false,
            only_matching: false,
        })
    }

    /// Search a file and return results - optimized version
    pub fn search_file(&self, path: &Path) -> AppResult<Option<SearchResult>> {
        // Use buffered reading for large files
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                // Skip files that can't be read (binary, permission issues, etc.)
                tracing::debug!("Skipping unreadable file {:?}: {}", path, e);
                return Ok(None);
            }
        };

        // Pre-allocate with reasonable capacity
        let mut matches = Vec::with_capacity(64);
        let mut line_count = 0;

        // Process lines - avoid unnecessary allocations
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
                    // Just record the line - avoid cloning if possible
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
    pub fn build_engine(&self) -> AppResult<SearchEngine> {
        let mut engine = SearchEngine::new(&self.pattern, self.ignore_case)?;
        engine.invert_match = self.invert_match;
        engine.only_matching = self.only_matching;
        Ok(engine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_engine_creation() {
        let engine = SearchEngine::new("hello", false).unwrap();
        assert!(!engine.ignore_case);
    }

    #[test]
    fn test_engine_case_insensitive() {
        let engine = SearchEngine::new("hello", true).unwrap();
        assert!(engine.ignore_case);
    }

    #[test]
    fn test_engine_invalid_pattern() {
        let result = SearchEngine::new("invalid[", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_simple_match() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        fs::write(path.join("test.txt"), "hello world\nhello rust").unwrap();
        
        let engine = SearchEngine::new("hello", false).unwrap();
        let result = engine.search_file(&path.join("test.txt")).unwrap();
        
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.total_matches, 2);
    }

    #[test]
    fn test_case_insensitive_match() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        // Each line is matched separately, so "hello" appears once per line
        fs::write(path.join("test.txt"), "Hello\nHELLO\nhello").unwrap();
        
        let engine = SearchEngine::new("hello", true).unwrap();
        let result = engine.search_file(&path.join("test.txt")).unwrap();
        
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.total_matches, 3);
    }

    #[test]
    fn test_no_match() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        fs::write(path.join("test.txt"), "goodbye world").unwrap();
        
        let engine = SearchEngine::new("hello", false).unwrap();
        let result = engine.search_file(&path.join("test.txt")).unwrap();
        
        assert!(result.is_none());
    }

    #[test]
    fn test_invert_match() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        fs::write(path.join("test.txt"), "hello\nworld\nhello").unwrap();
        
        let mut engine = SearchEngine::new("hello", false).unwrap();
        engine.invert_match = true;
        
        let result = engine.search_file(&path.join("test.txt")).unwrap();
        
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.total_matches, 1); // Only "world"
    }

    #[test]
    fn test_only_matching() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        fs::write(path.join("test.txt"), "hello world hello").unwrap();
        
        let mut engine = SearchEngine::new("hello", false).unwrap();
        engine.only_matching = true;
        
        let result = engine.search_file(&path.join("test.txt")).unwrap();
        
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.total_matches, 2);
    }

    #[test]
    fn test_regex_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        // Each line is matched separately
        fs::write(path.join("test.txt"), "test123\ntest456\ntest789").unwrap();
        
        let engine = SearchEngine::new(r"test\d+", false).unwrap();
        let result = engine.search_file(&path.join("test.txt")).unwrap();
        
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.total_matches, 3);
    }
}
