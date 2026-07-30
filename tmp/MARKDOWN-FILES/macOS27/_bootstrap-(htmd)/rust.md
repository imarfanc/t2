---
name: rust
sort: "6"
icon: "ph:dice-six-duotone"
category: rare
subcategory: macOS-reset-26.7b
subsubcategory: _bootstrap
description: "Rust toolchain installation via rustup and Xcode Command Line Tools setup"
date: 2026-5-1
tags:
  - rust
  - rustup
  - cargo
  - install
visibility: public
---

# Rust Installation

Rust requires Apple's C toolchain and linker (Xcode Command Line Tools), followed by `rustup`.

## 1. Prerequisite: Xcode Command Line Tools

Rust requires Apple's linker and C toolchain. If not already installed during initial bootstrap, refer to [terminal-first.md](./terminal-first.md#install-xcode-command-line-tools).

## 2. Install Rust with rustup

Official installer and toolchain manager ([rustup.rs](https://rustup.rs)):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Press **Enter** to accept default settings. This installs `rustc`, `cargo`, and `rustup`.

## 3. Shell Profile Setup & Verification

```sh
source "$HOME/.cargo/env"
rustc --version
cargo --version
```

## 4. Keeping Rust Updated

```sh
rustup update
```
