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

# Fetch the dev branch version
get_dev_version() {
    git fetch origin dev > /dev/null 2>&1 || error "Failed to fetch dev branch"
    DEV_VERSION=$(git show origin/dev:Cargo.toml | grep '^version' | sed 's/version = "//;s/"//')
    echo "$DEV_VERSION"
}

# Fetch the current branch version
get_current_version() {
    CURRENT_VERSION=$(grep '^version' ./gbf_core/Cargo.toml | sed 's/version = "//;s/"//')
    echo "$CURRENT_VERSION"
}

# Fetch the gbf_macros version
get_macros_version() {
    MACROS_VERSION=$(grep '^version' ./gbf_macros/Cargo.toml | sed 's/version = "//;s/"//')
    echo "$MACROS_VERSION"
}

# Fetch the README version
get_readme_version() {
    README_VERSION=$(grep 'gbf_core =' README.md | sed 's/.*gbf_core = "//;s/"//;s/.*\[dependencies\]//')
    if [ -z "$README_VERSION" ]; then
        error "Could not find version in README.md"
    fi
    echo "$README_VERSION"
}

# Parse version into major, minor, and patch components
parse_version() {
    VERSION="$1"
    MAJOR=$(echo "$VERSION" | cut -d. -f1)
    MINOR=$(echo "$VERSION" | cut -d. -f2)
    PATCH=$(echo "$VERSION" | cut -d. -f3)
}

# Check version increment when merging into main
check_main_merge_version() {
    if [ "$CURRENT_MAJOR" -gt "$MAIN_MAJOR" ]; then
        if [ "$CURRENT_MINOR" -ne 0 ] || [ "$CURRENT_PATCH" -ne 0 ]; then
            error "When bumping the major version, minor and patch must be set to 0. Found: $CURRENT_MAJOR.$CURRENT_MINOR.$CURRENT_PATCH"
        fi
    elif [ "$CURRENT_MAJOR" -eq "$MAIN_MAJOR" ]; then
        if [ "$CURRENT_MINOR" -le "$MAIN_MINOR" ]; then
            error "Minor version must be greater than the current main branch version ($MAIN_MINOR). Found: $CURRENT_MINOR"
        fi
        if [ "$CURRENT_PATCH" -ne 0 ]; then
            error "When bumping the minor version, patch must be set to 0. Found: $CURRENT_PATCH"
        fi
    else
        error "Invalid version bump. Either increment the major version or the minor version (with the rules above)."
    fi
}

# Check version increment when merging into dev
check_dev_merge_version() {
    if [ -z "$CURRENT_MAJOR" ] || [ -z "$DEV_MAJOR" ] || [ -z "$CURRENT_MINOR" ] || [ -z "$DEV_MINOR" ] || [ -z "$CURRENT_PATCH" ] || [ -z "$DEV_PATCH" ]; then
        error "One or more version components are not set."
    fi

    if [ "$CURRENT_MAJOR" -ne "$DEV_MAJOR" ] || [ "$CURRENT_MINOR" -ne "$DEV_MINOR" ]; then
        error "When merging into dev, only the patch version can change, and the major and minor versions must remain the same."
    fi
    if [ "$CURRENT_PATCH" -le "$DEV_PATCH" ]; then
        error "Patch version must be greater than the current dev branch version ($DEV_PATCH). Found: $CURRENT_PATCH"
    fi
}

# Check if README version matches the current version
check_readme_version() {
    if [ "$CURRENT_VERSION" != "$README_VERSION" ]; then
        error "Version mismatch: README.md version is $README_VERSION, but Cargo.toml version is $CURRENT_VERSION"
    fi
}

# Check if gbf_macros version matches the current version
check_macros_version() {
    if [ "$CURRENT_VERSION" != "$MACROS_VERSION" ]; then
        error "Version mismatch: gbf_macros/Cargo.toml version is $MACROS_VERSION, but gbf_core/Cargo.toml version is $CURRENT_VERSION"
    fi
}

# Main script logic
echo "Fetching main branch version..."
MAIN_VERSION=$(get_main_version)
echo "Main branch version: $MAIN_VERSION"

echo "Fetching dev branch version..."
DEV_VERSION=$(get_dev_version)
echo "Dev branch version: $DEV_VERSION"

echo "Fetching current branch version..."
CURRENT_VERSION=$(get_current_version)
echo "Current branch version: $CURRENT_VERSION"

echo "Fetching gbf_macros version..."
MACROS_VERSION=$(get_macros_version)
echo "gbf_macros version: $MACROS_VERSION"

echo "Fetching README version..."
README_VERSION=$(get_readme_version)
echo "README version: $README_VERSION"

echo "Parsing versions..."
parse_version "$MAIN_VERSION"
MAIN_MAJOR="$MAJOR"
MAIN_MINOR="$MINOR"
MAIN_PATCH="$PATCH"

parse_version "$DEV_VERSION"
DEV_MAJOR="$MAJOR"
DEV_MINOR="$MINOR"
DEV_PATCH="$PATCH"

parse_version "$CURRENT_VERSION"
CURRENT_MAJOR="$MAJOR"
CURRENT_MINOR="$MINOR"
CURRENT_PATCH="$PATCH"

# Determine the target branch
# Determine the target branch in a GitHub Actions environment
if [ -n "$GITHUB_BASE_REF" ]; then
    # In a PR, use GITHUB_BASE_REF to get the target branch
    TARGET_BRANCH="$GITHUB_BASE_REF"
else
    # For direct pushes, fall back to the current branch
    TARGET_BRANCH=$(git rev-parse --abbrev-ref HEAD)
fi

echo "Target branch detected: $TARGET_BRANCH"

# Proceed with the existing checks based on the target branch
if [[ "$TARGET_BRANCH" == "main" ]]; then
    echo "Checking version for merging into main..."
    check_main_merge_version
elif [[ "$TARGET_BRANCH" == "dev" ]]; then
    echo "Checking version for merging into dev..."
    check_dev_merge_version
else
    echo "No specific version check required for this branch."
fi

# Check README and gbf_macros versions
echo "Checking README version..."
check_readme_version

echo "Checking gbf_macros version..."
check_macros_version

echo "All version checks passed!"
