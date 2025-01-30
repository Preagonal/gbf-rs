#!/usr/bin/env python3

import tomlkit
import json
import os
import subprocess
import logging
import re
import argparse
from semantic_version import Version

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def get_branch_version(branch):
    """Fetch version from a branch's Cargo.toml using Git."""
    try:
        version = subprocess.check_output(
            f"git fetch origin {branch} && git show origin/{branch}:gbf_core/Cargo.toml",
            shell=True,
            text=True
        )
        cargo_content = tomlkit.loads(version)
        version_str = cargo_content.get('package', {}).get('version')
        if not version_str:
            raise ValueError(f"Version not found in branch {branch}")
        return Version(version_str)
    except subprocess.CalledProcessError as e:
        logger.error(f"Failed to fetch version from branch {branch}: {e}")
        raise

def get_local_cargo_version(path):
    """Fetch version from a local Cargo.toml file."""
    try:
        with open(path, 'r') as f:
            cargo_content = tomlkit.load(f)
            version_str = cargo_content.get('package', {}).get('version')
            if not version_str:
                raise ValueError(f"Version not found in {path}")
            return Version(version_str)
    except FileNotFoundError:
        logger.error(f"File {path} not found")
        raise

def get_local_package_version(path):
    """Fetch version from a package.json file."""
    try:
        with open(path, 'r') as f:
            package_content = json.load(f)
            version_str = package_content.get('version')
            if not version_str:
                raise ValueError(f"Version not found in {path}")
            return Version(version_str)
    except FileNotFoundError:
        logger.error(f"File {path} not found")
        raise
    except json.JSONDecodeError as e:
        logger.error(f"Failed to parse JSON in {path}: {e}")
        raise

def get_readme_version():
    """Fetch version from the README file."""
    try:
        with open('README.md', 'r') as f:
            content = f.read()
            version_match = re.search(r'gbf_core = "(.*?)"', content)
            if not version_match:
                raise ValueError("Version not found in README.md")
            return Version(version_match.group(1))
    except FileNotFoundError:
        logger.error("README.md not found")
        raise

def parse_version(version_str):
    """Parse a version string into a Version object."""
    try:
        return Version(version_str)
    except ValueError as e:
        logger.error(f"Invalid version string: {version_str}")
        raise

def check_main_merge_version(current_version, main_version):
    """Check version rules for merging into main branch."""
    if current_version > main_version:
        if not (current_version.minor == 0 and current_version.patch == 0):
            raise ValueError("When bumping major version, minor and patch must be 0.")
    elif current_version == main_version:
        if current_version.minor <= main_version.minor:
            raise ValueError("Minor version must be greater than main branch version ($main_minor).")
        if current_version.patch != 0:
            raise ValueError("When bumping minor version, patch must be set to 0.")
    else:
        raise ValueError("Invalid version bump. Increment the major version or the minor version.")

def check_dev_merge_version(current_version, dev_version):
    """Check version rules for merging into dev branch."""
    if (current_version.major, current_version.minor) != (dev_version.major, dev_version.minor):
        raise ValueError("When merging into dev, only the patch version can change.")
    if current_version <= dev_version:
        raise ValueError("Patch version must be greater than dev branch version ($dev_patch).")

def check_version_match(current_version, target_version, description):
    """Validate that two versions match."""
    if current_version != target_version:
        raise ValueError(f"Version mismatch: {description}. Expected {target_version}, found {current_version}.")

def bump_version(version, bump_type):
    """Bump major, minor, or patch version."""
    if bump_type == 'major':
        return Version(major=version.major + 1, minor=0, patch=0)
    elif bump_type == 'minor':
        return Version(major=version.major, minor=version.minor + 1, patch=0)
    elif bump_type == 'patch':
        return Version(major=version.major, minor=version.minor, patch=version.patch + 1)
    else:
        raise ValueError("Invalid bump type. Use 'major', 'minor', or 'patch'.")

def update_cargo_version(file_path, new_version):
    """Update version in a Cargo.toml file."""
    try:
        with open(file_path, 'r+') as f:
            cargo_content = tomlkit.load(f)
            if 'package' not in cargo_content:
                cargo_content['package'] = {}
            cargo_content['package']['version'] = str(new_version)
            f.seek(0)
            f.write(tomlkit.dumps(cargo_content))
            f.truncate()
        logger.info(f"Updated version in {file_path} to {new_version}")
    except Exception as e:
        logger.error(f"Failed to update version in {file_path}: {e}")
        raise

def update_package_version(file_path, new_version):
    """Update version in a package.json file."""
    try:
        with open(file_path, 'r+') as f:
            package_content = json.load(f)
            package_content['version'] = str(new_version)
            f.seek(0)
            f.write(json.dumps(package_content, indent=2))
            f.truncate()
        logger.info(f"Updated version in {file_path} to {new_version}")
    except Exception as e:
        logger.error(f"Failed to update version in {file_path}: {e}")
        raise

def update_readme_version(new_version):
    """Update version in the README file."""
    try:
        with open('README.md', 'r+') as f:
            content = f.read()
            new_content = re.sub(
                r'gbf_core = ".*?"',
                f'gbf_core = "{new_version}"',
                content
            )
            f.seek(0)
            f.write(new_content)
            f.truncate()
        logger.info(f"Updated README version to {new_version}")
    except Exception as e:
        logger.error(f"Failed to update README version: {e}")
        raise

def parse_args():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(description='Check and bump versions')
    parser.add_argument('--bump', 
                      choices=['patch', 'minor', 'major'],
                      help='Specify the type of version bump: patch, minor, or major')
    return parser.parse_args()

def main():
    try:
        args = parse_args()
        
        # Fetch versions
        logger.info("Fetching versions...")
        main_version = get_branch_version('main')
        dev_version = get_branch_version('dev')
        current_version = get_local_cargo_version('./gbf_core/Cargo.toml')
        macros_version = get_local_cargo_version('./gbf_macros/Cargo.toml')
        suite_version = get_local_cargo_version('./gbf_suite/Cargo.toml')
        web_version = get_local_package_version('./gbf_web/package.json')
        readme_version = get_readme_version()
        
        # Log versions
        logger.info(f"Main branch version: {main_version}")
        logger.info(f"Dev branch version: {dev_version}")
        logger.info(f"Current branch version: {current_version}")
        logger.info(f"gbf_macros version: {macros_version}")
        logger.info(f"gbf_suite version: {suite_version}")
        logger.info(f"gbf_web version: {web_version}")
        logger.info(f"README version: {readme_version}")
        
        # Determine target branch
        target_branch = subprocess.check_output(['git', 'rev-parse', '--abbrev-ref', 'HEAD']).decode().strip()
        if 'GITHUB_BASE_REF' in os.environ:
            target_branch = os.getenv('GITHUB_BASE_REF', target_branch)
        logger.info(f"Target branch detected: {target_branch}")

        # Validate versions
        logger.info("Checking README version...")
        check_version_match(current_version, readme_version, "README.md version mismatch")
        logger.info("Checking gbf_macros version...")
        check_version_match(current_version, macros_version, "gbf_macros/Cargo.toml version mismatch")
        logger.info("Checking gbf_suite version...")
        check_version_match(current_version, suite_version, "gbf_suite/Cargo.toml version mismatch")
        logger.info("Checking gbf_web version...")
        check_version_match(current_version, web_version, "gbf_web/package.json version mismatch")

        if args.bump:
            # Only bump version if all checks pass
            logger.info(f"Bumping version: {args.bump}")
            new_version = bump_version(current_version, args.bump)
            
            # Update versions in all relevant files
            update_cargo_version('./gbf_core/Cargo.toml', new_version)
            update_cargo_version('./gbf_macros/Cargo.toml', new_version)
            update_cargo_version('./gbf_suite/Cargo.toml', new_version)
            update_package_version('./gbf_web/package.json', new_version)
            update_readme_version(new_version)
            
            logger.info(f"Successfully bumped version to {new_version}")

            return
        
        # Perform version checks based on target branch
        if target_branch == 'main':
            logger.info("Checking version for merging into main...")
            check_main_merge_version(current_version, main_version)
        elif target_branch == 'dev':
            logger.info("Checking version for merging into dev...")
            check_dev_merge_version(current_version, dev_version)
        else:
            logger.info("No specific version check required for this branch.")
        
        logger.info("All version checks passed!")
    except Exception as e:
        logger.error(f"Error in version processing: {e}")
        raise

if __name__ == "__main__":
    main()