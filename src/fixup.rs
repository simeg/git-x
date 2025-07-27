use std::process::Command;

pub fn run(commit_hash: String, rebase: bool) {
    // Validate the commit hash exists
    if let Err(msg) = validate_commit_hash(&commit_hash) {
        eprintln!("{}", format_error_message(msg));
        return;
    }

    // Get current staged and unstaged changes
    let has_changes = match check_for_changes() {
        Ok(has_changes) => has_changes,
        Err(msg) => {
            eprintln!("{}", format_error_message(msg));
            return;
        }
    };

    if !has_changes {
        eprintln!("{}", format_no_changes_message());
        return;
    }

    // Get the short commit hash for better UX
    let short_hash = match get_short_commit_hash(&commit_hash) {
        Ok(hash) => hash,
        Err(msg) => {
            eprintln!("{}", format_error_message(msg));
            return;
        }
    };

    println!("{}", format_creating_fixup_message(&short_hash));

    // Create the fixup commit
    if let Err(msg) = create_fixup_commit(&commit_hash) {
        eprintln!("{}", format_error_message(msg));
        return;
    }

    println!("{}", format_fixup_created_message(&short_hash));

    // Optionally run interactive rebase with autosquash
    if rebase {
        println!("{}", format_starting_rebase_message());
        if let Err(msg) = run_autosquash_rebase(&commit_hash) {
            eprintln!("{}", format_error_message(msg));
            eprintln!("{}", format_manual_rebase_hint(&commit_hash));
            return;
        }
        println!("{}", format_rebase_success_message());
    } else {
        println!("{}", format_manual_rebase_hint(&commit_hash));
    }
}

// Helper function to validate commit hash exists
fn validate_commit_hash(commit_hash: &str) -> Result<(), &'static str> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", &format!("{commit_hash}^{{commit}}")])
        .output()
        .map_err(|_| "Failed to validate commit hash")?;

    if !output.status.success() {
        return Err("Commit hash does not exist");
    }

    Ok(())
}

// Helper function to check for changes to commit
fn check_for_changes() -> Result<bool, &'static str> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .status()
        .map_err(|_| "Failed to check for staged changes")?;

    // If staged changes exist, we're good
    if !output.success() {
        return Ok(true);
    }

    // Check for unstaged changes
    let output = Command::new("git")
        .args(["diff", "--quiet"])
        .status()
        .map_err(|_| "Failed to check for unstaged changes")?;

    // If unstaged changes exist, we need to stage them
    if !output.success() {
        return Err("You have unstaged changes. Please stage them first with 'git add'");
    }

    Ok(false)
}

// Helper function to get short commit hash
fn get_short_commit_hash(commit_hash: &str) -> Result<String, &'static str> {
    let output = Command::new("git")
        .args(["rev-parse", "--short", commit_hash])
        .output()
        .map_err(|_| "Failed to get short commit hash")?;

    if !output.status.success() {
        return Err("Failed to resolve commit hash");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// Helper function to create fixup commit
fn create_fixup_commit(commit_hash: &str) -> Result<(), &'static str> {
    let status = Command::new("git")
        .args(["commit", &format!("--fixup={commit_hash}")])
        .status()
        .map_err(|_| "Failed to create fixup commit")?;

    if !status.success() {
        return Err("Failed to create fixup commit");
    }

    Ok(())
}

// Helper function to run autosquash rebase
fn run_autosquash_rebase(commit_hash: &str) -> Result<(), &'static str> {
    // Find the parent of the target commit for rebase
    let output = Command::new("git")
        .args(["rev-parse", &format!("{commit_hash}^")])
        .output()
        .map_err(|_| "Failed to find parent commit")?;

    if !output.status.success() {
        return Err("Cannot rebase - commit has no parent");
    }

    let parent_hash_string = String::from_utf8_lossy(&output.stdout);
    let parent_hash = parent_hash_string.trim();

    let status = Command::new("git")
        .args(["rebase", "-i", "--autosquash", parent_hash])
        .status()
        .map_err(|_| "Failed to start interactive rebase")?;

    if !status.success() {
        return Err("Interactive rebase failed");
    }

    Ok(())
}

// Helper function to get git commit args for fixup
pub fn get_git_fixup_args() -> [&'static str; 2] {
    ["commit", "--fixup"]
}

// Helper function to get git rebase args
pub fn get_git_rebase_args() -> [&'static str; 3] {
    ["rebase", "-i", "--autosquash"]
}

// Helper function to format error message
pub fn format_error_message(msg: &str) -> String {
    format!("❌ {msg}")
}

// Helper function to format no changes message
pub fn format_no_changes_message() -> &'static str {
    "❌ No staged changes found. Please stage your changes first with 'git add'"
}

// Helper function to format creating fixup message
pub fn format_creating_fixup_message(short_hash: &str) -> String {
    format!("🔧 Creating fixup commit for {short_hash}...")
}

// Helper function to format fixup created message
pub fn format_fixup_created_message(short_hash: &str) -> String {
    format!("✅ Fixup commit created for {short_hash}")
}

// Helper function to format starting rebase message
pub fn format_starting_rebase_message() -> &'static str {
    "🔄 Starting interactive rebase with autosquash..."
}

// Helper function to format rebase success message
pub fn format_rebase_success_message() -> &'static str {
    "✅ Interactive rebase completed successfully"
}

// Helper function to format manual rebase hint
pub fn format_manual_rebase_hint(commit_hash: &str) -> String {
    format!("💡 To squash the fixup commit, run: git rebase -i --autosquash {commit_hash}^")
}

// Helper function to check if commit hash is valid format
pub fn is_valid_commit_hash_format(hash: &str) -> bool {
    if hash.is_empty() {
        return false;
    }

    // Must be 4-40 characters long (short to full hash)
    if hash.len() < 4 || hash.len() > 40 {
        return false;
    }

    // Must contain only hex characters
    hash.chars().all(|c| c.is_ascii_hexdigit())
}

// Helper function to format commit validation rules
pub fn get_commit_hash_validation_rules() -> &'static [&'static str] {
    &[
        "Must be 4-40 characters long",
        "Must contain only hex characters (0-9, a-f)",
        "Must reference an existing commit",
    ]
}