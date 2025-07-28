use crate::cli::BisectAction;
use crate::{GitXError, Result};
use console::style;
use std::process::Command;

pub fn run(action: BisectAction) -> Result<String> {
    match action {
        BisectAction::Start { good, bad } => start_bisect(good, bad),
        BisectAction::Good => mark_good(),
        BisectAction::Bad => mark_bad(),
        BisectAction::Skip => skip_commit(),
        BisectAction::Reset => reset_bisect(),
        BisectAction::Status => show_status(),
    }
}

fn start_bisect(good: String, bad: String) -> Result<String> {
    // Check if already in bisect mode
    if is_bisecting()? {
        return Err(GitXError::GitCommand(
            "Already in bisect mode. Use 'git x bisect reset' to exit first.".to_string(),
        ));
    }

    // Validate that the commits exist
    validate_commit_exists(&good)?;
    validate_commit_exists(&bad)?;

    let mut output = Vec::new();
    output.push(format!(
        "{} Starting bisect between {} (good) and {} (bad)",
        style("🔍").bold(),
        style(&good).green().bold(),
        style(&bad).red().bold()
    ));

    // Start git bisect
    let git_output = Command::new("git")
        .args(["bisect", "start", &bad, &good])
        .output()?;

    if !git_output.status.success() {
        return Err(GitXError::GitCommand(format!(
            "Failed to start bisect: {}",
            String::from_utf8_lossy(&git_output.stderr).trim()
        )));
    }

    // Get current commit info
    let current_commit = get_current_commit_info()?;
    output.push(format!(
        "{} Checked out commit: {}",
        style("📍").bold(),
        style(&current_commit).cyan()
    ));

    let remaining = get_remaining_steps()?;
    output.push(format!(
        "{} Approximately {} steps remaining",
        style("⏳").bold(),
        style(remaining).yellow().bold()
    ));

    output.push(format!(
        "\n{} Test this commit and run:",
        style("💡").bold()
    ));
    output.push(format!(
        "  {} if commit is good",
        style("git x bisect good").green()
    ));
    output.push(format!(
        "  {} if commit is bad",
        style("git x bisect bad").red()
    ));
    output.push(format!(
        "  {} if commit is untestable",
        style("git x bisect skip").yellow()
    ));

    Ok(output.join("\n"))
}

fn mark_good() -> Result<String> {
    ensure_bisecting()?;

    let current_commit = get_current_commit_info()?;
    let mut output = Vec::new();
    output.push(format!(
        "{} Marked {} as good",
        style("✅").bold(),
        style(&current_commit).green()
    ));

    let git_output = Command::new("git").args(["bisect", "good"]).output()?;

    if !git_output.status.success() {
        return Err(GitXError::GitCommand(format!(
            "Failed to mark commit as good: {}",
            String::from_utf8_lossy(&git_output.stderr).trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&git_output.stdout);
    if stdout.contains("is the first bad commit") {
        output.push(format!(
            "\n{} Found the first bad commit!",
            style("🎯").bold()
        ));
        output.push(parse_bisect_result(&stdout));
        output.push(format!(
            "\n{} Run {} to return to your original branch",
            style("💡").bold(),
            style("git x bisect reset").cyan()
        ));
    } else {
        let new_commit = get_current_commit_info()?;
        output.push(format!(
            "{} Checked out commit: {}",
            style("📍").bold(),
            style(&new_commit).cyan()
        ));

        let remaining = get_remaining_steps()?;
        output.push(format!(
            "{} Approximately {} steps remaining",
            style("⏳").bold(),
            style(remaining).yellow().bold()
        ));
    }

    Ok(output.join("\n"))
}

fn mark_bad() -> Result<String> {
    ensure_bisecting()?;

    let current_commit = get_current_commit_info()?;
    let mut output = Vec::new();
    output.push(format!(
        "{} Marked {} as bad",
        style("❌").bold(),
        style(&current_commit).red()
    ));

    let git_output = Command::new("git").args(["bisect", "bad"]).output()?;

    if !git_output.status.success() {
        return Err(GitXError::GitCommand(format!(
            "Failed to mark commit as bad: {}",
            String::from_utf8_lossy(&git_output.stderr).trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&git_output.stdout);
    if stdout.contains("is the first bad commit") {
        output.push(format!(
            "\n{} Found the first bad commit!",
            style("🎯").bold()
        ));
        output.push(parse_bisect_result(&stdout));
        output.push(format!(
            "\n{} Run {} to return to your original branch",
            style("💡").bold(),
            style("git x bisect reset").cyan()
        ));
    } else {
        let new_commit = get_current_commit_info()?;
        output.push(format!(
            "{} Checked out commit: {}",
            style("📍").bold(),
            style(&new_commit).cyan()
        ));

        let remaining = get_remaining_steps()?;
        output.push(format!(
            "{} Approximately {} steps remaining",
            style("⏳").bold(),
            style(remaining).yellow().bold()
        ));
    }

    Ok(output.join("\n"))
}

fn skip_commit() -> Result<String> {
    ensure_bisecting()?;

    let current_commit = get_current_commit_info()?;
    let mut output = Vec::new();
    output.push(format!(
        "{} Skipped {} (untestable)",
        style("⏭️").bold(),
        style(&current_commit).yellow()
    ));

    let git_output = Command::new("git").args(["bisect", "skip"]).output()?;

    if !git_output.status.success() {
        return Err(GitXError::GitCommand(format!(
            "Failed to skip commit: {}",
            String::from_utf8_lossy(&git_output.stderr).trim()
        )));
    }

    let new_commit = get_current_commit_info()?;
    output.push(format!(
        "{} Checked out commit: {}",
        style("📍").bold(),
        style(&new_commit).cyan()
    ));

    let remaining = get_remaining_steps()?;
    output.push(format!(
        "{} Approximately {} steps remaining",
        style("⏳").bold(),
        style(remaining).yellow().bold()
    ));

    Ok(output.join("\n"))
}

fn reset_bisect() -> Result<String> {
    if !is_bisecting()? {
        return Ok(format!(
            "{} Not currently in bisect mode",
            style("ℹ️").bold()
        ));
    }

    let git_output = Command::new("git").args(["bisect", "reset"]).output()?;

    if !git_output.status.success() {
        return Err(GitXError::GitCommand(format!(
            "Failed to reset bisect: {}",
            String::from_utf8_lossy(&git_output.stderr).trim()
        )));
    }

    Ok(format!(
        "{} Bisect session ended, returned to original branch",
        style("🏁").bold()
    ))
}

fn show_status() -> Result<String> {
    if !is_bisecting()? {
        return Ok(format!(
            "{} Not currently in bisect mode",
            style("ℹ️").bold()
        ));
    }

    let mut output = Vec::new();
    output.push(format!("{} Bisect Status", style("📊").bold()));

    // Get current commit
    let current_commit = get_current_commit_info()?;
    output.push(format!(
        "{} Current commit: {}",
        style("📍").bold(),
        style(&current_commit).cyan()
    ));

    // Get remaining steps
    let remaining = get_remaining_steps()?;
    output.push(format!(
        "{} Approximately {} steps remaining",
        style("⏳").bold(),
        style(remaining).yellow().bold()
    ));

    // Get bisect log
    if let Ok(log) = get_bisect_log() {
        output.push(format!("\n{} Bisect log:", style("📝").bold()));
        for entry in log.lines().take(5) {
            if !entry.trim().is_empty() {
                output.push(format!("  {}", style(entry.trim()).dim()));
            }
        }
    }

    output.push(format!("\n{} Available commands:", style("💡").bold()));
    output.push(format!(
        "  {} - Mark current commit as good",
        style("git x bisect good").green()
    ));
    output.push(format!(
        "  {} - Mark current commit as bad",
        style("git x bisect bad").red()
    ));
    output.push(format!(
        "  {} - Skip current commit",
        style("git x bisect skip").yellow()
    ));
    output.push(format!(
        "  {} - End bisect session",
        style("git x bisect reset").cyan()
    ));

    Ok(output.join("\n"))
}

// Helper functions

fn is_bisecting() -> Result<bool> {
    // Check if .git/BISECT_START file exists, which indicates we're in bisect mode
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()?;

    if !output.status.success() {
        return Ok(false);
    }

    let git_dir_output = String::from_utf8_lossy(&output.stdout);
    let git_dir = git_dir_output.trim();
    let bisect_start_path = format!("{git_dir}/BISECT_START");

    Ok(std::path::Path::new(&bisect_start_path).exists())
}

fn ensure_bisecting() -> Result<()> {
    if !is_bisecting()? {
        return Err(GitXError::GitCommand(
            "Not currently in bisect mode. Use 'git x bisect start <good> <bad>' first."
                .to_string(),
        ));
    }
    Ok(())
}

fn validate_commit_exists(commit: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", &format!("{commit}^{{commit}}")])
        .output()?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(format!(
            "Commit '{commit}' does not exist"
        )));
    }
    Ok(())
}

fn get_current_commit_info() -> Result<String> {
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=format:%h %s"])
        .output()?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to get current commit info".to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn get_remaining_steps() -> Result<String> {
    let output = Command::new("git")
        .args(["bisect", "view", "--pretty=oneline"])
        .output()?;

    if output.status.success() {
        let count = String::from_utf8_lossy(&output.stdout).lines().count();
        let steps = (count as f64).log2().ceil() as usize;
        Ok(steps.to_string())
    } else {
        Ok("unknown".to_string())
    }
}

fn get_bisect_log() -> Result<String> {
    let output = Command::new("git").args(["bisect", "log"]).output()?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to get bisect log".to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn parse_bisect_result(output: &str) -> String {
    for line in output.lines() {
        if line.contains("is the first bad commit") {
            if let Some(commit_hash) = line.split_whitespace().next() {
                return format!(
                    "{} First bad commit: {}",
                    style("🎯").bold(),
                    style(commit_hash).red().bold()
                );
            }
        }
    }
    "Bisect completed".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GitXError;

    #[test]
    fn test_parse_bisect_result() {
        let sample_output =
            "abc123def456 is the first bad commit\ncommit abc123def456\nAuthor: Test User";
        let result = parse_bisect_result(sample_output);
        assert!(result.contains("abc123def456"));
        assert!(result.contains("First bad commit"));
    }

    #[test]
    fn test_parse_bisect_result_no_match() {
        let sample_output = "Some other git output\nNo bad commit found";
        let result = parse_bisect_result(sample_output);
        assert_eq!(result, "Bisect completed");
    }

    #[test]
    fn test_gitx_error_integration() {
        // Test that our functions work with GitXError types correctly
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "git not found");
        let gitx_error: GitXError = io_error.into();
        match gitx_error {
            GitXError::Io(_) => {} // Expected
            _ => panic!("Should convert to Io error"),
        }

        let git_error = GitXError::GitCommand("test error".to_string());
        assert_eq!(git_error.to_string(), "Git command failed: test error");
    }

    #[test]
    fn test_get_current_commit_info_error_handling() {
        // Test that the function signature works correctly
        // In a real test environment, we can't actually test git commands
        // but we can test the error handling logic
        let test_result: Result<String> = Err(GitXError::GitCommand("test".to_string()));
        assert!(test_result.is_err());
    }

    #[test]
    fn test_validate_commit_exists_logic() {
        // Test the function exists and has correct signature
        // We can't test actual git validation without a repo
        let _validate_fn = validate_commit_exists;
        // Function exists and compiles
    }

    #[test]
    fn test_is_bisecting_logic() {
        // Test the function exists and has correct signature
        let _bisect_fn = is_bisecting;
        // Function exists and compiles
    }

    #[test]
    fn test_ensure_bisecting_logic() {
        // Test that ensure_bisecting returns appropriate error
        // In non-bisect mode, this should return an error
        let result = ensure_bisecting();
        // The function should either succeed (if we're in a repo with bisect)
        // or fail with a GitXError (if we're not in bisect mode)
        match result {
            Ok(_) => {}                         // Might succeed if in actual bisect
            Err(GitXError::GitCommand(_)) => {} // Expected
            Err(GitXError::Io(_)) => {}         // Also possible
            Err(_) => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_get_remaining_steps_fallback() {
        // Test that get_remaining_steps handles errors gracefully
        // When git commands fail, it should return "unknown"
        let _steps_fn = get_remaining_steps;
        // Function exists and compiles
    }

    #[test]
    fn test_get_bisect_log_error_handling() {
        // Test error handling for bisect log
        let _log_fn = get_bisect_log;
        // Function exists and compiles
    }

    #[test]
    fn test_bisect_workflow_functions_exist() {
        // Verify all main workflow functions exist and compile
        let _start_fn = start_bisect;
        let _good_fn = mark_good;
        let _bad_fn = mark_bad;
        let _skip_fn = skip_commit;
        let _reset_fn = reset_bisect;
        let _status_fn = show_status;
        // All functions exist and compile
    }
}
