#!/bin/bash

# Script to validate and publish neocrates crate to crates.io
# This script works with the workspace inheritance structure
# where the main crate is in neocrates/neocrates/ directory

set -e

echo "Publishing neocrates crate to crates.io..."
echo ""

# Check if we are in the root directory
if [ ! -f "Cargo.toml" ] || ! grep -q "\[workspace\]" Cargo.toml; then
    echo "Error: This script must be run from the project root directory"
    echo "Expected to find workspace Cargo.toml in current directory"
    exit 1
fi

# Check if neocrates crate directory exists
if [ ! -d "neocrates" ] || [ ! -f "neocrates/Cargo.toml" ]; then
    echo "Error: neocrates crate directory not found!"
    echo "Expected neocrates/Cargo.toml to exist"
    exit 1
fi

# Check if we have the lib.rs file
if [ ! -f "neocrates/src/lib.rs" ]; then
    echo "Error: neocrates/src/lib.rs not found!"
    exit 1
fi

# Navigate to the neocrates crate directory
cd neocrates

echo "Running cargo publish --dry-run --allow-dirty --registry crates-io to verify..."
echo ""

# Run dry-run with all necessary flags
if cargo publish --dry-run --allow-dirty --registry crates-io; then
    echo ""
    echo "✅ Dry-run completed successfully!"
    echo ""
    echo "To publish to crates.io, run the following command from the neocrates/neocrates directory:"
    echo ""
    echo "  cargo publish --registry crates-io"
    echo ""
    echo "If you have uncommitted changes, use:"
    echo "  cargo publish --allow-dirty --registry crates-io"
    echo ""
    echo "Note: The neocrates crate uses workspace inheritance,"
    echo "so all dependency versions are managed in the root Cargo.toml"
else
    echo ""
    echo "❌ Dry-run failed. Please check the error messages above."
    exit 1
fi
