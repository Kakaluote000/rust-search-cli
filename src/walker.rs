//! File traversal module - handles directory walking

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// File walker with filtering options
pub struct FileWalker {
    max_depth: Option<usize>,
    exclude_patterns: Vec<String>,
    file_types: Vec<String>,
    hidden: bool,
    follow_symlinks: bool,
}

impl FileWalker {
    pub fn new(
        max_depth: Option<usize>,
        exclude_patterns: Vec<String>,
        file_types: Vec<String>,
        hidden: bool,
        follow_symlinks: bool,
    ) -> Self {
        Self {
            max_depth,
            exclude_patterns,
            file_types,
            hidden,
            follow_symlinks,
        }
    }

    /// Walk directories and yield matching files
    pub fn walk(&self, root: &Path) -> Vec<PathBuf> {
        let depth = self.max_depth.unwrap_or(usize::MAX);
        
        WalkDir::new(root)
            .follow_links(self.follow_symlinks)
            .max_depth(depth)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| self.should_include_file(e.path()))
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    /// Check if file should be included
    fn should_include_file(&self, path: &Path) -> bool {
        // Skip hidden files unless enabled
        if !self.hidden {
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().starts_with('.') {
                    return false;
                }
            }
        }

        // Check exclude patterns
        for pattern in &self.exclude_patterns {
            if path.to_string_lossy().contains(pattern) {
                return false;
            }
        }

        // Check file type filters
        if !self.file_types.is_empty() {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy();
                if !self.file_types.iter().any(|t| t == &ext_str) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_walker_creation() {
        let walker = FileWalker::new(
            Some(3),
            vec!["target".to_string()],
            vec!["rs".to_string()],
            false,
            false,
        );
        assert!(walker.max_depth == Some(3));
    }
}
