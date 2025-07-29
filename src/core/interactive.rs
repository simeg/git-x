use crate::{GitXError, Result};
use dialoguer::{Confirm, FuzzySelect, Input};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

/// Interactive utilities with fuzzy search capabilities
pub struct Interactive;

impl Interactive {
    /// Check if we're running in an interactive environment
    pub fn is_interactive() -> bool {
        // Check for any test-related environment variables or conditions
        if std::env::var("CARGO_TARGET_TMPDIR").is_ok()
            || std::env::var("CI").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
            || std::env::var("GIT_X_NON_INTERACTIVE").is_ok()
            || !atty::is(atty::Stream::Stdin)
        {
            return false;
        }

        true
    }

    /// Show a fuzzy-searchable selection menu
    pub fn fuzzy_select<T: AsRef<str> + Clone + ToString>(
        items: &[T],
        prompt: &str,
        default: Option<usize>,
    ) -> Result<T> {
        let selection = FuzzySelect::new()
            .with_prompt(prompt)
            .items(items)
            .default(default.unwrap_or(0))
            .interact()
            .map_err(|e| GitXError::GitCommand(format!("Selection cancelled: {e}")))?;

        Ok(items[selection].clone())
    }

    /// Show an enhanced branch picker with fuzzy search
    pub fn branch_picker(branches: &[String], prompt: Option<&str>) -> Result<String> {
        if branches.is_empty() {
            return Err(GitXError::GitCommand("No branches available".to_string()));
        }

        let formatted_items: Vec<String> = branches
            .iter()
            .enumerate()
            .map(|(i, branch)| {
                let prefix = if i == 0 { "🌟 " } else { "📁 " };
                format!("{prefix}{branch}")
            })
            .collect();

        let prompt_text = prompt.unwrap_or("Select a branch");
        let selection = FuzzySelect::new()
            .with_prompt(prompt_text)
            .items(&formatted_items)
            .default(0)
            .interact()
            .map_err(|e| GitXError::GitCommand(format!("Selection cancelled: {e}")))?;

        Ok(branches[selection].clone())
    }

    /// Get text input with validation
    pub fn text_input(
        prompt: &str,
        default: Option<&str>,
        validator: Option<fn(&str) -> Result<()>>,
    ) -> Result<String> {
        let mut input_builder = Input::<String>::new().with_prompt(prompt);

        if let Some(default_val) = default {
            input_builder = input_builder.default(default_val.to_string());
        }

        let input = input_builder
            .interact_text()
            .map_err(|e| GitXError::GitCommand(format!("Input cancelled: {e}")))?;

        // Apply validation if provided
        if let Some(validate_fn) = validator {
            validate_fn(&input)?;
        }

        Ok(input)
    }

    /// Show a confirmation dialog
    pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
        Confirm::new()
            .with_prompt(prompt)
            .default(default)
            .interact()
            .map_err(|e| GitXError::GitCommand(format!("Confirmation cancelled: {e}")))
    }

    /// Find and rank items using fuzzy matching
    pub fn fuzzy_find<T: AsRef<str>>(
        items: &[T],
        query: &str,
        limit: Option<usize>,
    ) -> Vec<(usize, i64)> {
        let matcher = SkimMatcherV2::default();
        let mut results: Vec<(usize, i64)> = items
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                matcher
                    .fuzzy_match(item.as_ref(), query)
                    .map(|score| (idx, score))
            })
            .collect();

        // Sort by score (highest first)
        results.sort_by(|a, b| b.1.cmp(&a.1));

        if let Some(limit) = limit {
            results.truncate(limit);
        }

        results
    }

    /// Select from a list with automatic fallback for non-interactive mode
    pub fn select_or_first<T: AsRef<str> + Clone + ToString>(
        items: &[T],
        prompt: &str,
    ) -> Result<T> {
        if items.is_empty() {
            return Err(GitXError::GitCommand("No items to select from".to_string()));
        }

        if !Self::is_interactive() {
            // In non-interactive mode, just return the first item
            return Ok(items[0].clone());
        }

        Self::fuzzy_select(items, prompt, Some(0))
    }

    /// Confirm or auto-accept in non-interactive mode
    pub fn confirm_or_accept(prompt: &str, default: bool) -> Result<bool> {
        if !Self::is_interactive() {
            // In non-interactive mode, return the default
            return Ok(default);
        }

        Self::confirm(prompt, default)
    }
}

/// Builder for creating complex interactive workflows
pub struct InteractiveBuilder {
    steps: Vec<InteractiveStep>,
}

#[derive(Debug)]
enum InteractiveStep {
    Confirm {
        prompt: String,
        default: bool,
    },
    Select {
        prompt: String,
        items: Vec<String>,
    },
    Input {
        prompt: String,
        default: Option<String>,
    },
}

impl InteractiveBuilder {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn confirm(mut self, prompt: &str, default: bool) -> Self {
        self.steps.push(InteractiveStep::Confirm {
            prompt: prompt.to_string(),
            default,
        });
        self
    }

    pub fn select(mut self, prompt: &str, items: Vec<String>) -> Self {
        self.steps.push(InteractiveStep::Select {
            prompt: prompt.to_string(),
            items,
        });
        self
    }

    pub fn input(mut self, prompt: &str, default: Option<String>) -> Self {
        self.steps.push(InteractiveStep::Input {
            prompt: prompt.to_string(),
            default,
        });
        self
    }

    pub fn execute(self) -> Result<InteractiveResults> {
        let mut results = InteractiveResults::new();

        for step in self.steps {
            match step {
                InteractiveStep::Confirm { prompt, default } => {
                    let result = Interactive::confirm_or_accept(&prompt, default)?;
                    results.confirmations.push(result);
                }
                InteractiveStep::Select { prompt, items } => {
                    let result = Interactive::select_or_first(&items, &prompt)?;
                    results.selections.push(result);
                }
                InteractiveStep::Input { prompt, default } => {
                    let result = Interactive::text_input(&prompt, default.as_deref(), None)?;
                    results.inputs.push(result);
                }
            }
        }

        Ok(results)
    }
}

impl Default for InteractiveBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct InteractiveResults {
    pub confirmations: Vec<bool>,
    pub selections: Vec<String>,
    pub inputs: Vec<String>,
}

impl InteractiveResults {
    fn new() -> Self {
        Self {
            confirmations: Vec::new(),
            selections: Vec::new(),
            inputs: Vec::new(),
        }
    }

    pub fn get_confirmation(&self, index: usize) -> Option<bool> {
        self.confirmations.get(index).copied()
    }

    pub fn get_selection(&self, index: usize) -> Option<&str> {
        self.selections.get(index).map(|s| s.as_str())
    }

    pub fn get_input(&self, index: usize) -> Option<&str> {
        self.inputs.get(index).map(|s| s.as_str())
    }
}
