#!/bin/bash
# Release automation script for typeglass-cli
# Usage: ./scripts/release.sh <version>
# Example: ./scripts/release.sh 0.2.0

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Error: Version number required"
    echo "Usage: ./scripts/release.sh <version>"
    echo "Example: ./scripts/release.sh 0.2.0"
    exit 1
fi

# Validate version format (semantic versioning)
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Invalid version format. Must be X.Y.Z (e.g., 0.2.0)"
    exit 1
fi

echo "🚀 Starting release process for v$VERSION"

# Check we're on main/master branch
BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [[ "$BRANCH" != "main" && "$BRANCH" != "master" && "$BRANCH" != "feature/initial-commit" ]]; then
    echo "Error: Must be on main/master branch to release"
    echo "Current branch: $BRANCH"
    exit 1
fi

# Check for uncommitted changes
if [[ -n $(git status -s) ]]; then
    echo "Error: Uncommitted changes detected"
    git status -s
    exit 1
fi

# Update version in Cargo.toml
echo "📝 Updating Cargo.toml version to $VERSION"
sed -i.bak "s/^version = .*/version = \"$VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Update Cargo.lock
echo "🔒 Updating Cargo.lock"
cargo update -p typeglass-cli

# Run full test suite
echo "🧪 Running tests"
cargo test --all-features

# Run clippy
echo "📎 Running clippy"
cargo clippy --all-targets --all-features -- -D warnings

# Build release binary
echo "🔨 Building release binary"
cargo build --release

# Generate changelog entry
echo "📋 Generating changelog"
CHANGELOG_ENTRY=$(git log --oneline $(git describe --tags --abbrev=0 2>/dev/null || echo "")..HEAD | sed 's/^/- /')

# Update CHANGELOG.md
if [ ! -f CHANGELOG.md ]; then
    cat > CHANGELOG.md << CHANGELOG_HEADER
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

CHANGELOG_HEADER
fi

# Prepend new version to changelog
{
    echo "## [$VERSION] - $(date +%Y-%m-%d)"
    echo ""
    echo "$CHANGELOG_ENTRY"
    echo ""
    cat CHANGELOG.md
} > CHANGELOG.md.new
mv CHANGELOG.md.new CHANGELOG.md

# Commit version bump
echo "💾 Committing version bump"
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore(release): bump version to v$VERSION

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"

# Create git tag
echo "🏷️  Creating tag v$VERSION"
git tag -a "v$VERSION" -m "Release v$VERSION"

echo ""
echo "✅ Release v$VERSION ready!"
echo ""
echo "Next steps:"
echo "  1. Review the changelog in CHANGELOG.md"
echo "  2. Push changes: git push origin $BRANCH"
echo "  3. Push tag: git push origin v$VERSION"
echo "  4. GitHub Actions will build and publish the release"
echo ""
echo "To undo:"
echo "  git reset --hard HEAD~1"
echo "  git tag -d v$VERSION"
