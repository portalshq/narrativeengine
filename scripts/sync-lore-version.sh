#!/bin/bash
# Sync Lore version from cloud's versions.yaml to nap's hardcoded constants
#
# Usage:
#   ./scripts/sync-lore-version.sh [path-to-cloud-repo]          # interactive sync
#   ./scripts/sync-lore-version.sh --check-only [path-to-cloud-repo]  # CI gate (read-only)
#
# In --check-only mode the script downloads versions.yaml from GitHub
# (falling back to a local path or format-only validation) and exits 0
# if pins are current, 1 if stale or invalid.

set -euo pipefail

# ── Argument parsing ────────────────────────────────────────────────────

CHECK_ONLY=false
if [[ "${1:-}" == "--check-only" ]]; then
    CHECK_ONLY=true
    shift
fi

CLOUD_REPO="${1:-../cloud}"
VERSION_RS="crates/nap-core/src/server/version.rs"
VERSIONS_YAML_URL="https://raw.githubusercontent.com/portalshq/portals-cloud/main/infra/lore/versions.yaml"

# ── Helper: extract a YAML value ────────────────────────────────────────

yaml_value() {
    local section="$1" key="$2" file="$3"
    # Extract the section block, join continuation lines (backslash + newline + spaces),
    # then pull the quoted value for the given key.
    grep "^${section}:" -A 30 "$file" \
        | perl -0777 -pe 's/\\\n\s*//g' \
        | grep "  ${key}:" \
        | head -1 \
        | sed 's/.*: "\(.*\)".*/\1/'
}

# ── Helper: read current constants from version.rs ─────────────────────

rust_string_constant() {
    local name="$1"
    grep -A1 "pub const ${name}" "$VERSION_RS" | grep '"' | head -1 | cut -d'"' -f2
}

replace_rust_string_constant() {
    local name="$1" value="$2"
    RUST_CONSTANT_NAME="$name" RUST_CONSTANT_VALUE="$value" perl -0pi.bak -e '
        $name = $ENV{RUST_CONSTANT_NAME};
        $value = $ENV{RUST_CONSTANT_VALUE};
        $changed = s/(pub const \Q$name\E: &str =\s*)"[^"]*";/$1"$value";/s;
        die "constant $name not found\n" unless $changed == 1;
    ' "$VERSION_RS"
    rm -f "${VERSION_RS}.bak"
}

read_constants() {
    CUR_VERSION=$(rust_string_constant PINNED_LORE_VERSION)
    CUR_INSTALLER_SHA256=$(rust_string_constant PINNED_LORE_INSTALLER_SHA256)
    CUR_MANIFEST_SHA256=$(rust_string_constant PINNED_LORE_ARTIFACT_MANIFEST_SHA256)
    CUR_MANIFEST_URL=$(rust_string_constant PINNED_LORE_ARTIFACT_MANIFEST_URL)
    CUR_SIGNATURE_URL=$(rust_string_constant PINNED_LORE_SIGNATURE_BUNDLE_URL)
}

# ── Helper: format-only validation (no versions.yaml) ──────────────────

validate_format() {
    local ok=true
    echo "Format-only validation (no versions.yaml available):"
    echo "  PINNED_LORE_VERSION:                ${CUR_VERSION:-<empty>}"
    echo "  PINNED_LORE_INSTALLER_SHA256:       ${CUR_INSTALLER_SHA256:-<empty>}"
    echo "  PINNED_LORE_ARTIFACT_MANIFEST_SHA256: ${CUR_MANIFEST_SHA256:-<empty>}"
    echo "  PINNED_LORE_ARTIFACT_MANIFEST_URL:  ${CUR_MANIFEST_URL:-<empty>}"
    echo "  PINNED_LORE_SIGNATURE_BUNDLE_URL:   ${CUR_SIGNATURE_URL:-<empty>}"
    echo ""

    if [[ -z "$CUR_VERSION" ]]; then
        echo "  ✗ PINNED_LORE_VERSION is empty"; ok=false
    else
        echo "  ✓ PINNED_LORE_VERSION is non-empty"
    fi

    if [[ "$CUR_INSTALLER_SHA256" =~ ^[a-f0-9]{64}$ ]]; then
        echo "  ✓ PINNED_LORE_INSTALLER_SHA256 is valid"
    else
        echo "  ✗ PINNED_LORE_INSTALLER_SHA256 is empty or malformed"; ok=false
    fi

    if [[ "$CUR_MANIFEST_SHA256" =~ ^sha256:[a-f0-9]{64}$ ]]; then
        echo "  ✓ PINNED_LORE_ARTIFACT_MANIFEST_SHA256 is valid"
    else
        echo "  ✗ PINNED_LORE_ARTIFACT_MANIFEST_SHA256 is empty or malformed"; ok=false
    fi

    if [[ "$CUR_MANIFEST_URL" =~ ^https:// ]]; then
        echo "  ✓ PINNED_LORE_ARTIFACT_MANIFEST_URL is valid"
    else
        echo "  ✗ PINNED_LORE_ARTIFACT_MANIFEST_URL is empty or not HTTPS"; ok=false
    fi

    if [[ "$CUR_SIGNATURE_URL" =~ ^https:// ]]; then
        echo "  ✓ PINNED_LORE_SIGNATURE_BUNDLE_URL is valid"
    else
        echo "  ✗ PINNED_LORE_SIGNATURE_BUNDLE_URL is empty or not HTTPS"; ok=false
    fi

    echo ""
    if $ok; then
        echo "✓ All constants have valid format"
        return 0
    else
        echo "✗ One or more constants are empty or malformed"
        return 1
    fi
}

# ── Locate versions.yaml ───────────────────────────────────────────────

VERSIONS_FILE=""
SOURCE=""

# 1. Try local path
if [[ -f "$CLOUD_REPO/infra/lore/versions.yaml" ]]; then
    VERSIONS_FILE="$CLOUD_REPO/infra/lore/versions.yaml"
    SOURCE="local ($CLOUD_REPO)"
fi

# 2. Try GitHub raw download
if [[ -z "$VERSIONS_FILE" ]]; then
    TMP_YAML=$(mktemp /tmp/versions-XXXXXX.yaml)
    trap "rm -f '$TMP_YAML'" EXIT
    if curl -fsSL "$VERSIONS_YAML_URL" -o "$TMP_YAML" 2>/dev/null; then
        VERSIONS_FILE="$TMP_YAML"
        SOURCE="GitHub raw"
    fi
fi

# ── Validate version.rs exists ─────────────────────────────────────────

if [[ ! -f "$VERSION_RS" ]]; then
    echo "❌ Error: version.rs not found at: $VERSION_RS"
    exit 1
fi

read_constants

# ── --check-only: compare pins against versions.yaml ───────────────────

if $CHECK_ONLY; then
    echo "========================================"
    echo "  Lore Pin Check (--check-only)"
    echo "========================================"
    echo ""

    if [[ -z "$VERSIONS_FILE" ]]; then
        echo "⚠ No versions.yaml available (local or GitHub). Falling back to format validation."
        echo ""
        validate_format
        exit $?
    fi

    echo "✓ versions.yaml source: $SOURCE"
    echo ""

    # Extract expected values from versions.yaml
    EXPECT_VERSION=$(yaml_value "lore-client" "version" "$VERSIONS_FILE")
    EXPECT_INSTALLER_SHA256=$(yaml_value "lore-client" "installer_sha256" "$VERSIONS_FILE")
    EXPECT_MANIFEST_SHA256=$(yaml_value "lore-client" "artifact_manifest_sha256" "$VERSIONS_FILE")
    EXPECT_MANIFEST_URL=$(yaml_value "lore-client" "artifact_manifest_url" "$VERSIONS_FILE")
    EXPECT_SIGNATURE_URL=$(yaml_value "lore-client" "signature_bundle_url" "$VERSIONS_FILE")

    echo "Expected (from versions.yaml):"
    echo "  Lore version:             $EXPECT_VERSION"
    echo "  Installer SHA256:         $EXPECT_INSTALLER_SHA256"
    echo "  Manifest SHA256:          $EXPECT_MANIFEST_SHA256"
    echo "  Manifest URL:             $EXPECT_MANIFEST_URL"
    echo "  Signature bundle URL:     $EXPECT_SIGNATURE_URL"
    echo ""
    echo "Current (in version.rs):"
    echo "  PINNED_LORE_VERSION:                $CUR_VERSION"
    echo "  PINNED_LORE_INSTALLER_SHA256:       $CUR_INSTALLER_SHA256"
    echo "  PINNED_LORE_ARTIFACT_MANIFEST_SHA256: $CUR_MANIFEST_SHA256"
    echo "  PINNED_LORE_ARTIFACT_MANIFEST_URL:  $CUR_MANIFEST_URL"
    echo "  PINNED_LORE_SIGNATURE_BUNDLE_URL:   $CUR_SIGNATURE_URL"
    echo ""

    ok=true
    [[ "$CUR_VERSION" == "$EXPECT_VERSION" ]] \
        && echo "  ✓ PINNED_LORE_VERSION matches" \
        || { echo "  ✗ PINNED_LORE_VERSION mismatch: got '$CUR_VERSION', expected '$EXPECT_VERSION'"; ok=false; }

    [[ "$CUR_INSTALLER_SHA256" == "$EXPECT_INSTALLER_SHA256" ]] \
        && echo "  ✓ PINNED_LORE_INSTALLER_SHA256 matches" \
        || { echo "  ✗ PINNED_LORE_INSTALLER_SHA256 mismatch"; ok=false; }

    [[ "$CUR_MANIFEST_SHA256" == "$EXPECT_MANIFEST_SHA256" ]] \
        && echo "  ✓ PINNED_LORE_ARTIFACT_MANIFEST_SHA256 matches" \
        || { echo "  ✗ PINNED_LORE_ARTIFACT_MANIFEST_SHA256 mismatch: got '$CUR_MANIFEST_SHA256', expected '$EXPECT_MANIFEST_SHA256'"; ok=false; }

    [[ "$CUR_MANIFEST_URL" == "$EXPECT_MANIFEST_URL" ]] \
        && echo "  ✓ PINNED_LORE_ARTIFACT_MANIFEST_URL matches" \
        || { echo "  ✗ PINNED_LORE_ARTIFACT_MANIFEST_URL mismatch"; ok=false; }

    [[ "$CUR_SIGNATURE_URL" == "$EXPECT_SIGNATURE_URL" ]] \
        && echo "  ✓ PINNED_LORE_SIGNATURE_BUNDLE_URL matches" \
        || { echo "  ✗ PINNED_LORE_SIGNATURE_BUNDLE_URL mismatch"; ok=false; }

    echo ""
    if $ok; then
        echo "✓ Lore pins are current"
        exit 0
    else
        echo "✗ Lore pins are stale — run sync-lore-version.sh (without --check-only) to update"
        exit 1
    fi
fi

# ── Interactive sync ────────────────────────────────────────────────────

echo "========================================"
echo "  Sync Lore Version from Cloud to Nap"
echo "========================================"
echo ""

if [[ -z "$VERSIONS_FILE" ]]; then
    echo "❌ Error: versions.yaml not found"
    echo "  Tried: $CLOUD_REPO/infra/lore/versions.yaml"
    echo "  Tried: $VERSIONS_YAML_URL"
    echo "Usage: $0 [path-to-cloud-repo]"
    exit 1
fi

echo "✓ versions.yaml source: $SOURCE"
echo ""

# Extract lore-client information from versions.yaml
echo "Extracting Lore client information..."

LORE_VERSION=$(yaml_value "lore-client" "version" "$VERSIONS_FILE")
LORE_SOURCE_COMMIT=$(yaml_value "lore-client" "source_commit" "$VERSIONS_FILE")
LORE_INSTALLER_SHA256=$(yaml_value "lore-client" "installer_sha256" "$VERSIONS_FILE")
LORE_ARTIFACT_MANIFEST_URL=$(yaml_value "lore-client" "artifact_manifest_url" "$VERSIONS_FILE")
LORE_ARTIFACT_MANIFEST_SHA256=$(yaml_value "lore-client" "artifact_manifest_sha256" "$VERSIONS_FILE")
LORE_SIGNATURE_BUNDLE_URL=$(yaml_value "lore-client" "signature_bundle_url" "$VERSIONS_FILE")

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

echo "✓ version.rs found at: $VERSION_RS"
echo ""

# Show current values
echo "Current values in version.rs:"
echo "  PINNED_LORE_VERSION: $CUR_VERSION"
echo "  PINNED_LORE_INSTALLER_SHA256: $CUR_INSTALLER_SHA256"
echo "  PINNED_LORE_ARTIFACT_MANIFEST_SHA256: $CUR_MANIFEST_SHA256"
echo "  PINNED_LORE_ARTIFACT_MANIFEST_URL: $CUR_MANIFEST_URL"
echo "  PINNED_LORE_SIGNATURE_BUNDLE_URL: $CUR_SIGNATURE_URL"
echo ""

# Create backup
BACKUP_RS=$(mktemp "${VERSION_RS}.backup.XXXXXX")
INTEGRATION_RS="crates/nap-core/tests/lore_version_integration.rs"
BACKUP_INTEGRATION=""
if [[ -f "$INTEGRATION_RS" ]]; then
    BACKUP_INTEGRATION=$(mktemp "${INTEGRATION_RS}.backup.XXXXXX")
    cp "$INTEGRATION_RS" "$BACKUP_INTEGRATION"
fi
INSTALL_RS="crates/nap-core/src/server/install.rs"
BACKUP_INSTALL=""
if [[ -f "$INSTALL_RS" ]]; then
    BACKUP_INSTALL=$(mktemp "${INSTALL_RS}.backup.XXXXXX")
    cp "$INSTALL_RS" "$BACKUP_INSTALL"
fi
cp "$VERSION_RS" "$BACKUP_RS"
trap "rm -f '$BACKUP_RS' '${BACKUP_RS}' '${BACKUP_INTEGRATION}' '${BACKUP_INSTALL}'" EXIT
echo "✓ Backups created"

# ── Update version.rs ──────────────────────────────────────────────────

echo ""
echo "Updating version.rs..."

OLD_LORE_VERSION="$CUR_VERSION"
replace_rust_string_constant PINNED_LORE_VERSION "$LORE_VERSION"
replace_rust_string_constant PINNED_LORE_INSTALLER_SHA256 "$LORE_INSTALLER_SHA256"
replace_rust_string_constant PINNED_LORE_ARTIFACT_MANIFEST_SHA256 "$LORE_ARTIFACT_MANIFEST_SHA256"
replace_rust_string_constant PINNED_LORE_ARTIFACT_MANIFEST_URL "$LORE_ARTIFACT_MANIFEST_URL"
replace_rust_string_constant PINNED_LORE_SIGNATURE_BUNDLE_URL "$LORE_SIGNATURE_BUNDLE_URL"

# Keep exact-version fixtures and their comments aligned with the production
# pin. Mismatch tests use separate nightly/stable/semver values.
OLD_LORE_VERSION="$OLD_LORE_VERSION" NEW_LORE_VERSION="$LORE_VERSION" perl -pi.bak -e '
    s/\Q$ENV{OLD_LORE_VERSION}\E/$ENV{NEW_LORE_VERSION}/g
' "$VERSION_RS" "$INTEGRATION_RS" "$INSTALL_RS"
rm -f "${VERSION_RS}.bak" "${INTEGRATION_RS}.bak" "${INSTALL_RS}.bak"
echo "✓ version.rs updated"

# ── Update integration test file ───────────────────────────────────────

if [[ -f "$INTEGRATION_RS" ]]; then
    echo "Updating $INTEGRATION_RS..."

    echo "✓ $INTEGRATION_RS updated"
fi

# ── Update install.rs test ─────────────────────────────────────────────

if [[ -f "$INSTALL_RS" ]]; then
    echo "Updating $INSTALL_RS..."

    echo "✓ $INSTALL_RS updated"
fi

# ── Show results ───────────────────────────────────────────────────────

echo ""
echo "New values in version.rs:"
echo "  PINNED_LORE_VERSION: $(rust_string_constant PINNED_LORE_VERSION)"
echo "  PINNED_LORE_INSTALLER_SHA256: $(rust_string_constant PINNED_LORE_INSTALLER_SHA256)"
echo "  PINNED_LORE_ARTIFACT_MANIFEST_SHA256: $(rust_string_constant PINNED_LORE_ARTIFACT_MANIFEST_SHA256)"
echo "  PINNED_LORE_ARTIFACT_MANIFEST_URL: $(rust_string_constant PINNED_LORE_ARTIFACT_MANIFEST_URL)"
echo "  PINNED_LORE_SIGNATURE_BUNDLE_URL: $(rust_string_constant PINNED_LORE_SIGNATURE_BUNDLE_URL)"
echo ""

echo "========================================"
echo "  Changes Summary"
echo "========================================"
echo "--- version.rs"
diff -u "$BACKUP_RS" "$VERSION_RS" || true
if [[ -n "$BACKUP_INTEGRATION" && -f "$INTEGRATION_RS" ]]; then
    echo "--- $INTEGRATION_RS"
    diff -u "$BACKUP_INTEGRATION" "$INTEGRATION_RS" || true
fi
if [[ -n "$BACKUP_INSTALL" && -f "$INSTALL_RS" ]]; then
    echo "--- $INSTALL_RS"
    diff -u "$BACKUP_INSTALL" "$INSTALL_RS" || true
fi
echo ""

# Ask for confirmation
read -p "Apply these changes? [y/N] " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Changes cancelled. Restoring backups..."
    mv "$BACKUP_RS" "$VERSION_RS"
    [[ -n "$BACKUP_INTEGRATION" && -f "$BACKUP_INTEGRATION" ]] && mv "$BACKUP_INTEGRATION" "$INTEGRATION_RS"
    [[ -n "$BACKUP_INSTALL" && -f "$BACKUP_INSTALL" ]] && mv "$BACKUP_INSTALL" "$INSTALL_RS"
    exit 1
fi

# Remove backup on success
rm -f "$BACKUP_RS" "${BACKUP_RS}"
[[ -n "$BACKUP_INTEGRATION" ]] && rm -f "$BACKUP_INTEGRATION"
[[ -n "$BACKUP_INSTALL" ]] && rm -f "$BACKUP_INSTALL"

echo ""
echo "========================================"
echo "  ✓ Lore version sync completed"
echo "========================================"
echo ""
echo "Updated to Lore version: $LORE_VERSION"
echo ""

# Run cargo fmt to fix long lines produced by sed
if command -v cargo &>/dev/null; then
    echo "Running cargo fmt..."
    cargo fmt --all
    echo "✓ cargo fmt completed"
    echo ""
fi

echo "Next steps:"
echo "  1. Review the changes"
echo "  2. Run tests: cargo test"
echo "  3. Commit the changes"
echo ""
