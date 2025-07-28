# 🔬 git-x Internals

This document explains how each `git-x` subcommand works under the hood. We aim to make everything transparent so users can trust and understand every result, and even replicate the logic with raw Git if needed.

---

## `info`

### What it does:
- Displays a high-level overview of the current repository.

### Under the hood:
- `git rev-parse --show-toplevel` → Get the repo root.
- `git rev-parse --abbrev-ref HEAD` → Get current branch name.
- `git for-each-ref --format='%(upstream:short)'` → Find tracking branch.
- `git rev-list --left-right --count HEAD...@{upstream}` → Ahead/behind counts.
- `git log -1 --pretty=format:"%s (%cr)"` → Most recent commit summary.

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

## `health`

### What it does:
- Performs a comprehensive repository health check to identify potential issues and maintenance needs.

### Under the hood:
- `git rev-parse --git-dir` → Verify we're in a Git repository
- `git status --porcelain` → Check working directory status
- `git ls-files --others --exclude-standard` → Count untracked files
- `git for-each-ref --format='%(refname:short) %(committerdate:relative)' refs/heads/` → Identify stale branches
- `du -sh .git` → Check repository size
- `git diff --cached --name-only` → Check for staged changes

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
