#!/bin/bash
# Sync Lore version from cloud's versions.yaml to nap's hardcoded constants
# Usage: ./scripts/sync-lore-version.sh [path-to-cloud-repo]

set -euo pipefail

# Default to ../cloud if no path provided
CLOUD_REPO="${1:-../cloud}"
VERSIONS_FILE="$CLOUD_REPO/infra/lore/versions.yaml"
VERSION_RS="crates/nap-core/src/server/version.rs"

echo "========================================"
echo "  Sync Lore Version from Cloud to Nap"
echo "========================================"
echo ""

# Validate cloud repository exists
if [[ ! -d "$CLOUD_REPO" ]]; then
    echo "❌ Error: Cloud repository not found at: $CLOUD_REPO"
    echo "Please provide the correct path to the cloud repository."
    echo "Usage: $0 [path-to-cloud-repo]"
    exit 1
fi

echo "✓ Cloud repository found at: $CLOUD_REPO"

# Validate versions.yaml exists
if [[ ! -f "$VERSIONS_FILE" ]]; then
    echo "❌ Error: versions.yaml not found at: $VERSIONS_FILE"
    exit 1
fi

echo "✓ versions.yaml found at: $VERSIONS_FILE"
echo ""

# Extract lore-client information from versions.yaml
echo "Extracting Lore client information..."

# Use simple grep/sed to extract values (works for this YAML structure)
# This avoids Python dependency issues
LORE_VERSION=$(grep "^lore-client:" -A 20 "$VERSIONS_FILE" | grep "  version:" | sed 's/.*: "\(.*\)".*/\1/')
LORE_SOURCE_COMMIT=$(grep "^lore-client:" -A 20 "$VERSIONS_FILE" | grep "  source_commit:" | sed 's/.*: "\(.*\)".*/\1/')
LORE_INSTALLER_SHA256=$(grep "^lore-client:" -A 20 "$VERSIONS_FILE" | grep "  installer_sha256:" | sed 's/.*: "\(.*\)".*/\1/')
LORE_ARTIFACT_MANIFEST_URL=$(grep "^lore-client:" -A 20 "$VERSIONS_FILE" | grep "  artifact_manifest_url:" | sed 's/.*: "\(.*\)".*/\1/')
LORE_ARTIFACT_MANIFEST_SHA256=$(grep "^lore-client:" -A 20 "$VERSIONS_FILE" | grep "  artifact_manifest_sha256:" | sed 's/.*: "\(.*\)".*/\1/')
LORE_SIGNATURE_BUNDLE_URL=$(grep "^lore-client:" -A 20 "$VERSIONS_FILE" | grep "  signature_bundle_url:" | sed 's/.*: "\(.*\)".*/\1/')

# Validate required fields
if [[ -z "$LORE_VERSION" ]]; then
    echo "❌ Error: lore-client.version is missing or empty in versions.yaml"
    exit 1
fi

if [[ -z "$LORE_INSTALLER_SHA256" ]]; then
    echo "❌ Error: lore-client.installer_sha256 is missing or empty in versions.yaml"
    exit 1
fi

# Validate checksum format (64 hex characters)
if [[ ! "$LORE_INSTALLER_SHA256" =~ ^[a-f0-9]{64}$ ]]; then
    echo "❌ Error: installer_sha256 has invalid format: $LORE_INSTALLER_SHA256"
    echo "Expected 64 hexadecimal characters"
    exit 1
fi

# Validate artifact manifest SHA256 format if not empty
if [[ -n "$LORE_ARTIFACT_MANIFEST_SHA256" && ! "$LORE_ARTIFACT_MANIFEST_SHA256" =~ ^sha256:[a-f0-9]{64}$ ]]; then
    echo "❌ Error: artifact_manifest_sha256 has invalid format: $LORE_ARTIFACT_MANIFEST_SHA256"
    echo "Expected format: sha256: followed by 64 hexadecimal characters"
    exit 1
fi

# Validate URLs if not empty
if [[ -n "$LORE_ARTIFACT_MANIFEST_URL" && ! "$LORE_ARTIFACT_MANIFEST_URL" =~ ^https:// ]]; then
    echo "❌ Error: artifact_manifest_url has invalid format: $LORE_ARTIFACT_MANIFEST_URL"
    echo "Expected HTTPS URL"
    exit 1
fi

if [[ -n "$LORE_SIGNATURE_BUNDLE_URL" && ! "$LORE_SIGNATURE_BUNDLE_URL" =~ ^https:// ]]; then
    echo "❌ Error: signature_bundle_url has invalid format: $LORE_SIGNATURE_BUNDLE_URL"
    echo "Expected HTTPS URL"
    exit 1
fi

echo "✓ Extracted Lore version: $LORE_VERSION"
echo "✓ Extracted source commit: ${LORE_SOURCE_COMMIT:-<not set>}"
echo "✓ Extracted installer SHA256: $LORE_INSTALLER_SHA256"
echo "✓ Extracted artifact manifest URL: ${LORE_ARTIFACT_MANIFEST_URL:-<not set>}"
echo "✓ Extracted artifact manifest SHA256: ${LORE_ARTIFACT_MANIFEST_SHA256:-<not set>}"
echo "✓ Extracted signature bundle URL: ${LORE_SIGNATURE_BUNDLE_URL:-<not set>}"
echo ""

# Validate that version.rs exists
if [[ ! -f "$VERSION_RS" ]]; then
    echo "❌ Error: version.rs not found at: $VERSION_RS"
    exit 1
fi

echo "✓ version.rs found at: $VERSION_RS"
echo ""

# Show current values
echo "Current values in version.rs:"
echo "  PINNED_LORE_VERSION: $(grep 'pub const PINNED_LORE_VERSION' "$VERSION_RS" | cut -d'"' -f2)"
echo "  PINNED_LORE_INSTALLER_SHA256: $(grep 'pub const PINNED_LORE_INSTALLER_SHA256' "$VERSION_RS" | tail -1 | cut -d'"' -f2)"
echo "  PINNED_LORE_ARTIFACT_MANIFEST_SHA256: $(grep 'pub const PINNED_LORE_ARTIFACT_MANIFEST_SHA256' "$VERSION_RS" | cut -d'"' -f2)"
echo "  PINNED_LORE_ARTIFACT_MANIFEST_URL: $(grep 'pub const PINNED_LORE_ARTIFACT_MANIFEST_URL' "$VERSION_RS" | cut -d'"' -f2)"
echo "  PINNED_LORE_SIGNATURE_BUNDLE_URL: $(grep 'pub const PINNED_LORE_SIGNATURE_BUNDLE_URL' "$VERSION_RS" | cut -d'"' -f2)"
echo ""

# Create backup
BACKUP_FILE="${VERSION_RS}.backup"
cp "$VERSION_RS" "$BACKUP_FILE"
echo "✓ Backup created at: $BACKUP_FILE"
echo ""

# Update version.rs
echo "Updating version.rs..."

# Update PINNED_LORE_VERSION
sed -i.bak "s|^pub const PINNED_LORE_VERSION: &str = \".*\";|pub const PINNED_LORE_VERSION: \&str = \"$LORE_VERSION\";|" "$VERSION_RS"

# Update PINNED_LORE_INSTALLER_SHA256
sed -i.bak "s|^pub const PINNED_LORE_INSTALLER_SHA256: &str =|pub const PINNED_LORE_INSTALLER_SHA256: \&str =|" "$VERSION_RS"
sed -i.bak "s|^    \"[a-f0-9]*\";$|    \"$LORE_INSTALLER_SHA256\";|" "$VERSION_RS"

# Update PINNED_LORE_ARTIFACT_MANIFEST_SHA256
sed -i.bak "s|^pub const PINNED_LORE_ARTIFACT_MANIFEST_SHA256: &str = \".*\";|pub const PINNED_LORE_ARTIFACT_MANIFEST_SHA256: \&str = \"$LORE_ARTIFACT_MANIFEST_SHA256\";|" "$VERSION_RS"

# Update PINNED_LORE_ARTIFACT_MANIFEST_URL
sed -i.bak "s|^pub const PINNED_LORE_ARTIFACT_MANIFEST_URL: &str = \".*\";|pub const PINNED_LORE_ARTIFACT_MANIFEST_URL: \&str = \"$LORE_ARTIFACT_MANIFEST_URL\";|" "$VERSION_RS"

# Update PINNED_LORE_SIGNATURE_BUNDLE_URL
sed -i.bak "s|^pub const PINNED_LORE_SIGNATURE_BUNDLE_URL: &str = \".*\";|pub const PINNED_LORE_SIGNATURE_BUNDLE_URL: \&str = \"$LORE_SIGNATURE_BUNDLE_URL\";|" "$VERSION_RS"

# Clean up sed backup files
rm -f "${VERSION_RS}.bak"

echo "✓ version.rs updated"
echo ""

# Show new values
echo "New values in version.rs:"
echo "  PINNED_LORE_VERSION: $(grep 'pub const PINNED_LORE_VERSION' "$VERSION_RS" | cut -d'"' -f2)"
echo "  PINNED_LORE_INSTALLER_SHA256: $(grep 'pub const PINNED_LORE_INSTALLER_SHA256' "$VERSION_RS" | tail -1 | cut -d'"' -f2)"
echo "  PINNED_LORE_ARTIFACT_MANIFEST_SHA256: $(grep 'pub const PINNED_LORE_ARTIFACT_MANIFEST_SHA256' "$VERSION_RS" | cut -d'"' -f2)"
echo "  PINNED_LORE_ARTIFACT_MANIFEST_URL: $(grep 'pub const PINNED_LORE_ARTIFACT_MANIFEST_URL' "$VERSION_RS" | cut -d'"' -f2)"
echo "  PINNED_LORE_SIGNATURE_BUNDLE_URL: $(grep 'pub const PINNED_LORE_SIGNATURE_BUNDLE_URL' "$VERSION_RS" | cut -d'"' -f2)"
echo ""

# Show diff
echo "========================================"
echo "  Changes Summary"
echo "========================================"
diff -u "$BACKUP_FILE" "$VERSION_RS" || true
echo ""

# Ask for confirmation
read -p "Apply these changes? [y/N] " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Changes cancelled. Restoring backup..."
    mv "$BACKUP_FILE" "$VERSION_RS"
    exit 1
fi

# Remove backup on success
rm "$BACKUP_FILE"

echo ""
echo "========================================"
echo "  ✓ Lore version sync completed"
echo "========================================"
echo ""
echo "Updated to Lore version: $LORE_VERSION"
echo ""
echo "Next steps:"
echo "  1. Review the changes in version.rs"
echo "  2. Run tests: cargo test"
echo "  3. Commit the changes"
echo ""