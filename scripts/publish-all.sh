#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

CURRENT_VERSION=$(grep '^version = ' "$ROOT_DIR/Cargo.toml" | head -1 | sed 's/^version = "\(.*\)"/\1/')
WORKSPACE_PACKAGES=(
  "portalshq-px-cli"
  "px-mcp-server"
  "portalshq-px"
  "px-docgen"
  "px-server"
  "px-test-utils"
  "narrativeengine"
  "narrativeengine-py"
  "px-sdk-py"
  "px-sdk-ts"
)

usage() {
  cat <<EOF
Usage: $0 <version>

  <version> can be an explicit semver like 0.2.0, or one of:
    patch    bump the patch segment  (${CURRENT_VERSION} → next patch)
    minor    bump the minor segment  (${CURRENT_VERSION} → next minor)
    major    bump the major segment  (${CURRENT_VERSION} → next major)

This script will:
  1. Verify you're on a clean main branch
  2. Run the required local Lore resolver integration test
  3. Bump all release versions (Cargo, Cargo.lock, Python, TypeScript)
  4. Run pre-publish validation
  5. Commit the release
  6. Create an annotated tag (vX.Y.Z)
  7. Push main and tags to origin

After the push, GitHub Actions will start the publish workflow.
If the 'production' environment requires approval, approve it in GitHub.
EOF
}

compute_next_version() {
  local version="$1"
  local segment="$2"
  local major minor patch
  IFS='.' read -r major minor patch <<< "$version"
  case "$segment" in
    patch) echo "$major.$minor.$((patch + 1))" ;;
    minor) echo "$major.$((minor + 1)).0" ;;
    major) echo "$((major + 1)).0.0" ;;
    *)
      echo "Internal error: unsupported segment '$segment'" >&2
      exit 1
      ;;
  esac
}

if [ $# -ne 1 ]; then
  usage
  exit 1
fi

case "$1" in
  patch|minor|major)
    NEW_VERSION=$(compute_next_version "$CURRENT_VERSION" "$1")
    ;;
  *)
    NEW_VERSION="$1"
    if ! echo "$NEW_VERSION" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
      echo "Error: version must be X.Y.Z or one of: patch, minor, major" >&2
      echo "  Got: '$NEW_VERSION'" >&2
      exit 1
    fi
    ;;
esac

RELEASE_TAG="v$NEW_VERSION"

if [ "$NEW_VERSION" = "$CURRENT_VERSION" ]; then
  echo "Error: new version matches current version ($CURRENT_VERSION)" >&2
  exit 1
fi

if [ "$(git rev-parse --abbrev-ref HEAD)" != "main" ]; then
  echo "Error: must be on main branch to release (currently on '$(git rev-parse --abbrev-ref HEAD)')" >&2
  exit 1
fi

if [ -n "$(git status --porcelain)" ]; then
  echo "Error: working tree has uncommitted changes — commit or stash them first" >&2
  exit 1
fi

if git rev-parse "$RELEASE_TAG" >/dev/null 2>&1; then
  echo "Error: tag $RELEASE_TAG already exists locally" >&2
  exit 1
fi

# Fetch the latest remote changes to ensure checks are accurate
echo "Fetching origin main..."
git fetch origin main

if git ls-remote --tags origin "refs/tags/$RELEASE_TAG" | grep -q "$RELEASE_TAG"; then
  echo "Error: tag $RELEASE_TAG already exists on origin" >&2
  exit 1
fi

# Verify local branch includes all remote history (it can be ahead, but not behind)
if ! git merge-base --is-ancestor origin/main HEAD; then
  echo "Error: local main is missing changes from origin/main — pull/rebase first" >&2
  exit 1
fi

echo ""
echo "Running required local Lore resolver integration test..."
"$ROOT_DIR/scripts/test-integration-local.sh" \
  test_local_lore_remote_resolve_reads_manifest_from_parent_tree --exact
echo "✓ Local Lore resolver integration test passed"

echo "Bumping release version: $CURRENT_VERSION → $NEW_VERSION"

python3 - "$ROOT_DIR" "$CURRENT_VERSION" "$NEW_VERSION" <<'PY'
import json
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
current = sys.argv[2]
new = sys.argv[3]
workspace_packages = [
    "portalshq-px-cli",
    "px-mcp-server",
    "portalshq-px",
    "px-docgen",
    "px-server",
    "px-test-utils",
    "narrativeengine",
    "narrativeengine-py",
    "px-sdk-py",
    "px-sdk-ts",
    ]

replacements = {
    root / "Cargo.toml": [r'version = "(\d+\.\d+\.\d+)"'],
    root / "python/narrativeengine/pyproject.toml": [r'version = "(\d+\.\d+\.\d+)"'],
    root / "python/px-sdk/pyproject.toml": [r'version = "(\d+\.\d+\.\d+)"'],
    root / "typescript/narrativeengine/package.json": [r'"version": "(\d+\.\d+\.\d+)"'],
    root / "typescript/px-sdk/package.json": [r'"version": "(\d+\.\d+\.\d+)"'],
}

for path, patterns in replacements.items():
    text = path.read_text()
    for pattern in patterns:
        match = re.search(pattern, text)
        if not match:
            raise SystemExit(f"Expected version pattern not found in {path}: {pattern}")
        old_version = match.group(1)
        if old_version != current:
            print(f"note: {path} is at {old_version} (workspace at {current}); bumping to {new}")
        text = text[:match.start(1)] + new + text[match.end(1):]
    path.write_text(text)

for path in [
    root / "typescript/narrativeengine/package-lock.json",
    root / "typescript/px-sdk/package-lock.json",
]:
    document = json.loads(path.read_text())
    old_version = document.get("version")
    inner_version = document.get("packages", {}).get("", {}).get("version")
    if old_version != inner_version:
        raise SystemExit(f"Conflicting root package versions in {path}: {old_version} vs {inner_version}")
    if old_version is None:
        raise SystemExit(f"No root package version found in {path}")
    if old_version != current:
        print(f"note: {path} is at {old_version} (workspace at {current}); bumping to {new}")
    document["version"] = new
    document["packages"][""]["version"] = new
    path.write_text(json.dumps(document, indent=2) + "\n")

for path, package in [
    (root / "python/narrativeengine/uv.lock", "narrativeengine"),
    (root / "python/px-sdk/uv.lock", "px-sdk"),
]:
    text = path.read_text()
    pattern = rf'(\[\[package\]\]\nname = "{re.escape(package)}"\nversion = ")(\d+\.\d+\.\d+)(")'
    match = re.search(pattern, text)
    if not match:
        raise SystemExit(f"Expected package/version pair not found in {path}: {package}")
    if match.group(2) != current:
        print(f"note: {package} in {path} is at {match.group(2)} (workspace at {current}); bumping to {new}")
    text = text[:match.start(2)] + new + text[match.end(2):]
    path.write_text(text)

cargo_lock = root / "Cargo.lock"
text = cargo_lock.read_text()
for package in workspace_packages:
    pattern = rf'(name = "{re.escape(package)}"\nversion = ")(\d+\.\d+\.\d+)(")'
    match = re.search(pattern, text)
    if not match:
        raise SystemExit(f"Expected package/version pair not found in Cargo.lock: {package}")
    if match.group(2) != current:
        print(f"note: {package} in Cargo.lock is at {match.group(2)} (workspace at {current}); bumping to {new}")
    text = text[:match.start(2)] + new + text[match.end(2):]
cargo_lock.write_text(text)
PY

echo "✓ Versions updated"

echo ""
echo "Running pre-publish validation..."

npm --prefix typescript/narrativeengine ci
npm --prefix typescript/px-sdk ci
env GITHUB_REF_NAME="$RELEASE_TAG" node scripts/pre-publish-check.mjs
npm --prefix typescript/narrativeengine run build:types
npm --prefix typescript/px-sdk run build:types

echo "✓ Release validation passed"

echo ""
echo "Committing and tagging $RELEASE_TAG..."

git add Cargo.toml Cargo.lock crates/px-core/Cargo.toml crates/px-cli/Cargo.toml crates/px-mcp-server/Cargo.toml crates/px-docgen/Cargo.toml crates/px-server/Cargo.toml crates/px-test-utils/Cargo.toml crates/narrativeengine/Cargo.toml python/narrativeengine/pyproject.toml python/narrativeengine/uv.lock python/px-sdk/pyproject.toml python/px-sdk/uv.lock typescript/narrativeengine/package.json typescript/narrativeengine/package-lock.json typescript/px-sdk/Cargo.toml typescript/px-sdk/package.json typescript/px-sdk/package-lock.json
if [ -n "${RELEASE_COMMIT_COAUTHOR:-}" ]; then
  git commit -m "chore(release): cut $RELEASE_TAG" -m "Co-Authored-By: ${RELEASE_COMMIT_COAUTHOR}"
else
  git commit -m "chore(release): cut $RELEASE_TAG"
fi
git tag -a "$RELEASE_TAG" -m "$RELEASE_TAG"

echo ""
echo "Pushing main and tags..."

git push origin main --tags

echo ""
echo "✓ Pushed $RELEASE_TAG"
echo "→ GitHub Actions publish workflow should now be running"
echo "→ If the 'production' environment requires approval, approve it in GitHub"
echo "https://github.com/portalshq/narrativeengine/actions"
