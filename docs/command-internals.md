# 🔬 git-x Internals

This document explains how each `git-x` subcommand works under the hood. We aim to make everything transparent so users can trust and understand every result, and even replicate the logic with raw Git if needed.

---

## `bisect`

### What it does:
- Provides a simplified interface for Git's bisect functionality to find the commit that introduced a bug.

### Under the hood:
- **Start bisect session:**
  ```shell
  git bisect start <bad-commit> <good-commit>
  ```
  - Validates commit references exist using `git rev-parse --verify`
  - Checks if already in bisect mode by checking for `.git/BISECT_START` file existence
  - Displays current commit info and remaining steps estimate

- **Mark commits:**
  ```shell
  git bisect good  # Mark current commit as good
  git bisect bad   # Mark current commit as bad  
  git bisect skip  # Skip current commit (untestable)
  ```
  - Each command updates bisect state and checks out next commit
  - Parses output to detect when first bad commit is found
  - Shows remaining steps using logarithmic calculation

- **Show status:**
  ```shell
  git bisect log     # Show bisect history
  git bisect view    # Count remaining commits
  ```
  - Displays current commit, remaining steps, and recent bisect actions
  - Provides guidance on next steps

- **Reset bisect:**
  ```shell
  git bisect reset
  ```
  - Returns to original branch and cleans up bisect state
  - Safe to run even when not in bisect mode

---

## `info`

### What it does:
- Displays a comprehensive overview of the current repository including recent activity, branch comparisons, and PR status.

### Under the hood:
**Basic repository info:**
- `git rev-parse --show-toplevel` → Get the repo root and repository name.
- `git rev-parse --abbrev-ref HEAD` → Get current branch name.
- `git for-each-ref --format='%(upstream:short)'` → Find tracking branch.
- `git rev-list --left-right --count HEAD...@{upstream}` → Ahead/behind counts with upstream.
- `git diff --cached --name-only` → List staged files.
- `git status --porcelain` → Check working directory cleanliness.

**Enhanced features:**
- `git log --oneline --decorate --graph --all --max-count=8 --pretty=format:'%C(auto)%h %s %C(dim)(%cr) %C(bold blue)<%an>%C(reset)'` → Recent activity timeline with author info.
- `gh pr status --json currentBranch` → GitHub PR detection (if `gh` CLI available).
- `git rev-list --left-right --count main...HEAD` → Branch differences against main/master/develop branches.
- `git for-each-ref --sort=-committerdate refs/heads/ --format='%(refname:short)'` → Recent branches list (detailed mode).

---

## `graph`

### What it does:
- Visual Git log showing commits across branches.

### Under the hood:
- Executes:
  ```shell
  git log --oneline --graph --decorate --all
  ```

---

## `color-graph`

### What it does:
- Enhanced visual Git log with full color support, showing commits, branches, and author information.

### Under the hood:
- Executes:
  ```shell
  git log --oneline --graph --decorate --all --color=always --pretty=format:"%C(auto)%h%d %s %C(dim)(%an, %ar)%C(reset)"
  ```

---

## `contributors`

### What it does:
- Shows contributor statistics for the repository, including commit counts, percentages, email addresses, and date ranges.

### Under the hood:
- Executes:
  ```shell
  git log --all --format=%ae|%an|%ad --date=short
  ```
- Parses the output to group commits by email address
- Sorts contributors by commit count (descending)
- Calculates percentage contributions and date ranges
- Formats output with ranking icons (🥇🥈🥉👤) and styled text

### Key data processing:
- Groups commits by contributor email to handle name variations
- Tracks first and last commit dates for each contributor
- Sorts by commit count to show most active contributors first
- Calculates percentages based on total commit count
- Uses emoji ranking system for top 3 contributors

---

## `health`

### What it does:
- Performs a comprehensive repository health check with real-time progress indicators and detailed security reporting.

### Under the hood:
**Core health checks:**
- `git rev-parse --git-dir` → Verify we're in a Git repository
- `git config user.name` → Check Git user configuration
- `git config user.email` → Check Git email configuration
- `git remote` → Verify remote repositories are configured
- `git status --porcelain` → Check working directory status
- `git ls-files --others --exclude-standard` → Count untracked files
- `git for-each-ref --format='%(refname:short) %(committerdate:relative)' refs/heads/` → Identify stale branches
- `git count-objects -vH` → Check repository size with human-readable output
- `git diff --cached --name-only` → Check for staged changes

**Security checks with detailed reporting:**
- `git log --all --full-history --grep=password --grep=secret --grep=key --grep=token --grep=credential --pretty=format:'%h %s' -i` → Scan for potential credentials in commit messages with commit hashes and messages
- `git ls-files *.pem *.key *.p12 *.pfx *.jks` → Find potentially sensitive files and list specific filenames
- `git ls-files *.env*` → Find environment files that might contain secrets and show which files

**Repository optimization checks:**
- `git ls-files .gitignore` → Verify .gitignore exists
- `git ls-files *.log *.tmp *.swp *.bak .DS_Store Thumbs.db node_modules/ target/ .vscode/ .idea/` → Check for files that should be ignored
- Binary file detection using `git diff --no-index /dev/null <file> --numstat` → Identify large binary files with sizes and Git LFS recommendations
- Progress tracking using `indicatif` crate → Real-time progress bar showing current check being performed

---

## `technical-debt`

### What it does:
- Analyzes repository for technical debt indicators including large commits, file hotspots, long-lived branches, code churn, and binary files.

### Under the hood:
- **Large Commits Analysis:**
  ```shell
  git log --all --pretty=format:%h|%s|%an|%ad --date=short --numstat --since=6 months ago
  ```
  - Parses commit history with file change statistics
  - Identifies commits with >20 file changes
  - Sorts by number of files changed

- **File Hotspots Analysis:**
  ```shell
  git log --all --pretty=format: --name-only --since=6 months ago
  ```
  - Counts modification frequency per file
  - Categorizes risk levels: HIGH (>50), MED (>20), LOW (>5)
  - Excludes dotfiles and shows top modified files

- **Long-lived Branches Analysis:**
  ```shell
  git for-each-ref --format=%(refname:short)|%(committerdate:relative)|%(authorname) refs/heads/
  ```
  - Identifies branches older than 30 days
  - Excludes main/master/develop branches
  - Estimates days from relative date strings

- **Code Churn Analysis:**
  ```shell
  git log --all --pretty=format: --numstat --since=3 months ago
  ```
  - Aggregates additions/deletions per file
  - Calculates churn ratio (total changes / line changes)
  - Highlights files with high modification-to-content ratios

- **Binary Files Detection:**
  ```shell
  git ls-files
  ```
  - Scans tracked files for binary extensions
  - Checks common binary types: images, videos, audio, archives, executables, documents
  - Reports count and sample file paths

### Key metrics:
- Large commits indicate lack of atomic changes and potential review complexity
- File hotspots suggest architectural issues or missing abstractions
- Long-lived branches indicate potential merge conflicts and outdated code
- High churn files may need refactoring or better change management
- Binary files affect repository size and diff readability

---

## `prune-branches`

### What it does:
- Deletes local branches that are fully merged into the current one, skipping protected branches.

### Under the hood:
- `git branch --merged` → List merged branches.
- Filters out current branch and protected ones (`main`, `master`, `develop`, plus any in `--except`).
- Runs `git branch -d` for each candidate (or just prints in dry-run).

---

## `since [ref]`

### What it does:
- Lists commits since a given ref (e.g., `cb676ec`, `origin/main`).

### Under the hood:
- `git log <ref>..HEAD --oneline`

---

## `undo`

### What it does:
- Soft-resets the last commit, keeping changes in the working directory.

### Under the hood:
- `git reset --soft HEAD~1`

---

## `clean-branches`

### What it does:
- Deletes fully merged local branches, similar to `x prune-branches`, but doesn't take `--except`.

### Under the hood:
- Same as `x prune-branches`, minus the exceptions flag.

---

## `what [branch]`

### What it does:
- Compares current branch to another (default: `main`).
- Shows ahead/behind commit count and file changes.

### Under the hood:
- `git rev-list --left-right --count HEAD...<other>` → Commit divergence.
- `git diff --name-status HEAD..<other>` → File-level changes.

---

## `summary`

### What it does:
- Generates a short, human-friendly changelog grouped by day.

### Under the hood:
- `git log --since=<value> --pretty=format:%h|%ad|%s|%an|%cr --date=short`
- Parses the output, groups by date, adds emojis based on commit messages:
    - "fix"/"bug" → 🐛
    - "feat"/"add" → ✨
    - "remove"/"delete" → 🔥
    - "refactor" → 🛠
    - fallback → 🔹

---

## `rename-branch`

### What it does:
- Renames the current branch locally and updates remote tracking.

### Under the hood:
- `git rev-parse --abbrev-ref HEAD` → Get current branch name
- `git branch -m <old-name> <new-name>` → Rename local branch
- `git push origin :<old-name>` → Delete old remote branch
- `git push origin -u <new-name>` → Push new branch and set upstream

---

## `sync`

### What it does:
- Synchronizes current branch with its upstream using fetch + rebase/merge.

### Under the hood:
- `git rev-parse --abbrev-ref HEAD` → Get current branch
- `git rev-parse --abbrev-ref HEAD@{upstream}` → Get upstream branch
- `git fetch <remote>` → Fetch from remote
- `git rev-list --left-right --count <upstream>...HEAD` → Check sync status
- `git rebase <upstream>` or `git merge <upstream>` → Integrate changes

---

## `new`

### What it does:
- Creates and switches to a new branch with validation.

### Under the hood:
- Validates branch name against Git naming rules
- `git rev-parse --verify <base-branch>` → Verify base branch exists (if --from specified)
- `git checkout -b <new-branch> [<base-branch>]` → Create and switch to new branch

---

## `large-files`

### What it does:
- Identifies the largest files in repository history to help with cleanup.

### Under the hood:
- `git rev-list --objects --all` → Get all objects in history
- `git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)'` → Get object sizes
- Filters for blob objects, sorts by size, formats output

---

## `fixup`

### What it does:
- Creates a fixup commit for easier interactive rebasing.

### Under the hood:
- `git rev-parse --verify <commit-hash>` → Validate commit exists
- `git diff --cached --quiet` → Check for staged changes
- `git commit --fixup=<commit-hash>` → Create fixup commit
- Optional: `git rebase -i --autosquash <commit-hash>^` → Auto-rebase if --rebase flag

---

## `stash-branch`

### What it does:
- Advanced stash management with branch integration.

### Under the hood:

**`create` subcommand:**
- Validates branch name
- `git stash branch <branch-name> [<stash-ref>]` → Create branch from stash

**`clean` subcommand:**
- `git stash list --format="%gd %gt %gs"` → List all stashes
- Filters by age if --older-than specified
- `git stash drop <stash-ref>` → Remove old stashes

**`apply-by-branch` subcommand:**
- `git stash list --format="%gd %gt %gs"` → List all stashes
- Filters stashes by branch name pattern
- `git stash apply <stash-ref>` → Apply matching stashes

**`interactive` subcommand:**
- `git stash list --pretty=format:'%gd|%s'` → Get stash list for interactive menu
- Uses `dialoguer` crate for interactive TUI with fuzzy selection
- Supports multiple actions: apply, delete, create branch, show diff, list
- `git stash apply/drop/branch/show -p <stash-ref>` → Execute selected action
- Multi-select for batch operations (delete multiple stashes)

**`export` subcommand:**
- `git stash list --pretty=format:'%gd|%s'` → Get list of stashes to export
- `git stash show -p <stash-ref>` → Generate patch content for each stash
- Creates `.patch` files in specified output directory
- Sanitizes stash names for safe filenames (removes special characters)
- Supports exporting all stashes or a specific stash reference

---

## `upstream`

### What it does:
- Manages upstream branch relationships across the repository.

### Under the hood:

**`status` subcommand:**
- `git for-each-ref --format='%(refname:short) %(upstream:short)' refs/heads/` → List branches with upstreams
- `git rev-parse --abbrev-ref HEAD` → Identify current branch

**`set` subcommand:**
- `git rev-parse --verify <upstream>` → Validate upstream exists
- `git branch --set-upstream-to=<upstream>` → Set upstream for current branch

**`sync-all` subcommand:**
- `git for-each-ref --format='%(refname:short) %(upstream:short)' refs/heads/` → Find branches with upstreams
- For each branch: `git checkout <branch> && git fetch && git rebase/merge <upstream>`
- `git checkout <original-branch>` → Return to original branch

---

## `switch-recent`

### What it does:
- Provides an interactive picker to quickly switch between recently used branches.

### Under the hood:
- `git for-each-ref --sort=-committerdate --format='%(refname:short)' refs/heads/` → Get branches sorted by recent activity
- `git branch --show-current` → Get current branch to exclude from list
- Filters out current branch and limits to 10 most recent branches
- Uses `dialoguer::Select` for interactive terminal UI
- `git checkout <selected-branch>` → Switch to selected branch

### Features:
- Shows up to 10 most recently committed branches
- Excludes current branch from selection
- Visual indicators (🌟 for most recent, 📁 for others)
- Cancellable with Esc or Ctrl+C
- Arrow key navigation with Enter to select

---
