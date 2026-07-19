# NAP Integration Tests

This directory contains integration test suites for the NAP (Narrative Addressing Protocol) CLI.

## Test Suites

### 1. Local Lore Server Suite (`local_lore_suite.rs`)

Tests nap functionality against a local lore server.

**Requirements:**
- A running local lore server at `lore://localhost:41337`
- The `lore` binary in PATH
- Environment: `NAP_LORE_URL_BASE=lore://localhost:41337`

**Test Coverage:**
- Connect to local lore server and initialize nap
- Create a repository
- Clone a repository
- Create entities (characters, locations, scenes, props)
- Update repository files
- Add images to repository
- Resolve manifest URIs using nap resolver
- Resolve images from manifest URIs
- List entities
- View commit history
- Branch operations (create, list, switch)
- Tag operations (create, list)
- Status and doctor commands
- Remote operations (add, list, remove)
- Sync operations (push, pull, sync)
- Content hash computation

**Running the tests:**

**Option 1: Using the provided script (recommended)**
```bash
# Ensure local lore server is running
./scripts/test-integration-local.sh
```

**Option 2: Using Just (recommended for Rust projects)**
```bash
# Install just: cargo install just
just test-integration-local
```

**Option 3: Direct cargo command**
```bash
# Ensurelocal lore server is running
cargo test -p nap-cli --test local_lore_suite --features lore-e2e -- --test-threads=1
```

### 2. Portals Cloud Lore Server Suite (`cloud_lore_suite.rs`)

Tests nap functionality against the Portals Cloud lore server.

**Requirements:**
- Valid Portals Cloud lore server URL
- Valid authentication credentials
- The `lore` binary in PATH

**Environment Variables:**
- `NAP_LORE_URL_BASE`: Portals Cloud lore server URL (e.g., `lore://cloud.portals.ai`)
- `NAP_WORKSPACE_ID`: Workspace ID for Portals Cloud
- `PORTALS_CLOUD_AUTH_TOKEN`: Authentication token (if required)

**Test Coverage:**
- Connect to Portals Cloud and initialize nap
- Switch between backends (local to portals-cloud)
- Create a repository on cloud
- Clone a repository from cloud
- Create entities on cloud
- Update repository files on cloud
- Add images to repository on cloud
- Resolve manifest URIs from cloud
- Resolve images from manifest URIs on cloud
- List entities on cloud
- View commit history from cloud
- Branch operations on cloud
- Tag operations on cloud
- Status and doctor commands for cloud
- Remote operations for cloud
- Sync operations with cloud
- Content hash computation
- Query subtrees
- Resolve with branch selectors
- Validate manifests

**Running the tests:**

**Option 1: Using the provided script (recommended)**
```bash
# Set environment variables
export NAP_LORE_URL_BASE="lore://cloud.portals.ai"
export NAP_WORKSPACE_ID="your-workspace-id"
export PORTALS_CLOUD_AUTH_TOKEN="your-auth-token"

# Run the tests
./scripts/test-integration-cloud.sh
```

**Option 2: Using Just (recommended for Rust projects)**
```bash
# Install just: cargo install just
# Set environment variables
export NAP_LORE_URL_BASE="lore://cloud.portals.ai"
export NAP_WORKSPACE_ID="your-workspace-id"
export PORTALS_CLOUD_AUTH_TOKEN="your-auth-token"

# Run the tests
just test-integration-cloud
```

**Option 3: Direct cargo command**
```bash
# Set environment variables
export NAP_LORE_URL_BASE="lore://cloud.portals.ai"
export NAP_WORKSPACE_ID="your-workspace-id"
export PORTALS_CLOUD_AUTH_TOKEN="your-auth-token"

# Run the tests
cargo test -p nap-cli --test cloud_lore_suite --features lore-e2e -- --test-threads=1
```

## Common Test Utilities

Both test suites share common testing patterns:

### Helper Functions

- `nap_cmd()`: Returns a configured Command instance for the nap binary
- `create_test_image()`: Creates a minimal PNG test image (1x1 transparent pixel)
- `unique_universe_name()`: Generates unique repository names using timestamps

### Test Structure

Each test follows this pattern:
1. Create a temporary directory
2. Initialize nap with appropriate provider
3. Create a repository repository
4. Perform the operation being tested
5. Verify the result using assertions
6. Clean up (tempdir auto-cleanup)

## Running All Integration Tests

To run all integration tests:

```bash
# Local suite only
just test-integration-local
# or
./scripts/test-integration-local.sh

# Cloud suite only (requires environment setup)
just test-integration-cloud
# or
./scripts/test-integration-cloud.sh

# Both suites (requires both local and cloud setup)
just test-integration-local && just test-integration-cloud
```

## Troubleshooting

### Local Lore Server Tests Fail

1. Ensure lore server is running: `lore status`
2. Check the server URL: `echo $NAP_LORE_URL_BASE`
3. Verify lore binary is in PATH: `which lore`
4. Check server connectivity: `lore repository list`

### Cloud Lore Server Tests Fail

1. Verify environment variables are set:
   ```bash
   echo $NAP_LORE_URL_BASE
   echo $NAP_WORKSPACE_ID
   echo $PORTALS_CLOUD_AUTH_TOKEN
   ```
2. Check authentication credentials are valid
3. Verify network connectivity to cloud server
4. Check workspace permissions

### Test Timeout Issues

Increase timeout in `nap_cmd()` helper function if tests are timing out due to slow network or server response times.

## Adding New Tests

When adding new integration tests:

1. Choose the appropriate suite (local or cloud)
2. Use the existing helper functions
3. Follow the naming convention: `test_<feature>_<operation>`
4. Add descriptive assertions
5. Clean up resources (tempdir handles this automatically)
6. Update this README with the new test coverage

## CI/CD Integration

These integration tests can be integrated into CI/CD pipelines:

**GitHub Actions Example:**
```yaml
name: NAP Integration Tests

on: [push, pull_request]

jobs:
  local-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Start Lore Server
        run: lore server start
      - name: Run Local Integration Tests
        run: cargo test -p nap-cli --test local_lore_suite --features lore-e2e -- --test-threads=1

  cloud-tests:
    runs-on: ubuntu-latest
    needs: local-tests
    steps:
      - uses: actions/checkout@v3
      - name: Run Cloud Integration Tests
        env:
          NAP_LORE_URL_BASE: ${{ secrets.NAP_LORE_URL_BASE }}
          NAP_WORKSPACE_ID: ${{ secrets.NAP_WORKSPACE_ID }}
          PORTALS_CLOUD_AUTH_TOKEN: ${{ secrets.PORTALS_CLOUD_AUTH_TOKEN }}
        run: cargo test -p nap-cli --test cloud_lore_suite --features lore-e2e -- --test-threads=1
```

## Notes

- Tests use `--test-threads=1` to prevent race conditions between tests
- Temporary directories are automatically cleaned up after each test
- Each test uses unique repository names to avoid conflicts
- Tests are designed to be independent and can run in any order (within a suite)
