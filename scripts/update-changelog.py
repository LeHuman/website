#!/usr/bin/env python3
"""
Update CHANGELOG.md using git-cliff with version bumping.

This script:
1. Gets the latest git tag
2. Bumps the version (patch by default)
3. Runs git-cliff with the new version
4. Updates CHANGELOG.md
"""

import subprocess
import sys
import re


def run_command(cmd, check=True):
    """Run a shell command and return output."""
    try:
        result = subprocess.run(
            cmd,
            shell=True,
            capture_output=True,
            text=True,
            check=check
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError as e:
        if check:
            print(f"Error running command: {cmd}")
            print(f"Error: {e.stderr}")
            sys.exit(1)
        return None


def get_latest_tag():
    """Get the latest git tag."""
    tags = run_command("git tag --sort=-version:refname")
    if not tags:
        return None
    return tags.split('\n')[0]


def bump_version(version, bump_type="patch"):
    """Bump a semantic version."""
    if not version:
        return "0.1.0"
    
    # Remove 'v' prefix if present
    version = version.lstrip('v')
    
    # Parse version
    match = re.match(r'^(\d+)\.(\d+)\.(\d+)(?:-(\w+))?(?:\+(\d+))?$', version)
    if not match:
        print(f"Warning: Could not parse version '{version}', using 0.1.0")
        return "0.1.0"
    
    major, minor, patch, pre, build = match.groups()
    major, minor, patch = int(major), int(minor), int(patch)
    
    if bump_type == "major":
        major += 1
        minor = 0
        patch = 0
    elif bump_type == "minor":
        minor += 1
        patch = 0
    else:  # patch
        patch += 1
    
    new_version = f"{major}.{minor}.{patch}"
    if pre:
        new_version += f"-{pre}"
    if build:
        new_version += f"+{build}"
    
    return new_version


def main():
    # Get latest tag
    latest_tag = get_latest_tag()
    
    if latest_tag:
        print(f"Latest tag: {latest_tag}")
        new_version = bump_version(latest_tag, "patch")
    else:
        print("No tags found, starting from 0.1.0")
        new_version = "0.1.0"
    
    print(f"Bumping to: v{new_version}")
    
    # Run git-cliff with the new version
    print("\nUpdating CHANGELOG.md...")
    cmd = f'git-cliff --config cliff.toml --tag v{new_version} --output CHANGELOG.md'
    run_command(cmd)
    
    print(f"\n✅ CHANGELOG.md updated with version v{new_version}")
    print("\nNext steps:")
    print("  1. Review CHANGELOG.md")
    print("  2. Commit changes: git add CHANGELOG.md && git commit -m 'docs: Update CHANGELOG.md'")
    print(f"  3. Tag release: git tag v{new_version}")


if __name__ == "__main__":
    main()
