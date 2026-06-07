# Installation

## Requirements

- Rust 1.78 or later
- macOS (primary target) or Linux

## Install from source

```bash
git clone https://github.com/vatskyone/claux.git
cd claux/apps/cli

# Build and install the binary to ~/.cargo/bin/claux
cargo install --path .
```

Confirm the installation:

```bash
claux --version
# claux 0.7.2
```

## Shell completions

Claux can generate completion scripts for zsh, bash, and fish.

### zsh

```bash
mkdir -p ~/.zsh/completions
claux completions zsh > ~/.zsh/completions/_claux

# Add to ~/.zshrc if not already present:
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
echo 'autoload -U compinit && compinit' >> ~/.zshrc

source ~/.zshrc
```

### bash

```bash
claux completions bash >> ~/.bashrc
source ~/.bashrc
```

### fish

```bash
claux completions fish > ~/.config/fish/completions/claux.fish
```

## Updating

Pull the latest changes and reinstall:

```bash
cd claux/apps/cli
git pull
cargo install --path . --force
```

## Uninstalling

```bash
cargo uninstall claux
```

This removes the binary from `~/.cargo/bin`. Claux's local data (`~/.claude/claux/`) is not affected.

To also remove Claux's local data:

```bash
rm -rf ~/.claude/claux
```

This removes tags, checkpoints, config, and rate-limit data. Claude Code's own session files in `~/.claude/projects/` are not affected.
