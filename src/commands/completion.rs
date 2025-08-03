use crate::core::traits::Command;
use crate::{GitXError, Result};
use clap_complete::Shell;
use std::fs;
use std::path::{Path, PathBuf};

pub struct CompletionInstallCommand {
    shell: Shell,
}

impl CompletionInstallCommand {
    pub fn new(shell: Shell) -> Self {
        Self { shell }
    }

    fn get_completion_directory(&self) -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .map_err(|_| GitXError::Other("HOME environment variable not set".to_string()))?;

        match self.shell {
            Shell::Bash => {
                // Try bash completion directories in order of preference
                let dirs = vec![
                    format!("{}/.local/share/bash-completion/completions", home),
                    format!("{}/.bash_completion.d", home),
                ];

                for dir in dirs {
                    let path = PathBuf::from(&dir);
                    if path.parent().is_some_and(|p| p.exists()) {
                        if !path.exists() {
                            fs::create_dir_all(&path).map_err(|e| {
                                GitXError::Other(format!(
                                    "Failed to create directory {dir}: {e}"
                                ))
                            })?;
                        }
                        return Ok(path);
                    }
                }

                // Fallback: create the standard location
                let fallback =
                    PathBuf::from(format!("{home}/.local/share/bash-completion/completions"));
                fs::create_dir_all(&fallback).map_err(|e| {
                    GitXError::Other(format!("Failed to create completion directory: {e}"))
                })?;
                Ok(fallback)
            }
            Shell::Zsh => {
                // Try zsh completion directories
                let dirs = vec![
                    format!("{}/.local/share/zsh/site-functions", home),
                    format!("{}/.zsh/completions", home),
                ];

                for dir in dirs {
                    let path = PathBuf::from(&dir);
                    if path.parent().is_some_and(|p| p.exists()) {
                        if !path.exists() {
                            fs::create_dir_all(&path).map_err(|e| {
                                GitXError::Other(format!(
                                    "Failed to create directory {dir}: {e}"
                                ))
                            })?;
                        }
                        return Ok(path);
                    }
                }

                // Fallback: create the standard location
                let fallback = PathBuf::from(format!("{home}/.local/share/zsh/site-functions"));
                fs::create_dir_all(&fallback).map_err(|e| {
                    GitXError::Other(format!("Failed to create completion directory: {e}"))
                })?;
                Ok(fallback)
            }
            Shell::Fish => {
                let dir = format!("{home}/.config/fish/completions");
                let path = PathBuf::from(&dir);
                fs::create_dir_all(&path).map_err(|e| {
                    GitXError::Other(format!(
                        "Failed to create fish completions directory: {e}"
                    ))
                })?;
                Ok(path)
            }
            Shell::PowerShell => {
                Err(GitXError::Other(
                    "PowerShell completion installation not supported. Use 'git x completion powershell' and add to your profile manually.".to_string(),
                ))
            }
            Shell::Elvish => {
                Err(GitXError::Other(
                    "Elvish completion installation not supported. Use 'git x completion elvish' and add to rc.elv manually.".to_string(),
                ))
            }
            _ => {
                Err(GitXError::Other(format!(
                    "Unsupported shell: {:?}",
                    self.shell
                )))
            }
        }
    }

    fn get_completion_filename(&self) -> &str {
        match self.shell {
            Shell::Bash => "git-x",
            Shell::Zsh => "_git-x",
            Shell::Fish => "git-x.fish",
            _ => "git-x",
        }
    }

    fn generate_completion_script(&self) -> Result<String> {
        use crate::cli::Cli;
        use clap::CommandFactory;
        use clap_complete::generate;
        use std::io::Cursor;

        let mut cmd = Cli::command();
        let mut buf = Cursor::new(Vec::new());

        match self.shell {
            Shell::Bash => {
                generate(clap_complete::shells::Bash, &mut cmd, "git-x", &mut buf);
            }
            Shell::Zsh => {
                generate(clap_complete::shells::Zsh, &mut cmd, "git-x", &mut buf);
            }
            Shell::Fish => {
                generate(clap_complete::shells::Fish, &mut cmd, "git-x", &mut buf);
            }
            _ => {
                return Err(GitXError::Other(format!(
                    "Completion generation not supported for {:?}",
                    self.shell
                )));
            }
        }

        String::from_utf8(buf.into_inner())
            .map_err(|e| GitXError::Other(format!("Failed to generate completion script: {e}")))
    }

    fn get_shell_setup_instructions(&self, completion_path: &Path) -> String {
        match self.shell {
            Shell::Bash => {
                format!(
                    "Completion installed to: {}\n\n\
                    To enable bash completions, add this to your ~/.bashrc or ~/.bash_profile:\n\
                    if [[ -d ~/.local/share/bash-completion/completions ]]; then\n\
                        for completion in ~/.local/share/bash-completion/completions/*; do\n\
                            [[ -r \"$completion\" ]] && source \"$completion\"\n\
                        done\n\
                    fi\n\n\
                    Then restart your shell or run: source ~/.bashrc",
                    completion_path.display()
                )
            }
            Shell::Zsh => {
                format!(
                    "Completion installed to: {}\n\n\
                    To enable zsh completions, add this to your ~/.zshrc:\n\
                    if [[ -d ~/.local/share/zsh/site-functions ]]; then\n\
                        fpath=(~/.local/share/zsh/site-functions $fpath)\n\
                        autoload -U compinit && compinit\n\
                    fi\n\n\
                    Then restart your shell or run: source ~/.zshrc\n\
                    You may also need to clear the completion cache: rm -f ~/.zcompdump*",
                    completion_path.display()
                )
            }
            Shell::Fish => {
                format!(
                    "Completion installed to: {}\n\n\
                    Fish completions are automatically loaded from ~/.config/fish/completions/\n\
                    Restart your shell or run: fish -c 'complete --erase; source ~/.config/fish/config.fish'",
                    completion_path.display()
                )
            }
            _ => format!("Completion installed to: {}", completion_path.display()),
        }
    }
}

impl Command for CompletionInstallCommand {
    fn execute(&self) -> Result<String> {
        let completion_dir = self.get_completion_directory()?;
        let filename = self.get_completion_filename();
        let completion_path = completion_dir.join(filename);

        let completion_script = self.generate_completion_script()?;

        fs::write(&completion_path, completion_script).map_err(|e| {
            GitXError::Other(format!(
                "Failed to write completion file to {}: {}",
                completion_path.display(),
                e
            ))
        })?;

        let instructions = self.get_shell_setup_instructions(&completion_path);

        Ok(format!(
            "✅ Shell completion installed successfully!\n\n{instructions}"
        ))
    }

    fn name(&self) -> &'static str {
        "completion-install"
    }

    fn description(&self) -> &'static str {
        "Install shell completion to standard location"
    }
}
