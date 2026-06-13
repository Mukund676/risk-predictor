#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

echo "Initializing Investment Banking Risk Predictor pipeline with Conda..."

# 1. Create the main project directory
PROJECT_DIR="risk_predictor"
mkdir -p "$PROJECT_DIR"
cd "$PROJECT_DIR"

# 2. Set up the Conda environment
echo "Setting up Conda environment..."
# This hook allows the 'conda activate' command to work inside a bash script
eval "$(conda shell.bash hook)"

# Create a new environment named 'risk_predictor' with Python 3.11
conda create -n "$PROJECT_DIR" python=3.11 -y

# Activate the environment
conda activate "$PROJECT_DIR"

# 3. Install maturin
echo "Installing maturin..."
pip install maturin

# 4. Scaffold the Rust project with PyO3 bindings
# This creates a folder named 'rust_core' and automatically configures 
# the Cargo.toml and lib.rs for Python extension compilation.
echo "Initializing Rust core with PyO3..."
maturin new rust_core --bindings pyo3

echo "===================================================="
echo "Initialization complete!"
echo ""
echo "Next steps:"
echo "1. Run: cd $PROJECT_DIR"
echo "2. Run: conda activate $PROJECT_DIR"
echo "3. Pass the master prompt and the phase-0 schemas to your AI agent to populate the Rust/Python boundary."
echo "===================================================="