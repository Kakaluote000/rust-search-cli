//! Output module - handles result formatting and display

use crate::search::SearchResult;
use std::io::Write;

/// Output color modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

impl ColorMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "always" => ColorMode::Always,
            "never" => ColorMode::Never,
            _ => ColorMode::Auto,
        }
    }

    pub fn should_color(&self, is_tty: bool) -> bool {
        match self {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => is_tty,
        }
    }
}

/// Output formatter
pub struct OutputFormatter {
    color_mode: ColorMode,
    show_line_numbers: bool,
    only_matching: bool,
}

impl OutputFormatter {
    pub fn new(color_mode: &str, show_line_numbers: bool, only_matching: bool) -> Self {
        Self {
            color_mode: ColorMode::from_str(color_mode),
            show_line_numbers,
            only_matching,
        }
    }

    /// Print search result to stdout
    pub fn print_result(&self, result: &SearchResult) {
        for m in &result.matches {
            if self.only_matching {
                // Print only the matching part
                let line = &m.line[m.start..m.end];
                if self.color_mode.should_color(atty::is(atty::Stream::Stdout)) {
                    print!("{}", self.highlight(line, &RED));
                } else {
                    print!("{}", line);
                }
                println!();
            } else {
                // Print full line
                if self.show_line_numbers {
                    print!("{}:", m.line_number);
                }
                if self.color_mode.should_color(atty::is(atty::Stream::Stdout)) {
                    println!("{}", self.highlight_line(&m.line, &RED, &RESET));
                } else {
                    println!("{}", m.line);
                }
            }
        }
    }

    /// Print file header
    pub fn print_file_header(&self, file_path: &str) {
        println!("{}", file_path);
    }

    /// Print match count
    pub fn print_count(&self, file_path: &str, count: usize) {
        println!("{}:{}", file_path, count);
    }

    /// Print file name only
    pub fn print_filename(&self, file_path: &str) {
        println!("{}", file_path);
    }

    /// Highlight a string with ANSI codes
    fn highlight<'a>(&self, text: &'a str, color: &str) -> String {
        format!("{}{}{}", color, text, RESET)
    }

    /// Highlight matches in a line
    fn highlight_line<'a>(&self, line: &str, color: &str, reset: &str) -> String {
        // Simple implementation - just return the line with color
        format!("{}{}{}", color, line, reset)
    }
}

// ANSI color codes
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const RESET: &str = "\x1b[0m";

/// Check if stdout is a TTY
pub fn is_tty() -> bool {
    atty::is(atty::Stream::Stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_mode() {
        assert_eq!(ColorMode::from_str("always"), ColorMode::Always);
        assert_eq!(ColorMode::from_str("never"), ColorMode::Never);
        assert_eq!(ColorMode::from_str("auto"), ColorMode::Auto);
        assert_eq!(ColorMode::from_str("invalid"), ColorMode::Auto);
    }
}
