#!/bin/bash

# Check if the user is on Windows
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    echo "It looks like you are on Windows. Please install Rust using the Windows installer:"
    echo "https://www.rust-lang.org/tools/install"
    exit 1
fi

# Check if Rust is already installed
if command -v rustc >/dev/null 2>&1; then
    echo "Rust is already installed."
else
    echo "Rust is not installed. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # Add Rust to the current shell session
    source $HOME/.cargo/env
fi

# Build the Rust project
cargo build