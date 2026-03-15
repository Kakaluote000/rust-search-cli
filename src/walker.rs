//! File traversal module - handles directory walking

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// File walker with filtering options
#[derive(Clone)]
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

    /// Walk directories and yield matching files - optimized
    pub fn walk(&self, root: &Path) -> Vec<PathBuf> {
        let depth = self.max_depth.unwrap_or(usize::MAX);
        
        // Pre-allocate with estimated capacity
        let mut files = Vec::with_capacity(1024);
        
        for entry in WalkDir::new(root)
            .follow_links(self.follow_symlinks)
            .max_depth(depth)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| self.should_include_file(e.path()))
        {
            files.push(entry.path().to_path_buf());
        }
        
        files
    }

    /// Check if file should be included - optimized with early returns
    #[inline]
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
        let path_str = path.to_string_lossy();
        for pattern in &self.exclude_patterns {
            if path_str.contains(pattern) {
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
    use std::fs;
    use tempfile::TempDir;

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

    #[test]
    fn test_walker_excludes_hidden() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        
        // Create hidden file
        fs::write(path.join(".hidden"), "test").unwrap();
        // Create normal file
        fs::write(path.join("normal.txt"), "test").unwrap();
        
        let walker = FileWalker::new(None, vec![], vec![], false, false);
        let files = walker.walk(path);
        
        assert_eq!(files.len(), 1);
        assert!(files[0].file_name().unwrap().to_string_lossy() == "normal.txt");
    }

    #[test]
    fn test_walker_includes_hidden() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        
        // Create hidden file
        fs::write(path.join(".hidden"), "test").unwrap();
        
        let walker = FileWalker::new(None, vec![], vec![], true, false);
        let files = walker.walk(path);
        
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_walker_exclude_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        
        fs::write(path.join("target.rs"), "test").unwrap();
        fs::write(path.join("src.rs"), "test").unwrap();
        
        let walker = FileWalker::new(None, vec!["target".to_string()], vec![], false, false);
        let files = walker.walk(path);
        
        assert_eq!(files.len(), 1);
        assert!(files[0].file_name().unwrap().to_string_lossy() == "src.rs");
    }

    #[test]
    fn test_walker_file_type_filter() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        
        fs::write(path.join("test.rs"), "test").unwrap();
        fs::write(path.join("test.txt"), "test").unwrap();
        
        let walker = FileWalker::new(None, vec![], vec!["rs".to_string()], false, false);
        let files = walker.walk(path);
        
        assert_eq!(files.len(), 1);
        assert!(files[0].file_name().unwrap().to_string_lossy() == "test.rs");
    }
}
