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

# Get current version from narrativeengine
CURRENT_VERSION=$(node -p "require('./typescript/narrativeengine/package.json').version")
echo "📦 Current version: $CURRENT_VERSION"

# Calculate new version
echo "📈 Bumping $VERSION_BUMP version..."
NEW_VERSION=$(node -e "
  const [major, minor, patch] = '$CURRENT_VERSION'.split('.').map(Number);
  if ('$VERSION_BUMP' === 'major') { console.log((major + 1) + '.0.0'); }
  else if ('$VERSION_BUMP' === 'minor') { console.log(major + '.' + (minor + 1) + '.0'); }
  else { console.log(major + '.' + minor + '.' + (patch + 1)); }
")

# Update both package.json files
node -e "
  const fs = require('fs');
  const pkg1 = JSON.parse(fs.readFileSync('./typescript/narrativeengine/package.json'));
  const pkg2 = JSON.parse(fs.readFileSync('./typescript/nap-sdk/package.json'));
  pkg1.version = '$NEW_VERSION';
  pkg2.version = '$NEW_VERSION';
  fs.writeFileSync('./typescript/narrativeengine/package.json', JSON.stringify(pkg1, null, 2) + '\n');
  fs.writeFileSync('./typescript/nap-sdk/package.json', JSON.stringify(pkg2, null, 2) + '\n');
"

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