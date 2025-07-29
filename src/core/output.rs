use console::style;
use std::fmt;

/// Buffered output utility for better performance
pub struct BufferedOutput {
    lines: Vec<String>,
}

impl BufferedOutput {
    /// Create a new buffered output
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    /// Add a line to the buffer
    pub fn add_line(&mut self, line: String) {
        self.lines.push(line);
    }

    /// Add multiple lines to the buffer
    pub fn add_lines(&mut self, lines: Vec<String>) {
        self.lines.extend(lines);
    }

    /// Add a formatted line to the buffer
    pub fn add_formatted(&mut self, line: String) {
        self.lines.push(line);
    }

    /// Get all buffered content as a single string
    pub fn content(&self) -> String {
        self.lines.join("\n")
    }

    /// Print all buffered content to stdout
    pub fn flush(&self) {
        if !self.lines.is_empty() {
            println!("{}", self.content());
        }
    }

    /// Print all buffered content to stderr
    pub fn flush_err(&self) {
        if !self.lines.is_empty() {
            eprintln!("{}", self.content());
        }
    }

    /// Get the number of buffered lines
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

impl Default for BufferedOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for BufferedOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.lines.join("\n"))
    }
}

/// Common formatting utilities
pub struct Format;

impl Format {
    /// Format an error message with emoji
    pub fn error(msg: &str) -> String {
        format!("{} {}", style("❌").bold(), msg)
    }

    /// Format a success message with emoji
    pub fn success(msg: &str) -> String {
        format!("{} {}", style("✅").bold(), msg)
    }

    /// Format an info message with emoji
    pub fn info(msg: &str) -> String {
        format!("{} {}", style("ℹ️").bold(), msg)
    }

    /// Format a warning message with emoji
    pub fn warning(msg: &str) -> String {
        format!("{} {}", style("⚠️").bold().yellow(), msg)
    }

    /// Format text with bold styling
    pub fn bold(text: &str) -> String {
        style(text).bold().to_string()
    }

    /// Format text with color
    pub fn colored(text: &str, color: console::Color) -> String {
        style(text).fg(color).to_string()
    }
}

/// Output formatters for different data types
pub struct TableFormatter {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl TableFormatter {
    pub fn new(headers: Vec<String>) -> Self {
        Self {
            headers,
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    pub fn format(&self) -> String {
        if self.rows.is_empty() {
            return "No data to display".to_string();
        }

        let mut output = String::new();

        // Calculate column widths
        let mut widths: Vec<usize> = self.headers.iter().map(|h| h.len()).collect();
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // Format headers
        for (i, header) in self.headers.iter().enumerate() {
            if i > 0 {
                output.push_str("  ");
            }
            output.push_str(&format!(
                "{:<width$}",
                header,
                width = widths.get(i).unwrap_or(&0)
            ));
        }
        output.push('\n');

        // Add separator
        for (i, width) in widths.iter().enumerate() {
            if i > 0 {
                output.push_str("  ");
            }
            output.push_str(&"-".repeat(*width));
        }
        output.push('\n');

        // Format rows
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i > 0 {
                    output.push_str("  ");
                }
                output.push_str(&format!(
                    "{:<width$}",
                    cell,
                    width = widths.get(i).unwrap_or(&0)
                ));
            }
            output.push('\n');
        }

        output
    }
}

/// Progress indicator for long-running operations
pub struct ProgressIndicator {
    message: String,
    current: usize,
    total: Option<usize>,
}

impl ProgressIndicator {
    pub fn new(message: String) -> Self {
        Self {
            message,
            current: 0,
            total: None,
        }
    }

    pub fn with_total(message: String, total: usize) -> Self {
        Self {
            message,
            current: 0,
            total: Some(total),
        }
    }

    pub fn increment(&mut self) {
        self.current += 1;
        self.display();
    }

    pub fn set_current(&mut self, current: usize) {
        self.current = current;
        self.display();
    }

    pub fn finish(&self) {
        println!("{} ✅ Done", self.message);
    }

    fn display(&self) {
        if let Some(total) = self.total {
            let percentage = (self.current as f64 / total as f64 * 100.0) as usize;
            print!(
                "\r{} [{}/{}] {}%",
                self.message, self.current, total, percentage
            );
        } else {
            print!("\r{} [{}]", self.message, self.current);
        }
        // Flush stdout to ensure immediate display
        use std::io::{self, Write};
        let _ = io::stdout().flush();
    }
}
