#!/usr/bin/env bash

# 1. Check formatting
echo "Running cargo fmt check..."
cargo fmt --all -- --check
if [ $? -ne 0 ]; then
  echo "ERROR: Code is not formatted. Run 'cargo fmt --all' and re-commit."
  exit 1
fi

# 2. Lint the code
echo "Running cargo clippy..."
cargo clippy --workspace --all-targets -- -D warnings
if [ $? -ne 0 ]; then
  echo "ERROR: Clippy found warnings or errors. Please fix them and re-commit."
  exit 1
fi

# 3. Run tests
if [ "$IS_CI" = "true" ]; then
  echo "Running cargo test..."
  cargo test --workspace
  if [ $? -ne 0 ]; then
    echo "ERROR: Tests (including doctests) failed. Please fix and re-commit."
    exit 1
  fi
else
  echo "Running cargo test without doctests..."
  echo "Note: Doctests are not run locally to speed up the process."
  echo "      This will change when edition2024 is stable."
  cargo test -q --all-targets --lib
  if [ $? -ne 0 ]; then
    echo "ERROR: Tests (excluding doctests) failed. Please fix and re-commit."
    exit 1
  fi
fi

echo "All checks passed!"
exit 0
