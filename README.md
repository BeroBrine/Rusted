# Rusted

A lightweight, Vim-inspired text editor written in Rust.

## Overview

Rusted was born from my love for Vim and desire to learn Rust programming. As Vim has been my favorite editor for years, I wanted to combine the elegant modal editing philosophy that has made me productive with my journey into Rust development. This project serves both as a practical learning experience and as a tribute to the editor that has shaped my workflow.

By reimplementing the core features that make Vim special, I'm gaining deeper insights into both Rust's performance benefits and systems programming concepts while preserving the editing experience I've come to rely on. Rusted aims to maintain Vim's efficiency while exploring what modern Rust can bring to text editing.

## Features

### Currently Implemented

- **Vim-style Modal Editing**: Navigate and edit text efficiently in normal and insert modes
- **Core Navigation**: 
  - Basic movement keys (`h`, `j`, `k`, `l`)
  - Jump to line start/end (`0`, `$`)
  - Move to buffer start/end (`gg`, `G`)
- **Editing Commands**:
  - Undo functionality (`u`)
  - Delete line (`dd`)
  - Center view (`zz`) 
- **Chorded Key Support**: Properly handles multi-key commands like `dd` and `zz`
- **Syntax Highlighting**: Support for common programming languages

### Coming Soon

- **LSP Integration**: Code intelligence with the Language Server Protocol
- **Vim Command Line**: Support for ex commands with `:` prefix
- **Extended Keybindings**: More advanced Vim motions and text objects
- **Theme Support**: Import your favorite theme in rusted.
- **Custom Configuration**: Change Keybindings using custom configuration file.

## Installation

```bash
# Install from cargo
cargo install rusted

# Or build from source
git clone https://github.com/yourusername/rusted.git
cd rusted
cargo build --release
```

## Usage

```bash
# Open a file
rusted path/to/file.rs

# Open multiple files
rusted file1.rs file2.rs
```

## Keybindings

### Navigation
- `h`, `j`, `k`, `l`: Move cursor left, down, up, right
- `0`: Move to start of line
- `$`: Move to end of line
- `gg`: Move to start of buffer
- `G`: Move to end of buffer

### Editing
- `i`: Enter insert mode
- `Esc`: Return to normal mode
- `u`: Undo last change
- `dd`: Delete current line
- `zz`: Center view on cursor


### Configuration File - Coming Soon
```toml
# Example configuration
[editor]
line_numbers = true
tab_width = 4
theme = "monokai"

[keybindings]
# Custom keybindings can be defined here
```

## Acknowledgments

- Inspired by the legendary Vim editor
- Built with Rust for performance and safety
