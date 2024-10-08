# Syntax Highlighting for Sailfish Templates in VSCode

This directory contains Syntax Highlighting extension for sailfish templates in Visual Studio Code.

The extension is available at [VisualStudio Marketplace](https://marketplace.visualstudio.com/items?itemName=rust-sailfish.vscode-rust-sailfish).

## Features

- Full Rust syntax highlighting rules inside code blocks
- Auto-closing brackets for code blocks
- Folding for comment blocks

## Screenshots

![screenshot](https://github.com/rust-sailfish/sailfish/blob/main/syntax/vscode/screenshot.png?raw=true)

## Developer Instructions
1. Open the /sailfish/syntax/vscode/ directory in VS Code.
2. Run the following commands:
    `sudo apt install nodejs`
    `npm install typescript --save-dev`
    `npm install @eslint/js -D`
    `npm install typescript-eslint -D`
    `npm install -g @vscode/vsce`
    `vsce login rust-sailfish`
    `vsce publish`