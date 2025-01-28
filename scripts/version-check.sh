#!/usr/bin/env bash

# Display an error message and exit
error() {
    echo "ERROR: $1" >&2
    exit 1
}

# Fetch the version from a branch's Cargo.toml
get_branch_version() {
    local branch="$1"
    git fetch origin "$branch" > /dev/null 2>&1 || error "Failed to fetch branch: $branch"
    local version
    version=$(git show "origin/$branch:gbf_core/Cargo.toml" | grep '^version' | sed 's/version = "//;s/"//')
    [[ -z "$version" ]] && error "Failed to fetch version from branch: $branch"
    echo "$version"
}

# Fetch the version from a local Cargo.toml file
get_local_version() {
    local path="$1"
    local version
    version=$(grep '^version' "$path" | sed 's/version = "//;s/"//')
    [[ -z "$version" ]] && error "Failed to fetch version from $path"
    echo "$version"
}

# Fetch the version from the README file
get_readme_version() {
    local version
    version=$(grep 'gbf_core =' README.md | sed 's/.*gbf_core = "//;s/"//;s/.*\[dependencies\]//')
    [[ -z "$version" ]] && error "Could not find version in README.md"
    echo "$version"
}

# Parse a version string into major, minor, and patch components
parse_version() {
    local version="$1"
    [[ -z "$version" ]] && error "Version string is empty"
    local major minor patch
    major=$(echo "$version" | cut -d. -f1)
    minor=$(echo "$version" | cut -d. -f2)
    patch=$(echo "$version" | cut -d. -f3)
    [[ -z "$major" || -z "$minor" || -z "$patch" ]] && error "Invalid version string: $version"
    echo "$major $minor $patch"
}

# Check version increment rules for merging into main
check_main_merge_version() {
    local current_major=$1 current_minor=$2 current_patch=$3
    local main_major=$4 main_minor=$5 main_patch=$6

    if ((current_major > main_major)); then
        if ((current_minor != 0 || current_patch != 0)); then
            error "When bumping the major version, minor and patch must be set to 0."
        fi
    elif ((current_major == main_major)); then
        if ((current_minor <= main_minor)); then
            error "Minor version must be greater than the current main branch version ($main_minor)."
        fi
        if ((current_patch != 0)); then
            error "When bumping the minor version, patch must be set to 0."
        fi
    else
        error "Invalid version bump. Increment the major version or the minor version."
    fi
}

# Check version increment rules for merging into dev
check_dev_merge_version() {
    local current_major=$1 current_minor=$2 current_patch=$3
    local dev_major=$4 dev_minor=$5 dev_patch=$6

    if ((current_major != dev_major || current_minor != dev_minor)); then
        error "When merging into dev, only the patch version can change."
    fi
    if ((current_patch <= dev_patch)); then
        error "Patch version must be greater than the current dev branch version ($dev_patch)."
    fi
}

# Validate that two versions match
check_version_match() {
    local current_version="$1"
    local target_version="$2"
    local description="$3"
    if [[ "$current_version" != "$target_version" ]]; then
        error "Version mismatch: $description. Expected $target_version, found $current_version."
    fi
}

# Main script logic
echo "Fetching versions..."
MAIN_VERSION=$(get_branch_version main)
DEV_VERSION=$(get_branch_version dev)
CURRENT_VERSION=$(get_local_version ./gbf_core/Cargo.toml)
MACROS_VERSION=$(get_local_version ./gbf_macros/Cargo.toml)
SUITE_VERSION=$(get_local_version ./gbf_suite/Cargo.toml)
README_VERSION=$(get_readme_version)

echo "Main branch version: $MAIN_VERSION"
echo "Dev branch version: $DEV_VERSION"
echo "Current branch version: $CURRENT_VERSION"
echo "gbf_macros version: $MACROS_VERSION"
echo "gbf_suite version: $SUITE_VERSION"
echo "README version: $README_VERSION"

echo "Parsing versions..."
read MAIN_MAJOR MAIN_MINOR MAIN_PATCH < <(parse_version "$MAIN_VERSION")
read DEV_MAJOR DEV_MINOR DEV_PATCH < <(parse_version "$DEV_VERSION")
read CURRENT_MAJOR CURRENT_MINOR CURRENT_PATCH < <(parse_version "$CURRENT_VERSION")

# Determine target branch
if [[ -n "$GITHUB_BASE_REF" ]]; then
    TARGET_BRANCH="$GITHUB_BASE_REF"
else
    TARGET_BRANCH=$(git rev-parse --abbrev-ref HEAD)
fi
echo "Target branch detected: $TARGET_BRANCH"

# Perform version checks based on the target branch
if [[ "$TARGET_BRANCH" == "main" ]]; then
    echo "Checking version for merging into main..."
    check_main_merge_version "$CURRENT_MAJOR" "$CURRENT_MINOR" "$CURRENT_PATCH" "$MAIN_MAJOR" "$MAIN_MINOR" "$MAIN_PATCH"
elif [[ "$TARGET_BRANCH" == "dev" ]]; then
    echo "Checking version for merging into dev..."
    check_dev_merge_version "$CURRENT_MAJOR" "$CURRENT_MINOR" "$CURRENT_PATCH" "$DEV_MAJOR" "$DEV_MINOR" "$DEV_PATCH"
else
    echo "No specific version check required for this branch."
fi

# Validate README and gbf_macros versions
echo "Checking README version..."
check_version_match "$CURRENT_VERSION" "$README_VERSION" "README.md version mismatch"

echo "Checking gbf_macros version..."
check_version_match "$CURRENT_VERSION" "$MACROS_VERSION" "gbf_macros/Cargo.toml version mismatch"

echo "Checking gbf_suite version..."
check_version_match "$CURRENT_VERSION" "$SUITE_VERSION" "gbf_suite/Cargo.toml version mismatch"

echo "All version checks passed!"
