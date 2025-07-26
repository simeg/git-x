# 🔬 git-x Internals

This document explains how each `git-x` subcommand works under the hood. We aim to make everything transparent so users can trust and understand every result, and even replicate the logic with raw Git if needed.

---

## 🧠 `git xinfo`

### What it does:
- Displays a high-level overview of the current repository.

### Under the hood:
- `git rev-parse --show-toplevel` → Get the repo root.
- `git rev-parse --abbrev-ref HEAD` → Get current branch name.
- `git for-each-ref --format='%(upstream:short)'` → Find tracking branch.
- `git rev-list --left-right --count HEAD...@{upstream}` → Ahead/behind counts.
- `git log -1 --pretty=format:"%s (%cr)"` → Most recent commit summary.

---

## 📊 `git xgraph`

### What it does:
- Visual Git log showing commits across branches.

### Under the hood:
- Executes:
  ```shell
  git log --oneline --graph --decorate --all
  ```

---

## 🧹 `git x prune-branches`

### What it does:
- Deletes local branches that are fully merged into the current one, skipping protected branches.

### Under the hood:
- `git branch --merged` → List merged branches.
- Filters out current branch and protected ones (`main`, `master`, `develop`, plus any in `--except`).
- Runs `git branch -d` for each candidate (or just prints in dry-run).

---

## 🧪 `git xsince [ref]`

### What it does:
- Lists commits since a given ref (e.g., `main`, `origin/main`).

### Under the hood:
- `git log <ref>..HEAD --oneline`

---

## 💥 `git xundo`

### What it does:
- Soft-resets the last commit, keeping changes in the working directory.

### Under the hood:
- `git reset --soft HEAD~1`

---

## 🚚 `git xclean-branches`

### What it does:
- Deletes fully merged local branches, similar to `x prune-branches`, but doesn't take `--except`.

### Under the hood:
- Same as `x prune-branches`, minus the exceptions flag.

---

## 🧱 `git xwhat [branch]`

### What it does:
- Compares current branch to another (default: `main`).
- Shows ahead/behind commit count and file changes.

### Under the hood:
- `git rev-list --left-right --count HEAD...<other>` → Commit divergence.
- `git diff --name-status HEAD..<other>` → File-level changes.

---

## 🗞️ `git xsummary`

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
