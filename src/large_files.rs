use crate::common::BufferedOutput;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub size_bytes: u64,
    pub size_mb: f64,
}

impl FileInfo {
    pub fn new(path: String, size_bytes: u64) -> Self {
        let size_mb = size_bytes as f64 / (1024.0 * 1024.0);
        Self {
            path,
            size_bytes,
            size_mb,
        }
    }
}

pub fn run(limit: usize, threshold: Option<f64>) {
    let mut output = BufferedOutput::new();

    output.add_line(format_scan_start_message().to_string());

    // Get all file objects and their sizes
    let file_objects = match get_file_objects() {
        Ok(objects) => objects,
        Err(msg) => {
            eprintln!("{}", format_error_message(msg));
            return;
        }
    };

    if file_objects.is_empty() {
        output.add_line(format_no_files_message().to_string());
        output.flush();
        return;
    }

    // Find the largest files by path
    let mut large_files = find_largest_files(file_objects, threshold);

    // Sort by size (largest first)
    large_files.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    // Limit results
    large_files.truncate(limit);

    if large_files.is_empty() {
        output.add_line(format_no_large_files_message(threshold));
        output.flush();
        return;
    }

    output.add_line(format_results_header(large_files.len(), threshold));

    // Add all file results to buffer
    for (i, file) in large_files.iter().enumerate() {
        output.add_line(format_file_line(i + 1, file));
    }

    // Add summary
    let total_size: u64 = large_files.iter().map(|f| f.size_bytes).sum();
    let total_mb = total_size as f64 / (1024.0 * 1024.0);
    output.add_line(format_summary_message(large_files.len(), total_mb));

    // Flush all output at once for better performance
    output.flush();
}

// Helper function to get file objects from git
fn get_file_objects() -> Result<Vec<(String, String, u64)>, &'static str> {
    let output = Command::new("git")
        .args(get_rev_list_args())
        .output()
        .map_err(|_| "Failed to execute git rev-list")?;

    if !output.status.success() {
        return Err("Failed to get file objects from git history");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_git_objects(&stdout)
}

// Helper function to get git rev-list args
pub fn get_rev_list_args() -> [&'static str; 6] {
    [
        "rev-list",
        "--objects",
        "--all",
        "--no-object-names",
        "--filter=blob:none",
        "--",
    ]
}

// Helper function to parse git objects output
fn parse_git_objects(output: &str) -> Result<Vec<(String, String, u64)>, &'static str> {
    let mut objects = Vec::new();

    for line in output.lines() {
        let hash = line.trim();
        if hash.is_empty() || hash.len() != 40 {
            continue;
        }

        // Get object size
        if let Ok(size) = get_object_size(hash) {
            if size > 0 {
                // Get file paths for this object
                if let Ok(paths) = get_object_paths(hash) {
                    for path in paths {
                        objects.push((hash.to_string(), path, size));
                    }
                }
            }
        }
    }

    Ok(objects)
}

// Helper function to get object size
fn get_object_size(hash: &str) -> Result<u64, &'static str> {
    let output = Command::new("git")
        .args(["cat-file", "-s", hash])
        .output()
        .map_err(|_| "Failed to get object size")?;

    if !output.status.success() {
        return Err("Failed to get object size");
    }

    let size_str = String::from_utf8_lossy(&output.stdout);
    size_str.trim().parse().map_err(|_| "Invalid size format")
}

// Helper function to get object paths
fn get_object_paths(hash: &str) -> Result<Vec<String>, &'static str> {
    let output = Command::new("git")
        .args([
            "log",
            "--all",
            "--pretty=format:",
            "--name-only",
            "--diff-filter=A",
            "-S",
            hash,
        ])
        .output()
        .map_err(|_| "Failed to get object paths")?;

    if !output.status.success() {
        // Fallback: try to find the path using rev-list with object names
        return get_object_paths_fallback(hash);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let paths: Vec<String> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect();

    if paths.is_empty() {
        get_object_paths_fallback(hash)
    } else {
        Ok(paths)
    }
}

// Fallback method to get object paths
fn get_object_paths_fallback(hash: &str) -> Result<Vec<String>, &'static str> {
    let output = Command::new("git")
        .args(["rev-list", "--objects", "--all"])
        .output()
        .map_err(|_| "Failed to get object paths")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let paths: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[0] == hash {
                Some(parts[1..].join(" "))
            } else {
                None
            }
        })
        .collect();

    if paths.is_empty() {
        Ok(vec![format!("unknown-{}", &hash[0..8])])
    } else {
        Ok(paths)
    }
}

// Helper function to find largest files
fn find_largest_files(
    objects: Vec<(String, String, u64)>,
    threshold: Option<f64>,
) -> Vec<FileInfo> {
    let mut file_sizes: HashMap<String, u64> = HashMap::new();

    // Group by file path and take the maximum size
    for (_hash, path, size) in objects {
        file_sizes
            .entry(path)
            .and_modify(|current| *current = (*current).max(size))
            .or_insert(size);
    }

    let threshold_bytes = threshold.map(|mb| (mb * 1024.0 * 1024.0) as u64);

    file_sizes
        .into_iter()
        .filter(|(_, size)| threshold_bytes.is_none_or(|threshold| *size >= threshold))
        .map(|(path, size)| FileInfo::new(path, size))
        .collect()
}

// Helper function to format scan start message
pub fn format_scan_start_message() -> &'static str {
    "🔍 Scanning repository for large files..."
}

// Helper function to format error message
pub fn format_error_message(msg: &str) -> String {
    format!("❌ {msg}")
}

// Helper function to format no files message
pub fn format_no_files_message() -> &'static str {
    "ℹ️ No files found in repository history"
}

// Helper function to format no large files message
pub fn format_no_large_files_message(threshold: Option<f64>) -> String {
    match threshold {
        Some(mb) => format!("✅ No files found larger than {mb:.1} MB"),
        None => "✅ No large files found".to_string(),
    }
}

// Helper function to format results header
pub fn format_results_header(count: usize, threshold: Option<f64>) -> String {
    match threshold {
        Some(mb) => format!("📊 Top {count} files larger than {mb:.1} MB:"),
        None => format!("📊 Top {count} largest files:"),
    }
}

// Helper function to format file line
pub fn format_file_line(index: usize, file: &FileInfo) -> String {
    format!(
        "{index:2}. {size:>8.1} MB  {path}",
        size = file.size_mb,
        path = file.path
    )
}

// Helper function to format summary message
pub fn format_summary_message(count: usize, total_mb: f64) -> String {
    format!("\n📈 Total: {count} files, {total_mb:.1} MB")
}

// Helper function to convert bytes to human readable
pub fn format_size_human_readable(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{size:.0} {}", UNITS[unit_index])
    } else {
        format!("{size:.1} {}", UNITS[unit_index])
    }
}
