use std::process::Command;

pub fn run(branch_name: String, from: Option<String>) {
    // Validate branch name
    if let Err(msg) = validate_branch_name(&branch_name) {
        eprintln!("{}", format_error_message(msg));
        return;
    }

    // Check if branch already exists
    if branch_exists(&branch_name) {
        eprintln!("{}", format_branch_exists_message(&branch_name));
        return;
    }

    // Determine base branch
    let base_branch = match from {
        Some(ref branch) => {
            if !branch_exists(branch) && !is_valid_ref(branch) {
                eprintln!("{}", format_invalid_base_message(branch));
                return;
            }
            branch.clone()
        }
        None => match get_current_branch() {
            Ok(branch) => branch,
            Err(msg) => {
                eprintln!("{}", format_error_message(msg));
                return;
            }
        },
    };

    println!(
        "{}",
        format_creating_branch_message(&branch_name, &base_branch)
    );

    // Create the new branch
    if let Err(msg) = create_branch(&branch_name, &base_branch) {
        eprintln!("{}", format_error_message(msg));
        return;
    }

    // Switch to the new branch
    if let Err(msg) = switch_to_branch(&branch_name) {
        eprintln!("{}", format_error_message(msg));
        return;
    }

    println!("{}", format_success_message(&branch_name));
}

// Helper function to validate branch name
fn validate_branch_name(name: &str) -> Result<(), &'static str> {
    if name.is_empty() {
        return Err("Branch name cannot be empty");
    }

    if name.starts_with('-') {
        return Err("Branch name cannot start with a dash");
    }

    if name.contains("..") {
        return Err("Branch name cannot contain '..'");
    }

    if name.contains(' ') {
        return Err("Branch name cannot contain spaces");
    }

    // Check for invalid characters
    const INVALID_CHARS: &[char] = &['~', '^', ':', '?', '*', '[', '\\'];
    if name.chars().any(|c| INVALID_CHARS.contains(&c)) {
        return Err("Branch name contains invalid characters");
    }

    Ok(())
}

// Helper function to check if branch exists
fn branch_exists(branch_name: &str) -> bool {
    Command::new("git")
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch_name}"),
        ])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

// Helper function to check if ref is valid
fn is_valid_ref(ref_name: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", ref_name])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

// Helper function to get current branch
fn get_current_branch() -> Result<String, &'static str> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|_| "Failed to get current branch")?;

    if !output.status.success() {
        return Err("Not in a git repository");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// Helper function to create branch
fn create_branch(branch_name: &str, base_branch: &str) -> Result<(), &'static str> {
    let status = Command::new("git")
        .args(["branch", branch_name, base_branch])
        .status()
        .map_err(|_| "Failed to execute git branch command")?;

    if !status.success() {
        return Err("Failed to create branch");
    }

    Ok(())
}

// Helper function to switch to branch
fn switch_to_branch(branch_name: &str) -> Result<(), &'static str> {
    let status = Command::new("git")
        .args(["switch", branch_name])
        .status()
        .map_err(|_| "Failed to execute git switch command")?;

    if !status.success() {
        return Err("Failed to switch to new branch");
    }

    Ok(())
}

// Helper function to get branch validation error messages
pub fn get_validation_rules() -> &'static [&'static str] {
    &[
        "Cannot be empty",
        "Cannot start with a dash",
        "Cannot contain '..'",
        "Cannot contain spaces",
        "Cannot contain ~^:?*[\\",
    ]
}

// Helper function to format error message
pub fn format_error_message(msg: &str) -> String {
    format!("❌ {msg}")
}

// Helper function to format branch exists message
pub fn format_branch_exists_message(branch_name: &str) -> String {
    format!("❌ Branch '{branch_name}' already exists")
}

// Helper function to format invalid base message
pub fn format_invalid_base_message(base_branch: &str) -> String {
    format!("❌ Base branch or ref '{base_branch}' does not exist")
}

// Helper function to format creating branch message
pub fn format_creating_branch_message(branch_name: &str, base_branch: &str) -> String {
    format!("🌿 Creating branch '{branch_name}' from '{base_branch}'...")
}

// Helper function to format success message
pub fn format_success_message(branch_name: &str) -> String {
    format!("✅ Created and switched to branch '{branch_name}'")
}

// Helper function to get git branch creation args
pub fn get_git_branch_args() -> [&'static str; 2] {
    ["branch", "-"]
}

// Helper function to get git switch args
pub fn get_git_switch_args() -> [&'static str; 2] {
    ["switch", "-"]
}
