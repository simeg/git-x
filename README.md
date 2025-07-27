# 🚀 git-x – Superpowers for Git [![CI](https://github.com/simeg/git-x/actions/workflows/ci.yaml/badge.svg)](https://github.com/simeg/git-x/actions/workflows/ci.yaml)

**`git-x`** is a collection of smarter, faster, and more intuitive Git subcommands built to make your daily workflow suck less.

It wraps common Git actions in muscle-memory-friendly, no-brainer commands — perfect for solo developers, team leads, and anyone tired of typing `git log --oneline --graph --decorate --all` for the 400th time.

---

## 📚 Table of Contents

- [Why Does This Exist?](#why-does-this-exist)
- [Installation](#installation)
- [Example Commands](#example-commands)
    - [`info`](#info)
    - [`graph`](#graph)
    - [`color-graph`](#color-graph)
    - [`prune-branches`](#prune-branches)
    - [`since [ref]`](#since-ref)
    - [`undo`](#undo)
    - [`clean-branches`](#clean-branches)
    - [`what [branch]`](#what-branch)
    - [`summary`](#summary)
- [Git Integration: How `git-x` Just Works™](#git-integration-how-git-x-just-works)
- [What's Under the Hood?](#whats-under-the-hood)
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

### `prune-branches`

Deletes all **local branches** that have already been **merged into the current branch**, while skipping protected ones.

Useful for keeping your repo tidy after merging feature branches.

**Defaults:**
- Protected branches: `main`, `master`, `develop`
- Won’t delete current branch
- Will only delete branches that are *fully merged*

**Flags:**
- `--except <branches>` — Comma-separated list of branch names to exclude from deletion

```shell
git x prune-branches --except "release,v1.0-temp"
```

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

### `what [branch]`

> Show what’s different between this branch and another (default: main)

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

### `summary`

> Show a short, changelog-style summary of recent commits

```shell
git x summary
git x summary --since "2 days ago"
```

**Flags:**
- `--since` — Accepts natural date formats like "2 days ago", "last Monday", or exact dates like "2025-07-01". It uses Git’s built-in date parser, so most human-readable expressions work.

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

## Built With

- Language: Rust 🦀
- Shell: Integrates cleanly with Bash, Zsh, Fish, etc.
- Uses: native `git` under the hood — no black magic

---

## License

MIT
