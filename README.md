# 🚀 git-x – Superpowers for Git [![CI](https://github.com/simeg/git-x/actions/workflows/ci.yaml/badge.svg)](https://github.com/simeg/git-x/actions/workflows/ci.yaml) [![codecov](https://codecov.io/github/simeg/git-x/branch/master/graph/badge.svg?token=A661U2R66C)](https://codecov.io/github/simeg/git-x)

**`git-x`** is a collection of smarter, faster, and more intuitive Git subcommands built to make your daily workflow suck less.

It wraps common Git actions in muscle-memory-friendly, no-brainer commands — perfect for solo developers, team leads, and anyone tired of typing `git log --oneline --graph --decorate --all` for the 400th time.

⚠️ **PSA: Do you know shell tab completion? If so - I need your help! 🙏 See the [Tab Completion](#-tab-completion-) section** ⚠️

---

## 📚 Table of Contents

- [Why Does This Exist?](#why-does-this-exist)
- [Installation](#installation)
- [Example Commands](#example-commands)
    - [`clean-branches`](#clean-branches)
    - [`color-graph`](#color-graph)
    - [`contributors`](#contributors)
    - [`fixup`](#fixup)
    - [`graph`](#graph)
    - [`health`](#health)
    - [`info`](#info)
    - [`large-files`](#large-files)
    - [`new`](#new)
    - [`prune-branches`](#prune-branches)
    - [`rename-branch`](#rename-branch)
    - [`since [ref]`](#since-ref)
    - [`stash-branch`](#stash-branch)
    - [`summary`](#summary)
    - [`switch-recent`](#switch-recent)
    - [`sync`](#sync)
    - [`undo`](#undo)
    - [`upstream`](#upstream)
    - [`what [branch]`](#what-branch)
- [Git Integration: How `git-x` Just Works™](#git-integration-how-git-x-just-works)
- [What's Under the Hood?](#whats-under-the-hood)
- [Command Transparency](#command-transparency)
- [Roadmap Ideas](#roadmap-ideas)
- [Tab Completion](#-tab-completion-)
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

---

## Example Commands

---

### `clean-branches`

> Delete all fully merged local branches (except protected ones)

```shell
git x clean-branches
```

#### Output:

```shell
🧹 Deleted 5 merged branches:
- feature/refactor
- bugfix/signup-typo
...
```

---

### `color-graph`

> Colorized Git log with branches, remotes, HEADs, and author info

```shell
git x color-graph
```

#### Output:

Enhanced version of `git x graph` with:
- **Full color support** for branches, commits, and decorations
- **Author names and timestamps** for each commit
- **Rich formatting** that's easy to scan

(essentially wraps this)
```shell
git log --oneline --graph --decorate --all --color=always --pretty=format:"%C(auto)%h%d %s %C(dim)(%an, %ar)%C(reset)"
```

---

### `contributors`

> Show contributor statistics for the repository

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

### `fixup`

> Create fixup commits for easier interactive rebasing

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

### `graph`

> Pretty Git log with branches, remotes, and HEADs

```shell
git x graph
```

#### Output:

(essentially wraps this)
```shell
git log --oneline --graph --decorate --all
```

---

### `health`

> Check repository health and identify potential issues

```shell
git x health
```

#### Output:

```shell
Repository Health Check
=========================

✓ Working directory is clean
✓ No untracked files
✓ No stale branches (older than 1 month)
✓ Repository size: 524K (healthy)
✓ No staged changes

Health check complete!
```

#### What it checks:
- **Working directory status** - Detects uncommitted changes
- **Untracked files** - Counts files not under version control
- **Stale branches** - Identifies branches older than 1 month
- **Repository size** - Warns about large repositories that may need cleanup
- **Staged changes** - Shows files ready for commit

Useful for:
- Daily repository maintenance
- Pre-commit health checks
- Identifying cleanup opportunities
- Team onboarding (ensuring clean local state)

---

### `info`

> Show a high-level overview of the current repo

```shell
git x info
```

#### Output:

```shell
Repo: my-project
Branch: feature/auth
Tracking: origin/feature/auth
Ahead: 2 Behind: 0
Last Commit: "fix login bug" (2 hours ago)
```

---

### `large-files`

> Find largest files in repository history

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

### `new`

> Create and switch to a new branch

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

### `prune-branches`

Deletes all **local branches** that have already been **merged into the current branch**, while skipping protected ones.

Useful for keeping your repo tidy after merging feature branches.

**Defaults:**
- Protected branches: `main`, `master`, `develop`
- Won't delete current branch
- Will only delete branches that are *fully merged*

**Flags:**
- `--except <branches>` — Comma-separated list of branch names to exclude from deletion

```shell
git x prune-branches --except "release,v1.0-temp"
```

---

### `rename-branch`

> Rename the current branch locally and on remote

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

### `since [ref]`

> Show commits since a reference (e.g., `d926b4b`, my-branch, origin/main)

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

### `stash-branch`

> Advanced stash management with branch integration

```shell
git x stash-branch create new-feature
git x stash-branch clean --older-than 7d
git x stash-branch apply-by-branch feature-work
```

#### Subcommands:

**`create <branch-name>`** — Create a new branch from a stash
- `--stash <ref>` — Use specific stash (default: latest)

**`clean`** — Clean up old stashes
- `--older-than <time>` — Remove stashes older than specified time
- `--dry-run` — Show what would be cleaned without doing it

**`apply-by-branch <branch-name>`** — Apply stashes from a specific branch
- `--list` — List matching stashes instead of applying

Helps manage stashes more effectively by associating them with branches.

---

### `summary`

> Show a short, changelog-style summary of recent commits

```shell
git x summary
git x summary --since "2 days ago"
```

**Flags:**
- `--since` — Accepts natural date formats like "2 days ago", "last Monday", or exact dates like "2025-07-01". It uses Git's built-in date parser, so most human-readable expressions work.

#### Output:
```shell
🗞️ Commit summary since 3 days ago:

📅 2025-07-25
 - 🛠 fix: update token refresh logic (by Alice, 3 hours ago)
 - ✨ feat: add dark mode support (by Bob, 6 hours ago)

📅 2025-07-24
 - 🔥 remove unused dependencies (by Alice, 1 day ago)

📅 2025-07-23
 - 🐛 fix: handle null response in API call (by Carol, 2 days ago)
```

- Groups commits by day
- Shows commit message, author, and relative time
- Useful for writing daily stand-ups, changelogs, or review summaries
- Defaults to showing commits from the last 3 days
- Can be customized using `--since` (e.g. `--since "1 week ago"`)
- Sorts commits newest-first within each day

---

### `switch-recent`

> Interactive picker for recent branches

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

### `sync`

> Sync current branch with upstream (fetch + rebase/merge)

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

---

### `undo`

> Undo the last commit (without losing changes)

```shell
git x undo
```

#### Output:

```shell
Last commit undone (soft reset). Changes kept in working directory.
```

---

### `upstream`

> Manage upstream branch relationships

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

### `what [branch]`

> Show what's different between this branch and another (default: main)

```shell
git x what
git x what develop
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

---

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
git log --oneline --graph --decorate --all
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
git log --oneline --graph --decorate --all
```

**Why Rust?**
- **Fast startup** — Sub-millisecond command execution
- **Zero dependencies** — Single binary, no runtime requirements
- **Cross-platform** — Works on macOS, Linux, Windows
- **Memory safe** — No crashes, no memory leaks

---

## Roadmap Ideas

- `git x stash`: A smarter stash viewer with preview
- `git x prune`: Aggressively delete stale branches (with dry-run)
- `git x inspect`: Interactive blame / file history explorer

---

## 🚧 Tab Completion 🚧

I'm looking for help to get tab completion working! 🆘 By that I mean `git x <TAB>` should show available commands. I've given it my best shot without success 😅 so if anyone can help, that would be much appreciated!

Your shell expertise could make `git-x` so much more pleasant to use!

---

## Built With

- Language: Rust 🦀
- Shell: Integrates cleanly with Bash, Zsh, Fish, etc.
- Uses: native `git` under the hood — no black magic

---

## License

MIT
