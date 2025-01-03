#!/usr/bin/env bash

# Function to display errors and exit
error() {
    echo "ERROR: $1"
    exit 1
}

# Fetch the main branch version
get_main_version() {
    git fetch origin main > /dev/null 2>&1 || error "Failed to fetch main branch"
    MAIN_VERSION=$(git show origin/main:Cargo.toml | grep '^version' | sed 's/version = "//;s/"//')
    echo "$MAIN_VERSION"
}

# Fetch the current branch version
get_current_version() {
    CURRENT_VERSION=$(grep '^version' Cargo.toml | sed 's/version = "//;s/"//')
    echo "$CURRENT_VERSION"
}

# Extract version from README.md
get_readme_version() {
    README_VERSION=$(grep 'gbf-rs =' README.md | sed 's/.*gbf-rs = "//;s/"//;s/.*\[dependencies\]//')
    if [ -z "$README_VERSION" ]; then
        error "Could not find version in README.md"
    fi
    echo "$README_VERSION"
}

# Extract major, minor, and patch versions
parse_version() {
    VERSION="$1"
    MAJOR=$(echo "$VERSION" | cut -d. -f1)
    MINOR=$(echo "$VERSION" | cut -d. -f2)
    PATCH=$(echo "$VERSION" | cut -d. -f3)
}

# Ensure the major version is 0
check_major_version() {
    if [ "$MAJOR" -ne 0 ]; then
        error "Major version must be 0 during development. Found: $MAJOR"
    fi
}

# Ensure the patch version is incremented
check_patch_version() {
    if [ "$CURRENT_PATCH" -le "$MAIN_PATCH" ]; then
        error "Patch version must be greater than the current main branch version ($MAIN_PATCH). Found: $CURRENT_PATCH"
    fi
}

# Ensure the README version matches the current version
check_readme_version() {
    if [ "$CURRENT_VERSION" != "$README_VERSION" ]; then
        error "Version mismatch: README.md version is $README_VERSION, but Cargo.toml version is $CURRENT_VERSION"
    fi
}

# Main script logic
echo "Fetching main branch version..."
MAIN_VERSION=$(get_main_version)
echo "Main branch version: $MAIN_VERSION"

echo "Fetching current branch version..."
CURRENT_VERSION=$(get_current_version)
echo "Current branch version: $CURRENT_VERSION"

echo "Parsing versions..."
parse_version "$MAIN_VERSION"
MAIN_MAJOR="$MAJOR"
MAIN_PATCH="$PATCH"

parse_version "$CURRENT_VERSION"
CURRENT_MAJOR="$MAJOR"
CURRENT_PATCH="$PATCH"

echo "Fetching README version..."
README_VERSION=$(get_readme_version)
echo "README version: $README_VERSION"

echo "Checking major version..."
check_major_version

echo "Checking patch version..."
check_patch_version

echo "Checking README version..."
check_readme_version

echo "All version checks passed!"
