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

# Extract major and minor versions
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

# Ensure the minor version is incremented
check_minor_version() {
    if [ "$CURRENT_MINOR" -le "$MAIN_MINOR" ]; then
        error "Minor version must be greater than the current main branch version ($MAIN_MINOR). Found: $CURRENT_MINOR"
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
MAIN_MINOR="$MINOR"

parse_version "$CURRENT_VERSION"
CURRENT_MAJOR="$MAJOR"
CURRENT_MINOR="$MINOR"

echo "Checking major version..."
check_major_version

echo "Checking minor version..."
check_minor_version

echo "All version checks passed!"
