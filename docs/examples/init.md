```bash
# Initialize a new repository
px init toystory

# Initialize with local provider
px init toystory --provider local

# Initialize with remote provider
px init --provider remote --remote-url lore://localhost:41337 --workspace-id my-workspace

# Configure provider only (no repository creation)
px init --provider local

# Initialize with a remote origin
px init toystory --origin lore://localhost:41337/toystory
```
