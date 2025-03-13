#!/bin/bash
set -e

echo "Installing pre-commit hooks..."
pip install pre-commit
pre-commit install

echo "Pre-commit hooks installed successfully!"