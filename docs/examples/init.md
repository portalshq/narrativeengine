```bash
# Initialize a new universe
nap init starwars

# Initialize with local provider
nap init starwars --provider local

# Initialize with remote provider
nap init --provider remote --remote-url lore://localhost:41337 --workspace-id my-workspace

# Configure provider only (no universe creation)
nap init --provider local

# Initialize with a remote origin
nap init starwars --remote git@github.com:user/starwars.git
```
