# 🚀 git-x – Superpowers for Git [![CI](https://github.com/simeg/git-x/actions/workflows/ci.yaml/badge.svg)](https://github.com/simeg/git-x/actions/workflows/ci.yaml)

**`git-x`** is a collection of smarter, faster, and more intuitive Git subcommands built to make your daily workflow suck less.

It wraps common Git actions in muscle-memory-friendly, no-brainer commands — perfect for solo developers, team leads, and anyone tired of typing `git log --oneline --graph --decorate --all` for the 400th time.

![Banner](banner.png)

---

## 📚 Table of Contents

- [Why Does This Exist?](#why-does-this-exist)
- [Installation](#installation)
- [Example Commands](#example-commands)
    - [Repository Information & Analysis](#repository-information--analysis)
        - [`info`](#info) - High-level repository overview
        - [`health`](#health) - Repository health check
        - [`summary`](#summary) - Commit summary and stats
        - [`contributors`](#contributors) - Contributor statistics
        - [`technical-debt`](#technical-debt) - Code complexity analysis
        - [`large-files`](#large-files) - Find largest files
    - [Branch Management](#branch-management)
        - [`new`](#new) - Create and switch to new branch
        - [`rename-branch`](#rename-branch) - Rename current branch
        - [`switch-recent`](#switch-recent) - Interactive branch picker
        - [`clean-branches`](#clean-branches) - Delete all merged branches
        - [`prune-branches`](#prune-branches) - Delete branches merged into current
        - [`upstream`](#upstream) - Manage upstream relationships
    - [Commit History & Visualization](#commit-history--visualization)
        - [`graph`](#graph) - Pretty commit graph
        - [`color-graph`](#color-graph) - Colorized commit graph
        - [`since [ref]`](#since-ref) - Show commits since reference
        - [`what [branch]`](#what-branch) - Compare branches
    - [Commit Operations](#commit-operations)
        - [`fixup`](#fixup) - Create fixup commits
        - [`undo`](#undo) - Undo last commit safely
        - [`bisect`](#bisect) - Simplified bisect workflow
    - [Stash Management](#stash-management)
        - [`stash-branch`](#stash-branch) - Advanced stash operations
    - [Synchronization](#synchronization)
        - [`sync`](#sync) - Sync with upstream
- [Git Integration: How `git-x` Just Works™](#git-integration-how-git-x-just-works)
- [What's Under the Hood?](#whats-under-the-hood)
- [Performance](#performance)
- [Command Transparency](#command-transparency)
- [Roadmap Ideas](#roadmap-ideas)
- [Built With](#built-with)
- [License](#license)

---

## Why Does This Exist?

Git is powerful, but its UX is stuck in the early 2000s.

You're probably asking:

- “Why is this branch here?”
- “What changed since I last pushed?”
- “Can I just get a clean, visual summary of this repo?”
- “How do I undo that commit without wrecking everything?”

Most Git tools either:
- Show *too much* (spammy logs, unreadable diffs)
- Show *too little* (cryptic one-liners with no context)
- Or require memorizing a dozen flags

`git-x` fixes that by giving you **opinionated, purpose-built subcommands** that *just do the thing*.

> 💡 **Want to see git-x in action?** Check out our [**Real-World Scenarios**](docs/real-world-scenarios.md) document to see exactly how git-x commands solve everyday developer problems like code review cleanup, branch naming mistakes, urgent context switching, and complex remote workflows.

---

## Installation

```shell
cargo install git-x
```

Or clone and run manually:

```shell
git clone https://github.com/simeg/git-x
cd git-x
cargo install --path .
```

### Shell Completion

`git-x` supports tab completion for all shells.

#### Installation

Install completion files to standard directories:

```shell
# Install for your shell
git x completion-install bash
git x completion-install zsh
git x completion-install fish
```

This will:
- Install completion files to the standard system directories
- Provide shell-specific setup instructions
- Create directories if they don't exist

**Follow the printed instructions after installation to enable completions in your shell configuration.**

#### Troubleshooting

If tab completion doesn't work immediately, you may need to:

```shell
# For zsh - clear completion cache
rm ~/.zcompdump*

# For bash - refresh command hash
hash -r

# For fish - clear completion cache
fish -c "complete --erase"

# Then restart your shell
```

---

## Example Commands

---

## Repository Information & Analysis

### `info`

> Show a high-level overview of the current repo  
> [🔍 *Git commands*](docs/command-internals.md#info)

```shell
git x info
```

#### Output:

```shell
🗂️  Repository: git-x
📍 Current branch: master
🔗 Upstream: origin/master
✅ Status: Up to date
⚠️  Working directory: Has changes
📋 Staged files: None
❌ No open PR for current branch
📊 vs main: 2 ahead, 1 behind

📋 Recent activity:
   * a1b2c3d Add new feature (2 hours ago) <Alice>
   * d4e5f6g Fix bug in parser (4 hours ago) <Bob>
   * g7h8i9j Update documentation (1 day ago) <Charlie>
```

#### Enhanced Features:
- **Recent activity timeline** - Shows recent commits across all branches with author info
- **GitHub PR detection** - Automatically detects if current branch has an open pull request (requires `gh` CLI)
- **Branch comparisons** - Shows ahead/behind status compared to main branches
- **Detailed view** - Use any git-x command to see additional details

---

### `health`

> Check repository health and identify potential issues  
> [🔍 *Git commands*](docs/command-internals.md#health)

```shell
git x health
```

#### Output:

```shell
🏥 Repository Health Check
==============================
⠁ [00:00:01] [########################################] 8/8 Health check complete!
✅ Git configuration: OK
✅ Remotes: OK
✅ Branches: OK
✅ Working directory: Clean
✅ Repository size: OK
⚠️  Security: Potential issues found
✅ .gitignore: Looks good
✅ Binary files: OK

🔧 Found 3 issue(s):
   🔒 2 potentially sensitive commit message(s) found:
        • a1b2c3d Add API key configuration
        • d4e5f6g Update secret token handling
   🔐 1 potentially sensitive file(s) in repository:
        • config/private.key
   ⚠️  2 environment file(s) found - ensure no secrets are committed:
        • .env.local
        • .env.production
```

#### What it checks:
- **Git configuration** - Validates user.name and user.email settings
- **Remotes** - Ensures remote repositories are configured
- **Working directory status** - Detects uncommitted changes
- **Untracked files** - Counts files not under version control
- **Stale branches** - Identifies branches older than 1 month
- **Repository size** - Warns about large repositories that may need cleanup
- **Staged changes** - Shows files ready for commit
- **Security issues** - Scans for potential credentials in history and sensitive files
- **.gitignore effectiveness** - Suggests improvements to ignore patterns
- **Binary files** - Identifies large binary files that might benefit from Git LFS

#### Enhanced Features:
- **Progress Indicator**: Real-time progress bar showing current check being performed
- **Detailed Security Reporting**: Shows exactly which commits, files, and patterns triggered security warnings
- **Specific Recommendations**: Lists actual files and examples instead of just counts
- **Performance Optimized**: Efficiently scans large repositories with visual feedback

Useful for:
- Daily repository maintenance
- Pre-commit health checks
- Security auditing
- Identifying cleanup opportunities
- Team onboarding (ensuring clean local state)

---

### `summary`

> Show a short, changelog-style summary of recent commits  
> [🔍 *Git commands*](docs/command-internals.md#summary)

```shell
git x summary
git x summary --since "2 days ago"
```

**Flags:**
- `--since` — Accepts natural date formats like "2 days ago", "last Monday", or exact dates like "2025-07-01". It uses Git's built-in date parser, so most human-readable expressions work.

#### Output:

**Without `--since` flag (shows repository summary):**
```shell
📊 Repository Summary
==================================================
🗂️  Repository: git-x
📍 Current branch: master
🔗 Upstream: origin/master (up to date)
📈 Commits (1 month ago): 72
📁 Files: 63 total
```

**With `--since` flag (shows changelog-style commit history):**
```shell
📅 Commit Summary since 2 days ago:
==================================================

📆 2025-07-30
 - 🔹 Big re-architecture (by Simon Egersand, 4 hours ago)
 - 🐛 Fix remaining test failures (by Alice, 6 hours ago)

📆 2025-07-29
 - ✨ Add new features (by Bob, 1 day ago)
 - 🛠 Refactor core components (by Carol, 1 day ago)
```

- **Default behavior**: Shows repository overview with stats from the last month
- **With `--since`**: Groups commits by day with commit messages, authors, and timestamps
- Useful for writing daily stand-ups, changelogs, or review summaries
- Can be customized using `--since` (e.g. `--since "1 week ago"`)
- Sorts commits newest-first within each day

---

### `contributors`

> Show contributor statistics for the repository  
> [🔍 *Git commands*](docs/command-internals.md#contributors)

```shell
git x contributors
```

#### Output:

```shell
📊 Repository Contributors (15 total commits):

🥇 Alice Smith 10 commits (66.7%)
   📧 alice@example.com | 📅 2025-01-01 to 2025-01-20

🥈 Bob Jones 3 commits (20.0%)
   📧 bob@example.com | 📅 2025-01-05 to 2025-01-15

🥉 Charlie Brown 2 commits (13.3%)
   📧 charlie@example.com | 📅 2025-01-10 to 2025-01-12
```

Shows repository contributors ranked by commit count with email addresses and date ranges of their contributions.

---

### `technical-debt`

> Analyze code complexity and technical debt metrics  
> [🔍 *Git commands*](docs/command-internals.md#technical-debt)

```shell
git x technical-debt
```

#### Output:

```shell
🔍 Technical Debt Analysis

📊 Large Commits (>20 files changed)
   ✓ No large commits found

🔥 File Hotspots (frequently modified)
   1. 15 changes | HIGH | src/main.rs
   2. 8 changes | MED | src/lib.rs
   3. 6 changes | LOW | README.md

🌿 Long-lived Branches (>30 days)
   • feature/old-refactor | 3 months ago | Alice Smith
   • hotfix/legacy-fix | 6 weeks ago | Bob Jones

🔄 Code Churn (high add/delete ratio)
   1. +245 -189 | HIGH | src/parser.rs
   2. +156 -98 | MED | src/utils.rs

📦 Binary Files in Repository
   ! 3 binary files found
   • assets/logo.png
   • docs/manual.pdf
   ...

Analysis complete!
```

Analyzes repository for technical debt indicators including large commits, file modification hotspots, long-lived branches, code churn patterns, and binary file usage.

---

### `large-files`

> Find largest files in repository history  
> [🔍 *Git commands*](docs/command-internals.md#large-files)

```shell
git x large-files
git x large-files --limit 20 --threshold 5
```

#### Output:

```shell
🔍 Scanning repository for large files...

📁 Largest files in repository history:
  15.2 MB  assets/video.mp4
   8.7 MB  docs/presentation.pdf
   3.1 MB  images/hero-banner.png

💡 Found 3 files larger than 1.0 MB
```

**Flags:**
- `--limit <number>` — Number of files to show (default: 10)
- `--threshold <MB>` — Minimum file size in MB to include

Useful for identifying large files that may be slowing down your repository.

---

## Branch Management

### `new`

> Create and switch to a new branch  
> [🔍 *Git commands*](docs/command-internals.md#new)

```shell
git x new feature-branch
git x new hotfix --from main
```

#### Output:

```shell
🌿 Creating branch 'feature-branch' from 'current-branch'...
✅ Created and switched to branch 'feature-branch'
```

**Flags:**
- `--from <branch>` — Base the new branch off a specific branch instead of current

Validates branch names and prevents common Git naming issues.

---

### `rename-branch`

> Rename the current branch locally and on remote  
> [🔍 *Git commands*](docs/command-internals.md#rename-branch)

```shell
git x rename-branch new-feature-name
```

#### Output:

```shell
🔄 Renaming branch 'old-name' to 'new-feature-name'...
✅ Branch renamed successfully
```

Safely renames your current branch by:
- Renaming the local branch
- Updating the remote tracking branch
- Cleaning up old remote references

---

### `switch-recent`

> Interactive picker for recent branches  
> [🔍 *Git commands*](docs/command-internals.md#switch-recent)

```shell
git x switch-recent
```

#### Output:

```shell
? Select a branch to switch to:
  🌟 feature/auth-improvement
  📁 hotfix/login-bug
  📁 feature/dark-mode
  📁 main
```

Shows an interactive menu of your 10 most recently used branches (excluding current branch). Use arrow keys to navigate, Enter to select.

---

### `clean-branches`

> Delete all fully merged local branches (except protected ones)  
> [🔍 *Git commands*](docs/command-internals.md#clean-branches)

```shell
git x clean-branches
git x clean-branches --dry-run  # Preview what would be deleted
```

#### Output:

```shell
⚠️  Are you sure you want to clean merged branches?
This will delete 3 merged branches: feature/refactor, bugfix/signup-typo, hotfix/quick-fix
[y/N]: y

🧹 Deleted 3 merged branches:
- feature/refactor
- bugfix/signup-typo
- hotfix/quick-fix
```

**Flags:**
- `--dry-run` — Show which branches would be deleted without actually deleting them

**Note:** This command will prompt for confirmation before deleting branches to prevent accidental deletions.

---

### `prune-branches`

> Delete branches merged into current branch  
> [🔍 *Git commands*](docs/command-internals.md#prune-branches)

Deletes all **local branches** that have already been **merged into the current branch**, while skipping protected ones.

Useful for keeping your repo tidy after merging feature branches.

```shell
git x prune-branches
git x prune-branches --except "release,v1.0-temp"
git x prune-branches --dry-run  # Preview what would be deleted
```

#### Output:

```shell
⚠️  Are you sure you want to prune merged branches?
This will delete 2 merged branches: feature/completed-task, hotfix/old-bug
[y/N]: y

🧹 Deleted merged branch 'feature/completed-task'
🧹 Deleted merged branch 'hotfix/old-bug'
```

**Defaults:**
- Protected branches: `main`, `master`, `develop`
- Won't delete current branch
- Will only delete branches that are *fully merged*

**Flags:**
- `--except <branches>` — Comma-separated list of branch names to exclude from deletion
- `--dry-run` — Show which branches would be deleted without actually deleting them

**Note:** This command will prompt for confirmation before deleting branches to prevent accidental deletions.

---

### `upstream`

> Manage upstream branch relationships  
> [🔍 *Git commands*](docs/command-internals.md#upstream)

```shell
git x upstream status
git x upstream set origin/main
git x upstream sync-all --dry-run
```

#### Subcommands:

**`status`** — Show upstream status for all branches

```shell
🔗 Upstream status for all branches:
* main -> origin/main
  feature -> (no upstream)
  hotfix -> origin/hotfix
```

**`set <upstream>`** — Set upstream for current branch

**`sync-all`** — Sync all local branches with their upstreams
- `--dry-run` — Show what would be synced without doing it
- `--merge` — Use merge instead of rebase

Streamlines upstream branch management across your entire repository.

---

## Commit History & Visualization

### `graph`

> Pretty Git log with branches, remotes, and HEADs  
> [🔍 *Git commands*](docs/command-internals.md#graph)

```shell
git x graph
```

#### Output:

```shell
* fc27857 (HEAD -> master, origin/master) Make tests more robust
* d109d83 Fix remaining test failures with improved git repository detection
* ded10bb Apply code formatting and linting fixes
| * 71b448a (feature/auth-improvement) Apply code formatting and linting fixes
|/  
* 6c69a03 Fix failing tests on GitHub Actions with robust error handling
* 4f6565e Fix tests
* 433788a Update README.md
* 8594ff0 Implement comprehensive layered architecture for code structure reorganization
```

---

### `color-graph`

> Colorized Git log with branches, remotes, HEADs, and author info  
> [🔍 *Git commands*](docs/command-internals.md#color-graph)

```shell
git x color-graph
```

#### Output:

Enhanced version of `git x graph` with:
- **Full color support** for branches, commits, and decorations
- **Author names and timestamps** for each commit
- **Rich formatting** that's easy to scan

---

### `since [ref]`

> Show commits since a reference (e.g., `d926b4b`, my-branch, origin/main)  
> [🔍 *Git commands*](docs/command-internals.md#since-ref)

```shell
git x since origin/main
```

#### Output:

```shell
🔍 Commits since origin/main:
- 8f2d9b3 fix login bug
- b41a71e add auth test
```

---

### `what [branch]`

> Show what's different between this branch and another (default: main)  
> [🔍 *Git commands*](docs/command-internals.md#what-branch)

```shell
git x what
git x what --target develop
```

#### Output:

```shell
Branch: feature/new-ui vs main
+ 4 commits ahead
- 2 commits behind
Changes:
 - + new_ui.js
 - ~ App.tsx
 - - old_ui.css
```

**Flags:**
- `--target <branch>` — Branch to compare to (default: main)

---

## Commit Operations

### `fixup`

> Create fixup commits for easier interactive rebasing  
> [🔍 *Git commands*](docs/command-internals.md#fixup)

```shell
git x fixup abc123
git x fixup abc123 --rebase
```

#### Output:

```shell
🔧 Creating fixup commit for abc123...
✅ Fixup commit created for abc123
💡 To squash the fixup commit, run: git rebase -i --autosquash abc123^
```

**Flags:**
- `--rebase` — Automatically start interactive rebase with autosquash after creating fixup

Creates a fixup commit that can be automatically squashed during interactive rebase. Requires staged changes.

---

### `undo`

> Undo the last commit (without losing changes)  
> [🔍 *Git commands*](docs/command-internals.md#undo)

```shell
git x undo
```

#### Output:

```shell
Last commit undone (soft reset). Changes kept in working directory.
```

---

### `bisect`

> Simplified bisect workflow  
> [🔍 *Git commands*](docs/command-internals.md#bisect)

```shell
# Start bisect session
git x bisect start <good-commit> <bad-commit>

# Mark current commit as good/bad/untestable
git x bisect good
git x bisect bad
git x bisect skip

# Show bisect status
git x bisect status

# End bisect session
git x bisect reset
```

#### Example workflow:

```shell
# Start bisecting between a known good commit and HEAD
git x bisect start HEAD HEAD~10

# Git checks out a commit for testing
# Test your code, then mark the commit:
git x bisect bad    # if bug is present
git x bisect good   # if bug is not present
git x bisect skip   # if commit is untestable

# Repeat until git finds the first bad commit
git x bisect reset  # return to original branch
```

#### Output:

```shell
🔍 Starting bisect between abc123 (good) and def456 (bad)
📍 Checked out commit: 789abc Fix user authentication
⏳ Approximately 3 steps remaining

💡 Test this commit and run:
  git x bisect good if commit is good
  git x bisect bad if commit is bad
  git x bisect skip if commit is untestable
```

---

## Stash Management

### `stash-branch`

> Advanced stash management with branch integration  
> [🔍 *Git commands*](docs/command-internals.md#stash-branch)

```shell
git x stash-branch create new-feature
git x stash-branch clean --older-than 7d
git x stash-branch apply-by-branch feature-work
git x stash-branch interactive
git x stash-branch export ./patches
```

#### Subcommands:

**`create <branch-name>`** — Create a new branch from a stash
- `--stash <ref>` — Use specific stash (default: latest)

**`clean`** — Clean up old stashes
- `--older-than <time>` — Remove stashes older than specified time
- `--dry-run` — Show what would be cleaned without doing it

#### Example Output for `clean`:

```shell
🧹 Cleaning 3 stash(es):
  stash@{0}: WIP on feature: Add new component
  stash@{1}: On main: Fix typo in README
  stash@{2}: WIP on bugfix: Debug auth issue

⚠️  Are you sure you want to clean old stashes?
This will delete 3 stashes: stash@{0}, stash@{1}, stash@{2}
[y/N]: y

✅ Cleaned 3 stash(es)
```

**`apply-by-branch <branch-name>`** — Apply stashes from a specific branch
- `--list` — List matching stashes instead of applying

**`interactive`** — Interactive stash management with fuzzy search
- Visual menu for applying, deleting, or creating branches from stashes
- Supports multiple selection for batch operations
- Shows stash content and branch associations

**`export <output-dir>`** — Export stashes to patch files
- `--stash <ref>` — Export specific stash (default: all stashes)
- Creates `.patch` files that can be shared or archived
- Useful for backing up or sharing stash content

#### Example Output for `interactive`:

```shell
📋 What would you like to do?
❯ Apply selected stash
  Delete selected stashes
  Create branch from stash
  Show stash diff
  List all stashes
  Exit

🎯 Select stash to apply:
❯ stash@{0}: WIP on feature: Add authentication (from feature-auth)
  stash@{1}: On main: Fix README typo (from main)
  stash@{2}: WIP on bugfix: Debug API calls (from api-fixes)
```

Helps manage stashes more effectively by associating them with branches and providing modern interactive workflows.

**Note:** Interactive and destructive commands will prompt for confirmation to prevent accidental data loss.

---

## Synchronization

### `sync`

> Sync current branch with upstream (fetch + rebase/merge)  
> [🔍 *Git commands*](docs/command-internals.md#sync)

```shell
git x sync
git x sync --merge
```

#### Output:

```shell
🔄 Syncing branch 'feature' with 'origin/feature'...
⬇️ Branch is 2 commit(s) behind upstream
✅ Successfully rebased onto upstream
```

**Flags:**
- `--merge` — Use merge instead of rebase for integration

Automatically fetches from remote and integrates upstream changes into your current branch.


## Command Transparency

`git-x` believes in **complete transparency** — there's no magic, no hidden behavior, and no surprise side effects.

Every `git-x` command is a **thin wrapper** around standard Git operations that you could run yourself. Want to know exactly what's happening under the hood? Check out our [**Command Internals Documentation**](docs/command-internals.md).

**Why this matters:**
- **Trust** — You can verify every operation before and after
- **Learning** — Understand the Git commands you're actually running  
- **Debugging** — When something goes wrong, you know exactly what to investigate
- **Portability** — You can replicate any `git-x` workflow with plain Git

**Example:** When you run `git x graph`, it literally executes:
```shell
git log --graph --oneline --all -20
```

No database calls, no hidden state, no magic — just Git doing Git things, with better UX.

---

## Git Integration: How `git-x` Just Works™

Since `git-x` is installed as a standalone binary, Git automatically recognizes it as a subcommand when you type `git x [command]`.

This is Git's standard extension mechanism — no configuration needed.

**How it works:**
1. You run `git x info`
2. Git looks for an executable called `git-x` in your `$PATH`
3. Git runs `git-x info` and displays the output

**Why this rocks:**
- Zero setup required
- Works in any Git repo
- Integrates seamlessly with your existing Git workflow
- All your Git aliases, hooks, and config still work

---

## What's Under the Hood?

`git-x` is a thin, opinionated wrapper around native Git commands.

**Philosophy:**
- **No magic** — Every `git-x` command maps to standard Git operations
- **Readable** — You can see exactly what Git commands are being run
- **Predictable** — Follows Git's existing patterns and conventions
- **Fast** — Minimal overhead, direct subprocess calls

**Example:** `git x graph` literally runs:
```shell
git log --graph --oneline --all -20
```

**Why Rust?**
- **Fast startup** — Sub-millisecond command execution
- **Zero dependencies** — Single binary, no runtime requirements
- **Cross-platform** — Works on macOS, Linux, Windows
- **Memory safe** — No crashes, no memory leaks

---

## Performance

`git-x` is designed with speed and efficiency at its core. Every command has been optimized for performance through parallel execution and smart concurrency.

### Parallel & Async Architecture

All major analysis commands leverage:
- **Multi-threading** for CPU-intensive operations (file processing, data aggregation)
- **Async execution** for I/O-bound Git operations (fetching, status checks)
- **Concurrent Git calls** to minimize total execution time

### Under the Hood

```rust
// Example: Parallel Git operations
let (repo_info, branch_status, working_dir) = tokio::try_join!(
    AsyncGitOperations::repo_root(),
    AsyncGitOperations::branch_info_parallel(),
    AsyncGitOperations::is_working_directory_clean(),
)?;
```

```rust
// Example: Multi-threaded file processing
let large_files: Vec<LargeFile> = files
    .par_iter()
    .filter_map(|file| analyze_file_size(file))
    .collect();
```

### Performance Philosophy

- **Algorithmic optimization first** — Fix O(n²) problems before adding concurrency
- **Smart parallelization** — I/O operations run async, CPU work runs multi-threaded
- **Minimal overhead** — Native Git subprocess calls, no unnecessary abstractions
- **Responsive feedback** — Show progress and timing information

The result? Commands that feel instant, even on large repositories.

---

## Built With

- Language: Rust 🦀
- Shell: Integrates cleanly with Bash, Zsh, Fish, etc.
- Uses: native `git` under the hood — no black magic

---

## License

MIT
