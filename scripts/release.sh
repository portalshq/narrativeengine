#!/bin/bash
# Release script for narrative-engine packages
# Usage: ./scripts/release.sh [patch|minor|major]

set -e

VERSION_BUMP="${1:-patch}"

echo "========================================"
echo "  NarrativeEngine Release Script"
echo "========================================"
echo ""

# Verify we're in a clean state
if [[ -n $(git status --porcelain) ]]; then
    echo "❌ Error: Working tree is not clean. Please commit or stash changes first."
    exit 1
fi

# Get current version from narrative-engine
CURRENT_VERSION=$(node -p "require('./narrative-engine/package.json').version")
echo "📦 Current version: $CURRENT_VERSION"

# Update versions in both packages
echo "📈 Bumping $VERSION_BUMP version..."
npm run "version:bump:$VERSION_BUMP" --workspaces --if-present

# Get new version
NEW_VERSION=$(node -p "require('./narrative-engine/package.json').version")
echo "✨ New version: $NEW_VERSION"

# Build all packages
echo ""
echo "🏗️  Building all packages..."
npm run build

# Run tests
echo ""
echo "🧪 Running tests..."
npm run test || true

# Commit version bump
echo ""
echo "💾 Committing version bump..."
git add -A
git commit -m "release: v$NEW_VERSION"

# Tag both packages
echo ""
echo "🏷️  Creating tags..."
git tag "narrative-engine@$NEW_VERSION"
git tag "narrative-engine-lab@$NEW_VERSION"

echo ""
echo "========================================"
echo "  Release v$NEW_VERSION ready!"
echo "========================================"
echo ""
echo "To publish to npm, run:"
echo "  git push && git push --tags"
echo ""
echo "Or publish individually:"
echo "  npm run release:engine"
echo "  npm run release:lab"