# 🚀 git-x – Superpowers for Git

**`git-x`** is a collection of smarter, faster, and more intuitive Git subcommands built to make your daily workflow suck less.

It wraps common Git actions in muscle-memory-friendly, no-brainer commands — perfect for solo developers, team leads, and anyone tired of typing `git log --oneline --graph --decorate --all` for the 400th time.

---

## 📚 Table of Contents

- [🧨 Why Does This Exist?](#-why-does-this-exist)
- [🔧 Installation](#-installation)
- [🔌 Git Integration: How `git-x` Just Works™](#-git-integration-how-git-x-just-works)
- [🔬 What's Under the Hood?](#-whats-under-the-hood)
- [🔍 Example Commands](#-example-commands)
    - [🧠 `git xinfo`](#-git-xinfo)
    - [📊 `git xgraph`](#-git-xgraph)
    - [🧹 `git x prune-branches`](#-git-x-prune-branches)
    - [🧪 `git xsince`](#-git-xsince-ref)
    - [💥 `git xundo`](#-git-xundo)
    - [🚚 `git xclean-branches`](#-git-xclean-branches)
    - [🧱 `git xwhat`](#-git-xwhat-branch)
    - [🗞️ `git xsummary`](#-git-xsummary)
- [🛣️ Roadmap Ideas](#️-roadmap-ideas)
- [🛠 Built With](#-built-with)
- [📄 License](#-license)

---

## 🧨 Why Does This Exist?

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

## 🔧 Installation

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

### 🔌 Git Integration: How `git-x` Just Works™

Once installed, the `git-x` binary is automatically available as a Git subcommand.

Thanks to how Git is designed, **any executable in your `PATH` named `git-<something>` can be invoked as `git <something>`**.

So if you install `git-x`, you can run:
```shell
git x summary
git x clean-branches
git x what
```
and Git will execute the `git-x` binary with the remaining arguments.

You can even tab-complete like with any normal Git command — no aliases, symlinks, or shell hacks needed.

#### How it works:

When you type `git x summary`, Git looks for:
- `git-xsummary` (unlikely)
- Then falls back to `git-x` and passes `summary` as the first argument

This is exactly what `git-x` is built for — **one binary to rule many Git subcommands**.

> 💡 Bonus: You can also run `git-x summary` directly if you prefer.

---

### 🔬 What's Under the Hood?

Want to know exactly what each `git-x` command does?

We’ve got you covered. No magic here — just well-wrapped Git commands.

Check out the [Command Internals](./docs/command-internals.md) to see what each command runs behind the scenes, from `git log` to `git branch --merged`.

---

## 🔍 Example Commands

---

### 🧠 `git x info`

> Show a high-level overview of the current repo

```shell
git x info
```

#### Output:

```shell
📂 Repo: my-project
🔀 Branch: feature/auth
🌿 Tracking: origin/feature/auth
⬆️ Ahead: 2   ⬇️ Behind: 0
📌 Last Commit: "fix login bug" (2 hours ago)
```

---

### 📊 `git x graph`

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

### 🧹 `git x prune-branches`

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

### 🧪 `git x since [ref]`

> Show commits since a reference (e.g., main, origin/main)

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

### 💥 `git x undo`

> Undo the last commit (without losing changes)

```shell
git x undo
```

#### Output:

```shell
Last commit undone (soft reset). Changes kept in working directory.
```

---

### 🚚 `git x clean-branches`

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

### 🧱 `git x what [branch]`

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

### 🗞️ `git x summary`

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

## 🛣️ Roadmap Ideas

- `git x stash`: A smarter stash viewer with preview
- `git x prune`: Aggressively delete stale branches (with dry-run)
- `git x inspect`: Interactive blame / file history explorer

---

## 🛠 Built With

- Language: Rust 🦀
- Shell: Integrates cleanly with Bash, Zsh, Fish, etc.
- Uses: native `git` under the hood — no black magic

---

## License

MIT