#!/usr/bin/env pwsh

# 1. Check formatting
Write-Host "Running cargo fmt check..."
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Code is not formatted. Run 'cargo fmt --all' and re-commit." -ForegroundColor Red
    exit 1
}

# 2. Lint the code
Write-Host "Running cargo clippy..."
cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Clippy found warnings or errors. Please fix them and re-commit." -ForegroundColor Red
    exit 1
}

# 3. Run tests
Write-Host "Running cargo test..."
cargo test --workspace --all-targets
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Tests failed. Please fix and re-commit." -ForegroundColor Red
    exit 1
}

Write-Host "All checks passed!" -ForegroundColor Green
exit 0
