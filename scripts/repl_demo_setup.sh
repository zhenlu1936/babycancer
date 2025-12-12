#!/bin/bash

# BabyCancer REPL Demo Setup
# Creates demo files and directories for REPL demonstration

echo "=== Setting up BabyCancer REPL Demo ==="

# Clean and create demo directories
rm -rf demo_repl
mkdir -p demo_repl/{source,backup}

# Create demo files
echo "Project documentation" > demo_repl/source/readme.txt
echo "Configuration settings" > demo_repl/source/config.toml
echo "Application logs" > demo_repl/source/app.log

echo "✓ Demo directories created"
echo "✓ Demo files created in demo_repl/source/"
echo ""
echo "Setup complete! Run: ./target/release/babycancer"
