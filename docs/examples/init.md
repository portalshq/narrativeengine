```bash
# Initialize a new repository
nap init toystory

# Initialize with local provider
nap init toystory --provider local

# Initialize with remote provider
nap init --provider remote --remote-url lore://localhost:41337 --workspace-id my-workspace

# Configure provider only (no repository creation)
nap init --provider local

# Initialize with a remote origin
nap init toystory --origin lore://localhost:41337/toystory
```
