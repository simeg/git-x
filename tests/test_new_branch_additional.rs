// Additional tests for new_branch.rs to increase coverage

use git_x::new_branch::*;

#[test]
fn test_get_validation_rules() {
    let rules = get_validation_rules();
    assert!(!rules.is_empty());

    // Check that rules contain expected validation information
    let rules_text = rules.join(" ");
    assert!(
        rules_text.contains("empty")
            || rules_text.contains("dash")
            || rules_text.contains("spaces")
            || rules_text.contains("contain")
    );
}

#[test]
fn test_format_error_message() {
    assert_eq!(format_error_message("test error"), "❌ test error");
    assert_eq!(format_error_message(""), "❌ ");
    assert_eq!(
        format_error_message("Branch creation failed"),
        "❌ Branch creation failed"
    );
    assert_eq!(
        format_error_message("Invalid branch name"),
        "❌ Invalid branch name"
    );
}

#[test]
fn test_format_branch_exists_message() {
    assert_eq!(
        format_branch_exists_message("main"),
        "❌ Branch 'main' already exists"
    );
    assert_eq!(
        format_branch_exists_message("feature/test"),
        "❌ Branch 'feature/test' already exists"
    );
    assert_eq!(
        format_branch_exists_message(""),
        "❌ Branch '' already exists"
    );
}

#[test]
fn test_format_invalid_base_message() {
    assert_eq!(
        format_invalid_base_message("nonexistent"),
        "❌ Base branch or ref 'nonexistent' does not exist"
    );
    assert_eq!(
        format_invalid_base_message("origin/missing"),
        "❌ Base branch or ref 'origin/missing' does not exist"
    );
    assert_eq!(
        format_invalid_base_message(""),
        "❌ Base branch or ref '' does not exist"
    );
}

#[test]
fn test_format_creating_branch_message() {
    assert_eq!(
        format_creating_branch_message("new-branch", "main"),
        "🌿 Creating branch 'new-branch' from 'main'..."
    );
    assert_eq!(
        format_creating_branch_message("feature/awesome", "develop"),
        "🌿 Creating branch 'feature/awesome' from 'develop'..."
    );
    assert_eq!(
        format_creating_branch_message("", ""),
        "🌿 Creating branch '' from ''..."
    );
}

#[test]
fn test_format_success_message() {
    assert_eq!(
        format_success_message("new-branch"),
        "✅ Created and switched to branch 'new-branch'"
    );
    assert_eq!(
        format_success_message("feature/test"),
        "✅ Created and switched to branch 'feature/test'"
    );
    assert_eq!(
        format_success_message(""),
        "✅ Created and switched to branch ''"
    );
}

#[test]
fn test_get_git_branch_args() {
    let args = get_git_branch_args();
    assert_eq!(args.len(), 2);
    assert_eq!(args[0], "branch");
    assert_eq!(args[1], "-");
}

#[test]
fn test_get_git_switch_args() {
    let args = get_git_switch_args();
    assert_eq!(args.len(), 2);
    assert_eq!(args[0], "switch");
    assert_eq!(args[1], "-");
}

#[test]
fn test_message_formatting_edge_cases() {
    // Test with special characters and edge cases
    assert!(
        format_creating_branch_message("test/branch-123", "origin/main")
            .contains("test/branch-123")
    );
    assert!(
        format_creating_branch_message("test/branch-123", "origin/main").contains("origin/main")
    );

    assert!(format_success_message("feature/issue-456").contains("feature/issue-456"));
    assert!(format_branch_exists_message("hotfix/urgent").contains("hotfix/urgent"));
    assert!(
        format_invalid_base_message("refs/remotes/origin/feature")
            .contains("refs/remotes/origin/feature")
    );
}

#[test]
fn test_format_consistency() {
    // Test that all format functions return non-empty strings for reasonable inputs
    assert!(!format_error_message("test").is_empty());
    assert!(!format_branch_exists_message("test").is_empty());
    assert!(!format_invalid_base_message("test").is_empty());
    assert!(!format_creating_branch_message("test", "main").is_empty());
    assert!(!format_success_message("test").is_empty());

    // Test that they include expected emojis or symbols
    assert!(format_error_message("test").contains("❌"));
    assert!(format_branch_exists_message("test").contains("❌"));
    assert!(format_invalid_base_message("test").contains("❌"));
    assert!(format_creating_branch_message("test", "main").contains("🌿"));
    assert!(format_success_message("test").contains("✅"));
}
