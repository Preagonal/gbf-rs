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
echo "Running cargo test..."
cargo test
if [ $? -ne 0 ]; then
  echo "ERROR: Tests failed. Please fix and re-commit."
  exit 1
fi

echo "All checks passed!"
exit 0