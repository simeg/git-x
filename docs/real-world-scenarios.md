# 🌍 Real-World Scenarios: When git-x Commands Save the Day

This document shows practical, everyday situations where `git-x` commands solve real developer problems. Each scenario includes the problem context, traditional Git approach, and the simplified `git-x` solution.

---

## 🔧 `fixup` - Clean Up Your Commit History

### Scenario 1: Code Review Feedback
**The Problem:**
You submitted a pull request with this commit history:
```
abc123f Add user authentication system
def456g Add dashboard routing  
```

Your reviewer says: *"The auth system looks good, but you're missing input validation on the login form. Can you add that?"*

You add the validation and now need to incorporate it into the original auth commit.

**Traditional Git Approach:**
```shell
# Make your changes
git add login-form.js

# Create fixup commit manually
git commit --fixup=abc123f

# Interactive rebase to squash
git rebase -i --autosquash abc123f^
# Opens editor, you confirm the rebase plan, save and exit
```

**With git-x:**
```shell
# Make your changes  
git add login-form.js

# One command does it all
git x fixup abc123f --rebase
```

✅ **Result:** Your commit history stays clean, and the validation is properly incorporated into the auth commit.

---

### Scenario 2: Forgot to Add Tests
**The Problem:**
You just committed a new feature but realized you forgot to include the unit tests:

```
5a7b8c9 Add shopping cart functionality
```

You write the tests and want them to be part of the original feature commit, not a separate "add tests" commit.

**Traditional Git Approach:**
```shell
git add cart.test.js
git commit --fixup=5a7b8c9
git rebase -i --autosquash 5a7b8c9^
# Navigate the interactive rebase interface
```

**With git-x:**
```shell
git add cart.test.js
git x fixup 5a7b8c9 --rebase
```

✅ **Result:** Tests are included in the original feature commit, maintaining logical commit organization.

---

## 🏷️ `rename-branch` - Fix Branch Naming Mistakes

### Scenario 1: Typo in Branch Name
**The Problem:**
You created a branch for a user profile feature but made a typo:
```shell
git checkout -b user-proifle-page  # Oops! "proifle" instead of "profile"
```

You've made several commits and pushed to remote. Now you need to fix the typo everywhere.

**Traditional Git Approach:**
```shell
# Rename local branch
git branch -m user-proifle-page user-profile-page

# Delete old remote branch
git push origin :user-proifle-page

# Push new branch and set upstream
git push origin -u user-profile-page

# Update any PRs manually in GitHub/GitLab
```

**With git-x:**
```shell
git x rename-branch user-profile-page
```

✅ **Result:** Local branch renamed, remote updated, tracking fixed, all in one command.

---

### Scenario 2: Better Naming After Understanding Requirements
**The Problem:**
You started working on what you thought was a simple "login fix":
```
git checkout -b login-fix
```

After talking with the team, you realize this is actually a major authentication refactor that will take several days. The branch name no longer reflects the scope.

**Traditional Git Approach:**
```shell
git branch -m login-fix auth-system-refactor
git push origin :login-fix
git push origin -u auth-system-refactor
# Update PR title and description
# Notify team about branch name change
```

**With git-x:**
```shell
git x rename-branch auth-system-refactor
```

✅ **Result:** Branch name accurately reflects the work scope, easier for team collaboration.

---

## 📦 `stash-branch` - Manage Experimental Work

### Scenario 1: Interrupted by Urgent Bug Fix
**The Problem:**
You're halfway through implementing a complex feature when your manager says: *"Drop everything! Production is down, we need a hotfix for the payment system NOW!"*

You have uncommitted changes that aren't ready to commit, but you need to switch context immediately.

**Traditional Git Approach:**
```shell
# Stash your work
git stash push -m "WIP: new search algorithm - half implemented"

# Fix the urgent bug on master
git checkout master
git checkout -b hotfix-payment-bug
# ... fix the bug, commit, deploy ...

# Later, try to remember what you were doing
git stash list
git stash apply stash@{0}
# Hope you remember the context
```

**With git-x:**
```shell
# Stash and immediately create a proper branch for your work
git x stash-branch create search-algorithm-wip

# Handle the urgent fix
git checkout master
git checkout -b hotfix-payment-bug
# ... fix the bug ...

# Later, easily return to your feature work
git checkout search-algorithm-wip
# All your changes are there, properly organized
```

✅ **Result:** No lost work, clear context, proper branch organization.

---

### Scenario 2: Stash Cleanup After Team Sprint
**The Problem:**
After a busy sprint, your stash list looks like this:
```shell
git stash list
stash@{0}: WIP on feature-x: 2a3b4c5 Add validation
stash@{1}: On master: 1a2b3c4 Fix typo  
stash@{2}: WIP on feature-y: 9x8y7z6 Update styles
stash@{3}: On hotfix: 5d6e7f8 Debug logs
stash@{4}: WIP on feature-x: 3c4d5e6 Initial work
```

Most of these are outdated experiments or work that's been completed differently.

**Traditional Git Approach:**
```shell
# Manually inspect each stash
git stash show stash@{0}
git stash show stash@{1} 
# ... decide which ones to keep ...

# Manually drop old stashes
git stash drop stash@{1}
git stash drop stash@{3}
# Risk dropping the wrong ones
```

**With git-x:**
```shell
# Clean stashes older than 2 weeks
git x stash-branch clean --older-than 2w --dry-run
# Review what would be cleaned

git x stash-branch clean --older-than 2w
```

✅ **Result:** Clean workspace, no risk of accidentally losing recent work.

---

## ↶ `undo` - Fix Commit Mistakes

### Scenario 1: Committed Too Early
**The Problem:**
You're implementing OAuth integration and committed this:
```
git commit -m "Add OAuth integration"
```

Immediately after committing, you realize you forgot to:
- Add error handling for failed authentication
- Update the configuration documentation
- Add the new environment variables to `.env.example`

You want to include these in the same logical commit.

**Traditional Git Approach:**
```shell
# Undo the commit, keep changes
git reset --soft HEAD~1

# Make additional changes
# ... add error handling, docs, env vars ...
git add .

# Commit everything together
git commit -m "Add OAuth integration with error handling and docs"
```

**With git-x:**
```shell
git x undo

# Make additional changes
# ... add error handling, docs, env vars ...
git add .

git commit -m "Add OAuth integration with error handling and docs"
```

✅ **Result:** Clean, complete commit that includes all related changes.

---

### Scenario 2: Wrong Files Committed
**The Problem:**
You meant to commit just your feature changes, but accidentally included debug files:
```
git add .
git commit -m "Add user preferences feature"
```

You realize the commit includes:
- `debug.log` (should be gitignored)
- `temp-test-data.json` (not meant for version control)
- Your actual feature files (these should stay)

**Traditional Git Approach:**
```shell
# Undo the commit
git reset --soft HEAD~1

# Unstage everything
git reset

# Carefully add only the files you want
git add src/preferences.js
git add src/preferences.test.js
git add docs/preferences.md

# Recommit
git commit -m "Add user preferences feature"

# Update .gitignore for future
echo "debug.log" >> .gitignore
echo "temp-test-data.json" >> .gitignore
```

**With git-x:**
```shell
git x undo

# Fix the staging
git reset
git add src/preferences.js src/preferences.test.js docs/preferences.md

# Commit properly
git commit -m "Add user preferences feature"
```

✅ **Result:** Clean commit with only the intended files.

---

## 🔗 `upstream` - Manage Remote Relationships

### Scenario 1: Fork Synchronization
**The Problem:**
You're contributing to an open-source project. You forked `original-repo/awesome-project` to `your-username/awesome-project`. After working on your feature for a week, the original repository has 15 new commits you need to sync.

**Traditional Git Approach:**
```shell
# Add upstream remote (if not already done)
git remote add upstream https://github.com/original-repo/awesome-project.git

# Fetch upstream changes
git fetch upstream

# Check which branches need syncing
git branch -vv

# Sync master/main
git checkout main
git merge upstream/main

# Check if your feature branch needs updating
git checkout your-feature-branch
git rebase upstream/main

# Push updates
git push origin main
git push --force-with-lease origin your-feature-branch
```

**With git-x:**
```shell
# Set up upstream tracking for main branch
git checkout main
git x upstream set upstream/main

# Sync all branches that have upstream tracking
git x upstream sync-all

# Check status anytime
git x upstream status
```

✅ **Result:** All branches stay synchronized with minimal effort.

---

### Scenario 2: Complex Multi-Remote Workflow
**The Problem:**
Your team uses a complex Git workflow:
- `origin` = your fork
- `upstream` = main company repository  
- `staging` = staging environment repository

Different branches track different upstreams:
- `main` tracks `upstream/main`
- `feature-*` branches track `origin/main` 
- `staging` tracks `staging/main`

**Traditional Git Approach:**
```shell
# Manually track what's tracking what
git branch -vv

# Manually set up tracking for each branch
git checkout main
git branch --set-upstream-to=upstream/main

git checkout staging  
git branch --set-upstream-to=staging/main

# Manually sync each branch
git checkout main
git fetch upstream && git merge upstream/main

git checkout staging
git fetch staging && git merge staging/main

# Easy to make mistakes or forget branches
```

**With git-x:**
```shell
# Set up tracking once
git checkout main
git x upstream set upstream/main

git checkout staging
git x upstream set staging/main

# View all upstream relationships
git x upstream status

# Sync everything at once
git x upstream sync-all --dry-run  # Preview changes
git x upstream sync-all             # Apply changes
```

✅ **Result:** Clear visibility and easy management of complex upstream relationships.

---

### Scenario 3: Team Onboarding
**The Problem:**
A new developer joins your team and needs to understand the repository's remote setup and get their local branches synchronized. The project has:
- Multiple feature branches
- Different remote tracking configurations
- Some branches that are behind their upstreams

**Traditional Git Approach:**
```shell
# New developer runs various commands to understand setup
git remote -v
git branch -vv
git log --oneline main..origin/main
git log --oneline feature-x..origin/feature-x

# Manually sync each branch
git checkout main && git pull
git checkout feature-x && git pull  
# Repeat for each branch...
```

**With git-x:**
```shell
# Instantly see the upstream status of everything
git x upstream status

# Sync all branches that need it
git x upstream sync-all
```

✅ **Result:** New team member quickly understands and synchronizes the entire repository state.

---

## 🔍 `bisect` - Find the Bug-Introducing Commit

### Scenario 1: Regression Hunt
**The Problem:**
Your team noticed that user authentication broke sometime in the last 2 weeks. You have 47 commits since the last known-good release, and you need to find exactly which commit introduced the bug.

Manually checking each commit would take hours, and you're not sure which changes might be related to auth.

**Traditional Git Approach:**
```shell
# Start bisect manually
git bisect start
git bisect bad HEAD
git bisect good v2.1.0

# For each commit Git checks out:
# 1. Build and test the application
# 2. Figure out if the bug exists
# 3. Remember the right git bisect command
git bisect bad   # or good, or skip
# Repeat until found...

# Don't forget to clean up
git bisect reset
```

**With git-x:**
```shell
# Start with clear guidance
git x bisect start v2.1.0 HEAD

# Git-x guides you through each step
git x bisect bad     # Bug exists in this commit
git x bisect good    # Bug doesn't exist in this commit  
git x bisect skip    # Can't test this commit (build broken)

# Check progress anytime
git x bisect status

# Clean finish
git x bisect reset
```

✅ **Result:** Found the exact commit that broke auth in just 6 steps instead of checking 47 commits manually.

---

### Scenario 2: Performance Regression
**The Problem:**
Your app's dashboard page used to load in 200ms, but now it takes 3 seconds. The performance regression happened sometime in the past month (about 80 commits), but you're not sure which feature or optimization attempt caused it.

**Traditional Git Approach:**
```shell
# Start bisect
git bisect start HEAD HEAD~80

# For each commit:
# 1. Build the app
# 2. Run performance tests or manual testing
# 3. Decide if performance is acceptable
# 4. Remember the right bisect command
git bisect good  # or bad

# Easy to lose track of progress
# Hard to remember which commits you've tested
git bisect reset
```

**With git-x:**
```shell
# Clear workflow with progress tracking
git x bisect start HEAD~80 HEAD

# Test each commit with clear feedback
git x bisect bad     # Slow (3s load time)
git x bisect good    # Fast (200ms load time)

# See your progress at any time
git x bisect status
# Shows: Current commit, ~3 steps remaining, previous decisions

# Finish cleanly
git x bisect reset
```

✅ **Result:** Identified that commit `a7b8c9d` introducing async data fetching optimization actually caused the slowdown due to a race condition.

---

### Scenario 3: Flaky Test Investigation
**The Problem:**
A critical test started failing intermittently. It passes most of the time but fails about 30% of test runs. You need to find when this instability was introduced, but testing requires multiple runs per commit to be confident.

**Traditional Git Approach:**
```shell
git bisect start
git bisect bad HEAD
git bisect good v3.2.0

# For each commit, you need to:
# 1. Run the test multiple times
# 2. Keep track of results manually
# 3. Decide if the flakiness exists
# 4. Hope you don't lose track of where you are

git bisect skip  # When unsure due to randomness
git bisect reset
```

**With git-x:**
```shell
git x bisect start v3.2.0 HEAD

# Clear workflow for each commit
# Run: npm test -- --repeat=10
git x bisect bad     # Test failed 3/10 times (flaky)
git x bisect good    # Test passed 10/10 times (stable)
git x bisect skip    # Test infrastructure was broken

# Always know where you stand
git x bisect status

git x bisect reset
```

✅ **Result:** Discovered that commit `f4e5d6c` adding parallel test execution introduced a race condition in the test setup.

---

## 🎯 Summary

These scenarios show how `git-x` commands solve real problems that developers face daily:

- **`fixup`**: Maintains clean commit history during code reviews and iterative development
- **`rename-branch`**: Fixes naming mistakes and adapts to changing requirements  
- **`stash-branch`**: Manages context switching and experimental work effectively
- **`undo`**: Provides safe commit corrections without losing work
- **`upstream`**: Simplifies complex multi-remote workflows and team collaboration
- **`bisect`**: Efficiently finds bug-introducing commits through guided binary search

Each command reduces cognitive load and prevents the kind of Git mistakes that can derail productivity or damage repository history.